use crate::parsers::{claude_code::ClaudeCodeParser, gemini::GeminiParser, opencode::OpencodeParser, LogParser};
use sqlx::{Row, SqlitePool};
use std::path::PathBuf;

/// 追加パス文字列を Vec<String> に変換（カンマまたは改行区切り）
fn split_paths(paths_str: &str) -> Vec<String> {
    paths_str
        .split(|c| c == ',' || c == '\n')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && PathBuf::from(s).exists())
        .collect()
}

/// 全パーサーを実行し、新規レコードを DB に保存する
pub async fn refresh_all(pool: &SqlitePool) -> anyhow::Result<()> {
    // 設定を読み込み
    let settings = match load_settings(pool).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load settings for refresh: {}", e);
            None
        }
    };

    let additional_claude = settings
        .as_ref()
        .map(|s| split_paths(&s.additional_claude_code_paths))
        .unwrap_or_default();
    let additional_opencode = settings
        .as_ref()
        .map(|s| split_paths(&s.additional_opencode_paths))
        .unwrap_or_default();
    let additional_gemini = settings
        .as_ref()
        .map(|s| split_paths(&s.additional_gemini_paths))
        .unwrap_or_default();

    // Claude Code
    match ClaudeCodeParser.parse(pool, &additional_claude).await {
        Ok(records) if !records.is_empty() => {
            insert_records(pool, &records).await?;
        }
        Ok(_) => {}
        Err(e) => eprintln!("Parser error for claude_code: {}", e),
    }

    // opencode
    match OpencodeParser.parse(pool, &additional_opencode).await {
        Ok(records) if !records.is_empty() => {
            insert_records(pool, &records).await?;
        }
        Ok(_) => {}
        Err(e) => eprintln!("Parser error for opencode: {}", e),
    }

    // Gemini
    match GeminiParser.parse(pool, &additional_gemini).await {
        Ok(records) if !records.is_empty() => {
            insert_records(pool, &records).await?;
        }
        Ok(_) => {}
        Err(e) => eprintln!("Parser error for gemini: {}", e),
    }

    Ok(())
}

async fn load_settings(pool: &SqlitePool) -> anyhow::Result<Option<crate::commands::AppSettings>> {
    let rows = sqlx::query("SELECT key, value FROM app_settings")
        .fetch_all(pool)
        .await?;

    let mut settings = crate::commands::AppSettings::default();
    for r in rows {
        let key: String = r.try_get("key").unwrap_or_default();
        let value: String = r.try_get("value").unwrap_or_default();
        match key.as_str() {
            "claude_code_path" => settings.claude_code_path = value,
            "opencode_path" => settings.opencode_path = value,
            "gemini_path" => settings.gemini_path = value,
            "additional_claude_code_paths" => settings.additional_claude_code_paths = value,
            "additional_opencode_paths" => settings.additional_opencode_paths = value,
            "additional_gemini_paths" => settings.additional_gemini_paths = value,
            "input_cost_per_1k" => settings.input_cost_per_1k = value,
            "output_cost_per_1k" => settings.output_cost_per_1k = value,
            "auto_start" => settings.auto_start = value == "true",
            _ => {}
        }
    }
    Ok(Some(settings))
}

async fn insert_records(pool: &SqlitePool, records: &[super::parsers::UsageRecord]) -> anyhow::Result<()> {
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
