CREATE TABLE feed_products (
  id TEXT PRIMARY KEY,
  org_id TEXT,
  brand TEXT NOT NULL,
  product_code TEXT,
  name_th TEXT NOT NULL,
  target TEXT NOT NULL,
  stage_th TEXT,
  weight_from_g REAL,
  weight_to_g REAL,
  protein_pct REAL,
  fat_pct REAL,
  pellet_mm REAL,
  form TEXT,
  bag_kg REAL,
  price_ref REAL,
  price_date TEXT,
  source_url TEXT,
  verified INTEGER NOT NULL DEFAULT 0,
  active INTEGER NOT NULL DEFAULT 1,
  note TEXT,
  updated_by TEXT,
  updated_at TEXT NOT NULL
);
CREATE INDEX idx_feed_products_target ON feed_products(target, active);
ALTER TABLE feed_stock_moves ADD COLUMN product_id TEXT;
