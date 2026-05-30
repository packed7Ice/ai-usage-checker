pub mod claude_code;
pub mod gemini;
pub mod opencode;

use serde::{Deserialize, Serialize};

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

/// 各パーサーが実装するトレイト
#[async_trait::async_trait]
pub trait LogParser: Send + Sync {
    /// このパーサーが対象とするツール名
    fn tool_name(&self) -> &'static str;

    /// ログファイルをスキャンし、新しいレコードを返す
    /// `parse_state` テーブルとの連携は呼び出し側で行う
    async fn parse(&self) -> anyhow::Result<Vec<UsageRecord>>;
}
