mod commands;
mod db;
mod parsers;
mod scanner;

use std::fs::OpenOptions;
use std::io::Write;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager, RunEvent, WebviewWindowBuilder, WebviewUrl,
};

pub struct AppState {
    pub db_pool: sqlx::SqlitePool,
}

/// 簡易ログ出力（Windows Release ビルドでは標準エラー出力が見えないためファイルに書き出す）
fn write_log(msg: &str) {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        let log_dir = std::path::PathBuf::from(local_app_data).join("ai-usage-checker/logs");
        let _ = std::fs::create_dir_all(&log_dir);
        let log_file = log_dir.join("error.log");
        let mut file = match OpenOptions::new().create(true).append(true).open(&log_file) {
            Ok(f) => f,
            Err(_) => return,
        };
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(file, "[{}] {}", timestamp, msg);
    }
    // フォールバック: 標準エラー出力
    eprintln!("{}", msg);
}

/// メインウィンドウを表示する共通ロジック
fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        let _ = WebviewWindowBuilder::new(
            app,
            "main",
            WebviewUrl::App("/".into()),
        )
        .title("AI Usage Checker")
        .inner_size(960.0, 640.0)
        .min_inner_size(960.0, 640.0)
        .build();
    }
}

fn hide_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            // データベース初期化（エラーはログ出力して続行）
            let pool = match tauri::async_runtime::block_on(db::init_db(app.handle())) {
                Ok(pool) => pool,
                Err(e) => {
                    let msg = format!("Failed to initialize database: {}", e);
                    write_log(&msg);
                    return Err(msg.into());
                }
            };
            app.manage(AppState { db_pool: pool.clone() });

            // 起動時に一度スキャン
            let pool_clone = pool.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = scanner::refresh_all(&pool_clone).await {
                    write_log(&format!("Initial scan failed: {}", e));
                }
            });

            // 30分ごとの定期スキャン
            let pool_clone = pool.clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(30 * 60));
                loop {
                    interval.tick().await;
                    if let Err(e) = scanner::refresh_all(&pool_clone).await {
                        write_log(&format!("Periodic scan failed: {}", e));
                    }
                }
            });

            // トレイメニュー作成
            let menu = match (|| -> Result<_, Box<dyn std::error::Error>> {
                let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
                let open_i = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&open_i, &quit_i])?;
                Ok(menu)
            })() {
                Ok(menu) => menu,
                Err(e) => {
                    write_log(&format!("Failed to create tray menu: {}", e));
                    return Err(e);
                }
            };

            // トレイアイコン構築
            match TrayIconBuilder::with_id("main")
                .tooltip("AI CLI Usage Tracker")
                .icon_as_template(false)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "open" => show_main_window(&app),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                        show_main_window(tray.app_handle());
                    }
                })
                .build(app)
            {
                Ok(_) => {}
                Err(e) => {
                    write_log(&format!("Failed to build tray icon: {}", e));
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_usage_summary,
            commands::get_five_hour_blocks,
            commands::get_weekly_summary,
            commands::refresh_data,
            commands::get_settings,
            commands::set_setting,
            commands::set_settings,
        ])
        .build(tauri::generate_context!());

    let app = match app {
        Ok(app) => app,
        Err(e) => {
            write_log(&format!("Failed to build Tauri application: {}", e));
            std::process::exit(1);
        }
    };

    app.run(|_app_handle, event| match event {
        RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
            hide_main_window(_app_handle);
        }
        _ => {}
    });
}
