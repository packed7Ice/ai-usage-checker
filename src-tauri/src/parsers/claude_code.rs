use super::{get_parse_state, set_parse_state, LogParser, UsageRecord};
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::SqlitePool;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub struct ClaudeCodeParser;

impl ClaudeCodeParser {
    async fn parse_single_dir(&self, pool: &SqlitePool, base: &PathBuf) -> Result<Vec<UsageRecord>> {
        let pattern = base.join("**/*.jsonl");
        let pattern_str = pattern.to_string_lossy().replace('\\', "/");

        let mut records = Vec::new();

        for entry in glob::glob(&pattern_str)? {
            let path = entry?;
            let path_str = path.to_string_lossy().to_string();

            let meta = std::fs::metadata(&path)?;
            let mtime = meta.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;

            let (last_offset, last_mtime) = get_parse_state(pool, &path_str).await?;

            if mtime <= last_mtime {
                continue;
            }

            let file = File::open(&path)?;
            let reader = BufReader::new(file);

            let mut line_count = 0i64;
            for line in reader.lines() {
                let line = line?;
                line_count += 1;

                if line_count <= last_offset {
                    continue;
                }
                if line.trim().is_empty() {
                    continue;
                }

                let json: Value = match serde_json::from_str(&line) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                let timestamp = json
                    .get("timestamp")
                    .and_then(|v| v.as_str())
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc).timestamp());

                let usage = json.get("usage");
                let input_tokens = usage
                    .and_then(|u| u.get("input_tokens"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let output_tokens = usage
                    .and_then(|u| u.get("output_tokens"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let cache_tokens = usage
                    .and_then(|u| u.get("cache_read_input_tokens"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                if let Some(ts) = timestamp {
                    records.push(UsageRecord {
                        tool: self.tool_name().to_string(),
                        session_id: None,
                        recorded_at: ts,
                        input_tokens,
                        output_tokens,
                        cache_tokens,
                        cost_usd: 0.0,
                    });
                }
            }

            set_parse_state(pool, &path_str, line_count, mtime).await?;
        }

        Ok(records)
    }
}

#[async_trait]
impl LogParser for ClaudeCodeParser {
    fn tool_name(&self) -> &'static str {
        "claude_code"
    }

    async fn parse(&self, pool: &SqlitePool, extra_paths: &[String]) -> Result<Vec<UsageRecord>> {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .context("Failed to get home directory")?;

        let mut all_bases: Vec<PathBuf> = vec![PathBuf::from(home).join(".claude/projects")];

        for p in extra_paths {
            let pb = PathBuf::from(p);
            if pb.exists() {
                all_bases.push(pb);
            }
        }

        let mut all_records = Vec::new();
        for base in all_bases {
            match self.parse_single_dir(pool, &base).await {
                Ok(recs) => all_records.extend(recs),
                Err(e) => eprintln!("Failed to parse claude_code dir {:?}: {}", base, e),
            }
        }

        Ok(all_records)
    }
}
