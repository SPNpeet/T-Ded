//! เครื่องคำนวณสาธารณะ (ไม่ต้องล็อกอิน) ใช้ engine ตัวเดียวกับหน้าฟาร์ม

use aqua_engine::{compare_growth, health_score, recommend, simulate, FeedInput, HealthInput, SimulationInput, SpeciesProfile};
use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::{error::{ApiResult, AppError}, AppState};

pub async fn species(State(_st): State<AppState>) -> Json<Value> {
    Json(json!(SpeciesProfile::defaults()))
}

pub async fn rules(State(_st): State<AppState>) -> Json<Value> {
    Json(json!(aqua_engine::env::default_rules()))
}

/// body: { species_code, avg_weight_g, count, env?, meals_per_day?, farm_factor? }
pub async fn calc_recommend(State(_st): State<AppState>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    let code = b.get("species_code").and_then(|v| v.as_str()).unwrap_or("nile_tilapia");
    let sp = SpeciesProfile::by_code(code).ok_or_else(|| AppError::BadRequest("ไม่รู้จักชนิดปลา".into()))?;
    let input = FeedInput {
        species: sp,
        avg_weight_g: b.get("avg_weight_g").and_then(|v| v.as_f64()).ok_or_else(|| AppError::BadRequest("กรอกน้ำหนักปลา".into()))?,
        count: b.get("count").and_then(|v| v.as_f64()).ok_or_else(|| AppError::BadRequest("กรอกจำนวนปลา".into()))?,
        env: match b.get("env") {
            Some(v) if !v.is_null() => Some(serde_json::from_value(v.clone())?),
            _ => None,
        },
        rules: vec![],
        meals_per_day: b.get("meals_per_day").and_then(|v| v.as_u64()).map(|m| m as u8),
        farm_factor: b.get("farm_factor").and_then(|v| v.as_f64()),
    };
    Ok(Json(json!(recommend(&input))))
}

pub async fn calc_simulate(State(_st): State<AppState>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    let code = b.get("species_code").and_then(|v| v.as_str()).unwrap_or("nile_tilapia");
    let sp = SpeciesProfile::by_code(code).ok_or_else(|| AppError::BadRequest("ไม่รู้จักชนิดปลา".into()))?;
    let g = |k: &str| b.get(k).and_then(|v| v.as_f64());
    let input = SimulationInput {
        species: sp,
        count: g("count").ok_or_else(|| AppError::BadRequest("กรอกจำนวนปลา".into()))?,
        stock_weight_g: g("stock_weight_g").unwrap_or(30.0),
        target_weight_g: g("target_weight_g"),
        target_days: g("target_days").map(|d| d as u32),
        expected_survival_pct: g("expected_survival_pct").unwrap_or(85.0),
        fry_price_each: g("fry_price_each").unwrap_or(0.0),
        feed_price_per_kg: g("feed_price_per_kg").unwrap_or(0.0),
        other_cost_per_day: g("other_cost_per_day").unwrap_or(0.0),
        fixed_cost: g("fixed_cost").unwrap_or(0.0),
        sell_price_per_kg: g("sell_price_per_kg").unwrap_or(0.0),
        growth_scale: g("growth_scale"),
        avg_feed_factor: g("avg_feed_factor"),
        bag_kg: g("bag_kg"),
    };
    Ok(Json(json!(simulate(&input))))
}

pub async fn calc_growth(State(_st): State<AppState>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    let code = b.get("species_code").and_then(|v| v.as_str()).unwrap_or("nile_tilapia");
    let sp = SpeciesProfile::by_code(code).ok_or_else(|| AppError::BadRequest("ไม่รู้จักชนิดปลา".into()))?;
    let g = |k: &str| b.get(k).and_then(|v| v.as_f64());
    let prev = match (g("prev_day"), g("prev_weight_g")) {
        (Some(d), Some(w)) => Some((d as u32, w)),
        _ => None,
    };
    Ok(Json(json!(compare_growth(
        &sp,
        g("stock_weight_g").unwrap_or(30.0),
        g("day").unwrap_or(0.0) as u32,
        g("actual_g").ok_or_else(|| AppError::BadRequest("กรอกน้ำหนักจริง".into()))?,
        prev,
        g("target_g"),
    ))))
}

pub async fn calc_health(State(_st): State<AppState>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    let code = b.get("species_code").and_then(|v| v.as_str()).unwrap_or("nile_tilapia");
    let sp = SpeciesProfile::by_code(code).ok_or_else(|| AppError::BadRequest("ไม่รู้จักชนิดปลา".into()))?;
    let input: HealthInput = serde_json::from_value(b.get("input").cloned().unwrap_or(json!({})))?;
    Ok(Json(json!(health_score(&input, &sp.water))))
}

pub async fn nutrition_stages(State(_st): State<AppState>, axum::extract::Path(code): axum::extract::Path<String>) -> Json<Value> {
    Json(json!({ "stages": aqua_engine::stages_for(&code), "tips": aqua_engine::feed_tips().into_iter().map(|(a, b)| json!({ "title": a, "body": b })).collect::<Vec<_>>() }))
}

pub async fn nutrition_ingredients(State(_st): State<AppState>) -> Json<Value> {
    Json(json!(aqua_engine::default_ingredients()))
}

/// body: { ingredients: [...], batch_kg? } หรือ { pearson: { protein_a, protein_b, target } }
pub async fn calc_mix(State(_st): State<AppState>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    if let Some(p) = b.get("pearson") {
        let g = |k: &str| p.get(k).and_then(|v| v.as_f64()).unwrap_or(0.0);
        return Ok(Json(json!({ "pearson": aqua_engine::pearson_square(g("protein_a"), g("protein_b"), g("target")) })));
    }
    let list: Vec<aqua_engine::Ingredient> = serde_json::from_value(b.get("ingredients").cloned().unwrap_or(json!([])))?;
    Ok(Json(json!(aqua_engine::feed_mix(&list, b.get("batch_kg").and_then(|v| v.as_f64())))))
}
