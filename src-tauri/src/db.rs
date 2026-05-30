use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::str::FromStr;
use tauri::{AppHandle, Manager};

/// アプリ内 SQLite データベースへの接続プールを初期化する
pub async fn init_db(app_handle: &AppHandle) -> Result<SqlitePool, sqlx::Error> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");

    std::fs::create_dir_all(&app_data_dir).ok();

    let db_path = app_data_dir.join("usage_tracker.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.to_string_lossy());

    let options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = SqlitePool::connect_with(options).await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}
