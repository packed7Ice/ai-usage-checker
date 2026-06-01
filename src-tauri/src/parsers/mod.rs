pub mod claude_code;
pub mod gemini;
pub mod opencode;

use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

/// パース結果として返す1レコード
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsageRecord {
    pub tool: String,
    pub session_id: Option<String>,
    pub recorded_at: i64, // Unix timestamp (seconds)
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_tokens: u64,
    pub cost_usd: f64,
}

/// parse_state テーブルから最終読み取り状態を取得
pub async fn get_parse_state(
    pool: &SqlitePool,
    source_path: &str,
) -> anyhow::Result<(i64, i64)> {
    let row = sqlx::query("SELECT last_offset, last_mtime FROM parse_state WHERE source_path = ?1")
        .bind(source_path)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(r) => {
            let offset: i64 = r.try_get("last_offset").unwrap_or(0);
            let mtime: i64 = r.try_get("last_mtime").unwrap_or(0);
            Ok((offset, mtime))
        }
        None => Ok((0, 0)),
    }
}

/// parse_state テーブルを更新
pub async fn set_parse_state(
    pool: &SqlitePool,
    source_path: &str,
    last_offset: i64,
    last_mtime: i64,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO parse_state (source_path, last_offset, last_mtime) VALUES (?1, ?2, ?3) ON CONFLICT(source_path) DO UPDATE SET last_offset = excluded.last_offset, last_mtime = excluded.last_mtime"
    )
    .bind(source_path)
    .bind(last_offset)
    .bind(last_mtime)
    .execute(pool)
    .await?;
    Ok(())
}

/// 各パーサーが実装するトレイト
#[async_trait::async_trait]
pub trait LogParser: Send + Sync {
    /// このパーサーが対象とするツール名
    fn tool_name(&self) -> &'static str;

    /// ログファイルをスキャンし、新しいレコードを返す
    /// pool を通じて parse_state テーブルにアクセスし、差分読み取りを行う
    async fn parse(&self, pool: &SqlitePool) -> anyhow::Result<Vec<UsageRecord>>;
}
