use serde::{Deserialize, Serialize};

use crate::env::{compute_adjustment, default_rules, AdjustRule, EnvInput, Reason};
use crate::species::SpeciesProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedInput {
    pub species: SpeciesProfile,
    /// น้ำหนักเฉลี่ยต่อตัว (กรัม)
    pub avg_weight_g: f64,
    /// จำนวนปลาที่มีชีวิตในบ่อ
    pub count: f64,
    /// ข้อมูลสภาพแวดล้อม ถ้า None = ให้ตามมาตรฐาน
    pub env: Option<EnvInput>,
    /// กติกาปรับ ถ้าว่างใช้ default_rules()
    #[serde(default)]
    pub rules: Vec<AdjustRule>,
    /// จำนวนมื้อต่อวัน ถ้า None ใช้ค่าของชนิดปลา
    pub meals_per_day: Option<u8>,
    /// ตัวคูณที่ฟาร์มตั้งเอง (เช่น 0.9 ถ้าใช้อาหารโปรตีนสูง) ค่าเริ่มต้น 1
    pub farm_factor: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// % ของน้ำหนักตัวต่อวันตามตาราง
    pub pct: f64,
    pub pellet_mm: f64,
    /// ชีวมวลรวม (กก.)
    pub biomass_kg: f64,
    /// อาหารมาตรฐานก่อนปรับ (กก./วัน)
    pub base_kg: f64,
    pub factor: f64,
    /// อาหารที่แนะนำจริง (กก./วัน)
    pub final_kg: f64,
    pub meals_per_day: u8,
    pub per_meal_kg: f64,
    pub band: String,
    pub reasons: Vec<Reason>,
    pub temp_optimal: bool,
    /// คำเตือน เช่น นอกช่วงตาราง
    pub warnings: Vec<String>,
    /// ประโยคสรุปสั้นสำหรับหน้าแรก
    pub headline_th: String,
}

pub fn recommend(input: &FeedInput) -> Recommendation {
    let sp = &input.species;
    let w = input.avg_weight_g.max(0.0);
    let n = input.count.max(0.0);
    let mut warnings = Vec::new();

    if let (Some(first), Some(last)) = (sp.feed_table.first(), sp.feed_table.last()) {
        if w < first.weight_g {
            warnings.push(format!(
                "น้ำหนัก {} ก. ต่ำกว่าช่วงตาราง ({} ก.) ใช้ {}% โดยประมาณ",
                fmt(w),
                fmt(first.weight_g),
                first.pct
            ));
        } else if w > last.weight_g {
            warnings.push(format!(
                "น้ำหนัก {} ก. เกินช่วงตาราง ({} ก.) ใช้ {}% โดยประมาณ",
                fmt(w),
                fmt(last.weight_g),
                last.pct
            ));
        }
    }
    if sp.approximate {
        warnings.push(format!("ตารางของ{}เป็นค่าเริ่มต้นโดยประมาณ ควรปรับตามฟาร์ม", sp.name_th));
    }

    let pct = sp.feed_pct(w);
    let pellet = sp.pellet_mm(w);
    let biomass_kg = w * n / 1000.0;
    let base_kg = biomass_kg * pct / 100.0;

    let rules = if input.rules.is_empty() { default_rules() } else { input.rules.clone() };
    let temp_ideal = (sp.water.temp_ideal_min, sp.water.temp_ideal_max);
    let adj = match &input.env {
        Some(env) => compute_adjustment(env, &rules, temp_ideal),
        None => compute_adjustment(&EnvInput::default(), &rules, temp_ideal),
    };
    let farm_factor = input.farm_factor.unwrap_or(1.0).clamp(0.1, 2.0);
    let factor = crate::round(adj.factor * farm_factor, 3);
    let final_kg = crate::round(base_kg * factor, 2);
    let meals = input.meals_per_day.unwrap_or(sp.meals_per_day).max(1);
    let per_meal = crate::round(final_kg / meals as f64, 2);

    let pct_change = ((factor - 1.0) * 100.0).round() as i64;
    let headline = if input.env.is_none() {
        format!("ให้ {} กก. ตามมาตรฐาน", fmt2(final_kg))
    } else if pct_change == 0 {
        format!("ให้ {} กก. สภาพปกติ", fmt2(final_kg))
    } else if pct_change < 0 {
        format!("ให้ {} กก. ลดลง {}% จากมาตรฐาน", fmt2(final_kg), pct_change.abs())
    } else {
        format!("ให้ {} กก. เพิ่ม {}% จากมาตรฐาน", fmt2(final_kg), pct_change)
    };

    Recommendation {
        pct: crate::round(pct, 2),
        pellet_mm: pellet,
        biomass_kg: crate::round(biomass_kg, 2),
        base_kg: crate::round(base_kg, 2),
        factor,
        final_kg,
        meals_per_day: meals,
        per_meal_kg: per_meal,
        band: adj.band,
        reasons: adj.reasons,
        temp_optimal: adj.temp_optimal,
        warnings,
        headline_th: headline,
    }
}

fn fmt(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 {
        format!("{}", v.round() as i64)
    } else {
        format!("{:.1}", v)
    }
}
fn fmt2(v: f64) -> String {
    format!("{:.2}", v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::StressLevel;
    use approx::assert_abs_diff_eq;

    fn base(env: Option<EnvInput>) -> FeedInput {
        FeedInput {
            species: SpeciesProfile::nile_tilapia(),
            avg_weight_g: 300.0,
            count: 1000.0,
            env,
            rules: vec![],
            meals_per_day: None,
            farm_factor: None,
        }
    }

    #[test]
    fn prototype_standard() {
        let r = recommend(&base(None));
        assert_abs_diff_eq!(r.base_kg, 9.0);
        assert_abs_diff_eq!(r.final_kg, 9.0);
        assert_abs_diff_eq!(r.per_meal_kg, 4.5);
        assert_abs_diff_eq!(r.pellet_mm, 3.0);
        assert!(r.warnings.is_empty());
    }

    #[test]
    fn prototype_cold_morning() {
        let env = EnvInput { tmax_c: Some(30.0), tmin_c: Some(22.0), rain_mm: Some(0.0), cloud_pct: Some(40.0), ..Default::default() };
        let r = recommend(&base(Some(env)));
        assert_abs_diff_eq!(r.final_kg, 6.30);
        assert_eq!(r.band, "cut");
    }

    #[test]
    fn prototype_floor() {
        let env = EnvInput {
            tmax_c: Some(36.0),
            tmin_c: Some(19.0),
            rain_mm: Some(40.0),
            cloud_pct: Some(90.0),
            stress: StressLevel::Gasping,
            ..Default::default()
        };
        let r = recommend(&base(Some(env)));
        assert_abs_diff_eq!(r.final_kg, 3.60);
    }

    #[test]
    fn out_of_range_warns() {
        let mut i = base(None);
        i.avg_weight_g = 20.0;
        let r = recommend(&i);
        assert_eq!(r.warnings.len(), 1);
        assert_abs_diff_eq!(r.pct, 4.5);
    }
}
