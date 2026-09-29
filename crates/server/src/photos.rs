//! รูปถ่ายบ่อ: ถ่ายจากกล้องในแอป ประทับเวลา+พิกัดลงบนรูปตั้งแต่ในเครื่อง แล้วเก็บเป็น BLOB พร้อมข้อมูลเวลา/พิกัดแยกไว้ค้นหา
//! รูปแรกที่มีพิกัดแม่นพอจะกลายเป็นตำแหน่งของบ่อ และรูปแรกของบ่อจะเป็นรูปหน้าปก

use axum::{
    extract::{Path, Query, State},
    http::header,
    response::{IntoResponse, Response},
    Json,
};
use base64::Engine;
use serde_json::{json, Value};
use sqlx::Row;

use crate::{
    api::{assert_farm_access, farm_of_pond},
    auth::AuthUser,
    db::{new_id, now_iso, rows_to_json},
    error::{ApiResult, AppError},
    AppState,
};

const MAX_IMAGE_BYTES: usize = 3 * 1024 * 1024;
const MAX_THUMB_BYTES: usize = 200 * 1024;
const MAX_PHOTOS_PER_POND: i64 = 300;
/// พิกัดที่คลาดเคลื่อนเกินนี้ไม่ใช้เป็นตำแหน่งบ่อ
const POND_LOCATION_MAX_ERROR_M: f64 = 100.0;

const META_COLS: &str = "id, pond_id, farm_id, taken_at, lat, lng, accuracy_m, kind, caption, width, height, mime, length(image) AS size_bytes, created_by, created_at";

fn text(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(|x| x.as_str()).map(|x| x.trim().to_string()).filter(|x| !x.is_empty())
}
fn num(v: &Value, k: &str) -> Option<f64> {
    v.get(k).and_then(|x| x.as_f64()).filter(|x| x.is_finite())
}

fn decode(b64: &str, max: usize, label: &str) -> ApiResult<Vec<u8>> {
    let raw = b64.split_once(',').map(|(_, d)| d).unwrap_or(b64);
    let bytes = base64::engine::general_purpose::STANDARD.decode(raw.trim()).map_err(|_| AppError::BadRequest(format!("{label}เสีย อ่านไม่ได้")))?;
    if bytes.len() > max {
        return Err(AppError::BadRequest(format!("{label}ใหญ่เกินไป")));
    }
    Ok(bytes)
}

/// ยอมรับเฉพาะไฟล์ที่เป็นรูปจริง (ดูจากไบต์หัวไฟล์ ไม่เชื่อชนิดที่ส่งมา)
fn sniff_mime(b: &[u8]) -> Option<&'static str> {
    if b.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if b.starts_with(&[0x89, b'P', b'N', b'G']) {
        Some("image/png")
    } else if b.len() > 12 && &b[0..4] == b"RIFF" && &b[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

async fn pond_of_photo(st: &AppState, id: &str) -> ApiResult<(String, String)> {
    let r = sqlx::query("SELECT pond_id, farm_id FROM pond_photos WHERE id = ?").bind(id).fetch_optional(&st.db).await?.ok_or(AppError::NotFound)?;
    Ok((r.get("pond_id"), r.get("farm_id")))
}

pub async fn list(State(st): State<AppState>, user: AuthUser, Path(pond_id): Path<String>) -> ApiResult<Json<Value>> {
    let farm_id = farm_of_pond(&st, &pond_id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    let rows = sqlx::query(&format!("SELECT {META_COLS} FROM pond_photos WHERE pond_id = ? ORDER BY taken_at DESC")).bind(&pond_id).fetch_all(&st.db).await?;
    Ok(Json(json!(rows_to_json(&rows))))
}

pub async fn create(State(st): State<AppState>, user: AuthUser, Path(pond_id): Path<String>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    let farm_id = farm_of_pond(&st, &pond_id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;

    let client_id = text(&b, "client_id");
    if let Some(cid) = &client_id {
        if let Some(r) = sqlx::query("SELECT id FROM pond_photos WHERE client_id = ?").bind(cid).fetch_optional(&st.db).await? {
            return Ok(Json(json!({ "id": r.get::<String, _>("id"), "duplicate": true })));
        }
    }
    let n: i64 = sqlx::query("SELECT COUNT(*) AS n FROM pond_photos WHERE pond_id = ?").bind(&pond_id).fetch_one(&st.db).await?.get("n");
    if n >= MAX_PHOTOS_PER_POND {
        return Err(AppError::BadRequest(format!("บ่อนี้มีรูปครบ {MAX_PHOTOS_PER_POND} รูปแล้ว ลบรูปเก่าที่ไม่ใช้ก่อน")));
    }

    let image = decode(&text(&b, "image").ok_or_else(|| AppError::BadRequest("ไม่มีรูป".into()))?, MAX_IMAGE_BYTES, "รูป")?;
    let mime = sniff_mime(&image).ok_or_else(|| AppError::BadRequest("ไฟล์นี้ไม่ใช่รูปภาพ".into()))?;
    let thumb = match text(&b, "thumb") {
        Some(t) => {
            let bytes = decode(&t, MAX_THUMB_BYTES, "รูปย่อ")?;
            sniff_mime(&bytes).ok_or_else(|| AppError::BadRequest("รูปย่อไม่ใช่รูปภาพ".into()))?;
            Some(bytes)
        }
        None => None,
    };

    // เก็บเป็น UTC ทั้งหมด เรียงตามเวลาได้ถูกแม้เครื่องที่ส่งมาตั้งโซนเวลาต่างกัน
    let taken_at = text(&b, "taken_at")
        .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
        .map(|t| t.with_timezone(&chrono::Utc).to_rfc3339())
        .unwrap_or_else(now_iso);
    let lat = num(&b, "lat").filter(|v| (-90.0..=90.0).contains(v));
    let lng = num(&b, "lng").filter(|v| (-180.0..=180.0).contains(v));
    let (lat, lng) = if lat.is_some() && lng.is_some() { (lat, lng) } else { (None, None) };
    let acc = num(&b, "accuracy_m").filter(|v| *v >= 0.0);

    let id = new_id();
    let mut tx = st.db.begin().await?;
    sqlx::query("INSERT INTO pond_photos (id, client_id, pond_id, farm_id, taken_at, lat, lng, accuracy_m, kind, caption, width, height, mime, image, thumb, created_by, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(&client_id)
        .bind(&pond_id)
        .bind(&farm_id)
        .bind(&taken_at)
        .bind(lat)
        .bind(lng)
        .bind(acc)
        .bind(text(&b, "kind"))
        .bind(text(&b, "caption").map(|c| c.chars().take(200).collect::<String>()))
        .bind(num(&b, "width").map(|v| v as i64))
        .bind(num(&b, "height").map(|v| v as i64))
        .bind(mime)
        .bind(&image)
        .bind(&thumb)
        .bind(&user.id)
        .bind(now_iso())
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE ponds SET cover_photo_id = ? WHERE id = ? AND cover_photo_id IS NULL").bind(&id).bind(&pond_id).execute(&mut *tx).await?;
    let mut located = false;
    if let (Some(la), Some(ln)) = (lat, lng) {
        if acc.map_or(true, |a| a <= POND_LOCATION_MAX_ERROR_M) {
            let r = sqlx::query("UPDATE ponds SET lat = ?, lng = ?, location_accuracy_m = ? WHERE id = ? AND lat IS NULL")
                .bind(la)
                .bind(ln)
                .bind(acc)
                .bind(&pond_id)
                .execute(&mut *tx)
                .await?;
            located = r.rows_affected() > 0;
        }
    }
    tx.commit().await?;
    Ok(Json(json!({ "id": id, "pond_located": located })))
}

pub async fn image(State(st): State<AppState>, user: AuthUser, Path(id): Path<String>, Query(q): Query<Value>) -> ApiResult<Response> {
    let (_, farm_id) = pond_of_photo(&st, &id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    let want_thumb = q.get("thumb").and_then(|v| v.as_str()) == Some("1");
    let r = sqlx::query("SELECT mime, image, thumb FROM pond_photos WHERE id = ?").bind(&id).fetch_one(&st.db).await?;
    let full: Vec<u8> = r.get("image");
    let thumb: Option<Vec<u8>> = r.get("thumb");
    let bytes = if want_thumb { thumb.unwrap_or(full) } else { full };
    let mime = sniff_mime(&bytes).unwrap_or("image/jpeg");
    Ok(([(header::CONTENT_TYPE, mime), (header::CACHE_CONTROL, "private, max-age=31536000, immutable")], bytes).into_response())
}

pub async fn update(State(st): State<AppState>, user: AuthUser, Path(id): Path<String>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    let (pond_id, farm_id) = pond_of_photo(&st, &id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    if b.get("caption").is_some() || b.get("kind").is_some() {
        sqlx::query("UPDATE pond_photos SET caption = CASE WHEN ? THEN ? ELSE caption END, kind = CASE WHEN ? THEN ? ELSE kind END WHERE id = ?")
            .bind(b.get("caption").is_some())
            .bind(text(&b, "caption"))
            .bind(b.get("kind").is_some())
            .bind(text(&b, "kind"))
            .bind(&id)
            .execute(&st.db)
            .await?;
    }
    if b.get("cover").and_then(|v| v.as_bool()) == Some(true) {
        sqlx::query("UPDATE ponds SET cover_photo_id = ? WHERE id = ?").bind(&id).bind(&pond_id).execute(&st.db).await?;
    }
    if b.get("use_location").and_then(|v| v.as_bool()) == Some(true) {
        let r = sqlx::query("SELECT lat, lng, accuracy_m FROM pond_photos WHERE id = ?").bind(&id).fetch_one(&st.db).await?;
        let (la, ln): (Option<f64>, Option<f64>) = (r.get("lat"), r.get("lng"));
        if la.is_none() || ln.is_none() {
            return Err(AppError::BadRequest("รูปนี้ไม่มีพิกัด".into()));
        }
        sqlx::query("UPDATE ponds SET lat = ?, lng = ?, location_accuracy_m = ? WHERE id = ?")
            .bind(la)
            .bind(ln)
            .bind(r.get::<Option<f64>, _>("accuracy_m"))
            .bind(&pond_id)
            .execute(&st.db)
            .await?;
    }
    Ok(Json(json!({ "ok": true })))
}

pub async fn remove(State(st): State<AppState>, user: AuthUser, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    let (pond_id, farm_id) = pond_of_photo(&st, &id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    let mut tx = st.db.begin().await?;
    sqlx::query("DELETE FROM pond_photos WHERE id = ?").bind(&id).execute(&mut *tx).await?;
    // รูปปกถูกลบ ใช้รูปล่าสุดที่เหลือแทน
    sqlx::query("UPDATE ponds SET cover_photo_id = (SELECT id FROM pond_photos WHERE pond_id = ? ORDER BY taken_at DESC LIMIT 1) WHERE id = ? AND cover_photo_id = ?")
        .bind(&pond_id)
        .bind(&pond_id)
        .bind(&id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    let _ = sqlx::query("INSERT INTO audit_log (user_id, action, entity, entity_id, detail_json, at) VALUES (?, 'delete', 'pond_photo', ?, NULL, ?)").bind(&user.id).bind(&id).bind(now_iso()).execute(&st.db).await;
    Ok(Json(json!({ "ok": true })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_real_images_pass() {
        assert_eq!(sniff_mime(&[0xFF, 0xD8, 0xFF, 0xE0, 0, 0]), Some("image/jpeg"));
        assert_eq!(sniff_mime(b"<svg onload=alert(1)>"), None);
        assert_eq!(sniff_mime(b"RIFF\0\0\0\0WEBPVP8 "), Some("image/webp"));
    }

    #[test]
    fn data_url_prefix_is_stripped() {
        let b = decode("data:image/jpeg;base64,/9j/", 10, "รูป").unwrap();
        assert_eq!(b, vec![0xFF, 0xD8, 0xFF]);
        assert!(decode("/9j/4AAQ", 2, "รูป").is_err());
    }
}
