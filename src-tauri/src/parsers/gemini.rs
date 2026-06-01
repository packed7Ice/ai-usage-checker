use super::{get_parse_state, set_parse_state, LogParser, UsageRecord};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use sqlx::SqlitePool;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub struct GeminiParser;

#[async_trait]
impl LogParser for GeminiParser {
    fn tool_name(&self) -> &'static str {
        "gemini"
    }

    async fn parse(&self, pool: &SqlitePool) -> Result<Vec<UsageRecord>> {
        let base = resolve_base_path()?;
        let pattern = base.join("*/chats/*.json");
        let pattern_str = pattern.to_string_lossy().replace('\\', "/");

        let mut all_records = Vec::new();

        for entry in glob::glob(&pattern_str)? {
            let path = entry?;
            let path_str = path.to_string_lossy().to_string();

            let meta = std::fs::metadata(&path)?;
            let mtime = meta.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;

            let (_, last_mtime) = get_parse_state(pool, &path_str).await?;

            if mtime <= last_mtime {
                // 未変更ファイルはスキップ
                continue;
            }

            let file = File::open(&path)?;
            let reader = BufReader::new(file);
            let json: Value = serde_json::from_reader(reader)?;

            // Gemini の JSON は配列またはオブジェクト
            let entries = match json {
                Value::Array(arr) => arr,
                Value::Object(_) => vec![json],
                _ => continue,
            };

            for item in entries {
                if let Some(meta) = item.get("usageMetadata") {
                    let input = meta
                        .get("promptTokenCount")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let output = meta
                        .get("candidatesTokenCount")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    let recorded_at = item
                        .get("createTime")
                        .and_then(|v| v.as_str())
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.timestamp())
                        .or_else(|| {
                            std::fs::metadata(&path)
                                .ok()
                                .and_then(|m| m.modified().ok())
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| d.as_secs() as i64)
                        })
                        .unwrap_or_else(|| chrono::Utc::now().timestamp());

                    all_records.push(UsageRecord {
                        tool: self.tool_name().to_string(),
                        session_id: None,
                        recorded_at,
                        input_tokens: input,
                        output_tokens: output,
                        cache_tokens: 0,
                        cost_usd: 0.0,
                    });
                }
            }

            set_parse_state(pool, &path_str, 0, mtime).await?;
        }

        Ok(all_records)
    }
}

fn resolve_base_path() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("GEMINI_CLI_HOME") {
        return Ok(PathBuf::from(dir).join("tmp"));
    }
    if let Ok(dir) = std::env::var("GEMINI_DATA_DIR") {
        return Ok(PathBuf::from(dir).join("tmp"));
    }

    if cfg!(target_os = "windows") {
        let home = std::env::var("USERPROFILE").context("USERPROFILE not set")?;
        Ok(PathBuf::from(home).join(".gemini/tmp"))
    } else {
        let home = std::env::var("HOME").context("HOME not set")?;
        Ok(PathBuf::from(home).join(".gemini/tmp"))
    }
}
