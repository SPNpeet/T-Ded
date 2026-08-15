//! ค่าตั้งระบบที่แก้ได้จากหน้าแอดมิน (เก็บใน DB) เช่น LINE token — ถ้าไม่มีใน DB จะใช้ค่าจาก env

use axum::{extract::State, Json};
use serde_json::{json, Value};
use sqlx::Row;

use crate::{auth::AuthUser, db::now_iso, error::{ApiResult, AppError}, AppState};

pub const LINE_SECRET: &str = "line_channel_secret";
pub const LINE_TOKEN: &str = "line_channel_access_token";
pub const LINE_ADD_FRIEND: &str = "line_add_friend_url";
pub const LINE_ENABLED: &str = "line_enabled";

pub async fn get(st: &AppState, key: &str) -> Option<String> {
    let row = sqlx::query("SELECT value FROM app_settings WHERE key = ?").bind(key).fetch_optional(&st.db).await.ok()??;
    let v: Option<String> = row.get("value");
    v.filter(|s| !s.trim().is_empty())
}

/// ค่าจาก DB ก่อน ถ้าไม่มีใช้ env
pub async fn line_secret(st: &AppState) -> Option<String> {
    get(st, LINE_SECRET).await.or_else(|| st.cfg.line_secret.clone())
}
pub async fn line_token(st: &AppState) -> Option<String> {
    get(st, LINE_TOKEN).await.or_else(|| st.cfg.line_token.clone())
}
pub async fn line_add_friend(st: &AppState) -> Option<String> {
    get(st, LINE_ADD_FRIEND).await.or_else(|| st.cfg.line_add_friend_url.clone())
}

async fn set(st: &AppState, key: &str, value: Option<&str>, user: &str) -> ApiResult<()> {
    sqlx::query("INSERT INTO app_settings (key, value, updated_by, updated_at) VALUES (?, ?, ?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_by = excluded.updated_by, updated_at = excluded.updated_at")
        .bind(key)
        .bind(value)
        .bind(user)
        .bind(now_iso())
        .execute(&st.db)
        .await?;
    Ok(())
}

fn mask(v: &Option<String>) -> Value {
    match v {
        Some(s) if s.len() > 8 => json!(format!("{}...{}", &s[..4], &s[s.len() - 4..])),
        Some(_) => json!("****"),
        None => Value::Null,
    }
}

/// อ่านสถานะ LINE (ไม่ส่ง token เต็มกลับไป)
pub async fn line_status(State(st): State<AppState>, user: AuthUser) -> ApiResult<Json<Value>> {
    if !user.is_staff() {
        return Err(AppError::Forbidden);
    }
    let secret = line_secret(&st).await;
    let token = line_token(&st).await;
    let from_db = get(&st, LINE_TOKEN).await.is_some();
    let linked: i64 = sqlx::query("SELECT COUNT(*) AS n FROM users WHERE line_user_id IS NOT NULL AND org_id = ?").bind(&user.org_id).fetch_one(&st.db).await?.get("n");
    let sent: i64 = sqlx::query("SELECT COUNT(*) AS n FROM notifications WHERE channel = 'line' AND sent_at IS NOT NULL").fetch_one(&st.db).await?.get("n");
    let base = std::env::var("PUBLIC_BASE_URL").unwrap_or_default();
    Ok(Json(json!({
        "configured": secret.is_some() && token.is_some(),
        "has_secret": secret.is_some(),
        "has_token": token.is_some(),
        "secret_masked": mask(&secret),
        "token_masked": mask(&token),
        "from_db": from_db,
        "add_friend_url": line_add_friend(&st).await,
        "webhook_url": if base.is_empty() { Value::Null } else { json!(format!("{}/api/line/webhook", base.trim_end_matches('/'))) },
        "linked_users": linked,
        "messages_sent": sent,
    })))
}

pub async fn line_save(State(st): State<AppState>, user: AuthUser, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    if !user.is_staff() {
        return Err(AppError::Forbidden);
    }
    let s = |k: &str| b.get(k).and_then(|v| v.as_str()).map(|x| x.trim().to_string());
    if let Some(v) = s("channel_secret") {
        set(&st, LINE_SECRET, if v.is_empty() { None } else { Some(&v) }, &user.id).await?;
    }
    if let Some(v) = s("channel_access_token") {
        set(&st, LINE_TOKEN, if v.is_empty() { None } else { Some(&v) }, &user.id).await?;
    }
    if let Some(v) = s("add_friend_url") {
        set(&st, LINE_ADD_FRIEND, if v.is_empty() { None } else { Some(&v) }, &user.id).await?;
    }
    let _ = sqlx::query("INSERT INTO audit_log (user_id, action, entity, entity_id, at) VALUES (?, 'update', 'settings', 'line', ?)").bind(&user.id).bind(now_iso()).execute(&st.db).await;
    Ok(Json(json!({ "ok": true })))
}

/// ทดสอบส่งข้อความหาตัวเอง (ต้องผูก LINE แล้ว)
pub async fn line_test(State(st): State<AppState>, user: AuthUser) -> ApiResult<Json<Value>> {
    if !user.is_staff() {
        return Err(AppError::Forbidden);
    }
    let row = sqlx::query("SELECT line_user_id FROM users WHERE id = ?").bind(&user.id).fetch_one(&st.db).await?;
    let lid: Option<String> = row.get("line_user_id");
    let Some(lid) = lid else { return Err(AppError::BadRequest("บัญชีนี้ยังไม่ได้เชื่อม LINE (ไปที่ ตั้งค่า > เชื่อม LINE)".into())) };
    crate::line::push_text(&st, &lid, "ทดสอบจากระบบทีเด็ดปลาน้ำจืด: ตั้งค่า LINE สำเร็จแล้ว").await?;
    Ok(Json(json!({ "ok": true })))
}
