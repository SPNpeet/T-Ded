CREATE TABLE subscriptions (
  org_id TEXT PRIMARY KEY REFERENCES orgs(id) ON DELETE CASCADE,
  plan TEXT NOT NULL DEFAULT 'trial',
  status TEXT NOT NULL DEFAULT 'active',
  max_farms INTEGER NOT NULL DEFAULT 1,
  max_ponds INTEGER NOT NULL DEFAULT 5,
  max_members INTEGER NOT NULL DEFAULT 3,
  price_per_month REAL NOT NULL DEFAULT 0,
  started_at TEXT NOT NULL,
  expires_at TEXT,
  note TEXT,
  updated_by TEXT,
  updated_at TEXT NOT NULL
);

CREATE TABLE payments (
  id TEXT PRIMARY KEY,
  org_id TEXT NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
  amount REAL NOT NULL,
  method TEXT,
  reference TEXT,
  period_from TEXT,
  period_to TEXT,
  paid_at TEXT NOT NULL,
  recorded_by TEXT,
  note TEXT,
  created_at TEXT NOT NULL
);
CREATE INDEX idx_payments_org ON payments(org_id, paid_at);
