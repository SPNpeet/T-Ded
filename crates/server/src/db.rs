use serde_json::{json, Map, Value};
use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteRow}, Column, Row, SqlitePool, TypeInfo, ValueRef};
use std::str::FromStr;

pub type Db = SqlitePool;

pub async fn connect(url: &str) -> anyhow_lite::Result<Db> {
    let opts = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(10))
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new().max_connections(8).connect_with(opts).await?;
    migrate(&pool).await?;
    Ok(pool)
}

/// migration แบบเรียงลำดับ เก็บเวอร์ชันในตาราง schema_version — เพิ่มไฟล์ใหม่ต่อท้ายเสมอ ห้ามแก้ของเก่า
const MIGRATIONS: &[(&str, &str)] = &[
    ("0001_init", include_str!("../migrations/0001_init.sql")),
    ("0002_feed_nutrition", include_str!("../migrations/0002_feed_nutrition.sql")),
    ("0003_feed_products", include_str!("../migrations/0003_feed_products.sql")),
];

async fn migrate(pool: &Db) -> anyhow_lite::Result<()> {
    sqlx::query("CREATE TABLE IF NOT EXISTS schema_version (name TEXT PRIMARY KEY, applied_at TEXT NOT NULL)")
        .execute(pool)
        .await?;
    for (name, sql) in MIGRATIONS {
        let done: Option<(String,)> = sqlx::query_as("SELECT name FROM schema_version WHERE name = ?")
            .bind(name)
            .fetch_optional(pool)
            .await?;
        if done.is_some() {
            continue;
        }
        let mut tx = pool.begin().await?;
        for stmt in split_sql(sql) {
            sqlx::query(&stmt).execute(&mut *tx).await.map_err(|e| anyhow_lite::Error(format!("migration {name}: {e}\n{stmt}")))?;
        }
        sqlx::query("INSERT INTO schema_version (name, applied_at) VALUES (?, ?)")
            .bind(name)
            .bind(chrono::Utc::now().to_rfc3339())
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        tracing::info!(migration = name, "applied");
    }
    Ok(())
}

fn split_sql(sql: &str) -> Vec<String> {
    sql.split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && !s.starts_with("--"))
        .map(|s| s.to_string())
        .collect()
}

/// แปลงแถว SQLite เป็น JSON ตามชนิดคอลัมน์ (ใช้แทนการประกาศ struct ทุกตาราง)
pub fn row_to_json(row: &SqliteRow) -> Value {
    let mut m = Map::new();
    for col in row.columns() {
        let name = col.name().to_string();
        let idx = col.ordinal();
        let raw = row.try_get_raw(idx);
        let v = match raw {
            Ok(r) if r.is_null() => Value::Null,
            Ok(r) => match r.type_info().name() {
                "INTEGER" | "INT" | "BIGINT" => row.try_get::<i64, _>(idx).map(Value::from).unwrap_or(Value::Null),
                "REAL" | "FLOAT" | "DOUBLE" | "NUMERIC" => row.try_get::<f64, _>(idx).map(|f| json!(f)).unwrap_or(Value::Null),
                "BOOLEAN" | "BOOL" => row.try_get::<bool, _>(idx).map(Value::from).unwrap_or(Value::Null),
                _ => row
                    .try_get::<String, _>(idx)
                    .map(|s| {
                        // คอลัมน์ *_json เก็บ JSON string ให้คืนเป็น object
                        if name.ends_with("_json") {
                            serde_json::from_str(&s).unwrap_or(Value::String(s))
                        } else {
                            Value::String(s)
                        }
                    })
                    .or_else(|_| row.try_get::<f64, _>(idx).map(|f| json!(f)))
                    .or_else(|_| row.try_get::<i64, _>(idx).map(Value::from))
                    .unwrap_or(Value::Null),
            },
            Err(_) => Value::Null,
        };
        m.insert(name, v);
    }
    Value::Object(m)
}

pub fn rows_to_json(rows: &[SqliteRow]) -> Vec<Value> {
    rows.iter().map(row_to_json).collect()
}

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// วันที่วันนี้ตามเวลาไทย (UTC+7) รูปแบบ YYYY-MM-DD
pub fn today_bkk() -> String {
    let tz = chrono::FixedOffset::east_opt(7 * 3600).unwrap();
    chrono::Utc::now().with_timezone(&tz).format("%Y-%m-%d").to_string()
}

pub mod anyhow_lite {
    #[derive(Debug)]
    pub struct Error(pub String);
    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.0)
        }
    }
    impl std::error::Error for Error {}
    impl From<sqlx::Error> for Error {
        fn from(e: sqlx::Error) -> Self {
            Error(e.to_string())
        }
    }
    pub type Result<T> = std::result::Result<T, Error>;
}
