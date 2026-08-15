use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfInput {
    pub stocked_count: f64,
    pub stock_weight_g: f64,
    /// ปลาตายสะสมที่บันทึก
    pub dead_count: f64,
    /// ปลาที่จับออกไปแล้ว (ตัว, กก.)
    pub harvested_count: f64,
    pub harvested_kg: f64,
    /// น้ำหนักเฉลี่ยปัจจุบัน
    pub avg_weight_g: f64,
    /// อาหารที่ให้สะสม (กก.)
    pub feed_kg_total: f64,
    /// ต้นทุนสะสมทุกประเภท (บาท) รวมอาหาร ลูกปลา ยา ไฟ แรงงาน
    pub cost_total: f64,
    /// ต้นทุนอาหารสะสม (บาท)
    pub feed_cost_total: f64,
    /// รายได้จากการจับสะสม (บาท)
    pub revenue_total: f64,
    /// วันที่เลี้ยงมาแล้ว
    pub day: u32,
    /// ราคาขายต่อ กก. ณ วันนี้ (ถ้ามี) เพื่อตีมูลค่าปลาในบ่อ
    pub price_per_kg: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Performance {
    pub alive_count: f64,
    pub survival_pct: f64,
    pub biomass_kg: f64,
    pub initial_biomass_kg: f64,
    /// น้ำหนักที่ผลิตได้ (ชีวมวลตอนนี้ + จับแล้ว - ตอนปล่อย)
    pub gain_kg: f64,
    pub fcr: Option<f64>,
    /// เกรด FCR: excellent | good | fair | poor
    pub fcr_grade: Option<String>,
    pub fcr_grade_th: Option<String>,
    /// ต้นทุนต่อ กก. ที่ผลิตได้ (บาท)
    pub cost_per_kg: Option<f64>,
    pub feed_cost_per_kg: Option<f64>,
    /// มูลค่าปลาในบ่อวันนี้ (บาท) ถ้ามีราคา
    pub stock_value: Option<f64>,
    /// กำไรขาดทุนถ้าจับวันนี้ (มูลค่าปลา + รายได้แล้ว - ต้นทุนสะสม)
    pub profit_if_harvest_today: Option<f64>,
    pub adg_overall: f64,
    /// อัตราตายเฉลี่ยต่อวัน (สัดส่วน) ใช้ต่อในการพยากรณ์
    pub daily_mortality_rate: f64,
}

pub fn performance(i: &PerfInput) -> Performance {
    let alive = (i.stocked_count - i.dead_count - i.harvested_count).max(0.0);
    let survival = if i.stocked_count > 0.0 { (i.stocked_count - i.dead_count) / i.stocked_count * 100.0 } else { 0.0 };
    let biomass = alive * i.avg_weight_g / 1000.0;
    let initial = i.stocked_count * i.stock_weight_g / 1000.0;
    let gain = biomass + i.harvested_kg - initial;

    let fcr = if gain > 0.0 && i.feed_kg_total > 0.0 { Some(crate::round(i.feed_kg_total / gain, 2)) } else { None };
    let (grade, grade_th) = match fcr {
        Some(f) if f <= 1.3 => (Some("excellent"), Some("ดีเยี่ยม")),
        Some(f) if f <= 1.6 => (Some("good"), Some("ดี")),
        Some(f) if f <= 2.0 => (Some("fair"), Some("พอใช้")),
        Some(_) => (Some("poor"), Some("ต้องปรับปรุง")),
        None => (None, None),
    };

    let produced = biomass + i.harvested_kg;
    let cost_per_kg = if produced > 0.0 && i.cost_total > 0.0 { Some(crate::round(i.cost_total / produced, 2)) } else { None };
    let feed_cost_per_kg = if produced > 0.0 && i.feed_cost_total > 0.0 { Some(crate::round(i.feed_cost_total / produced, 2)) } else { None };
    let stock_value = i.price_per_kg.map(|p| crate::round(biomass * p, 0));
    let profit_today = stock_value.map(|v| crate::round(v + i.revenue_total - i.cost_total, 0));
    let adg = if i.day > 0 { (i.avg_weight_g - i.stock_weight_g) / i.day as f64 } else { 0.0 };
    let mortality_rate = if i.day > 0 && i.stocked_count > 0.0 { i.dead_count / i.stocked_count / i.day as f64 } else { 0.0 };

    Performance {
        alive_count: alive,
        survival_pct: crate::round(survival, 1),
        biomass_kg: crate::round(biomass, 1),
        initial_biomass_kg: crate::round(initial, 1),
        gain_kg: crate::round(gain, 1),
        fcr,
        fcr_grade: grade.map(String::from),
        fcr_grade_th: grade_th.map(String::from),
        cost_per_kg,
        feed_cost_per_kg,
        stock_value,
        profit_if_harvest_today: profit_today,
        adg_overall: crate::round(adg, 2),
        daily_mortality_rate: mortality_rate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn fcr_and_costs() {
        let p = performance(&PerfInput {
            stocked_count: 5000.0,
            stock_weight_g: 30.0,
            dead_count: 250.0,
            harvested_count: 0.0,
            harvested_kg: 0.0,
            avg_weight_g: 500.0,
            feed_kg_total: 3200.0,
            cost_total: 120_000.0,
            feed_cost_total: 96_000.0,
            revenue_total: 0.0,
            day: 84,
            price_per_kg: Some(60.0),
        });
        assert_abs_diff_eq!(p.alive_count, 4750.0);
        assert_abs_diff_eq!(p.survival_pct, 95.0);
        assert_abs_diff_eq!(p.biomass_kg, 2375.0);
        assert_abs_diff_eq!(p.gain_kg, 2225.0);
        assert_abs_diff_eq!(p.fcr.unwrap(), 1.44);
        assert_eq!(p.fcr_grade.as_deref(), Some("good"));
        assert_abs_diff_eq!(p.stock_value.unwrap(), 142_500.0);
        assert_abs_diff_eq!(p.profit_if_harvest_today.unwrap(), 22_500.0);
    }
}
