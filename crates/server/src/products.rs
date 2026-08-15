//! แคตตาล็อกอาหารปลาที่ขายในไทย: seed จากไฟล์ + เจ้าหน้าที่/แอดมินแก้เพิ่มได้ + ผู้ใช้เลือกตอนรับเข้าสต๊อก

use axum::{extract::{Path, Query, State}, Json};
use serde_json::{json, Value};
use sqlx::Row;

use crate::{
    auth::AuthUser,
    db::{new_id, now_iso, rows_to_json},
    error::{ApiResult, AppError},
    AppState,
};

const SEED: &str = include_str!("../seed/feed_products.json");

/// ใส่ข้อมูลตั้งต้นครั้งเดียว (ถ้าตารางว่าง)
pub async fn seed_if_empty(st: &AppState) -> ApiResult<()> {
    let n: i64 = sqlx::query("SELECT COUNT(*) AS n FROM feed_products").fetch_one(&st.db).await?.get("n");
    if n > 0 {
        return Ok(());
    }
    let items: Vec<Value> = serde_json::from_str(SEED)?;
    for it in items {
        insert_product(st, None, &it, None).await?;
    }
    tracing::info!("feed products seeded");
    Ok(())
}

async fn insert_product(st: &AppState, org_id: Option<&str>, b: &Value, user: Option<&str>) -> ApiResult<String> {
    let id = new_id();
    let s = |k: &str| b.get(k).and_then(|v| v.as_str()).map(|x| x.to_string());
    let f = |k: &str| b.get(k).and_then(|v| v.as_f64());
    let brand = s("brand").ok_or_else(|| AppError::BadRequest("กรอกยี่ห้อ".into()))?;
    let name = s("name_th").ok_or_else(|| AppError::BadRequest("กรอกชื่อสินค้า".into()))?;
    sqlx::query("INSERT INTO feed_products (id, org_id, brand, product_code, name_th, target, stage_th, weight_from_g, weight_to_g, protein_pct, fat_pct, pellet_mm, form, bag_kg, price_ref, price_date, source_url, verified, active, note, updated_by, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?, ?)")
        .bind(&id)
        .bind(org_id)
        .bind(brand)
        .bind(s("product_code"))
        .bind(name)
        .bind(s("target").unwrap_or_else(|| "all".into()))
        .bind(s("stage_th"))
        .bind(f("weight_from_g"))
        .bind(f("weight_to_g"))
        .bind(f("protein_pct"))
        .bind(f("fat_pct"))
        .bind(f("pellet_mm"))
        .bind(s("form"))
        .bind(f("bag_kg"))
        .bind(f("price_ref"))
        .bind(s("price_date"))
        .bind(s("source_url"))
        .bind(b.get("verified").and_then(|v| v.as_i64().or_else(|| v.as_bool().map(|x| x as i64))).unwrap_or(0))
        .bind(s("note"))
        .bind(user)
        .bind(now_iso())
        .execute(&st.db)
        .await?;
    Ok(id)
}

/// รายการสินค้า (สาธารณะ): กรองตามชนิดปลาได้ ?target=tilapia|catfish|herbivore|carnivore ; species_code=nile_tilapia จะ map ให้
pub async fn list(State(st): State<AppState>, Query(q): Query<Value>) -> ApiResult<Json<Value>> {
    let target = q.get("target").and_then(|v| v.as_str()).map(String::from).or_else(|| {
        q.get("species_code").and_then(|v| v.as_str()).map(|c| match c {
            "nile_tilapia" | "red_tilapia" => "tilapia".to_string(),
            "catfish" => "catfish".to_string(),
            _ => "all".to_string(),
        })
    });
    let rows = match target.as_deref() {
        Some(t) if t != "all" => {
            // ปลานิล/ทับทิม กินอาหารกลุ่มปลากินพืชได้ด้วย
            let extra = if t == "tilapia" { "herbivore" } else { "" };
            sqlx::query("SELECT * FROM feed_products WHERE active = 1 AND (target = ? OR target = 'all' OR target = ?) ORDER BY brand, weight_from_g, name_th").bind(t).bind(extra).fetch_all(&st.db).await?
        }
        _ => sqlx::query("SELECT * FROM feed_products WHERE active = 1 ORDER BY brand, target, weight_from_g, name_th").fetch_all(&st.db).await?,
    };
    Ok(Json(json!(rows_to_json(&rows))))
}

pub async fn create(State(st): State<AppState>, user: AuthUser, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    if !user.is_staff() {
        return Err(AppError::Forbidden);
    }
    let id = insert_product(&st, Some(&user.org_id), &b, Some(&user.id)).await?;
    Ok(Json(json!({ "id": id })))
}

pub async fn update(State(st): State<AppState>, user: AuthUser, Path(id): Path<String>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    if !user.is_staff() {
        return Err(AppError::Forbidden);
    }
    let s = |k: &str| b.get(k).and_then(|v| v.as_str()).map(|x| x.to_string());
    let f = |k: &str| b.get(k).and_then(|v| v.as_f64());
    sqlx::query("UPDATE feed_products SET brand = COALESCE(?, brand), product_code = COALESCE(?, product_code), name_th = COALESCE(?, name_th), target = COALESCE(?, target), stage_th = COALESCE(?, stage_th), weight_from_g = COALESCE(?, weight_from_g), weight_to_g = COALESCE(?, weight_to_g), protein_pct = COALESCE(?, protein_pct), fat_pct = COALESCE(?, fat_pct), pellet_mm = COALESCE(?, pellet_mm), form = COALESCE(?, form), bag_kg = COALESCE(?, bag_kg), price_ref = COALESCE(?, price_ref), price_date = COALESCE(?, price_date), source_url = COALESCE(?, source_url), verified = COALESCE(?, verified), active = COALESCE(?, active), note = COALESCE(?, note), updated_by = ?, updated_at = ? WHERE id = ?")
        .bind(s("brand")).bind(s("product_code")).bind(s("name_th")).bind(s("target")).bind(s("stage_th"))
        .bind(f("weight_from_g")).bind(f("weight_to_g")).bind(f("protein_pct")).bind(f("fat_pct")).bind(f("pellet_mm"))
        .bind(s("form")).bind(f("bag_kg")).bind(f("price_ref")).bind(s("price_date")).bind(s("source_url"))
        .bind(b.get("verified").and_then(|v| v.as_i64().or_else(|| v.as_bool().map(|x| x as i64))))
        .bind(b.get("active").and_then(|v| v.as_i64().or_else(|| v.as_bool().map(|x| x as i64))))
        .bind(s("note")).bind(&user.id).bind(now_iso()).bind(&id)
        .execute(&st.db)
        .await?;
    Ok(Json(json!({ "ok": true })))
}

pub async fn remove(State(st): State<AppState>, user: AuthUser, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    if !user.is_staff() {
        return Err(AppError::Forbidden);
    }
    sqlx::query("UPDATE feed_products SET active = 0, updated_by = ?, updated_at = ? WHERE id = ?").bind(&user.id).bind(now_iso()).bind(&id).execute(&st.db).await?;
    Ok(Json(json!({ "ok": true })))
}
