use super::{LogParser, UsageRecord};
use anyhow::Result;
use async_trait::async_trait;

pub struct OpencodeParser;

#[async_trait]
impl LogParser for OpencodeParser {
    fn tool_name(&self) -> &'static str {
        "opencode"
    }

    async fn parse(&self) -> Result<Vec<UsageRecord>> {
        // TODO: opencode SQLite/JSONL パーサー実装
        Ok(vec![])
    }
}
