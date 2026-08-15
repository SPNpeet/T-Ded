use argon2::{password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};
use axum::{extract::{FromRequestParts, State}, http::request::Parts, Json};
use base64::Engine;
use rand::RngCore;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;

use crate::{db::{new_id, now_iso, row_to_json}, error::{ApiResult, AppError}, AppState};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: String,
    pub org_id: String,
    pub role: String,
    pub name: String,
}

impl AuthUser {
    pub fn is_staff(&self) -> bool {
        self.role == "officer" || self.role == "admin"
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;
        let token = header.strip_prefix("Bearer ").ok_or(AppError::Unauthorized)?;
        let row = sqlx::query(
            "SELECT u.id, u.org_id, u.role, u.name FROM sessions s JOIN users u ON u.id = s.user_id WHERE s.token = ?",
        )
        .bind(token)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;
        let _ = sqlx::query("UPDATE sessions SET last_seen_at = ? WHERE token = ?")
            .bind(now_iso())
            .bind(token)
            .execute(&state.db)
            .await;
        Ok(AuthUser {
            id: row.get("id"),
            org_id: row.get("org_id"),
            role: row.get("role"),
            name: row.get("name"),
        })
    }
}

fn hash_pin(pin: &str) -> ApiResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(pin.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(e.to_string()))
}

fn verify_pin(pin: &str, hash: &str) -> bool {
    PasswordHash::new(hash)
        .map(|h| Argon2::default().verify_password(pin.as_bytes(), &h).is_ok())
        .unwrap_or(false)
}

fn new_token() -> String {
    let mut b = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut b);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b)
}

fn normalize_phone(p: &str) -> String {
    p.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn validate_pin(pin: &str) -> ApiResult<()> {
    if pin.len() < 4 || pin.len() > 8 || !pin.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::BadRequest("รหัส PIN ต้องเป็นตัวเลข 4-8 หลัก".into()));
    }
    Ok(())
}

#[derive(Deserialize)]
pub struct RegisterReq {
    pub farm_name: String,
    pub name: String,
    pub phone: String,
    pub pin: String,
    pub province: Option<String>,
    pub district: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    /// รหัสหน่วยงาน (ถ้าเจ้าหน้าที่ให้มา) เพื่อเข้ากลุ่มเดียวกัน
    pub org_code: Option<String>,
}

pub async fn register(State(st): State<AppState>, Json(req): Json<RegisterReq>) -> ApiResult<Json<Value>> {
    let phone = normalize_phone(&req.phone);
    if phone.len() < 9 {
        return Err(AppError::BadRequest("เบอร์โทรไม่ถูกต้อง".into()));
    }
    validate_pin(&req.pin)?;
    if req.farm_name.trim().is_empty() || req.name.trim().is_empty() {
        return Err(AppError::BadRequest("กรอกชื่อฟาร์มและชื่อผู้ใช้".into()));
    }
    let exists = sqlx::query("SELECT id FROM users WHERE phone = ?").bind(&phone).fetch_optional(&st.db).await?;
    if exists.is_some() {
        return Err(AppError::BadRequest("เบอร์นี้สมัครแล้ว กรุณาเข้าสู่ระบบ".into()));
    }

    let now = now_iso();
    let org_id = match req.org_code.as_deref().filter(|c| !c.trim().is_empty()) {
        Some(code) => {
            let r = sqlx::query("SELECT id FROM orgs WHERE id = ? OR name = ?").bind(code).bind(code).fetch_optional(&st.db).await?;
            match r {
                Some(r) => r.get::<String, _>("id"),
                None => return Err(AppError::BadRequest("ไม่พบรหัสหน่วยงาน".into())),
            }
        }
        None => {
            let id = new_id();
            sqlx::query("INSERT INTO orgs (id, name, created_at) VALUES (?, ?, ?)")
                .bind(&id)
                .bind(format!("{} (ส่วนตัว)", req.farm_name.trim()))
                .bind(&now)
                .execute(&st.db)
                .await?;
            id
        }
    };

    let user_id = new_id();
    sqlx::query("INSERT INTO users (id, org_id, phone, pin_hash, name, role, created_at) VALUES (?, ?, ?, ?, ?, 'owner', ?)")
        .bind(&user_id)
        .bind(&org_id)
        .bind(&phone)
        .bind(hash_pin(&req.pin)?)
        .bind(req.name.trim())
        .bind(&now)
        .execute(&st.db)
        .await?;

    let farm_id = new_id();
    sqlx::query(
        "INSERT INTO farms (id, org_id, name, province, district, lat, lng, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&farm_id)
    .bind(&org_id)
    .bind(req.farm_name.trim())
    .bind(&req.province)
    .bind(&req.district)
    .bind(req.lat)
    .bind(req.lng)
    .bind(&now)
    .bind(&now)
    .execute(&st.db)
    .await?;
    sqlx::query("INSERT INTO farm_members (farm_id, user_id, role) VALUES (?, ?, 'owner')")
        .bind(&farm_id)
        .bind(&user_id)
        .execute(&st.db)
        .await?;

    let token = issue_session(&st, &user_id, None).await?;
    Ok(Json(json!({ "token": token, "user": user_json(&st, &user_id).await?, "farm_id": farm_id })))
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub phone: String,
    pub pin: String,
    pub device: Option<String>,
}

pub async fn login(State(st): State<AppState>, Json(req): Json<LoginReq>) -> ApiResult<Json<Value>> {
    let phone = normalize_phone(&req.phone);
    let row = sqlx::query("SELECT id, pin_hash FROM users WHERE phone = ?").bind(&phone).fetch_optional(&st.db).await?;
    let Some(row) = row else { return Err(AppError::BadRequest("ไม่พบเบอร์นี้ หรือ PIN ไม่ถูกต้อง".into())) };
    let hash: String = row.get("pin_hash");
    if !verify_pin(&req.pin, &hash) {
        return Err(AppError::BadRequest("ไม่พบเบอร์นี้ หรือ PIN ไม่ถูกต้อง".into()));
    }
    let user_id: String = row.get("id");
    let token = issue_session(&st, &user_id, req.device.as_deref()).await?;
    Ok(Json(json!({ "token": token, "user": user_json(&st, &user_id).await? })))
}

pub async fn logout(State(st): State<AppState>, user: AuthUser, headers: axum::http::HeaderMap) -> ApiResult<Json<Value>> {
    if let Some(t) = headers.get("authorization").and_then(|v| v.to_str().ok()).and_then(|v| v.strip_prefix("Bearer ")) {
        sqlx::query("DELETE FROM sessions WHERE token = ? AND user_id = ?").bind(t).bind(&user.id).execute(&st.db).await?;
    }
    Ok(Json(json!({ "ok": true })))
}

pub async fn me(State(st): State<AppState>, user: AuthUser) -> ApiResult<Json<Value>> {
    Ok(Json(user_json(&st, &user.id).await?))
}

#[derive(Deserialize)]
pub struct ChangePinReq {
    pub old_pin: String,
    pub new_pin: String,
}

pub async fn change_pin(State(st): State<AppState>, user: AuthUser, Json(req): Json<ChangePinReq>) -> ApiResult<Json<Value>> {
    validate_pin(&req.new_pin)?;
    let row = sqlx::query("SELECT pin_hash FROM users WHERE id = ?").bind(&user.id).fetch_one(&st.db).await?;
    let hash: String = row.get("pin_hash");
    if !verify_pin(&req.old_pin, &hash) {
        return Err(AppError::BadRequest("PIN เดิมไม่ถูกต้อง".into()));
    }
    sqlx::query("UPDATE users SET pin_hash = ? WHERE id = ?").bind(hash_pin(&req.new_pin)?).bind(&user.id).execute(&st.db).await?;
    Ok(Json(json!({ "ok": true })))
}

async fn issue_session(st: &AppState, user_id: &str, device: Option<&str>) -> ApiResult<String> {
    let token = new_token();
    let now = now_iso();
    sqlx::query("INSERT INTO sessions (token, user_id, created_at, last_seen_at, device) VALUES (?, ?, ?, ?, ?)")
        .bind(&token)
        .bind(user_id)
        .bind(&now)
        .bind(&now)
        .bind(device)
        .execute(&st.db)
        .await?;
    Ok(token)
}

pub async fn user_json(st: &AppState, user_id: &str) -> ApiResult<Value> {
    let row = sqlx::query("SELECT u.id, u.org_id, u.phone, u.name, u.role, u.line_user_id IS NOT NULL AS line_linked, o.name AS org_name FROM users u JOIN orgs o ON o.id = u.org_id WHERE u.id = ?")
        .bind(user_id)
        .fetch_one(&st.db)
        .await?;
    let mut u = row_to_json(&row);
    let farms = sqlx::query(
        "SELECT f.* FROM farms f JOIN farm_members m ON m.farm_id = f.id WHERE m.user_id = ? ORDER BY f.created_at",
    )
    .bind(user_id)
    .fetch_all(&st.db)
    .await?;
    u["farms"] = Value::Array(crate::db::rows_to_json(&farms));
    Ok(u)
}

/// สร้างบัญชีเจ้าหน้าที่/แอดมิน (เฉพาะแอดมิน หรือ bootstrap ผ่าน env ADMIN_PHONE/ADMIN_PIN ตอนเริ่มระบบ)
pub async fn ensure_bootstrap_admin(st: &AppState) -> ApiResult<()> {
    let (Ok(phone), Ok(pin)) = (std::env::var("ADMIN_PHONE"), std::env::var("ADMIN_PIN")) else { return Ok(()) };
    let phone = normalize_phone(&phone);
    let exists = sqlx::query("SELECT id FROM users WHERE phone = ?").bind(&phone).fetch_optional(&st.db).await?;
    if exists.is_some() {
        return Ok(());
    }
    let now = now_iso();
    let org_id = "org_main".to_string();
    sqlx::query("INSERT OR IGNORE INTO orgs (id, name, created_at) VALUES (?, ?, ?)")
        .bind(&org_id)
        .bind(std::env::var("ORG_NAME").unwrap_or_else(|_| "หน่วยส่งเสริม ทีเด็ดปลาน้ำจืด".into()))
        .bind(&now)
        .execute(&st.db)
        .await?;
    sqlx::query("INSERT INTO users (id, org_id, phone, pin_hash, name, role, created_at) VALUES (?, ?, ?, ?, 'ผู้ดูแลระบบ', 'admin', ?)")
        .bind(new_id())
        .bind(&org_id)
        .bind(&phone)
        .bind(hash_pin(&pin)?)
        .bind(&now)
        .execute(&st.db)
        .await?;
    tracing::info!("bootstrap admin created");
    Ok(())
}

/// แอดมิน/เจ้าหน้าที่สร้างผู้ใช้ในองค์กรเดียวกัน (เจ้าหน้าที่คนใหม่ หรือคนงานฟาร์ม)
#[derive(Deserialize)]
pub struct CreateUserReq {
    pub name: String,
    pub phone: String,
    pub pin: String,
    pub role: String,
    pub farm_id: Option<String>,
}

pub async fn create_user(State(st): State<AppState>, user: AuthUser, Json(req): Json<CreateUserReq>) -> ApiResult<Json<Value>> {
    let allowed = match req.role.as_str() {
        "worker" => true,
        "officer" | "owner" => user.is_staff(),
        "admin" => user.role == "admin",
        _ => false,
    };
    if !allowed {
        return Err(AppError::Forbidden);
    }
    let phone = normalize_phone(&req.phone);
    validate_pin(&req.pin)?;
    if sqlx::query("SELECT id FROM users WHERE phone = ?").bind(&phone).fetch_optional(&st.db).await?.is_some() {
        return Err(AppError::BadRequest("เบอร์นี้มีผู้ใช้แล้ว".into()));
    }
    let id = new_id();
    sqlx::query("INSERT INTO users (id, org_id, phone, pin_hash, name, role, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(&user.org_id)
        .bind(&phone)
        .bind(hash_pin(&req.pin)?)
        .bind(req.name.trim())
        .bind(&req.role)
        .bind(now_iso())
        .execute(&st.db)
        .await?;
    if let Some(fid) = &req.farm_id {
        crate::api::assert_farm_access(&st, &user, fid).await?;
        sqlx::query("INSERT OR IGNORE INTO farm_members (farm_id, user_id, role) VALUES (?, ?, ?)")
            .bind(fid)
            .bind(&id)
            .bind(&req.role)
            .execute(&st.db)
            .await?;
    }
    Ok(Json(json!({ "id": id })))
}
