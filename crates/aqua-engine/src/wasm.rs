//! WASM bindings: รับ/ส่ง JSON เพื่อให้แอปใช้ engine ตัวเดียวกับ server แบบออฟไลน์

use wasm_bindgen::prelude::*;

use crate::{
    env::default_rules,
    feed::{recommend, FeedInput},
    forecast::{project, simulate, ProjectionInput, SimulationInput},
    growth::compare_growth,
    health::{health_score, HealthInput},
    perf::{performance, PerfInput},
    species::SpeciesProfile,
    water::{assess_water, WaterSample},
};

fn to_js<T: serde::Serialize>(v: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(v).map_err(|e| JsValue::from_str(&e.to_string()))
}
fn from_js<T: for<'de> serde::Deserialize<'de>>(v: JsValue) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(v).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn engine_version() -> String {
    crate::ENGINE_VERSION.to_string()
}

#[wasm_bindgen]
pub fn species_defaults() -> Result<JsValue, JsValue> {
    to_js(&SpeciesProfile::defaults())
}

#[wasm_bindgen]
pub fn rules_default() -> Result<JsValue, JsValue> {
    to_js(&default_rules())
}

#[wasm_bindgen]
pub fn feed_recommend(input: JsValue) -> Result<JsValue, JsValue> {
    let i: FeedInput = from_js(input)?;
    to_js(&recommend(&i))
}

#[wasm_bindgen]
pub fn growth_compare(
    species: JsValue,
    stock_weight_g: f64,
    day: u32,
    actual_g: f64,
    prev_day: Option<u32>,
    prev_weight_g: Option<f64>,
    target_g: Option<f64>,
) -> Result<JsValue, JsValue> {
    let sp: SpeciesProfile = from_js(species)?;
    let prev = match (prev_day, prev_weight_g) {
        (Some(d), Some(w)) => Some((d, w)),
        _ => None,
    };
    to_js(&compare_growth(&sp, stock_weight_g, day, actual_g, prev, target_g))
}

#[wasm_bindgen]
pub fn perf_calc(input: JsValue) -> Result<JsValue, JsValue> {
    let i: PerfInput = from_js(input)?;
    to_js(&performance(&i))
}

#[wasm_bindgen]
pub fn forecast_project(input: JsValue) -> Result<JsValue, JsValue> {
    let i: ProjectionInput = from_js(input)?;
    to_js(&project(&i))
}

#[wasm_bindgen]
pub fn forecast_simulate(input: JsValue) -> Result<JsValue, JsValue> {
    let i: SimulationInput = from_js(input)?;
    to_js(&simulate(&i))
}

#[wasm_bindgen]
pub fn water_assess(sample: JsValue, species: JsValue) -> Result<JsValue, JsValue> {
    let s: WaterSample = from_js(sample)?;
    let sp: SpeciesProfile = from_js(species)?;
    to_js(&assess_water(&s, &sp.water))
}

#[wasm_bindgen]
pub fn pond_health(input: JsValue, species: JsValue) -> Result<JsValue, JsValue> {
    let i: HealthInput = from_js(input)?;
    let sp: SpeciesProfile = from_js(species)?;
    to_js(&health_score(&i, &sp.water))
}
