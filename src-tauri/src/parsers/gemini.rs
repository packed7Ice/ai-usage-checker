use super::{LogParser, UsageRecord};
use anyhow::Result;
use async_trait::async_trait;

pub struct GeminiParser;

#[async_trait]
impl LogParser for GeminiParser {
    fn tool_name(&self) -> &'static str {
        "gemini"
    }

    async fn parse(&self) -> Result<Vec<UsageRecord>> {
        // TODO: Gemini CLI JSON パーサー実装
        Ok(vec![])
    }
}
