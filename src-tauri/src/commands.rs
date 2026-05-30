use serde::{Deserialize, Serialize};
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

// Tauri IPC コマンド

#[tauri::command]
pub async fn get_usage_summary(
    _state: State<'_, crate::AppState>,
) -> Result<UsageSummary, String> {
    // TODO: 実際の集計を実装
    Ok(UsageSummary {
        total_input: 0,
        total_output: 0,
        total_cache: 0,
        total_cost_usd: 0.0,
    })
}

#[tauri::command]
pub async fn get_five_hour_blocks(
    _state: State<'_, crate::AppState>,
    _tool: String,
    _days: u32,
) -> Result<Vec<FiveHourBlock>, String> {
    // TODO: 実際の集計を実装
    Ok(vec![])
}

#[tauri::command]
pub async fn get_weekly_summary(
    _state: State<'_, crate::AppState>,
) -> Result<WeeklySummary, String> {
    // TODO: 実際の集計を実装
    Ok(WeeklySummary {
        week_start: 0,
        per_day: vec![],
        total_input: 0,
        total_output: 0,
        total_cost_usd: 0.0,
    })
}

#[tauri::command]
pub async fn refresh_data(
    _state: State<'_, crate::AppState>,
) -> Result<(), String> {
    // TODO: ログ再スキャンを実装
    Ok(())
}
