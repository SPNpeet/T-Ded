// ผูกมาตรการความปลอดภัยเข้ากับ server (รันครั้งเดียวจาก web/)
import { readFileSync, writeFileSync } from 'node:fs'
const R = (p) => '../crates/server/' + p
const edit = (file, pairs) => {
  let c = readFileSync(R(file), 'utf8')
  for (const [a, b] of pairs) {
    if (!c.includes(a)) throw new Error(file + ' missing: ' + a.slice(0, 60))
    c = c.replace(a, b)
  }
  writeFileSync(R(file), c)
}

// 1) auth.rs — ล็อกเอาต์อัตโนมัติเมื่อผิดซ้ำ, PIN อ่อนห้ามใช้, session หมดอายุ, บันทึกล็อก
edit('src/auth.rs', [
  [
    'fn validate_pin(pin: &str) -> ApiResult<()> {\n    if pin.len() < 4 || pin.len() > 8 || !pin.chars().all(|c| c.is_ascii_digit()) {\n        return Err(AppError::BadRequest("รหัส PIN ต้องเป็นตัวเลข 4-8 หลัก".into()));\n    }\n    Ok(())\n}',
    'fn validate_pin(pin: &str) -> ApiResult<()> {\n    if pin.len() < 4 || pin.len() > 8 || !pin.chars().all(|c| c.is_ascii_digit()) {\n        return Err(AppError::BadRequest("รหัส PIN ต้องเป็นตัวเลข 4-8 หลัก".into()));\n    }\n    if crate::security::is_weak_pin(pin) {\n        return Err(AppError::BadRequest("PIN นี้เดาง่ายเกินไป (เช่น 1234, 0000, เลขซ้ำ) กรุณาตั้งใหม่".into()));\n    }\n    Ok(())\n}',
  ],
  [
    'pub async fn login(State(st): State<AppState>, Json(req): Json<LoginReq>) -> ApiResult<Json<Value>> {\n    let phone = normalize_phone(&req.phone);\n    let row = sqlx::query("SELECT id, pin_hash FROM users WHERE phone = ?").bind(&phone).fetch_optional(&st.db).await?;\n    let Some(row) = row else { return Err(AppError::BadRequest("ไม่พบเบอร์นี้ หรือ PIN ไม่ถูกต้อง".into())) };\n    let hash: String = row.get("pin_hash");\n    if !verify_pin(&req.pin, &hash) {\n        return Err(AppError::BadRequest("ไม่พบเบอร์นี้ หรือ PIN ไม่ถูกต้อง".into()));\n    }\n    let user_id: String = row.get("id");',
    'pub async fn login(State(st): State<AppState>, Json(req): Json<LoginReq>) -> ApiResult<Json<Value>> {\n    let phone = normalize_phone(&req.phone);\n    // ล็อกชั่วคราวเมื่อกรอกผิดซ้ำ (ISO 27001 A.8.5)\n    if let Some(wait) = st.login_guard.locked_for(&phone) {\n        let mins = (wait / 60) + 1;\n        audit_auth(&st, None, "login_locked", &phone).await;\n        return Err(AppError::BadRequest(format!("กรอกผิดหลายครั้งเกินไป กรุณารออีก {mins} นาทีแล้วลองใหม่")));\n    }\n    let row = sqlx::query("SELECT id, pin_hash FROM users WHERE phone = ?").bind(&phone).fetch_optional(&st.db).await?;\n    let Some(row) = row else {\n        st.login_guard.record_failure(&phone);\n        audit_auth(&st, None, "login_failed", &phone).await;\n        return Err(AppError::BadRequest("ไม่พบเบอร์นี้ หรือ PIN ไม่ถูกต้อง".into()));\n    };\n    let hash: String = row.get("pin_hash");\n    if !verify_pin(&req.pin, &hash) {\n        st.login_guard.record_failure(&phone);\n        let uid: String = row.get("id");\n        audit_auth(&st, Some(&uid), "login_failed", &phone).await;\n        return Err(AppError::BadRequest("ไม่พบเบอร์นี้ หรือ PIN ไม่ถูกต้อง".into()));\n    }\n    st.login_guard.record_success(&phone);\n    let user_id: String = row.get("id");\n    audit_auth(&st, Some(&user_id), "login_ok", &phone).await;',
  ],
  [
    'async fn issue_session(st: &AppState, user_id: &str, device: Option<&str>) -> ApiResult<String> {',
    'async fn audit_auth(st: &AppState, user_id: Option<&str>, action: &str, phone: &str) {\n    // เก็บเฉพาะ 4 ตัวท้ายของเบอร์ ไม่เก็บเบอร์เต็มในล็อก (ลดข้อมูลส่วนบุคคล)\n    let masked = if phone.len() > 4 { format!("xxx{}", &phone[phone.len() - 4..]) } else { "xxx".into() };\n    let _ = sqlx::query("INSERT INTO audit_log (user_id, action, entity, entity_id, at) VALUES (?, ?, \'auth\', ?, ?)")\n        .bind(user_id)\n        .bind(action)\n        .bind(masked)\n        .bind(now_iso())\n        .execute(&st.db)\n        .await;\n}\n\nasync fn issue_session(st: &AppState, user_id: &str, device: Option<&str>) -> ApiResult<String> {',
  ],
  // session หมดอายุตามเวลาไม่ใช้งาน และอายุสูงสุด
  [
    '        let row = sqlx::query(\n            "SELECT u.id, u.org_id, u.role, u.name FROM sessions s JOIN users u ON u.id = s.user_id WHERE s.token = ?",\n        )',
    '        let row = sqlx::query(\n            "SELECT u.id, u.org_id, u.role, u.name FROM sessions s JOIN users u ON u.id = s.user_id\n             WHERE s.token = ?\n               AND s.last_seen_at > datetime(\'now\', ?)\n               AND s.created_at > datetime(\'now\', ?)",\n        )',
  ],
  [
    '        .bind(token)\n        .fetch_optional(&state.db)\n        .await?\n        .ok_or(AppError::Unauthorized)?;',
    '        .bind(token)\n        .bind(format!("-{} days", crate::security::SESSION_IDLE_DAYS))\n        .bind(format!("-{} days", crate::security::SESSION_MAX_DAYS))\n        .fetch_optional(&state.db)\n        .await?\n        .ok_or(AppError::Unauthorized)?;',
  ],
])

// 2) main.rs — โมดูล, state, security headers
edit('src/main.rs', [
  ['mod products;', 'mod products;\nmod security;'],
  [
    'pub struct AppState {\n    pub db: db::Db,\n    pub http: reqwest::Client,\n    pub cfg: Arc<Config>,\n}',
    'pub struct AppState {\n    pub db: db::Db,\n    pub http: reqwest::Client,\n    pub cfg: Arc<Config>,\n    pub login_guard: Arc<security::LoginGuard>,\n}',
  ],
  [
    '        cfg: Arc::new(cfg),\n    };',
    '        cfg: Arc::new(cfg),\n        login_guard: Arc::new(security::LoginGuard::new()),\n    };',
  ],
  [
    '        .layer(cors)\n        .layer(CompressionLayer::new())',
    '        .layer(axum::middleware::from_fn(security::security_headers))\n        .layer(cors)\n        .layer(CompressionLayer::new())',
  ],
  // งานบ้าน: ล้างรายการล็อกอินเก่าและ session หมดอายุทุกชั่วโมง
  [
    '    line::spawn_scheduler(state.clone());',
    '    line::spawn_scheduler(state.clone());\n    {\n        let st = state.clone();\n        tokio::spawn(async move {\n            loop {\n                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;\n                st.login_guard.sweep();\n                let _ = sqlx::query("DELETE FROM sessions WHERE last_seen_at < datetime(\'now\', ?) OR created_at < datetime(\'now\', ?)")\n                    .bind(format!("-{} days", security::SESSION_IDLE_DAYS))\n                    .bind(format!("-{} days", security::SESSION_MAX_DAYS))\n                    .execute(&st.db)\n                    .await;\n            }\n        });\n    }',
  ],
])
console.log('security wired')
