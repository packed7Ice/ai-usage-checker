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
