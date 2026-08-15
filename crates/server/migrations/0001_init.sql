CREATE TABLE orgs (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE users (
  id TEXT PRIMARY KEY,
  org_id TEXT NOT NULL REFERENCES orgs(id),
  phone TEXT NOT NULL UNIQUE,
  pin_hash TEXT NOT NULL,
  name TEXT NOT NULL,
  role TEXT NOT NULL CHECK (role IN ('owner','worker','officer','admin')),
  line_user_id TEXT,
  line_link_code TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE sessions (
  token TEXT PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL,
  device TEXT
);

CREATE TABLE farms (
  id TEXT PRIMARY KEY,
  org_id TEXT NOT NULL REFERENCES orgs(id),
  name TEXT NOT NULL,
  province TEXT,
  district TEXT,
  lat REAL,
  lng REAL,
  meals_per_day INTEGER NOT NULL DEFAULT 2,
  farm_factor REAL NOT NULL DEFAULT 1.0,
  bag_kg REAL NOT NULL DEFAULT 20,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE farm_members (
  farm_id TEXT NOT NULL REFERENCES farms(id) ON DELETE CASCADE,
  user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  role TEXT NOT NULL,
  PRIMARY KEY (farm_id, user_id)
);

CREATE TABLE ponds (
  id TEXT PRIMARY KEY,
  farm_id TEXT NOT NULL REFERENCES farms(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  pond_type TEXT NOT NULL DEFAULT 'earthen',
  area_rai REAL,
  area_m2 REAL,
  depth_m REAL,
  sort_order INTEGER NOT NULL DEFAULT 0,
  active INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL
);

CREATE TABLE crops (
  id TEXT PRIMARY KEY,
  pond_id TEXT NOT NULL REFERENCES ponds(id) ON DELETE CASCADE,
  farm_id TEXT NOT NULL REFERENCES farms(id) ON DELETE CASCADE,
  species_code TEXT NOT NULL,
  stocked_at TEXT NOT NULL,
  stocked_count INTEGER NOT NULL,
  stock_weight_g REAL NOT NULL,
  fry_price_each REAL NOT NULL DEFAULT 0,
  target_weight_g REAL,
  target_harvest_at TEXT,
  status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','closed')),
  closed_at TEXT,
  note TEXT,
  created_at TEXT NOT NULL
);
CREATE INDEX idx_crops_pond ON crops(pond_id, status);

CREATE TABLE daily_logs (
  id TEXT PRIMARY KEY,
  client_id TEXT UNIQUE,
  crop_id TEXT NOT NULL REFERENCES crops(id) ON DELETE CASCADE,
  log_date TEXT NOT NULL,
  recommended_kg REAL,
  fed_kg REAL,
  factor REAL,
  mortality INTEGER NOT NULL DEFAULT 0,
  feeding_response INTEGER NOT NULL DEFAULT 0,
  weather_json TEXT,
  reasons_json TEXT,
  note TEXT,
  photo_url TEXT,
  created_by TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (crop_id, log_date)
);

CREATE TABLE weighings (
  id TEXT PRIMARY KEY,
  client_id TEXT UNIQUE,
  crop_id TEXT NOT NULL REFERENCES crops(id) ON DELETE CASCADE,
  weigh_date TEXT NOT NULL,
  sample_count INTEGER,
  avg_weight_g REAL NOT NULL,
  method TEXT,
  note TEXT,
  created_at TEXT NOT NULL
);
CREATE INDEX idx_weighings_crop ON weighings(crop_id, weigh_date);

CREATE TABLE water_quality (
  id TEXT PRIMARY KEY,
  client_id TEXT UNIQUE,
  pond_id TEXT NOT NULL REFERENCES ponds(id) ON DELETE CASCADE,
  crop_id TEXT,
  measured_at TEXT NOT NULL,
  do_mg_l REAL,
  ph REAL,
  temp_c REAL,
  nh3 REAL,
  no2 REAL,
  secchi_cm REAL,
  color TEXT,
  note TEXT,
  created_at TEXT NOT NULL
);
CREATE INDEX idx_water_pond ON water_quality(pond_id, measured_at);

CREATE TABLE feed_stock_moves (
  id TEXT PRIMARY KEY,
  client_id TEXT UNIQUE,
  farm_id TEXT NOT NULL REFERENCES farms(id) ON DELETE CASCADE,
  move_date TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('in','out','adjust')),
  brand TEXT,
  pellet_mm REAL,
  bags REAL,
  kg REAL NOT NULL,
  price_total REAL,
  crop_id TEXT,
  ref_log_id TEXT,
  note TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE expenses (
  id TEXT PRIMARY KEY,
  client_id TEXT UNIQUE,
  crop_id TEXT NOT NULL REFERENCES crops(id) ON DELETE CASCADE,
  expense_date TEXT NOT NULL,
  category TEXT NOT NULL,
  amount REAL NOT NULL,
  note TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE harvests (
  id TEXT PRIMARY KEY,
  client_id TEXT UNIQUE,
  crop_id TEXT NOT NULL REFERENCES crops(id) ON DELETE CASCADE,
  harvest_date TEXT NOT NULL,
  count INTEGER,
  kg REAL NOT NULL,
  price_per_kg REAL,
  buyer TEXT,
  note TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE treatments (
  id TEXT PRIMARY KEY,
  client_id TEXT UNIQUE,
  crop_id TEXT NOT NULL REFERENCES crops(id) ON DELETE CASCADE,
  start_date TEXT NOT NULL,
  end_date TEXT,
  product TEXT NOT NULL,
  dose TEXT,
  withdrawal_days INTEGER NOT NULL DEFAULT 0,
  symptom TEXT,
  note TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE weather_cache (
  key TEXT PRIMARY KEY,
  payload_json TEXT NOT NULL,
  fetched_at TEXT NOT NULL
);

CREATE TABLE market_prices (
  id TEXT PRIMARY KEY,
  species_code TEXT NOT NULL,
  province TEXT,
  price_per_kg REAL NOT NULL,
  size_note TEXT,
  source TEXT,
  reported_by TEXT,
  price_date TEXT NOT NULL,
  created_at TEXT NOT NULL
);
CREATE INDEX idx_prices ON market_prices(species_code, price_date);

CREATE TABLE disease_reports (
  id TEXT PRIMARY KEY,
  farm_id TEXT,
  province TEXT,
  district TEXT,
  lat REAL,
  lng REAL,
  species_code TEXT,
  symptom TEXT NOT NULL,
  severity TEXT NOT NULL DEFAULT 'medium',
  report_date TEXT NOT NULL,
  note TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE species_overrides (
  org_id TEXT NOT NULL,
  code TEXT NOT NULL,
  profile_json TEXT NOT NULL,
  updated_by TEXT,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (org_id, code)
);

CREATE TABLE rule_overrides (
  org_id TEXT PRIMARY KEY,
  rules_json TEXT NOT NULL,
  updated_by TEXT,
  updated_at TEXT NOT NULL
);

CREATE TABLE health_history (
  crop_id TEXT NOT NULL,
  score_date TEXT NOT NULL,
  score INTEGER NOT NULL,
  PRIMARY KEY (crop_id, score_date)
);

CREATE TABLE notifications (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  channel TEXT NOT NULL,
  title TEXT,
  body TEXT NOT NULL,
  sent_at TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE announcements (
  id TEXT PRIMARY KEY,
  org_id TEXT NOT NULL,
  title TEXT NOT NULL,
  body TEXT NOT NULL,
  created_by TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE audit_log (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id TEXT,
  action TEXT NOT NULL,
  entity TEXT,
  entity_id TEXT,
  detail_json TEXT,
  at TEXT NOT NULL
);
