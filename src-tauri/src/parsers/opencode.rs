use super::{LogParser, UsageRecord};
use anyhow::{Context, Result};
use async_trait::async_trait;
use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;

pub struct OpencodeParser;

#[async_trait]
impl LogParser for OpencodeParser {
    fn tool_name(&self) -> &'static str {
        "opencode"
    }

    async fn parse(&self) -> Result<Vec<UsageRecord>> {
        let db_path = resolve_db_path()?;

        if !db_path.exists() {
            // DB が存在しない場合は空の結果を返す（初回起動時など）
            return Ok(vec![]);
        }

        let conn = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .with_context(|| format!("Failed to open opencode DB at {:?}", db_path))?;

        let mut stmt = conn
            .prepare(
                "SELECT tokens_in, tokens_out, created_at FROM message WHERE created_at IS NOT NULL"
            )
            .with_context(|| "Failed to prepare opencode query. Schema may be incompatible.")?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, Option<i64>>(0)?,
                row.get::<_, Option<i64>>(1)?,
                row.get::<_, Option<i64>>(2)?,
            ))
        })?;

        let mut records = Vec::new();
        for row in rows {
            let (tokens_in, tokens_out, created_at) = row?;
            if let (Some(tin), Some(tout), Some(ts)) = (tokens_in, tokens_out, created_at) {
                records.push(UsageRecord {
                    tool: self.tool_name().to_string(),
                    session_id: None,
                    recorded_at: ts,
                    input_tokens: tin as u64,
                    output_tokens: tout as u64,
                    cache_tokens: 0,
                    cost_usd: 0.0,
                });
            }
        }

        Ok(records)
    }
}

fn resolve_db_path() -> Result<PathBuf> {
    if cfg!(target_os = "windows") {
        let local_app_data = std::env::var("LOCALAPPDATA")
            .context("LOCALAPPDATA not set")?;
        Ok(PathBuf::from(local_app_data).join("opencode/opencode.db"))
    } else {
        let home = std::env::var("HOME")
            .context("HOME not set")?;
        Ok(PathBuf::from(home).join(".local/share/opencode/opencode.db"))
    }
}
