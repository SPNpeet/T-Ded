//! หลังบ้านสำหรับเจ้าหน้าที่ส่งเสริม/แอดมิน: ภาพรวมทุกฟาร์ม แก้ตาราง/กติกา ประกาศ ผู้ใช้ audit

use aqua_engine::{AdjustRule, SpeciesProfile};
use axum::{extract::{Path, Query, State}, Json};
use serde_json::{json, Value};
use sqlx::Row;

use crate::{
    auth::AuthUser,
    db::{new_id, now_iso, row_to_json, rows_to_json, today_bkk},
    error::{ApiResult, AppError},
    snapshot::{crop_snapshot, SnapshotOpts},
    AppState,
};

fn staff(user: &AuthUser) -> ApiResult<()> {
    if user.is_staff() { Ok(()) } else { Err(AppError::Forbidden) }
}

/// ทุกฟาร์มในองค์กร พร้อมสถานะล่าสุด: บันทึกล่าสุดเมื่อไหร่ กี่บ่อ คะแนนสุขภาพเฉลี่ย FCR
pub async fn farms_overview(State(st): State<AppState>, user: AuthUser) -> ApiResult<Json<Value>> {
    staff(&user)?;
    let farms = sqlx::query(
        "SELECT f.*,
           (SELECT COUNT(*) FROM crops c WHERE c.farm_id = f.id AND c.status = 'active') AS active_crops,
           (SELECT MAX(l.log_date) FROM daily_logs l JOIN crops c ON c.id = l.crop_id WHERE c.farm_id = f.id) AS last_log_date,
           (SELECT GROUP_CONCAT(u.name, ', ') FROM farm_members m JOIN users u ON u.id = m.user_id WHERE m.farm_id = f.id AND m.role = 'owner') AS owners,
           (SELECT u.phone FROM farm_members m JOIN users u ON u.id = m.user_id WHERE m.farm_id = f.id AND m.role = 'owner' LIMIT 1) AS owner_phone
         FROM farms f WHERE f.org_id = ? ORDER BY f.name",
    )
    .bind(&user.org_id)
    .fetch_all(&st.db)
    .await?;
    let today = today_bkk();
    let mut out = Vec::new();
    for r in farms {
        let mut f = row_to_json(&r);
        let fid = f["id"].as_str().unwrap_or("").to_string();
        let scores = sqlx::query("SELECT AVG(h.score) AS s FROM health_history h JOIN crops c ON c.id = h.crop_id WHERE c.farm_id = ? AND c.status = 'active' AND h.score_date = ?").bind(&fid).bind(&today).fetch_one(&st.db).await?;
        f["health_avg"] = json!(scores.get::<Option<f64>, _>("s").map(|x| x.round()));
        let days_silent = f["last_log_date"].as_str().map(|d| {
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok().and_then(|x| chrono::NaiveDate::parse_from_str(&today, "%Y-%m-%d").ok().map(|t| (t - x).num_days())).unwrap_or(0)
        });
        f["days_silent"] = json!(days_silent);
        f["needs_attention"] = json!(days_silent.map(|d| d >= 3).unwrap_or(f["active_crops"].as_i64().unwrap_or(0) > 0) || f["health_avg"].as_f64().map(|h| h < 50.0).unwrap_or(false));
        out.push(f);
    }
    Ok(Json(json!(out)))
}

/// รายละเอียดฟาร์มสำหรับเจ้าหน้าที่: ทุกบ่อพร้อมสแนปช็อต
pub async fn farm_detail(State(st): State<AppState>, user: AuthUser, Path(farm_id): Path<String>) -> ApiResult<Json<Value>> {
    staff(&user)?;
    let crops = sqlx::query("SELECT id FROM crops WHERE farm_id = ? AND status = 'active'").bind(&farm_id).fetch_all(&st.db).await?;
    let mut items = Vec::new();
    let opts = SnapshotOpts { with_weather: true, with_forecast: true, ..Default::default() };
    for r in crops {
        let id: String = r.get("id");
        items.push(crop_snapshot(&st, &id, &today_bkk(), &user.org_id, &opts).await?);
    }
    Ok(Json(json!(items)))
}

pub async fn get_rules(State(st): State<AppState>, user: AuthUser) -> ApiResult<Json<Value>> {
    let rules = crate::snapshot::rules_for(&st, &user.org_id).await?;
    let custom = sqlx::query("SELECT updated_at, updated_by FROM rule_overrides WHERE org_id = ?").bind(&user.org_id).fetch_optional(&st.db).await?;
    Ok(Json(json!({ "rules": rules, "custom": custom.map(|r| row_to_json(&r)), "defaults": aqua_engine::env::default_rules() })))
}

pub async fn put_rules(State(st): State<AppState>, user: AuthUser, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    staff(&user)?;
    let rules: Vec<AdjustRule> = serde_json::from_value(b.get("rules").cloned().unwrap_or(json!([])))?;
    if rules.is_empty() {
        sqlx::query("DELETE FROM rule_overrides WHERE org_id = ?").bind(&user.org_id).execute(&st.db).await?;
    } else {
        for r in &rules {
            if r.factor <= 0.0 || r.factor > 2.0 {
                return Err(AppError::BadRequest("ตัวคูณต้องอยู่ระหว่าง 0-2".into()));
            }
        }
        sqlx::query("INSERT INTO rule_overrides (org_id, rules_json, updated_by, updated_at) VALUES (?, ?, ?, ?) ON CONFLICT(org_id) DO UPDATE SET rules_json = excluded.rules_json, updated_by = excluded.updated_by, updated_at = excluded.updated_at")
            .bind(&user.org_id)
            .bind(serde_json::to_string(&rules)?)
            .bind(&user.id)
            .bind(now_iso())
            .execute(&st.db)
            .await?;
    }
    audit_admin(&st, &user, "put_rules", "rules", &user.org_id).await;
    Ok(Json(json!({ "ok": true })))
}

pub async fn get_species(State(st): State<AppState>, user: AuthUser) -> ApiResult<Json<Value>> {
    let mut list = Vec::new();
    for sp in SpeciesProfile::defaults() {
        let custom = crate::snapshot::species_for(&st, &user.org_id, &sp.code).await?;
        let is_custom = sqlx::query("SELECT 1 FROM species_overrides WHERE org_id = ? AND code = ?").bind(&user.org_id).bind(&sp.code).fetch_optional(&st.db).await?.is_some();
        list.push(json!({ "profile": custom, "custom": is_custom, "default": sp }));
    }
    Ok(Json(json!(list)))
}

pub async fn put_species(State(st): State<AppState>, user: AuthUser, Path(code): Path<String>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    staff(&user)?;
    if SpeciesProfile::by_code(&code).is_none() {
        return Err(AppError::BadRequest("ไม่รู้จักชนิดปลา".into()));
    }
    if b.get("reset").and_then(|v| v.as_bool()).unwrap_or(false) {
        sqlx::query("DELETE FROM species_overrides WHERE org_id = ? AND code = ?").bind(&user.org_id).bind(&code).execute(&st.db).await?;
        return Ok(Json(json!({ "ok": true, "reset": true })));
    }
    let profile: SpeciesProfile = serde_json::from_value(b.get("profile").cloned().unwrap_or(Value::Null))?;
    if profile.code != code || profile.feed_table.len() < 2 || profile.growth.len() < 2 {
        return Err(AppError::BadRequest("ตารางไม่ครบ (ต้องมีอย่างน้อย 2 แถว)".into()));
    }
    for w in profile.feed_table.windows(2) {
        if w[1].weight_g <= w[0].weight_g {
            return Err(AppError::BadRequest("น้ำหนักในตารางอาหารต้องเรียงจากน้อยไปมาก".into()));
        }
    }
    sqlx::query("INSERT INTO species_overrides (org_id, code, profile_json, updated_by, updated_at) VALUES (?, ?, ?, ?, ?) ON CONFLICT(org_id, code) DO UPDATE SET profile_json = excluded.profile_json, updated_by = excluded.updated_by, updated_at = excluded.updated_at")
        .bind(&user.org_id)
        .bind(&code)
        .bind(serde_json::to_string(&profile)?)
        .bind(&user.id)
        .bind(now_iso())
        .execute(&st.db)
        .await?;
    audit_admin(&st, &user, "put_species", "species", &code).await;
    Ok(Json(json!({ "ok": true })))
}

pub async fn list_users(State(st): State<AppState>, user: AuthUser) -> ApiResult<Json<Value>> {
    staff(&user)?;
    let rows = sqlx::query("SELECT u.id, u.name, u.phone, u.role, u.created_at, u.line_user_id IS NOT NULL AS line_linked, (SELECT GROUP_CONCAT(f.name, ', ') FROM farm_members m JOIN farms f ON f.id = m.farm_id WHERE m.user_id = u.id) AS farms FROM users u WHERE u.org_id = ? ORDER BY u.role, u.name")
        .bind(&user.org_id)
        .fetch_all(&st.db)
        .await?;
    Ok(Json(json!(rows_to_json(&rows))))
}

pub async fn create_announcement(State(st): State<AppState>, user: AuthUser, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    staff(&user)?;
    let title = b.get("title").and_then(|v| v.as_str()).map(str::trim).filter(|s| !s.is_empty()).ok_or_else(|| AppError::BadRequest("กรอกหัวข้อ".into()))?;
    let body = b.get("body").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    let id = new_id();
    sqlx::query("INSERT INTO announcements (id, org_id, title, body, created_by, created_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(&user.org_id)
        .bind(title)
        .bind(&body)
        .bind(&user.id)
        .bind(now_iso())
        .execute(&st.db)
        .await?;
    // ส่งเข้า LINE ให้ทุกคนที่ผูกไว้ (ถ้าตั้งค่า token)
    let users = sqlx::query("SELECT line_user_id FROM users WHERE org_id = ? AND line_user_id IS NOT NULL").bind(&user.org_id).fetch_all(&st.db).await?;
    for u in users {
        let lid: String = u.get("line_user_id");
        let _ = crate::line::push_text(&st, &lid, &format!("ประกาศจากหน่วยส่งเสริม\n{}\n{}", title, body)).await;
    }
    Ok(Json(json!({ "id": id })))
}

pub async fn list_announcements(State(st): State<AppState>, user: AuthUser) -> ApiResult<Json<Value>> {
    let rows = sqlx::query("SELECT a.*, u.name AS author FROM announcements a LEFT JOIN users u ON u.id = a.created_by WHERE a.org_id = ? ORDER BY a.created_at DESC LIMIT 50").bind(&user.org_id).fetch_all(&st.db).await?;
    Ok(Json(json!(rows_to_json(&rows))))
}

pub async fn audit_list(State(st): State<AppState>, user: AuthUser, Query(q): Query<Value>) -> ApiResult<Json<Value>> {
    staff(&user)?;
    let limit = q.get("limit").and_then(|v| v.as_str()).and_then(|v| v.parse::<i64>().ok()).unwrap_or(200);
    let rows = sqlx::query("SELECT a.*, u.name AS user_name FROM audit_log a LEFT JOIN users u ON u.id = a.user_id WHERE u.org_id = ? OR a.user_id IS NULL ORDER BY a.id DESC LIMIT ?").bind(&user.org_id).bind(limit).fetch_all(&st.db).await?;
    Ok(Json(json!(rows_to_json(&rows))))
}

/// สถิติกลุ่ม (ไม่ระบุตัวตน) เพื่อให้เกษตรกรเทียบกับค่าเฉลี่ย
pub async fn benchmark(State(st): State<AppState>, user: AuthUser) -> ApiResult<Json<Value>> {
    let today = today_bkk();
    let crops = sqlx::query("SELECT c.id FROM crops c JOIN farms f ON f.id = c.farm_id WHERE f.org_id = ? AND c.status = 'active'").bind(&user.org_id).fetch_all(&st.db).await?;
    let opts = SnapshotOpts { with_weather: false, with_forecast: false, ..Default::default() };
    let mut fcrs = Vec::new();
    let mut survs = Vec::new();
    let mut healths = Vec::new();
    for r in crops.iter().take(200) {
        let id: String = r.get("id");
        if let Ok(s) = crop_snapshot(&st, &id, &today, &user.org_id, &opts).await {
            if let Some(f) = s["performance"]["fcr"].as_f64() { fcrs.push(f); }
            if let Some(v) = s["performance"]["survival_pct"].as_f64() { survs.push(v); }
            if let Some(h) = s["health"]["score"].as_f64() { healths.push(h); }
        }
    }
    let avg = |v: &Vec<f64>| if v.is_empty() { None } else { Some((v.iter().sum::<f64>() / v.len() as f64 * 100.0).round() / 100.0) };
    Ok(Json(json!({ "n_crops": crops.len(), "fcr_avg": avg(&fcrs), "survival_avg": avg(&survs), "health_avg": avg(&healths) })))
}

async fn audit_admin(st: &AppState, user: &AuthUser, action: &str, entity: &str, entity_id: &str) {
    let _ = sqlx::query("INSERT INTO audit_log (user_id, action, entity, entity_id, at) VALUES (?, ?, ?, ?, ?)")
        .bind(&user.id)
        .bind(action)
        .bind(entity)
        .bind(entity_id)
        .bind(now_iso())
        .execute(&st.db)
        .await;
}
