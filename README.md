# AI CLI Usage Tracker

Claude Code、opencode、Gemini CLI の **5時間ブロック別／週間トークン使用量** を可視化する、Windows / Zorin OS 対応のトレイ常駐型デスクトップアプリケーション。

---

## 対応ツール

| ツール | データソース |
|--------|-------------|
| Claude Code | `~/.claude/projects/**/*.jsonl` |
| opencode | `~/.local/share/opencode/opencode.db` |
| Gemini CLI | `~/.gemini/tmp/*/chats/*.json` |

各 CLI のローカルログをポーリングで読み取り、完全ローカル動作（外部API通信なし）。

---

## インストール

### Windows

1. [Releases](https://github.com/zutuu/ai-usage-checker/releases) から `ai-usage-checker_0.x.x_x64-setup.exe` をダウンロード
2. インストーラーを実行
3. タスクバーの通知領域にトレイアイコンが表示されます

### Zorin OS / Ubuntu

1. `.deb` パッケージをダウンロード
2. ターミナルでインストール

```bash
sudo dpkg -i ai-usage-checker_0.x.x_amd64.deb
sudo apt-get install -f   # 依存関係があれば自動解決
```

3. アプリケーションメニューから「AI Usage Checker」を起動

---

## 使い方

### 起動時

アプリ起動時、メインウィンドウは非表示で、**システムトレイアイコンとして常駐**します。

### トレイアイコン操作

| 操作 | 動作 |
|------|------|
| **左クリック**（Windows） | メインウィンドウを表示・データを自動更新 |
| **右クリック → Open**（Linux） | メインウィンドウを表示（Linux は左クリック未対応） |
| **右クリック → Quit** | アプリを終了 |

### メインウィンドウ

```
┌──────────────────────────────────────────────────┐
│  AI CLI Usage Tracker          [Refresh]          │
├──────────────────────────────────────────────────┤
│  [5-Hour Blocks] [Weekly Summary] [Settings]     │
├──────────────────────────────────────────────────┤
│  Usage Summary                                    │
│  Input: xxx,xxx  Output: xxx,xxx  Cache: xxx,xxx │
│  Est. Cost: $xx.xx                                │
├──────────────────────────────────────────────────┤
│  [チャート表示エリア]                              │
└──────────────────────────────────────────────────┘
```

#### タブ

- **5-Hour Blocks**: 直近7日間の5時間ブロック別使用量を棒グラフで表示
- **Weekly Summary**: 直近4週間のツール別積み上げ棒グラフ
- **Settings**: アプリ設定

### データ更新

- **自動更新**: 起動時と30分ごとにバックグラウンドでスキャン
- **手動更新**: 「Refresh」ボタンで即座に再スキャン

---

## 設定（Settings）

### カスタムパス

各 CLI のログ保存先がデフォルトと異なる場合、カスタムパスを指定できます。

| 設定項目 | デフォルト値（Windows） | デフォルト値（Linux） |
|----------|------------------------|----------------------|
| Claude Code Log Path | `%USERPROFILE%\.claude\projects` | `~/.claude/projects` |
| Opencode DB Path | `%LOCALAPPDATA%\opencode\opencode.db` | `~/.local/share/opencode/opencode.db` |
| Gemini CLI Log Path | `%USERPROFILE%\.gemini\tmp` | `~/.gemini/tmp` |

※ 空欄のままにするとデフォルトパスが使用されます。

### コスト単価設定

推定コスト計算に使用する単価を設定できます（1K tokens あたりの USD）。

| 項目 | デフォルト |
|------|-----------|
| Input Cost per 1K tokens | $0.003 |
| Output Cost per 1K tokens | $0.015 |

### 自動起動

「Auto-start on login」にチェックを入れると、OS ログイン時に自動でアプリが起動します。

---

## 技術スタック

| レイヤー | 技術 |
|----------|------|
| フレームワーク | Tauri v2 |
| バックエンド | Rust |
| フロントエンド | TypeScript + React |
| ビルドツール | Vite |
| ローカルDB | SQLite（WALモード） |
| チャート | Recharts |

---

## 開発

### 要件

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/)（npm 付き）

### セットアップ

```bash
npm install
```

### 開発サーバー起動

```bash
npm run tauri dev
```

### ビルド

```bash
# Windows
npm run tauri build -- --target x86_64-pc-windows-msvc

# Linux
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

---

## OS別の注意点

### Windows

- WebView2 が必要です（Windows 11 は標準搭載、Windows 10 はインストールが必要な場合あり）
- トレイアイコンは `.ico` 形式（16x16, 32x32, 48x48 マルチサイズ）

### Zorin OS（Linux）

- Tauri v2 の左クリックイベントは Linux で未サポートのため、**右クリックメニュー「Open」でウィンドウ表示**
- 依存パッケージ: `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`

---

## ライセンス

MIT
