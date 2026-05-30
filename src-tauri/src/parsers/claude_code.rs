use super::{LogParser, UsageRecord};
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub struct ClaudeCodeParser;

#[async_trait]
impl LogParser for ClaudeCodeParser {
    fn tool_name(&self) -> &'static str {
        "claude_code"
    }

    async fn parse(&self) -> Result<Vec<UsageRecord>> {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .context("Failed to get home directory")?;

        let base = PathBuf::from(home).join(".claude/projects");
        let pattern = base.join("**/*.jsonl");
        let pattern_str = pattern.to_string_lossy().replace('\\', "/");

        let mut records = Vec::new();

        for entry in glob::glob(&pattern_str)? {
            let path = entry?;
            let file = File::open(&path)?;
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line = line?;
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
        }

        Ok(records)
    }
}
