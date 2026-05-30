# AI CLI Usage Tracker — 開発コンテキスト

## プロジェクト概要

Claude Code・opencode・Gemini CLI の **5時間ブロック別／週間トークン使用量** を可視化する  
Windows / Zorin OS 対応のトレイ常駐型デスクトップアプリケーション。

---

## 技術スタック

| レイヤー | 技術 |
|----------|------|
| フレームワーク | **Tauri v2**（最新安定版: v2.10.x） |
| バックエンド | **Rust**（edition 2021） |
| フロントエンド | **TypeScript + React**（または Svelte） |
| ビルドツール | **Vite** |
| ローカルDB | **SQLite**（`sqlx` crate、WALモード） |
| スタイリング | Tailwind CSS v4 |
| チャート | **Recharts**（または D3.js） |

---

## 対応OS

- **Windows 10/11**（WebView2 必須）
- **Zorin OS 17+**（Ubuntu 22.04 ベース、WebKitGTK）

---

## アプリの基本挙動

### 起動時

```
アプリ起動
  └─ メインウィンドウ非表示（visible: false）
  └─ システムトレイアイコンとして常駐
       ├─ Windows: タスクバー通知領域
       └─ Zorin OS: システムステータスエリア（右クリックメニュー対応）
```

### トレイアイコン操作

| 操作 | 動作 |
|------|------|
| 左クリック | ウィンドウを生成（または表示）＋最新データをロード |
| ウィンドウを閉じる | UIを非表示（またはDestroyして）メモリ解放 |
| 右クリック | コンテキストメニュー（「開く」「終了」） |

> **Linux 注意点**: Tauri v2 の左クリックイベントは Linux 未対応。  
> 右クリックメニューの「開く」でウィンドウ表示を代替する。

### ウィンドウライフサイクル

```rust
// トレイクリック時の推奨パターン（Rust側）
TrayIconEvent::Click { .. } => {
    match app.get_webview_window("main") {
        Some(win) => {
            // 既存ウィンドウを前面に出す
            win.show().unwrap();
            win.set_focus().unwrap();
        }
        None => {
            // ウィンドウを新規作成してデータロード
            let win = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("/".into()))
                .title("AI CLI Usage Tracker")
                .inner_size(900.0, 600.0)
                .visible(true)
                .build()
                .unwrap();
        }
    }
}
```

---

## データソース（ローカルファイル読み取り）

各 CLI ツールのローカルログを**ポーリング（起動時 + 定期）**で読み取る。  
外部API通信なし、完全ローカル動作。

### Claude Code

| OS | パス |
|----|------|
| Windows | `%USERPROFILE%\.claude\projects\**\*.jsonl` |
| Linux | `~/.claude/projects/**/*.jsonl` |

- フォーマット: **JSONL**（1行1イベント）
- トークン情報: `usage.input_tokens`, `usage.output_tokens`, `usage.cache_read_input_tokens` フィールド
- タイムスタンプ: `timestamp` フィールド（ISO 8601）

```jsonc
// JSONL 1行のサンプル（usage抜粋）
{
  "type": "assistant",
  "timestamp": "2025-06-01T10:23:45.000Z",
  "usage": {
    "input_tokens": 1234,
    "output_tokens": 567,
    "cache_read_input_tokens": 890
  }
}
```

### opencode

| OS | パス |
|----|------|
| Windows | `%LOCALAPPDATA%\opencode\opencode.db`（v1.2+）|
| Linux | `~/.local/share/opencode/opencode.db`（v1.2+）|
| Linux (旧) | `~/.local/share/opencode/storage/message/` |

- フォーマット: **SQLite DB**（v1.2+）または JSONL（旧）
- v1.2+ は SQLite を直接クエリ可能（`rusqlite` で読み取り）
- テーブル例: `message`テーブルに `tokens_in`, `tokens_out`, `created_at` カラム想定

```sql
-- opencode DB からトークン集計（参考クエリ）
SELECT
  date(created_at, 'unixepoch') AS day,
  SUM(tokens_in)  AS input_tokens,
  SUM(tokens_out) AS output_tokens
FROM message
GROUP BY day
ORDER BY day DESC;
```

### Gemini CLI

| OS | パス |
|----|------|
| Windows | `%USERPROFILE%\.gemini\tmp\*\chats\*.json` |
| Linux | `~/.gemini/tmp/*/chats/*.json` |

- フォーマット: **JSON**（チャットセッション単位）
- トークン情報: `usageMetadata.promptTokenCount`, `usageMetadata.candidatesTokenCount`
- 環境変数 `GEMINI_CLI_HOME` または `GEMINI_DATA_DIR` でパス上書き可能

---

## ローカルデータベース設計（SQLite）

アプリ内 DB はパース済みデータのキャッシュ・集計用。  
パス: `{tauri::api::path::BaseDirectory::AppData}/usage_tracker.db`

### スキーマ

```sql
-- マイグレーション: 001_initial.sql
CREATE TABLE IF NOT EXISTS usage_records (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    tool          TEXT    NOT NULL,  -- 'claude_code' | 'opencode' | 'gemini'
    session_id    TEXT,
    recorded_at   INTEGER NOT NULL,  -- Unix timestamp (秒)
    input_tokens  INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    cache_tokens  INTEGER NOT NULL DEFAULT 0,
    cost_usd      REAL    NOT NULL DEFAULT 0.0
);

CREATE INDEX IF NOT EXISTS idx_usage_tool_time
    ON usage_records (tool, recorded_at);

-- 最終パース位置を記録（差分読み取り用）
CREATE TABLE IF NOT EXISTS parse_state (
    source_path   TEXT    PRIMARY KEY,
    last_offset   INTEGER NOT NULL DEFAULT 0,
    last_mtime    INTEGER NOT NULL DEFAULT 0
);
```

### Rust側の実装方針

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri          = { version = "2", features = ["tray-icon", "devtools"] }
sqlx           = { version = "0.8", features = ["runtime-tokio", "sqlite", "macros"] }
serde          = { version = "1", features = ["derive"] }
serde_json     = "1"
tokio          = { version = "1", features = ["full"] }
chrono         = { version = "0.4", features = ["serde"] }
glob           = "0.3"
rusqlite       = { version = "0.31", features = ["bundled"] }  # opencode DB読み取り用
```

---

## Rustバックエンド設計

### ディレクトリ構成

```
src-tauri/
├── src/
│   ├── main.rs          # エントリポイント・Tauriセットアップ
│   ├── lib.rs           # コマンド登録
│   ├── db.rs            # SQLite接続プール・マイグレーション
│   ├── tray.rs          # トレイアイコン・イベント処理
│   ├── parsers/
│   │   ├── mod.rs
│   │   ├── claude_code.rs   # JSONL パーサー
│   │   ├── opencode.rs      # SQLite / JSONL パーサー
│   │   └── gemini.rs        # JSON パーサー
│   ├── aggregator.rs    # 5時間ブロック・週間集計ロジック
│   └── commands.rs      # Tauri IPC コマンド
├── migrations/
│   └── 001_initial.sql
└── Cargo.toml
```

### Tauri コマンド（IPC）

```rust
// フロントエンドから呼び出す Tauri コマンド

/// 最新の使用量データを返す（トレイクリック時に呼ばれる）
#[tauri::command]
async fn get_usage_summary(
    state: tauri::State<'_, AppState>,
) -> Result<UsageSummary, String>

/// 5時間ブロック別データを返す
#[tauri::command]
async fn get_five_hour_blocks(
    state: tauri::State<'_, AppState>,
    tool: String,        // "all" | "claude_code" | "opencode" | "gemini"
    days: u32,           // 取得日数（デフォルト 7）
) -> Result<Vec<FiveHourBlock>, String>

/// 週間サマリーを返す
#[tauri::command]
async fn get_weekly_summary(
    state: tauri::State<'_, AppState>,
) -> Result<WeeklySummary, String>

/// ログを再スキャンする（手動更新）
#[tauri::command]
async fn refresh_data(
    state: tauri::State<'_, AppState>,
) -> Result<(), String>
```

### データ型（Rust → TypeScript 共通）

```rust
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FiveHourBlock {
    pub tool: String,
    pub block_start: i64,   // Unix timestamp
    pub block_end:   i64,
    pub input_tokens:  u64,
    pub output_tokens: u64,
    pub cache_tokens:  u64,
    pub cost_usd: f64,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct WeeklySummary {
    pub week_start: i64,
    pub per_day: Vec<DaySummary>,
    pub total_input:  u64,
    pub total_output: u64,
    pub total_cost_usd: f64,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DaySummary {
    pub date: String,       // "YYYY-MM-DD"
    pub tool_breakdown: Vec<ToolUsage>,
}
```

---

## フロントエンド設計

### ディレクトリ構成

```
src/
├── App.tsx
├── main.tsx
├── components/
│   ├── FiveHourChart.tsx     # 5時間ブロック棒グラフ
│   ├── WeeklyChart.tsx       # 週間積み上げ棒グラフ
│   ├── ToolSelector.tsx      # ツール切り替えタブ
│   ├── SummaryCard.tsx       # 合計トークン・コスト表示
│   └── RefreshButton.tsx
├── hooks/
│   ├── useUsageData.ts       # Tauri コマンド呼び出し
│   └── useAutoRefresh.ts     # 定期更新
└── types/
    └── usage.ts              # 型定義（Rust側と一致）
```

### UIレイアウト

```
┌──────────────────────────────────────────────────┐
│  AI CLI Usage Tracker          [更新] [設定]      │
├──────────────────────────────────────────────────┤
│  [All] [Claude Code] [opencode] [Gemini]         │
├──────────────────────────────────────────────────┤
│  今週の合計                                       │
│  Input: 1,234,567 tokens  Output: 456,789 tokens │
│  推定コスト: $12.34                               │
├──────────────────────────────────────────────────┤
│  5時間ブロック別使用量（直近7日）                 │
│  ┌─────────────────────────────────────────────┐ │
│  │ [棒グラフ: 横軸=日付+時刻ブロック, 縦軸=Token]│ │
│  └─────────────────────────────────────────────┘ │
├──────────────────────────────────────────────────┤
│  週間推移（過去4週）                              │
│  ┌─────────────────────────────────────────────┐ │
│  │ [積み上げ棒グラフ: 横軸=週, ツール別色分け] │ │
│  └─────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

### Tauri コマンド呼び出し（TypeScript）

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { FiveHourBlock, WeeklySummary } from '../types/usage';

// 5時間ブロックデータ取得
const blocks = await invoke<FiveHourBlock[]>('get_five_hour_blocks', {
  tool: 'all',
  days: 7,
});

// 週間サマリー取得
const weekly = await invoke<WeeklySummary>('get_weekly_summary');

// 手動更新
await invoke('refresh_data');
```

---

## Tauri 設定

### tauri.conf.json（主要部分）

```json
{
  "app": {
    "windows": [],
    "trayIcon": {
      "iconPath": "icons/tray-icon.png",
      "iconAsTemplate": false,
      "menuOnLeftClick": false,
      "title": "AI Usage Tracker",
      "tooltip": "AI CLI Usage Tracker"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["msi", "deb"],
    "identifier": "com.example.ai-usage-tracker",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.ico"],
    "windows": {
      "webviewInstallMode": { "type": "downloadBootstrapper" }
    }
  },
  "security": {
    "csp": null
  }
}
```

### capabilities/default.json（必要な権限）

```json
{
  "identifier": "default",
  "description": "Default capability",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:path:default",
    "core:window:allow-create",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-close",
    "core:tray:default",
    "core:fs:allow-read-files"
  ]
}
```

---

## 5時間ブロック集計ロジック

Claude Code の課金は **5時間ごとにリセット**される仕様に合わせた集計。

```rust
/// タイムスタンプを5時間ブロックのインデックスに変換
pub fn to_five_hour_block(timestamp_secs: i64) -> i64 {
    // 5時間 = 18000秒
    (timestamp_secs / 18000) * 18000
}

/// 直近 N 日の 5時間ブロック別集計
pub async fn aggregate_five_hour_blocks(
    pool: &SqlitePool,
    tool: &str,
    days: u32,
) -> Result<Vec<FiveHourBlock>, sqlx::Error> {
    let since = chrono::Utc::now().timestamp() - (days as i64 * 86400);
    
    let rows = sqlx::query!(r#"
        SELECT
            tool,
            (recorded_at / 18000 * 18000) AS block_start,
            (recorded_at / 18000 * 18000 + 18000) AS block_end,
            SUM(input_tokens)  AS input_tokens,
            SUM(output_tokens) AS output_tokens,
            SUM(cache_tokens)  AS cache_tokens,
            SUM(cost_usd)      AS cost_usd
        FROM usage_records
        WHERE recorded_at >= ?
          AND (? = 'all' OR tool = ?)
        GROUP BY tool, block_start
        ORDER BY block_start DESC
    "#, since, tool, tool)
    .fetch_all(pool)
    .await?;
    
    Ok(rows.into_iter().map(|r| FiveHourBlock {
        tool: r.tool,
        block_start: r.block_start,
        block_end: r.block_end,
        input_tokens: r.input_tokens as u64,
        output_tokens: r.output_tokens as u64,
        cache_tokens: r.cache_tokens as u64,
        cost_usd: r.cost_usd,
    }).collect())
}
```

---

## OS別の実装注意点

### Windows

- トレイアイコンは **`.ico` 形式**が必要（16x16, 32x32, 48x48 を含むマルチサイズ ICO）
- ホームディレクトリは `%USERPROFILE%`（`std::env::var("USERPROFILE")`）
- Claude Code パス: `%USERPROFILE%\.claude\projects\`
- opencode パス: `%LOCALAPPDATA%\opencode\opencode.db`
- Gemini CLI パス: `%USERPROFILE%\.gemini\tmp\`
- ビルドターゲット: `msi`（または `nsis`）

### Zorin OS（Linux/GTK）

- トレイはシステムトレイ拡張（`AppIndicator` または `StatusNotifier`）経由
- 左クリックイベントは Tauri v2 Linux 未サポート → **右クリックメニューで代替**
- ホームディレクトリは `$HOME`（`std::env::var("HOME")`）
- Claude Code パス: `~/.claude/projects/`
- opencode パス: `~/.local/share/opencode/opencode.db`
- Gemini CLI パス: `~/.gemini/tmp/`
- ビルドターゲット: `deb`（AppImage も検討）
- 依存パッケージ: `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`

---

## ビルド・開発手順

```bash
# 依存インストール
pnpm install

# 開発サーバー起動
pnpm tauri dev

# Windows向けビルド（Windowsマシン上で実行）
pnpm tauri build --target x86_64-pc-windows-msvc

# Linux向けビルド（Zorin OS / Ubuntu上で実行）
pnpm tauri build --target x86_64-unknown-linux-gnu

# SQLx マイグレーションファイル作成
cd src-tauri
cargo sqlx migrate add initial
```

---

## 実装ロードマップ

### Phase 1: 基盤
- [ ] Tauri v2 プロジェクト初期化（`pnpm create tauri-app`）
- [ ] トレイアイコン常駐・ウィンドウ表示/非表示の実装
- [ ] SQLite DB セットアップ（sqlx + マイグレーション）
- [ ] Claude Code JSONL パーサー実装

### Phase 2: データ収集
- [ ] opencode SQLite パーサー実装
- [ ] Gemini CLI JSON パーサー実装
- [ ] 差分パース（`parse_state` テーブル活用）
- [ ] 定期バックグラウンドスキャン（起動時 + 30分ごと）

### Phase 3: 可視化
- [ ] 5時間ブロック棒グラフ（Recharts）
- [ ] 週間積み上げグラフ
- [ ] ツール別色分け表示
- [ ] サマリーカード（合計トークン・推定コスト）

### Phase 4: 仕上げ
- [ ] Windows MSI パッケージング
- [ ] Zorin OS .deb パッケージング
- [ ] 自動起動設定（オプション）
- [ ] 設定画面（カスタムパス、コスト単価設定）

---

## 参考リポジトリ・ドキュメント

- Tauri v2 公式: https://v2.tauri.app/
- Tauri System Tray: https://v2.tauri.app/learn/system-tray/
- Tauri SQL Plugin: https://v2.tauri.app/plugin/sql/
- ccusage（参考実装）: https://github.com/ryoppippi/ccusage
- tokscale（データソースパス参考）: https://github.com/junhoyeo/tokscale
- sqlx + Tauri 実装例: https://tauritutorials.com/blog/building-a-todo-app-in-tauri-with-sqlite-and-sqlx
