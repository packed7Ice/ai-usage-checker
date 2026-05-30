use super::{LogParser, UsageRecord};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub struct GeminiParser;

#[async_trait]
impl LogParser for GeminiParser {
    fn tool_name(&self) -> &'static str {
        "gemini"
    }

    async fn parse(&self) -> Result<Vec<UsageRecord>> {
        let base = resolve_base_path()?;
        let pattern = base.join("*/chats/*.json");
        let pattern_str = pattern.to_string_lossy().replace('\\', "/");

        let mut records = Vec::new();

        for entry in glob::glob(&pattern_str)? {
            let path = entry?;
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

                    // Gemini JSON には個別のタイムスタンプがない場合が多い。
                    // ファイルの更新時刻をフォールバックとして使用
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

                    records.push(UsageRecord {
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
        }

        Ok(records)
    }
}

fn resolve_base_path() -> Result<PathBuf> {
    // 環境変数での上書きを優先
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
