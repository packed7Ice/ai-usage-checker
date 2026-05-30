mod commands;
mod db;
mod parsers;
mod scanner;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager, WebviewWindowBuilder, WebviewUrl,
};

pub struct AppState {
    pub db_pool: sqlx::SqlitePool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--flag1", "--flag2"]),
        ))
        .setup(|app| {
            // データベース初期化
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            let pool = rt.block_on(db::init_db(app.handle()))
                .expect("Failed to initialize database");
            app.manage(AppState { db_pool: pool.clone() });

            // 起動時に一度スキャン（軽量のため非同期で実行）
            let pool_clone = pool.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = scanner::refresh_all(&pool_clone).await {
                    eprintln!("Initial scan failed: {}", e);
                }
            });

            // 30分ごとの定期スキャン
            let pool_clone = pool.clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(30 * 60));
                loop {
                    interval.tick().await;
                    if let Err(e) = scanner::refresh_all(&pool_clone).await {
                        eprintln!("Periodic scan failed: {}", e);
                    }
                }
            });

            // トレイメニュー作成
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let open_i = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_i, &quit_i])?;

            // トレイアイコン構築
            let _tray = TrayIconBuilder::with_id("main")
                .tooltip("AI CLI Usage Tracker")
                .icon_as_template(false)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "open" => {
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
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button, .. } = event {
                        if button == MouseButton::Left {
                            let app = tray.app_handle();
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
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_usage_summary,
            commands::get_five_hour_blocks,
            commands::get_weekly_summary,
            commands::refresh_data,
            commands::get_settings,
            commands::set_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
