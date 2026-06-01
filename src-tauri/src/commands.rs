use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::State;

#[derive(Serialize, Deserialize)]
pub struct FiveHourBlock {
    pub tool: String,
    pub block_start: i64,
    pub block_end: i64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_tokens: u64,
    pub cost_usd: f64,
}

#[derive(Serialize, Deserialize)]
pub struct WeeklySummary {
    pub week_start: i64,
    pub per_day: Vec<DaySummary>,
    pub total_input: u64,
    pub total_output: u64,
    pub total_cost_usd: f64,
}

#[derive(Serialize, Deserialize)]
pub struct DaySummary {
    pub date: String,
    pub tool_breakdown: Vec<ToolUsage>,
}

#[derive(Serialize, Deserialize)]
pub struct ToolUsage {
    pub tool: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_tokens: u64,
    pub cost_usd: f64,
}

#[derive(Serialize, Deserialize)]
pub struct UsageSummary {
    pub total_input: u64,
    pub total_output: u64,
    pub total_cache: u64,
    pub total_cost_usd: f64,
}

#[tauri::command]
pub async fn get_usage_summary(
    state: State<'_, crate::AppState>,
) -> Result<UsageSummary, String> {
    let row = sqlx::query(
        r#"
        SELECT
            COALESCE(SUM(input_tokens), 0) as total_input,
            COALESCE(SUM(output_tokens), 0) as total_output,
            COALESCE(SUM(cache_tokens), 0) as total_cache,
            COALESCE(SUM(cost_usd), 0.0) as total_cost_usd
        FROM usage_records
        "#,
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(UsageSummary {
        total_input: row.try_get::<i64, _>("total_input").unwrap_or(0) as u64,
        total_output: row.try_get::<i64, _>("total_output").unwrap_or(0) as u64,
        total_cache: row.try_get::<i64, _>("total_cache").unwrap_or(0) as u64,
        total_cost_usd: row.try_get::<f64, _>("total_cost_usd").unwrap_or(0.0),
    })
}

#[tauri::command]
pub async fn get_five_hour_blocks(
    state: State<'_, crate::AppState>,
    tool: String,
    days: u32,
) -> Result<Vec<FiveHourBlock>, String> {
    let since = chrono::Utc::now().timestamp() - (days as i64 * 86400);

    let rows = sqlx::query(
        r#"
        SELECT
            tool,
            (recorded_at / 18000 * 18000) as block_start,
            SUM(input_tokens) as input_tokens,
            SUM(output_tokens) as output_tokens,
            SUM(cache_tokens) as cache_tokens,
            SUM(cost_usd) as cost_usd
        FROM usage_records
        WHERE recorded_at >= ?1
          AND (?2 = 'all' OR tool = ?2)
        GROUP BY tool, block_start
        ORDER BY block_start DESC
        "#,
    )
    .bind(since)
    .bind(tool)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| FiveHourBlock {
            tool: r.try_get("tool").unwrap_or_default(),
            block_start: r.try_get("block_start").unwrap_or(0),
            block_end: r.try_get::<i64, _>("block_start").unwrap_or(0) + 18000,
            input_tokens: r.try_get::<i64, _>("input_tokens").unwrap_or(0) as u64,
            output_tokens: r.try_get::<i64, _>("output_tokens").unwrap_or(0) as u64,
            cache_tokens: r.try_get::<i64, _>("cache_tokens").unwrap_or(0) as u64,
            cost_usd: r.try_get::<f64, _>("cost_usd").unwrap_or(0.0),
        })
        .collect())
}

#[tauri::command]
pub async fn get_weekly_summary(
    state: State<'_, crate::AppState>,
) -> Result<WeeklySummary, String> {
    let since = chrono::Utc::now().timestamp() - (28 * 86400);

    let rows = sqlx::query(
        r#"
        SELECT
            date(recorded_at, 'unixepoch') as day,
            tool,
            SUM(input_tokens) as input_tokens,
            SUM(output_tokens) as output_tokens,
            SUM(cache_tokens) as cache_tokens,
            SUM(cost_usd) as cost_usd
        FROM usage_records
        WHERE recorded_at >= ?1
        GROUP BY day, tool
        ORDER BY day DESC
        "#,
    )
    .bind(since)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut days: std::collections::BTreeMap<String, Vec<ToolUsage>> = std::collections::BTreeMap::new();
    let mut total_input = 0u64;
    let mut total_output = 0u64;
    let mut total_cost = 0.0f64;

    for r in rows {
        let day: String = r.try_get("day").unwrap_or_default();
        let usage = ToolUsage {
            tool: r.try_get("tool").unwrap_or_default(),
            input_tokens: r.try_get::<i64, _>("input_tokens").unwrap_or(0) as u64,
            output_tokens: r.try_get::<i64, _>("output_tokens").unwrap_or(0) as u64,
            cache_tokens: r.try_get::<i64, _>("cache_tokens").unwrap_or(0) as u64,
            cost_usd: r.try_get::<f64, _>("cost_usd").unwrap_or(0.0),
        };
        total_input += usage.input_tokens;
        total_output += usage.output_tokens;
        total_cost += usage.cost_usd;
        days.entry(day).or_default().push(usage);
    }

    let per_day: Vec<DaySummary> = days
        .into_iter()
        .map(|(date, tool_breakdown)| DaySummary {
            date,
            tool_breakdown,
        })
        .collect();

    let week_start = chrono::Utc::now().timestamp() - (28 * 86400);

    Ok(WeeklySummary {
        week_start,
        per_day,
        total_input,
        total_output,
        total_cost_usd: total_cost,
    })
}

#[tauri::command]
pub async fn refresh_data(
    state: State<'_, crate::AppState>,
) -> Result<(), String> {
    crate::scanner::refresh_all(&state.db_pool)
        .await
        .map_err(|e| e.to_string())
}

// Settings commands

#[derive(Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub claude_code_path: String,
    pub opencode_path: String,
    pub gemini_path: String,
    pub input_cost_per_1k: String,
    pub output_cost_per_1k: String,
    pub auto_start: bool,
}

#[tauri::command]
pub async fn get_settings(
    state: State<'_, crate::AppState>,
) -> Result<AppSettings, String> {
    let rows = sqlx::query("SELECT key, value FROM app_settings")
        .fetch_all(&state.db_pool)
        .await
        .map_err(|e| e.to_string())?;

    let mut settings = AppSettings::default();
    for r in rows {
        let key: String = r.try_get("key").unwrap_or_default();
        let value: String = r.try_get("value").unwrap_or_default();
        match key.as_str() {
            "claude_code_path" => settings.claude_code_path = value,
            "opencode_path" => settings.opencode_path = value,
            "gemini_path" => settings.gemini_path = value,
            "input_cost_per_1k" => settings.input_cost_per_1k = value,
            "output_cost_per_1k" => settings.output_cost_per_1k = value,
            "auto_start" => settings.auto_start = value == "true",
            _ => {}
        }
    }
    Ok(settings)
}

#[tauri::command]
pub async fn set_setting(
    state: State<'_, crate::AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    sqlx::query("INSERT INTO app_settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value")
        .bind(&key)
        .bind(&value)
        .execute(&state.db_pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn set_settings(
    state: State<'_, crate::AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    let pairs = [
        ("claude_code_path", settings.claude_code_path),
        ("opencode_path", settings.opencode_path),
        ("gemini_path", settings.gemini_path),
        ("input_cost_per_1k", settings.input_cost_per_1k),
        ("output_cost_per_1k", settings.output_cost_per_1k),
        ("auto_start", if settings.auto_start { "true" } else { "false" }.to_string()),
    ];

    for (key, value) in &pairs {
        sqlx::query("INSERT INTO app_settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value")
            .bind(key)
            .bind(value)
            .execute(&state.db_pool)
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
