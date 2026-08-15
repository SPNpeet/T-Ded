// ผูกระบบแพ็กเกจเข้ากับ server (รันครั้งเดียวจาก web/)
import { readFileSync, writeFileSync } from 'node:fs'
const R = (p) => '../crates/server/' + p
const edit = (file, pairs) => {
  let c = readFileSync(R(file), 'utf8')
  for (const [a, b] of pairs) {
    if (!c.includes(a)) throw new Error(file + ' missing: ' + a.slice(0, 70))
    c = c.replace(a, b)
  }
  writeFileSync(R(file), c)
}

edit('src/db.rs', [
  [
    '("0004_settings", include_str!("../migrations/0004_settings.sql")),',
    '("0004_settings", include_str!("../migrations/0004_settings.sql")),\n    ("0005_subscriptions", include_str!("../migrations/0005_subscriptions.sql")),',
  ],
])

edit('src/main.rs', [
  ['mod api;', 'mod api;\nmod billing;'],
  [
    '        .route("/benchmark", get(admin::benchmark))',
    '        .route("/benchmark", get(admin::benchmark))\n        .route("/subscription", get(billing::my_subscription))\n        .route("/admin/subscriptions", get(billing::list_subscriptions))\n        .route("/admin/subscriptions/{org_id}", axum::routing::put(billing::set_subscription))\n        .route("/admin/subscriptions/{org_id}/payments", post(billing::record_payment))\n        .route("/admin/revenue", get(billing::revenue_summary))',
  ],
])

// เริ่มสิทธิ์ทดลองใช้ตอนสมัคร และตรวจสิทธิ์ก่อนสร้างฟาร์ม/บ่อ/ผู้ใช้
edit('src/auth.rs', [
  [
    '    let token = issue_session(&st, &user_id, None).await?;\n    Ok(Json(json!({ "token": token, "user": user_json(&st, &user_id).await?, "farm_id": farm_id })))',
    '    crate::billing::start_trial(&st, &org_id).await?;\n    let token = issue_session(&st, &user_id, None).await?;\n    Ok(Json(json!({ "token": token, "user": user_json(&st, &user_id).await?, "farm_id": farm_id })))',
  ],
  [
    '    let phone = normalize_phone(&req.phone);\n    validate_pin(&req.pin)?;\n    if sqlx::query("SELECT id FROM users WHERE phone = ?").bind(&phone).fetch_optional(&st.db).await?.is_some() {',
    '    let phone = normalize_phone(&req.phone);\n    validate_pin(&req.pin)?;\n    crate::billing::check_can_add(&st, &user.org_id, "สมาชิก").await?;\n    if sqlx::query("SELECT id FROM users WHERE phone = ?").bind(&phone).fetch_optional(&st.db).await?.is_some() {',
  ],
])

edit('src/api.rs', [
  [
    'pub async fn create_farm(State(st): State<AppState>, user: AuthUser, Json(b): Json<Value>) -> ApiResult<Json<Value>> {\n    let name = s(&b, "name").ok_or_else(|| AppError::BadRequest("กรอกชื่อฟาร์ม".into()))?;',
    'pub async fn create_farm(State(st): State<AppState>, user: AuthUser, Json(b): Json<Value>) -> ApiResult<Json<Value>> {\n    let name = s(&b, "name").ok_or_else(|| AppError::BadRequest("กรอกชื่อฟาร์ม".into()))?;\n    crate::billing::check_can_add(&st, &user.org_id, "ฟาร์ม").await?;',
  ],
  [
    'pub async fn create_pond(State(st): State<AppState>, user: AuthUser, Path(farm_id): Path<String>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {\n    assert_farm_access(&st, &user, &farm_id).await?;',
    'pub async fn create_pond(State(st): State<AppState>, user: AuthUser, Path(farm_id): Path<String>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {\n    assert_farm_access(&st, &user, &farm_id).await?;\n    crate::billing::check_can_add(&st, &user.org_id, "บ่อ").await?;',
  ],
])
console.log('billing wired')
