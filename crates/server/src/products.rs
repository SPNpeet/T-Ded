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

/// ซิงก์แคตตาล็อกกลางจากไฟล์ทุกครั้งที่เริ่มระบบ (แถวที่หน่วยงานแก้เองจะไม่ถูกทับ)
pub async fn seed_if_empty(st: &AppState) -> ApiResult<()> {
    let items: Vec<Value> = serde_json::from_str(SEED)?;
    let mut n = 0;
    for it in items {
        let code = it.get("product_code").and_then(|v| v.as_str()).unwrap_or("");
        let brand = it.get("brand").and_then(|v| v.as_str()).unwrap_or("");
        let id = format!("seed:{brand}:{code}");
        let existing = sqlx::query("SELECT id, updated_by FROM feed_products WHERE id = ?").bind(&id).fetch_optional(&st.db).await?;
        match existing {
            // เคยมีและยังไม่มีใครแก้ -> อัปเดตให้ตรงไฟล์ล่าสุด
            Some(r) if r.get::<Option<String>, _>("updated_by").is_none() => {
                update_from_seed(st, &id, &it).await?;
                n += 1;
            }
            Some(_) => {}
            None => {
                insert_product_with_id(st, &id, None, &it, None).await?;
                n += 1;
            }
        }
    }
    // ลบแถวที่ค้างจากการ seed แบบเก่า (id เป็น uuid และไม่มีคนแก้)
    sqlx::query("DELETE FROM feed_products WHERE org_id IS NULL AND updated_by IS NULL AND id NOT LIKE 'seed:%'").execute(&st.db).await?;
    tracing::info!(synced = n, "feed products catalog synced");
    Ok(())
}

async fn update_from_seed(st: &AppState, id: &str, b: &Value) -> ApiResult<()> {
    let s = |k: &str| b.get(k).and_then(|v| v.as_str()).map(|x| x.to_string());
    let f = |k: &str| b.get(k).and_then(|v| v.as_f64());
    sqlx::query("UPDATE feed_products SET brand = ?, product_code = ?, name_th = ?, target = ?, stage_th = ?, weight_from_g = ?, weight_to_g = ?, protein_pct = ?, fat_pct = ?, pellet_mm = ?, form = ?, bag_kg = ?, price_ref = ?, source_url = ?, verified = ?, note = ?, active = 1, updated_at = ? WHERE id = ?")
        .bind(s("brand")).bind(s("product_code")).bind(s("name_th")).bind(s("target").unwrap_or_else(|| "all".into())).bind(s("stage_th"))
        .bind(f("weight_from_g")).bind(f("weight_to_g")).bind(f("protein_pct")).bind(f("fat_pct")).bind(f("pellet_mm"))
        .bind(s("form")).bind(f("bag_kg")).bind(f("price_ref")).bind(s("source_url"))
        .bind(b.get("verified").and_then(|v| v.as_i64().or_else(|| v.as_bool().map(|x| x as i64))).unwrap_or(0))
        .bind(s("note")).bind(now_iso()).bind(id)
        .execute(&st.db)
        .await?;
    Ok(())
}

async fn insert_product(st: &AppState, org_id: Option<&str>, b: &Value, user: Option<&str>) -> ApiResult<String> {
    insert_product_with_id(st, &new_id(), org_id, b, user).await
}

async fn insert_product_with_id(st: &AppState, id: &str, org_id: Option<&str>, b: &Value, user: Option<&str>) -> ApiResult<String> {
    let id = id.to_string();
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
