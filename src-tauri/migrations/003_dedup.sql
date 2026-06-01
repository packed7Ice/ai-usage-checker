-- マイグレーション: 003_dedup.sql
-- 重複データ蓄積を防ぐため source_hash 列を追加

ALTER TABLE usage_records ADD COLUMN source_hash TEXT;

-- 既存データに簡易 source_hash を設定（tool + recorded_at の組み合わせ）
UPDATE usage_records SET source_hash = tool || ':' || recorded_at WHERE source_hash IS NULL;

-- source_hash で一意性を保証
CREATE UNIQUE INDEX idx_usage_source_hash ON usage_records(source_hash);
