use axum::{extract::{Query, State}, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;

use crate::{db::{now_iso, today_bkk}, error::{ApiResult, AppError}, AppState};

#[derive(Deserialize)]
pub struct WeatherQuery {
    pub lat: f64,
    pub lng: f64,
    pub date: Option<String>,
}

/// พยากรณ์/ข้อมูลย้อนหลังรายวันจาก Open-Meteo ผ่าน server (แก้ CORS/token และ cache ให้)
pub async fn get_weather(State(st): State<AppState>, Query(q): Query<WeatherQuery>) -> ApiResult<Json<Value>> {
    let date = q.date.unwrap_or_else(today_bkk);
    Ok(Json(fetch_daily(&st, q.lat, q.lng, &date).await?))
}

pub async fn fetch_daily(st: &AppState, lat: f64, lng: f64, date: &str) -> ApiResult<Value> {
    if date.len() != 10 {
        return Err(AppError::BadRequest("วันที่ต้องเป็น YYYY-MM-DD".into()));
    }
    let key = format!("{:.2},{:.2},{}", lat, lng, date);
    let today = today_bkk();
    // cache: วันย้อนหลังเก็บถาวร, วันนี้/อนาคตเก็บ 3 ชั่วโมง
    if let Some(row) = sqlx::query("SELECT payload_json, fetched_at FROM weather_cache WHERE key = ?").bind(&key).fetch_optional(&st.db).await? {
        let payload: String = row.get("payload_json");
        let fetched: String = row.get("fetched_at");
        let fresh = date.as_bytes() < today.as_bytes()
            || chrono::DateTime::parse_from_rfc3339(&fetched)
                .map(|t| chrono::Utc::now().signed_duration_since(t).num_minutes() < 180)
                .unwrap_or(false);
        if fresh {
            if let Ok(v) = serde_json::from_str::<Value>(&payload) {
                return Ok(v);
            }
        }
    }

    let is_past = date.as_bytes() < today.as_bytes();
    let base = if is_past { "https://archive-api.open-meteo.com/v1/archive" } else { "https://api.open-meteo.com/v1/forecast" };
    let url = format!(
        "{base}?latitude={lat}&longitude={lng}&daily=temperature_2m_max,temperature_2m_min,precipitation_sum,cloud_cover_mean,weather_code&timezone=Asia%2FBangkok&start_date={date}&end_date={date}"
    );
    let resp = st.http.get(&url).send().await?;
    if !resp.status().is_success() {
        return Err(AppError::Internal(format!("open-meteo HTTP {}", resp.status())));
    }
    let raw: Value = resp.json().await?;
    let daily = raw.get("daily").ok_or_else(|| AppError::Internal("open-meteo: no daily".into()))?;
    let first = |k: &str| daily.get(k).and_then(|a| a.get(0)).cloned().unwrap_or(Value::Null);
    let out = json!({
        "date": date,
        "lat": lat,
        "lng": lng,
        "source": if is_past { "open-meteo-archive" } else { "open-meteo-forecast" },
        "tmax_c": first("temperature_2m_max"),
        "tmin_c": first("temperature_2m_min"),
        "rain_mm": first("precipitation_sum"),
        "cloud_pct": first("cloud_cover_mean"),
        "weather_code": first("weather_code"),
        "fetched_at": now_iso(),
    });
    sqlx::query("INSERT INTO weather_cache (key, payload_json, fetched_at) VALUES (?, ?, ?) ON CONFLICT(key) DO UPDATE SET payload_json = excluded.payload_json, fetched_at = excluded.fetched_at")
        .bind(&key)
        .bind(out.to_string())
        .bind(now_iso())
        .execute(&st.db)
        .await?;
    Ok(out)
}

/// พยากรณ์ล่วงหน้า 7 วัน (สำหรับเตือนอากาศแปรปรวน)
#[derive(Deserialize)]
pub struct ForecastQuery {
    pub lat: f64,
    pub lng: f64,
    pub days: Option<u8>,
}

pub async fn get_forecast(State(st): State<AppState>, Query(q): Query<ForecastQuery>) -> ApiResult<Json<Value>> {
    let days = q.days.unwrap_or(7).clamp(1, 16);
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&daily=temperature_2m_max,temperature_2m_min,precipitation_sum,cloud_cover_mean,weather_code&timezone=Asia%2FBangkok&forecast_days={}",
        q.lat, q.lng, days
    );
    let raw: Value = st.http.get(&url).send().await?.json().await?;
    let daily = raw.get("daily").cloned().unwrap_or(json!({}));
    let n = daily.get("time").and_then(|t| t.as_array()).map(|a| a.len()).unwrap_or(0);
    let get = |k: &str, i: usize| daily.get(k).and_then(|a| a.get(i)).cloned().unwrap_or(Value::Null);
    let mut out = Vec::new();
    for i in 0..n {
        out.push(json!({
            "date": get("time", i),
            "tmax_c": get("temperature_2m_max", i),
            "tmin_c": get("temperature_2m_min", i),
            "rain_mm": get("precipitation_sum", i),
            "cloud_pct": get("cloud_cover_mean", i),
            "weather_code": get("weather_code", i),
        }));
    }
    Ok(Json(json!({ "days": out })))
}
