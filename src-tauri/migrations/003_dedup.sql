-- マイグレーション: 003_dedup.sql
-- 重複データ蓄積を防ぐため source_hash 列を追加

ALTER TABLE usage_records ADD COLUMN source_hash TEXT;

-- 既存データに一意な source_hash を設定（重複を避けるため id を含める）
UPDATE usage_records SET source_hash = tool || ':' || recorded_at || ':' || id WHERE source_hash IS NULL;

-- source_hash で一意性を保証
CREATE UNIQUE INDEX IF NOT EXISTS idx_usage_source_hash ON usage_records(source_hash);
