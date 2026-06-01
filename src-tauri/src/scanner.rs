use crate::parsers::{claude_code::ClaudeCodeParser, gemini::GeminiParser, opencode::OpencodeParser, LogParser, UsageRecord};
use sqlx::SqlitePool;

/// 全パーサーを実行し、新規レコードを DB に保存する
pub async fn refresh_all(pool: &SqlitePool) -> anyhow::Result<()> {
    let parsers: Vec<Box<dyn LogParser>> = vec![
        Box::new(ClaudeCodeParser),
        Box::new(OpencodeParser),
        Box::new(GeminiParser),
    ];

    for parser in parsers {
        match parser.parse().await {
            Ok(records) => {
                if !records.is_empty() {
                    insert_records(pool, &records).await?;
                }
            }
            Err(e) => {
                eprintln!("Parser error for {}: {}", parser.tool_name(), e);
            }
        }
    }

    Ok(())
}

async fn insert_records(pool: &SqlitePool, records: &[UsageRecord]) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;

    for r in records {
        let source_hash = format!("{}:{}:{}:{}", r.tool, r.recorded_at, r.input_tokens, r.output_tokens);
        sqlx::query(
            r#"
            INSERT INTO usage_records (tool, session_id, recorded_at, input_tokens, output_tokens, cache_tokens, cost_usd, source_hash)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(&r.tool)
        .bind(&r.session_id)
        .bind(r.recorded_at)
        .bind(r.input_tokens as i64)
        .bind(r.output_tokens as i64)
        .bind(r.cache_tokens as i64)
        .bind(r.cost_usd)
        .bind(source_hash)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
