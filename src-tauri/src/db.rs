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

    // マイグレーション実行（エラー時は手動修復を試みる）
    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        eprintln!("Migration failed, attempting manual repair: {}", e);
        if let Err(repair_err) = manual_repair(&pool).await {
            eprintln!("Manual repair also failed: {}", repair_err);
        }
    }

    Ok(pool)
}

/// マイグレーション失敗時の手動修復
async fn manual_repair(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // source_hash 列が存在しない場合は追加
    let _ = sqlx::query("ALTER TABLE usage_records ADD COLUMN source_hash TEXT")
        .execute(pool)
        .await;

    // 既存データに一意な source_hash を設定（id を含めて重複を回避）
    let _ = sqlx::query(
        "UPDATE usage_records SET source_hash = tool || ':' || recorded_at || ':' || id WHERE source_hash IS NULL"
    )
    .execute(pool)
    .await;

    // UNIQUE INDEX を作成（IF NOT EXISTS で重複作成を回避）
    let _ = sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_usage_source_hash ON usage_records(source_hash)"
    )
    .execute(pool)
    .await;

    // app_settings テーブルが存在しない場合は作成
    let _ = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )
        "#
    )
    .execute(pool)
    .await;

    // parse_state テーブルが存在しない場合は作成
    let _ = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS parse_state (
            source_path TEXT PRIMARY KEY,
            last_offset INTEGER NOT NULL DEFAULT 0,
            last_mtime INTEGER NOT NULL DEFAULT 0
        )
        "#
    )
    .execute(pool)
    .await;

    Ok(())
}
