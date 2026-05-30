-- マイグレーション: 002_settings.sql
CREATE TABLE IF NOT EXISTS app_settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- デフォルト値を挿入
INSERT OR IGNORE INTO app_settings (key, value) VALUES
    ('claude_code_path', ''),
    ('opencode_path', ''),
    ('gemini_path', ''),
    ('input_cost_per_1k', '0.003'),
    ('output_cost_per_1k', '0.015'),
    ('auto_start', 'false');
