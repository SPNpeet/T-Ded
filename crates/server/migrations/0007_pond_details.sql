ALTER TABLE ponds ADD COLUMN shape TEXT;
ALTER TABLE ponds ADD COLUMN width_m REAL;
ALTER TABLE ponds ADD COLUMN length_m REAL;
ALTER TABLE ponds ADD COLUMN diameter_m REAL;
ALTER TABLE ponds ADD COLUMN water_source TEXT;
ALTER TABLE ponds ADD COLUMN aeration TEXT;
ALTER TABLE ponds ADD COLUMN note TEXT;
ALTER TABLE ponds ADD COLUMN lat REAL;
ALTER TABLE ponds ADD COLUMN lng REAL;
ALTER TABLE ponds ADD COLUMN location_accuracy_m REAL;
ALTER TABLE ponds ADD COLUMN cover_photo_id TEXT;

CREATE TABLE pond_photos (
  id TEXT PRIMARY KEY,
  client_id TEXT UNIQUE,
  pond_id TEXT NOT NULL REFERENCES ponds(id) ON DELETE CASCADE,
  farm_id TEXT NOT NULL REFERENCES farms(id) ON DELETE CASCADE,
  taken_at TEXT NOT NULL,
  lat REAL,
  lng REAL,
  accuracy_m REAL,
  kind TEXT,
  caption TEXT,
  width INTEGER,
  height INTEGER,
  mime TEXT NOT NULL,
  image BLOB NOT NULL,
  thumb BLOB,
  created_by TEXT,
  created_at TEXT NOT NULL
);
CREATE INDEX idx_pond_photos_pond ON pond_photos(pond_id, taken_at);
