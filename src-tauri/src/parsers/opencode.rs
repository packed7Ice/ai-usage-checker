use super::{get_parse_state, set_parse_state, LogParser, UsageRecord};
use anyhow::{Context, Result};
use async_trait::async_trait;
use rusqlite::{Connection, OpenFlags};
use sqlx::SqlitePool;
use std::path::PathBuf;

pub struct OpencodeParser;

#[async_trait]
impl LogParser for OpencodeParser {
    fn tool_name(&self) -> &'static str {
        "opencode"
    }

    async fn parse(&self, pool: &SqlitePool) -> Result<Vec<UsageRecord>> {
        let db_path = resolve_db_path()?;

        if !db_path.exists() {
            return Ok(vec![]);
        }

        // parse_state から最後に読み取った created_at の最大値を取得
        let (last_offset, _) = get_parse_state(pool, "opencode").await?;
        let last_created_at = last_offset; // opencode では offset を created_at として使用

        let records = {
            let conn = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                .with_context(|| format!("Failed to open opencode DB at {:?}", db_path))?;

            let mut stmt = conn
                .prepare(
                    "SELECT tokens_in, tokens_out, created_at FROM message WHERE created_at IS NOT NULL AND created_at > ?1 ORDER BY created_at ASC"
                )
                .with_context(|| "Failed to prepare opencode query. Schema may be incompatible.")?;

            let rows = stmt.query_map([last_created_at], |row| {
                Ok((
                    row.get::<_, Option<i64>>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                ))
            })?;

            let mut records = Vec::new();
            let mut max_created_at = last_created_at;

            for row in rows {
                let (tokens_in, tokens_out, created_at) = row?;
                if let (Some(tin), Some(tout), Some(ts)) = (tokens_in, tokens_out, created_at) {
                    if ts > max_created_at {
                        max_created_at = ts;
                    }
                    records.push((ts, tin, tout, max_created_at));
                }
            }

            records
        }; // conn と stmt をここで drop

        let mut result = Vec::new();
        let mut max_created_at = last_created_at;
        for (ts, tin, tout, max_ts) in &records {
            max_created_at = *max_ts;
            result.push(UsageRecord {
                tool: self.tool_name().to_string(),
                session_id: None,
                recorded_at: *ts,
                input_tokens: *tin as u64,
                output_tokens: *tout as u64,
                cache_tokens: 0,
                cost_usd: 0.0,
            });
        }

        // 新しいレコードがあれば parse_state を更新
        if !result.is_empty() {
            let meta = std::fs::metadata(&db_path)?;
            let mtime = meta.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;
            set_parse_state(pool, "opencode", max_created_at, mtime).await?;
        }

        Ok(result)
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
