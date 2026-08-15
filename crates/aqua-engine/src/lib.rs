//! aqua-engine: กฎการเลี้ยงปลาน้ำจืดทั้งหมดอยู่ที่นี่ที่เดียว
//! ใช้ทั้งฝั่ง server และคอมไพล์เป็น WASM ให้แอปใช้ออฟไลน์ ผลลัพธ์ต้องตรงกัน
//!
//! โมดูล
//! - species  : โปรไฟล์ชนิดปลา ตารางอัตราให้อาหาร ตารางการเจริญเติบโตมาตรฐาน เกณฑ์น้ำ
//! - feed     : คำนวณอาหารต่อวัน (interpolate ตาราง + ตัวปรับสภาพแวดล้อม)
//! - env      : กติกาปรับตามอากาศ/น้ำ/พฤติกรรมปลา (data-driven แก้ได้จาก DB)
//! - growth   : เทียบน้ำหนักจริงกับมาตรฐาน ADG
//! - perf     : FCR อัตรารอด ต้นทุนต่อกิโล
//! - forecast : พยากรณ์รุ่น (จับเมื่อไหร่ ใช้อาหารเท่าไหร่ กำไรเท่าไหร่) และจำลองก่อนลงเลี้ยง
//! - water    : ประเมินคุณภาพน้ำรายค่า พร้อมคำแนะนำ
//! - health   : คะแนนสุขภาพบ่อ 0-100

pub mod env;
pub mod feed;
pub mod forecast;
pub mod growth;
pub mod health;
pub mod perf;
pub mod species;
pub mod water;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use env::{AdjustRule, EnvInput, Metric, Op, RuleGroup, StressLevel};
pub use feed::{recommend, FeedInput, Recommendation};
pub use forecast::{project, simulate, Projection, ProjectionDay, ProjectionInput, SimulationInput};
pub use growth::{compare_growth, GrowthCompare};
pub use health::{health_score, HealthInput, HealthScore};
pub use perf::{performance, PerfInput, Performance};
pub use species::{FeedRateRow, GrowthRow, SpeciesProfile, WaterThresholds};
pub use water::{assess_water, WaterAssessment, WaterSample};

pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// ปัดทศนิยม n ตำแหน่ง ใช้ให้ตัวเลขฝั่ง server และ WASM ตรงกันเป๊ะ
pub(crate) fn round(v: f64, places: i32) -> f64 {
    let m = 10f64.powi(places);
    (v * m).round() / m
}
