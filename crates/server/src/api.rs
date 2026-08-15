//! CRUD ฟาร์ม บ่อ รุ่นการเลี้ยง บันทึกประจำวัน ชั่ง น้ำ สต๊อก ค่าใช้จ่าย จับ ยา + sync ออฟไลน์

use axum::{extract::{Path, Query, State}, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;

use crate::{
    auth::AuthUser,
    db::{new_id, now_iso, row_to_json, rows_to_json, today_bkk},
    error::{ApiResult, AppError},
    AppState,
};

// ---------- สิทธิ์ ----------

pub async fn assert_farm_access(st: &AppState, user: &AuthUser, farm_id: &str) -> ApiResult<()> {
    let row = sqlx::query("SELECT org_id FROM farms WHERE id = ?").bind(farm_id).fetch_optional(&st.db).await?.ok_or(AppError::NotFound)?;
    let org: String = row.get("org_id");
    if user.is_staff() && org == user.org_id {
        return Ok(());
    }
    let m = sqlx::query("SELECT 1 FROM farm_members WHERE farm_id = ? AND user_id = ?").bind(farm_id).bind(&user.id).fetch_optional(&st.db).await?;
    if m.is_some() {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

pub async fn farm_of_pond(st: &AppState, pond_id: &str) -> ApiResult<String> {
    let r = sqlx::query("SELECT farm_id FROM ponds WHERE id = ?").bind(pond_id).fetch_optional(&st.db).await?.ok_or(AppError::NotFound)?;
    Ok(r.get("farm_id"))
}

pub async fn farm_of_crop(st: &AppState, crop_id: &str) -> ApiResult<String> {
    let r = sqlx::query("SELECT farm_id FROM crops WHERE id = ?").bind(crop_id).fetch_optional(&st.db).await?.ok_or(AppError::NotFound)?;
    Ok(r.get("farm_id"))
}

async fn audit(st: &AppState, user: &AuthUser, action: &str, entity: &str, entity_id: &str, detail: Option<&Value>) {
    let _ = sqlx::query("INSERT INTO audit_log (user_id, action, entity, entity_id, detail_json, at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&user.id)
        .bind(action)
        .bind(entity)
        .bind(entity_id)
        .bind(detail.map(|d| d.to_string()))
        .bind(now_iso())
        .execute(&st.db)
        .await;
}

fn s(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(|x| x.as_str()).map(|x| x.trim().to_string()).filter(|x| !x.is_empty())
}
fn f(v: &Value, k: &str) -> Option<f64> {
    v.get(k).and_then(|x| x.as_f64().or_else(|| x.as_str().and_then(|s| s.parse().ok())))
}
fn i(v: &Value, k: &str) -> Option<i64> {
    v.get(k).and_then(|x| x.as_i64().or_else(|| x.as_f64().map(|f| f as i64)).or_else(|| x.as_str().and_then(|s| s.parse().ok())))
}
fn date_or_today(v: &Value, k: &str) -> String {
    s(v, k).unwrap_or_else(today_bkk)
}
fn req_f(v: &Value, k: &str, label: &str) -> ApiResult<f64> {
    f(v, k).ok_or_else(|| AppError::BadRequest(format!("กรอก{label}")))
}

// ---------- ฟาร์ม ----------

pub async fn list_farms(State(st): State<AppState>, user: AuthUser) -> ApiResult<Json<Value>> {
    let rows = if user.is_staff() {
        sqlx::query("SELECT * FROM farms WHERE org_id = ? ORDER BY name").bind(&user.org_id).fetch_all(&st.db).await?
    } else {
        sqlx::query("SELECT f.* FROM farms f JOIN farm_members m ON m.farm_id = f.id WHERE m.user_id = ? ORDER BY f.name")
            .bind(&user.id)
            .fetch_all(&st.db)
            .await?
    };
    Ok(Json(json!(rows_to_json(&rows))))
}

pub async fn create_farm(State(st): State<AppState>, user: AuthUser, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    let name = s(&b, "name").ok_or_else(|| AppError::BadRequest("กรอกชื่อฟาร์ม".into()))?;
    let id = new_id();
    let now = now_iso();
    sqlx::query("INSERT INTO farms (id, org_id, name, province, district, lat, lng, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(&user.org_id)
        .bind(&name)
        .bind(s(&b, "province"))
        .bind(s(&b, "district"))
        .bind(f(&b, "lat"))
        .bind(f(&b, "lng"))
        .bind(&now)
        .bind(&now)
        .execute(&st.db)
        .await?;
    sqlx::query("INSERT INTO farm_members (farm_id, user_id, role) VALUES (?, ?, 'owner')").bind(&id).bind(&user.id).execute(&st.db).await?;
    audit(&st, &user, "create", "farm", &id, None).await;
    Ok(Json(json!({ "id": id })))
}

pub async fn get_farm(State(st): State<AppState>, user: AuthUser, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    assert_farm_access(&st, &user, &id).await?;
    let row = sqlx::query("SELECT * FROM farms WHERE id = ?").bind(&id).fetch_one(&st.db).await?;
    let mut farm = row_to_json(&row);
    let ponds = sqlx::query("SELECT * FROM ponds WHERE farm_id = ? AND active = 1 ORDER BY sort_order, name").bind(&id).fetch_all(&st.db).await?;
    farm["ponds"] = json!(rows_to_json(&ponds));
    let members = sqlx::query("SELECT u.id, u.name, u.phone, m.role FROM farm_members m JOIN users u ON u.id = m.user_id WHERE m.farm_id = ?")
        .bind(&id)
        .fetch_all(&st.db)
        .await?;
    farm["members"] = json!(rows_to_json(&members));
    Ok(Json(farm))
}

pub async fn update_farm(State(st): State<AppState>, user: AuthUser, Path(id): Path<String>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    assert_farm_access(&st, &user, &id).await?;
    sqlx::query(
        "UPDATE farms SET name = COALESCE(?, name), province = COALESCE(?, province), district = COALESCE(?, district), lat = COALESCE(?, lat), lng = COALESCE(?, lng), meals_per_day = COALESCE(?, meals_per_day), farm_factor = COALESCE(?, farm_factor), bag_kg = COALESCE(?, bag_kg), updated_at = ? WHERE id = ?",
    )
    .bind(s(&b, "name"))
    .bind(s(&b, "province"))
    .bind(s(&b, "district"))
    .bind(f(&b, "lat"))
    .bind(f(&b, "lng"))
    .bind(i(&b, "meals_per_day"))
    .bind(f(&b, "farm_factor"))
    .bind(f(&b, "bag_kg"))
    .bind(now_iso())
    .bind(&id)
    .execute(&st.db)
    .await?;
    audit(&st, &user, "update", "farm", &id, Some(&b)).await;
    Ok(Json(json!({ "ok": true })))
}

// ---------- บ่อ ----------

pub async fn create_pond(State(st): State<AppState>, user: AuthUser, Path(farm_id): Path<String>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    assert_farm_access(&st, &user, &farm_id).await?;
    let name = s(&b, "name").ok_or_else(|| AppError::BadRequest("กรอกชื่อบ่อ".into()))?;
    let id = new_id();
    let area_rai = f(&b, "area_rai");
    let area_m2 = f(&b, "area_m2").or(area_rai.map(|r| r * 1600.0));
    let order: i64 = sqlx::query("SELECT COALESCE(MAX(sort_order), 0) + 1 AS n FROM ponds WHERE farm_id = ?").bind(&farm_id).fetch_one(&st.db).await?.get("n");
    sqlx::query("INSERT INTO ponds (id, farm_id, name, pond_type, area_rai, area_m2, depth_m, sort_order, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(&farm_id)
        .bind(&name)
        .bind(s(&b, "pond_type").unwrap_or_else(|| "earthen".into()))
        .bind(area_rai)
        .bind(area_m2)
        .bind(f(&b, "depth_m"))
        .bind(order)
        .bind(now_iso())
        .execute(&st.db)
        .await?;
    audit(&st, &user, "create", "pond", &id, None).await;
    Ok(Json(json!({ "id": id })))
}

pub async fn update_pond(State(st): State<AppState>, user: AuthUser, Path(id): Path<String>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    let farm_id = farm_of_pond(&st, &id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    sqlx::query("UPDATE ponds SET name = COALESCE(?, name), pond_type = COALESCE(?, pond_type), area_rai = COALESCE(?, area_rai), area_m2 = COALESCE(?, area_m2), depth_m = COALESCE(?, depth_m), active = COALESCE(?, active), sort_order = COALESCE(?, sort_order) WHERE id = ?")
        .bind(s(&b, "name"))
        .bind(s(&b, "pond_type"))
        .bind(f(&b, "area_rai"))
        .bind(f(&b, "area_m2"))
        .bind(f(&b, "depth_m"))
        .bind(i(&b, "active"))
        .bind(i(&b, "sort_order"))
        .bind(&id)
        .execute(&st.db)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

// ---------- รุ่นการเลี้ยง ----------

pub async fn create_crop(State(st): State<AppState>, user: AuthUser, Path(pond_id): Path<String>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    let farm_id = farm_of_pond(&st, &pond_id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    let active = sqlx::query("SELECT id FROM crops WHERE pond_id = ? AND status = 'active'").bind(&pond_id).fetch_optional(&st.db).await?;
    if active.is_some() {
        return Err(AppError::BadRequest("บ่อนี้มีรุ่นที่เลี้ยงอยู่ ปิดรุ่นเดิมก่อน".into()));
    }
    let species = s(&b, "species_code").unwrap_or_else(|| "nile_tilapia".into());
    if aqua_engine::SpeciesProfile::by_code(&species).is_none() {
        return Err(AppError::BadRequest("ไม่รู้จักชนิดปลา".into()));
    }
    let count = i(&b, "stocked_count").filter(|c| *c > 0).ok_or_else(|| AppError::BadRequest("กรอกจำนวนปลาที่ปล่อย".into()))?;
    let w = req_f(&b, "stock_weight_g", "น้ำหนักปลาตอนปล่อย")?;
    let id = new_id();
    let stocked_at = date_or_today(&b, "stocked_at");
    sqlx::query("INSERT INTO crops (id, pond_id, farm_id, species_code, stocked_at, stocked_count, stock_weight_g, fry_price_each, target_weight_g, target_harvest_at, note, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(&pond_id)
        .bind(&farm_id)
        .bind(&species)
        .bind(&stocked_at)
        .bind(count)
        .bind(w)
        .bind(f(&b, "fry_price_each").unwrap_or(0.0))
        .bind(f(&b, "target_weight_g"))
        .bind(s(&b, "target_harvest_at"))
        .bind(s(&b, "note"))
        .bind(now_iso())
        .execute(&st.db)
        .await?;
    // ค่าลูกปลาเป็นค่าใช้จ่ายแรกของรุ่น
    if let Some(p) = f(&b, "fry_price_each").filter(|p| *p > 0.0) {
        sqlx::query("INSERT INTO expenses (id, crop_id, expense_date, category, amount, note, created_at) VALUES (?, ?, ?, 'fry', ?, 'ค่าลูกปลา', ?)")
            .bind(new_id())
            .bind(&id)
            .bind(&stocked_at)
            .bind(p * count as f64)
            .bind(now_iso())
            .execute(&st.db)
            .await?;
    }
    // ชั่งครั้งแรก = น้ำหนักปล่อย
    sqlx::query("INSERT INTO weighings (id, crop_id, weigh_date, sample_count, avg_weight_g, method, note, created_at) VALUES (?, ?, ?, NULL, ?, 'stocking', 'น้ำหนักตอนปล่อย', ?)")
        .bind(new_id())
        .bind(&id)
        .bind(&stocked_at)
        .bind(w)
        .bind(now_iso())
        .execute(&st.db)
        .await?;
    audit(&st, &user, "create", "crop", &id, Some(&b)).await;
    Ok(Json(json!({ "id": id })))
}

pub async fn update_crop(State(st): State<AppState>, user: AuthUser, Path(id): Path<String>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    let farm_id = farm_of_crop(&st, &id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    sqlx::query("UPDATE crops SET target_weight_g = COALESCE(?, target_weight_g), target_harvest_at = COALESCE(?, target_harvest_at), note = COALESCE(?, note), stocked_count = COALESCE(?, stocked_count), stock_weight_g = COALESCE(?, stock_weight_g), stocked_at = COALESCE(?, stocked_at) WHERE id = ?")
        .bind(f(&b, "target_weight_g"))
        .bind(s(&b, "target_harvest_at"))
        .bind(s(&b, "note"))
        .bind(i(&b, "stocked_count"))
        .bind(f(&b, "stock_weight_g"))
        .bind(s(&b, "stocked_at"))
        .bind(&id)
        .execute(&st.db)
        .await?;
    audit(&st, &user, "update", "crop", &id, Some(&b)).await;
    Ok(Json(json!({ "ok": true })))
}

pub async fn close_crop(State(st): State<AppState>, user: AuthUser, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    let farm_id = farm_of_crop(&st, &id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    sqlx::query("UPDATE crops SET status = 'closed', closed_at = ? WHERE id = ?").bind(today_bkk()).bind(&id).execute(&st.db).await?;
    audit(&st, &user, "close", "crop", &id, None).await;
    Ok(Json(json!({ "ok": true })))
}

pub async fn list_crops(State(st): State<AppState>, user: AuthUser, Path(farm_id): Path<String>, Query(q): Query<Value>) -> ApiResult<Json<Value>> {
    assert_farm_access(&st, &user, &farm_id).await?;
    let status = q.get("status").and_then(|v| v.as_str()).unwrap_or("active");
    let rows = if status == "all" {
        sqlx::query("SELECT c.*, p.name AS pond_name FROM crops c JOIN ponds p ON p.id = c.pond_id WHERE c.farm_id = ? ORDER BY c.stocked_at DESC").bind(&farm_id).fetch_all(&st.db).await?
    } else {
        sqlx::query("SELECT c.*, p.name AS pond_name FROM crops c JOIN ponds p ON p.id = c.pond_id WHERE c.farm_id = ? AND c.status = ? ORDER BY p.sort_order, p.name")
            .bind(&farm_id)
            .bind(status)
            .fetch_all(&st.db)
            .await?
    };
    Ok(Json(json!(rows_to_json(&rows))))
}

// ---------- บันทึกประจำวัน ----------

pub async fn upsert_log(State(st): State<AppState>, user: AuthUser, Path(crop_id): Path<String>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    let farm_id = farm_of_crop(&st, &crop_id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    let id = insert_log(&st, &user, &crop_id, &farm_id, &b).await?;
    Ok(Json(json!({ "id": id })))
}

/// upsert ตาม (crop_id, log_date): บันทึกซ้ำวันเดิม = อัปเดต (idempotent สำหรับ sync ออฟไลน์)
pub async fn insert_log(st: &AppState, user: &AuthUser, crop_id: &str, farm_id: &str, b: &Value) -> ApiResult<String> {
    let date = date_or_today(b, "log_date");
    let existing = sqlx::query("SELECT id, fed_kg FROM daily_logs WHERE crop_id = ? AND log_date = ?").bind(crop_id).bind(&date).fetch_optional(&st.db).await?;
    let now = now_iso();
    let fed = f(b, "fed_kg");
    let id = match existing {
        Some(r) => {
            let id: String = r.get("id");
            sqlx::query("UPDATE daily_logs SET fed_kg = COALESCE(?, fed_kg), recommended_kg = COALESCE(?, recommended_kg), factor = COALESCE(?, factor), mortality = COALESCE(?, mortality), feeding_response = COALESCE(?, feeding_response), weather_json = COALESCE(?, weather_json), reasons_json = COALESCE(?, reasons_json), note = COALESCE(?, note), photo_url = COALESCE(?, photo_url), updated_at = ? WHERE id = ?")
                .bind(fed)
                .bind(f(b, "recommended_kg"))
                .bind(f(b, "factor"))
                .bind(i(b, "mortality"))
                .bind(i(b, "feeding_response"))
                .bind(b.get("weather").filter(|v| !v.is_null()).map(|v| v.to_string()))
                .bind(b.get("reasons").filter(|v| !v.is_null()).map(|v| v.to_string()))
                .bind(s(b, "note"))
                .bind(s(b, "photo_url"))
                .bind(&now)
                .bind(&id)
                .execute(&st.db)
                .await?;
            id
        }
        None => {
            let id = new_id();
            sqlx::query("INSERT INTO daily_logs (id, client_id, crop_id, log_date, recommended_kg, fed_kg, factor, mortality, feeding_response, weather_json, reasons_json, note, photo_url, created_by, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
                .bind(&id)
                .bind(s(b, "client_id"))
                .bind(crop_id)
                .bind(&date)
                .bind(f(b, "recommended_kg"))
                .bind(fed)
                .bind(f(b, "factor"))
                .bind(i(b, "mortality").unwrap_or(0))
                .bind(i(b, "feeding_response").unwrap_or(0))
                .bind(b.get("weather").filter(|v| !v.is_null()).map(|v| v.to_string()))
                .bind(b.get("reasons").filter(|v| !v.is_null()).map(|v| v.to_string()))
                .bind(s(b, "note"))
                .bind(s(b, "photo_url"))
                .bind(&user.id)
                .bind(&now)
                .bind(&now)
                .execute(&st.db)
                .await?;
            id
        }
    };
    // ตัดสต๊อกอาหารอัตโนมัติ: ลบรายการเดิมของ log นี้แล้วบันทึกใหม่ตามค่าล่าสุด
    if let Some(kg) = fed {
        sqlx::query("DELETE FROM feed_stock_moves WHERE ref_log_id = ?").bind(&id).execute(&st.db).await?;
        if kg > 0.0 {
            sqlx::query("INSERT INTO feed_stock_moves (id, farm_id, move_date, kind, kg, crop_id, ref_log_id, note, created_at) VALUES (?, ?, ?, 'out', ?, ?, ?, 'ให้อาหารตามบันทึกประจำวัน', ?)")
                .bind(new_id())
                .bind(farm_id)
                .bind(&date)
                .bind(kg)
                .bind(crop_id)
                .bind(&id)
                .bind(&now)
                .execute(&st.db)
                .await?;
        }
    }
    // บันทึกน้ำพร้อมกัน (ถ้าส่งมา)
    if let Some(w) = b.get("water").filter(|v| v.is_object()) {
        let pond: String = sqlx::query("SELECT pond_id FROM crops WHERE id = ?").bind(crop_id).fetch_one(&st.db).await?.get("pond_id");
        insert_water(st, &pond, Some(crop_id), w).await?;
    }
    Ok(id)
}

pub async fn list_logs(State(st): State<AppState>, user: AuthUser, Path(crop_id): Path<String>, Query(q): Query<Value>) -> ApiResult<Json<Value>> {
    let farm_id = farm_of_crop(&st, &crop_id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    let limit = q.get("limit").and_then(|v| v.as_str()).and_then(|v| v.parse::<i64>().ok()).unwrap_or(60);
    let rows = sqlx::query("SELECT * FROM daily_logs WHERE crop_id = ? ORDER BY log_date DESC LIMIT ?").bind(&crop_id).bind(limit).fetch_all(&st.db).await?;
    Ok(Json(json!(rows_to_json(&rows))))
}

// ---------- ชั่งน้ำหนัก ----------

pub async fn create_weighing(State(st): State<AppState>, user: AuthUser, Path(crop_id): Path<String>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    let farm_id = farm_of_crop(&st, &crop_id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    let id = insert_weighing(&st, &crop_id, &b).await?;
    audit(&st, &user, "create", "weighing", &id, Some(&b)).await;
    Ok(Json(json!({ "id": id })))
}

pub async fn insert_weighing(st: &AppState, crop_id: &str, b: &Value) -> ApiResult<String> {
    let w = req_f(b, "avg_weight_g", "น้ำหนักเฉลี่ย")?;
    if w <= 0.0 {
        return Err(AppError::BadRequest("น้ำหนักต้องมากกว่า 0".into()));
    }
    // ถ้าส่งมาเป็นน้ำหนักรวม + จำนวนตัวอย่าง ให้คิดเฉลี่ยให้
    let id = new_id();
    sqlx::query("INSERT INTO weighings (id, client_id, crop_id, weigh_date, sample_count, avg_weight_g, method, note, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(s(b, "client_id"))
        .bind(crop_id)
        .bind(date_or_today(b, "weigh_date"))
        .bind(i(b, "sample_count"))
        .bind(w)
        .bind(s(b, "method").unwrap_or_else(|| "sample".into()))
        .bind(s(b, "note"))
        .bind(now_iso())
        .execute(&st.db)
        .await?;
    Ok(id)
}

pub async fn list_weighings(State(st): State<AppState>, user: AuthUser, Path(crop_id): Path<String>) -> ApiResult<Json<Value>> {
    let farm_id = farm_of_crop(&st, &crop_id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    let rows = sqlx::query("SELECT * FROM weighings WHERE crop_id = ? ORDER BY weigh_date, created_at").bind(&crop_id).fetch_all(&st.db).await?;
    Ok(Json(json!(rows_to_json(&rows))))
}

// ---------- คุณภาพน้ำ ----------

pub async fn create_water(State(st): State<AppState>, user: AuthUser, Path(pond_id): Path<String>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    let farm_id = farm_of_pond(&st, &pond_id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    let crop: Option<String> = sqlx::query("SELECT id FROM crops WHERE pond_id = ? AND status = 'active'").bind(&pond_id).fetch_optional(&st.db).await?.map(|r| r.get("id"));
    let id = insert_water(&st, &pond_id, crop.as_deref(), &b).await?;
    Ok(Json(json!({ "id": id })))
}

pub async fn insert_water(st: &AppState, pond_id: &str, crop_id: Option<&str>, b: &Value) -> ApiResult<String> {
    let id = new_id();
    let at = s(b, "measured_at").unwrap_or_else(|| now_iso());
    sqlx::query("INSERT INTO water_quality (id, client_id, pond_id, crop_id, measured_at, do_mg_l, ph, temp_c, nh3, no2, secchi_cm, color, note, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(s(b, "client_id"))
        .bind(pond_id)
        .bind(crop_id)
        .bind(&at)
        .bind(f(b, "do_mg_l"))
        .bind(f(b, "ph"))
        .bind(f(b, "temp_c"))
        .bind(f(b, "nh3"))
        .bind(f(b, "no2"))
        .bind(f(b, "secchi_cm"))
        .bind(s(b, "color"))
        .bind(s(b, "note"))
        .bind(now_iso())
        .execute(&st.db)
        .await?;
    Ok(id)
}

pub async fn list_water(State(st): State<AppState>, user: AuthUser, Path(pond_id): Path<String>, Query(q): Query<Value>) -> ApiResult<Json<Value>> {
    let farm_id = farm_of_pond(&st, &pond_id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    let limit = q.get("limit").and_then(|v| v.as_str()).and_then(|v| v.parse::<i64>().ok()).unwrap_or(90);
    let rows = sqlx::query("SELECT * FROM water_quality WHERE pond_id = ? ORDER BY measured_at DESC LIMIT ?").bind(&pond_id).bind(limit).fetch_all(&st.db).await?;
    Ok(Json(json!(rows_to_json(&rows))))
}

// ---------- สต๊อกอาหาร ----------

pub async fn create_stock_move(State(st): State<AppState>, user: AuthUser, Path(farm_id): Path<String>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    assert_farm_access(&st, &user, &farm_id).await?;
    let id = insert_stock_move(&st, &farm_id, &b).await?;
    audit(&st, &user, "create", "feed_stock", &id, Some(&b)).await;
    Ok(Json(json!({ "id": id })))
}

pub async fn insert_stock_move(st: &AppState, farm_id: &str, b: &Value) -> ApiResult<String> {
    let kind = s(b, "kind").unwrap_or_else(|| "in".into());
    if !["in", "out", "adjust"].contains(&kind.as_str()) {
        return Err(AppError::BadRequest("kind ต้องเป็น in/out/adjust".into()));
    }
    let bag_kg: f64 = sqlx::query("SELECT bag_kg FROM farms WHERE id = ?").bind(farm_id).fetch_one(&st.db).await?.get("bag_kg");
    let bags = f(b, "bags");
    let kg = f(b, "kg").or(bags.map(|x| x * f(b, "bag_kg").unwrap_or(bag_kg))).ok_or_else(|| AppError::BadRequest("กรอกจำนวน กก. หรือกระสอบ".into()))?;
    let id = new_id();
    sqlx::query("INSERT INTO feed_stock_moves (id, client_id, farm_id, move_date, kind, brand, pellet_mm, bags, kg, price_total, crop_id, note, created_at, protein_pct, form, product_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(s(b, "client_id"))
        .bind(farm_id)
        .bind(date_or_today(b, "move_date"))
        .bind(&kind)
        .bind(s(b, "brand"))
        .bind(f(b, "pellet_mm"))
        .bind(bags)
        .bind(kg)
        .bind(f(b, "price_total"))
        .bind(s(b, "crop_id"))
        .bind(s(b, "note"))
        .bind(now_iso())
        .bind(f(b, "protein_pct"))
        .bind(s(b, "form"))
        .bind(s(b, "product_id"))
        .execute(&st.db)
        .await?;
    Ok(id)
}

pub async fn stock_summary(State(st): State<AppState>, user: AuthUser, Path(farm_id): Path<String>) -> ApiResult<Json<Value>> {
    assert_farm_access(&st, &user, &farm_id).await?;
    Ok(Json(stock_summary_json(&st, &farm_id).await?))
}

pub async fn stock_summary_json(st: &AppState, farm_id: &str) -> ApiResult<Value> {
    let row = sqlx::query(
        "SELECT
           COALESCE(SUM(CASE WHEN kind='in' THEN kg WHEN kind='out' THEN -kg ELSE kg END),0.0) AS balance_kg,
           COALESCE(SUM(CASE WHEN kind='in' THEN kg END),0.0) AS in_kg,
           COALESCE(SUM(CASE WHEN kind='in' THEN price_total END),0.0) AS in_price,
           COALESCE(SUM(CASE WHEN kind='out' THEN kg END),0.0) AS out_kg
         FROM feed_stock_moves WHERE farm_id = ?",
    )
    .bind(farm_id)
    .fetch_one(&st.db)
    .await?;
    let balance: f64 = row.get("balance_kg");
    let in_kg: f64 = row.get("in_kg");
    let in_price: f64 = row.get("in_price");
    let avg_price = if in_kg > 0.0 { in_price / in_kg } else { 0.0 };
    // อัตราใช้เฉลี่ย 7 วัน
    let used7: f64 = sqlx::query("SELECT COALESCE(SUM(kg),0.0) AS k FROM feed_stock_moves WHERE farm_id = ? AND kind = 'out' AND move_date >= date('now','-7 days')")
        .bind(farm_id)
        .fetch_one(&st.db)
        .await?
        .get("k");
    let per_day = used7 / 7.0;
    let days_left = if per_day > 0.0 { Some((balance / per_day).floor()) } else { None };
    let bag_kg: f64 = sqlx::query("SELECT bag_kg FROM farms WHERE id = ?").bind(farm_id).fetch_one(&st.db).await?.get("bag_kg");
    let recent = sqlx::query("SELECT * FROM feed_stock_moves WHERE farm_id = ? ORDER BY move_date DESC, created_at DESC LIMIT 30").bind(farm_id).fetch_all(&st.db).await?;
    let last_in = sqlx::query("SELECT brand, protein_pct, pellet_mm, form, price_total, kg FROM feed_stock_moves WHERE farm_id = ? AND kind = 'in' ORDER BY move_date DESC, created_at DESC LIMIT 1").bind(farm_id).fetch_optional(&st.db).await?;
    let current_feed = last_in.map(|r| { let v = row_to_json(&r); let kg = v["kg"].as_f64().unwrap_or(0.0); let price = v["price_total"].as_f64(); json!({ "brand": v["brand"], "protein_pct": v["protein_pct"], "pellet_mm": v["pellet_mm"], "form": v["form"], "price_per_kg": price.filter(|_| kg > 0.0).map(|p| (p / kg * 100.0).round() / 100.0) }) });
    Ok(json!({
        "balance_kg": (balance * 10.0).round() / 10.0,
        "balance_bags": (balance / bag_kg * 10.0).round() / 10.0,
        "bag_kg": bag_kg,
        "avg_price_per_kg": (avg_price * 100.0).round() / 100.0,
        "used_per_day_7d": (per_day * 100.0).round() / 100.0,
        "days_left": days_left,
        "low": days_left.map(|d| d <= 5.0).unwrap_or(false),
        "moves": rows_to_json(&recent),
        "current_feed": current_feed,
    }))
}

// ---------- ค่าใช้จ่าย / จับ / ยา ----------

pub async fn create_expense(State(st): State<AppState>, user: AuthUser, Path(crop_id): Path<String>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    let farm_id = farm_of_crop(&st, &crop_id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    let id = insert_expense(&st, &crop_id, &b).await?;
    Ok(Json(json!({ "id": id })))
}
pub async fn insert_expense(st: &AppState, crop_id: &str, b: &Value) -> ApiResult<String> {
    let amount = req_f(b, "amount", "จำนวนเงิน")?;
    let id = new_id();
    sqlx::query("INSERT INTO expenses (id, client_id, crop_id, expense_date, category, amount, note, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(s(b, "client_id"))
        .bind(crop_id)
        .bind(date_or_today(b, "expense_date"))
        .bind(s(b, "category").unwrap_or_else(|| "other".into()))
        .bind(amount)
        .bind(s(b, "note"))
        .bind(now_iso())
        .execute(&st.db)
        .await?;
    Ok(id)
}
pub async fn list_expenses(State(st): State<AppState>, user: AuthUser, Path(crop_id): Path<String>) -> ApiResult<Json<Value>> {
    let farm_id = farm_of_crop(&st, &crop_id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    let rows = sqlx::query("SELECT * FROM expenses WHERE crop_id = ? ORDER BY expense_date DESC").bind(&crop_id).fetch_all(&st.db).await?;
    Ok(Json(json!(rows_to_json(&rows))))
}

pub async fn create_harvest(State(st): State<AppState>, user: AuthUser, Path(crop_id): Path<String>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    let farm_id = farm_of_crop(&st, &crop_id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    let id = insert_harvest(&st, &crop_id, &b).await?;
    audit(&st, &user, "create", "harvest", &id, Some(&b)).await;
    Ok(Json(json!({ "id": id })))
}
pub async fn insert_harvest(st: &AppState, crop_id: &str, b: &Value) -> ApiResult<String> {
    let kg = req_f(b, "kg", "น้ำหนักที่จับ")?;
    let id = new_id();
    sqlx::query("INSERT INTO harvests (id, client_id, crop_id, harvest_date, count, kg, price_per_kg, buyer, note, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(s(b, "client_id"))
        .bind(crop_id)
        .bind(date_or_today(b, "harvest_date"))
        .bind(i(b, "count"))
        .bind(kg)
        .bind(f(b, "price_per_kg"))
        .bind(s(b, "buyer"))
        .bind(s(b, "note"))
        .bind(now_iso())
        .execute(&st.db)
        .await?;
    // ราคาที่ขายได้จริง = ข้อมูลราคาตลาดของพื้นที่ (ไม่ระบุฟาร์ม)
    if let Some(p) = f(b, "price_per_kg") {
        let r = sqlx::query("SELECT c.species_code, f.province FROM crops c JOIN farms f ON f.id = c.farm_id WHERE c.id = ?").bind(crop_id).fetch_one(&st.db).await?;
        let species: String = r.get("species_code");
        let province: Option<String> = r.get("province");
        sqlx::query("INSERT INTO market_prices (id, species_code, province, price_per_kg, source, price_date, created_at) VALUES (?, ?, ?, ?, 'harvest', ?, ?)")
            .bind(new_id())
            .bind(species)
            .bind(province)
            .bind(p)
            .bind(date_or_today(b, "harvest_date"))
            .bind(now_iso())
            .execute(&st.db)
            .await?;
    }
    Ok(id)
}
pub async fn list_harvests(State(st): State<AppState>, user: AuthUser, Path(crop_id): Path<String>) -> ApiResult<Json<Value>> {
    let farm_id = farm_of_crop(&st, &crop_id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    let rows = sqlx::query("SELECT * FROM harvests WHERE crop_id = ? ORDER BY harvest_date DESC").bind(&crop_id).fetch_all(&st.db).await?;
    Ok(Json(json!(rows_to_json(&rows))))
}

pub async fn create_treatment(State(st): State<AppState>, user: AuthUser, Path(crop_id): Path<String>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    let farm_id = farm_of_crop(&st, &crop_id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    let id = insert_treatment(&st, &crop_id, &b).await?;
    Ok(Json(json!({ "id": id })))
}
pub async fn insert_treatment(st: &AppState, crop_id: &str, b: &Value) -> ApiResult<String> {
    let product = s(b, "product").ok_or_else(|| AppError::BadRequest("กรอกชื่อยา/สารที่ใช้".into()))?;
    let id = new_id();
    sqlx::query("INSERT INTO treatments (id, client_id, crop_id, start_date, end_date, product, dose, withdrawal_days, symptom, note, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(s(b, "client_id"))
        .bind(crop_id)
        .bind(date_or_today(b, "start_date"))
        .bind(s(b, "end_date"))
        .bind(product)
        .bind(s(b, "dose"))
        .bind(i(b, "withdrawal_days").unwrap_or(0))
        .bind(s(b, "symptom"))
        .bind(s(b, "note"))
        .bind(now_iso())
        .execute(&st.db)
        .await?;
    Ok(id)
}
pub async fn list_treatments(State(st): State<AppState>, user: AuthUser, Path(crop_id): Path<String>) -> ApiResult<Json<Value>> {
    let farm_id = farm_of_crop(&st, &crop_id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    let rows = sqlx::query("SELECT * FROM treatments WHERE crop_id = ? ORDER BY start_date DESC").bind(&crop_id).fetch_all(&st.db).await?;
    Ok(Json(json!(rows_to_json(&rows))))
}

// ---------- ราคาตลาด / รายงานโรค ----------

pub async fn list_prices(State(st): State<AppState>, Query(q): Query<Value>) -> ApiResult<Json<Value>> {
    let species = q.get("species").and_then(|v| v.as_str()).unwrap_or("nile_tilapia");
    let province = q.get("province").and_then(|v| v.as_str());
    let rows = match province {
        Some(p) => sqlx::query("SELECT * FROM market_prices WHERE species_code = ? AND province = ? ORDER BY price_date DESC LIMIT 60").bind(species).bind(p).fetch_all(&st.db).await?,
        None => sqlx::query("SELECT * FROM market_prices WHERE species_code = ? ORDER BY price_date DESC LIMIT 60").bind(species).fetch_all(&st.db).await?,
    };
    let latest = rows.first().map(row_to_json);
    let avg30: Option<f64> = sqlx::query("SELECT AVG(price_per_kg) AS a FROM market_prices WHERE species_code = ? AND price_date >= date('now','-30 days')").bind(species).fetch_one(&st.db).await?.get("a");
    Ok(Json(json!({ "latest": latest, "avg_30d": avg30, "history": rows_to_json(&rows) })))
}

pub async fn create_price(State(st): State<AppState>, user: AuthUser, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    let price = req_f(&b, "price_per_kg", "ราคา")?;
    let id = new_id();
    sqlx::query("INSERT INTO market_prices (id, species_code, province, price_per_kg, size_note, source, reported_by, price_date, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(s(&b, "species_code").unwrap_or_else(|| "nile_tilapia".into()))
        .bind(s(&b, "province"))
        .bind(price)
        .bind(s(&b, "size_note"))
        .bind(if user.is_staff() { "officer" } else { "farmer" })
        .bind(&user.id)
        .bind(date_or_today(&b, "price_date"))
        .bind(now_iso())
        .execute(&st.db)
        .await?;
    Ok(Json(json!({ "id": id })))
}

pub async fn create_disease_report(State(st): State<AppState>, user: AuthUser, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    let symptom = s(&b, "symptom").ok_or_else(|| AppError::BadRequest("กรอกอาการ".into()))?;
    let farm_id = s(&b, "farm_id");
    let (province, district, lat, lng) = match &farm_id {
        Some(fid) => {
            assert_farm_access(&st, &user, fid).await?;
            let r = sqlx::query("SELECT province, district, lat, lng FROM farms WHERE id = ?").bind(fid).fetch_one(&st.db).await?;
            (r.get::<Option<String>, _>("province"), r.get::<Option<String>, _>("district"), r.get::<Option<f64>, _>("lat"), r.get::<Option<f64>, _>("lng"))
        }
        None => (s(&b, "province"), s(&b, "district"), f(&b, "lat"), f(&b, "lng")),
    };
    let id = new_id();
    sqlx::query("INSERT INTO disease_reports (id, farm_id, province, district, lat, lng, species_code, symptom, severity, report_date, note, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(farm_id)
        .bind(province)
        .bind(district)
        .bind(lat)
        .bind(lng)
        .bind(s(&b, "species_code"))
        .bind(symptom)
        .bind(s(&b, "severity").unwrap_or_else(|| "medium".into()))
        .bind(date_or_today(&b, "report_date"))
        .bind(s(&b, "note"))
        .bind(now_iso())
        .execute(&st.db)
        .await?;
    Ok(Json(json!({ "id": id })))
}

/// รายงานโรคในพื้นที่ 30 วันล่าสุด แบบไม่ระบุฟาร์ม (พิกัดปัดเป็น 0.1 องศา ~ 10 กม.)
pub async fn list_disease_reports(State(st): State<AppState>, Query(q): Query<Value>) -> ApiResult<Json<Value>> {
    let province = q.get("province").and_then(|v| v.as_str());
    let rows = match province {
        Some(p) => sqlx::query("SELECT province, district, ROUND(lat,1) AS lat, ROUND(lng,1) AS lng, species_code, symptom, severity, report_date FROM disease_reports WHERE province = ? AND report_date >= date('now','-30 days') ORDER BY report_date DESC LIMIT 200").bind(p).fetch_all(&st.db).await?,
        None => sqlx::query("SELECT province, district, ROUND(lat,1) AS lat, ROUND(lng,1) AS lng, species_code, symptom, severity, report_date FROM disease_reports WHERE report_date >= date('now','-30 days') ORDER BY report_date DESC LIMIT 200").fetch_all(&st.db).await?,
    };
    Ok(Json(json!(rows_to_json(&rows))))
}

// ---------- sync ออฟไลน์ ----------

#[derive(Deserialize)]
pub struct SyncOp {
    pub op: String,
    pub target_id: String,
    pub body: Value,
    pub client_id: Option<String>,
}

/// รับชุดคำสั่งจากคิวออฟไลน์ ทำทีละรายการ ตอบผลรายรายการ (client_id ซ้ำ = ข้ามโดยไม่ error)
pub async fn sync(State(st): State<AppState>, user: AuthUser, Json(ops): Json<Vec<SyncOp>>) -> ApiResult<Json<Value>> {
    let mut results = Vec::new();
    for op in ops {
        let mut body = op.body.clone();
        if let Some(cid) = &op.client_id {
            body["client_id"] = json!(cid);
            let dup = sqlx::query(
                "SELECT 1 FROM daily_logs WHERE client_id = ?1 UNION SELECT 1 FROM weighings WHERE client_id = ?1 UNION SELECT 1 FROM water_quality WHERE client_id = ?1 UNION SELECT 1 FROM feed_stock_moves WHERE client_id = ?1 UNION SELECT 1 FROM expenses WHERE client_id = ?1 UNION SELECT 1 FROM harvests WHERE client_id = ?1 UNION SELECT 1 FROM treatments WHERE client_id = ?1",
            )
            .bind(cid)
            .fetch_optional(&st.db)
            .await?;
            if dup.is_some() && op.op != "log" {
                results.push(json!({ "client_id": cid, "status": "duplicate" }));
                continue;
            }
        }
        let r: ApiResult<String> = async {
            match op.op.as_str() {
                "log" => {
                    let farm = farm_of_crop(&st, &op.target_id).await?;
                    assert_farm_access(&st, &user, &farm).await?;
                    insert_log(&st, &user, &op.target_id, &farm, &body).await
                }
                "weighing" => {
                    assert_farm_access(&st, &user, &farm_of_crop(&st, &op.target_id).await?).await?;
                    insert_weighing(&st, &op.target_id, &body).await
                }
                "water" => {
                    assert_farm_access(&st, &user, &farm_of_pond(&st, &op.target_id).await?).await?;
                    let crop: Option<String> = sqlx::query("SELECT id FROM crops WHERE pond_id = ? AND status = 'active'").bind(&op.target_id).fetch_optional(&st.db).await?.map(|r| r.get("id"));
                    insert_water(&st, &op.target_id, crop.as_deref(), &body).await
                }
                "stock" => {
                    assert_farm_access(&st, &user, &op.target_id).await?;
                    insert_stock_move(&st, &op.target_id, &body).await
                }
                "expense" => {
                    assert_farm_access(&st, &user, &farm_of_crop(&st, &op.target_id).await?).await?;
                    insert_expense(&st, &op.target_id, &body).await
                }
                "harvest" => {
                    assert_farm_access(&st, &user, &farm_of_crop(&st, &op.target_id).await?).await?;
                    insert_harvest(&st, &op.target_id, &body).await
                }
                "treatment" => {
                    assert_farm_access(&st, &user, &farm_of_crop(&st, &op.target_id).await?).await?;
                    insert_treatment(&st, &op.target_id, &body).await
                }
                other => Err(AppError::BadRequest(format!("ไม่รู้จักคำสั่ง {other}"))),
            }
        }
        .await;
        match r {
            Ok(id) => results.push(json!({ "client_id": op.client_id, "status": "ok", "id": id })),
            Err(e) => results.push(json!({ "client_id": op.client_id, "status": "error", "error": e.to_string() })),
        }
    }
    Ok(Json(json!({ "results": results })))
}

// ---------- ส่งออก ----------

pub async fn export_crop_csv(State(st): State<AppState>, user: AuthUser, Path(crop_id): Path<String>) -> ApiResult<axum::response::Response> {
    use axum::response::IntoResponse;
    let farm_id = farm_of_crop(&st, &crop_id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    let rows = sqlx::query("SELECT log_date, recommended_kg, fed_kg, factor, mortality, feeding_response, note FROM daily_logs WHERE crop_id = ? ORDER BY log_date").bind(&crop_id).fetch_all(&st.db).await?;
    let mut csv = String::from("\u{feff}วันที่,อาหารแนะนำ (กก.),ให้จริง (กก.),ตัวปรับ,ตาย (ตัว),การกิน,หมายเหตุ\n");
    for r in rows {
        let v = row_to_json(&r);
        let cell = |k: &str| v.get(k).map(|x| match x { Value::String(s) => s.replace(',', " "), Value::Null => String::new(), o => o.to_string() }).unwrap_or_default();
        csv.push_str(&format!("{},{},{},{},{},{},{}\n", cell("log_date"), cell("recommended_kg"), cell("fed_kg"), cell("factor"), cell("mortality"), cell("feeding_response"), cell("note")));
    }
    Ok((
        [(axum::http::header::CONTENT_TYPE, "text/csv; charset=utf-8"), (axum::http::header::CONTENT_DISPOSITION, "attachment; filename=\"crop-log.csv\"")],
        csv,
    )
        .into_response())
}
