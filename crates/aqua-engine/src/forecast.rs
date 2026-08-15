use serde::{Deserialize, Serialize};

use crate::species::SpeciesProfile;

/// พยากรณ์รุ่นที่เลี้ยงอยู่ต่อจากวันนี้ไปจนถึงเป้าหมาย
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionInput {
    pub species: SpeciesProfile,
    /// วันที่เลี้ยงมาแล้ว (0 = ยังไม่เริ่ม)
    pub day: u32,
    pub avg_weight_g: f64,
    pub alive_count: f64,
    /// อัตราตายต่อวัน (สัดส่วน เช่น 0.0005) ถ้าไม่รู้ใส่ 0
    pub daily_mortality_rate: f64,
    /// น้ำหนักเป้าหมาย (ก.) ถ้า None ใช้ market_weight ของชนิดปลา
    pub target_weight_g: Option<f64>,
    /// หรือกำหนดจำนวนวันที่จะเลี้ยงต่อ (ถ้าใส่จะหยุดตามนี้)
    pub target_days: Option<u32>,
    /// ตัวคูณการโตเทียบมาตรฐาน (1.0 = ตามเกณฑ์, 0.9 = ช้ากว่า 10%)
    pub growth_scale: f64,
    /// ตัวปรับอาหารเฉลี่ยที่คาดไว้ (1.0 = ตามตาราง)
    pub avg_feed_factor: f64,
    /// ราคาอาหารต่อ กก. (บาท)
    pub feed_price_per_kg: f64,
    /// ต้นทุนอื่นต่อวัน (ไฟ แรงงาน ฯลฯ)
    pub other_cost_per_day: f64,
    /// ต้นทุนที่จ่ายไปแล้วถึงวันนี้
    pub cost_so_far: f64,
    pub feed_kg_so_far: f64,
    /// ราคาขายคาดการณ์ต่อ กก.
    pub sell_price_per_kg: f64,
    /// กก. ต่อกระสอบ (ค่าเริ่มต้น 20)
    pub bag_kg: Option<f64>,
    /// เพดานวันจำลอง (ค่าเริ่มต้น 400)
    pub max_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionDay {
    pub day: u32,
    pub avg_weight_g: f64,
    pub alive_count: f64,
    pub biomass_kg: f64,
    pub feed_kg_day: f64,
    pub feed_kg_cum: f64,
    pub cost_cum: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Projection {
    /// วันที่จะถึงเป้าหมาย (นับจากวันปล่อย)
    pub harvest_day: u32,
    /// อีกกี่วันจากวันนี้
    pub days_remaining: u32,
    pub final_avg_weight_g: f64,
    pub final_count: f64,
    pub final_biomass_kg: f64,
    pub survival_pct: f64,
    /// อาหารที่ต้องใช้ต่อจากนี้
    pub feed_kg_remaining: f64,
    pub feed_bags_remaining: f64,
    pub feed_kg_total: f64,
    pub feed_cost_remaining: f64,
    pub cost_remaining: f64,
    pub cost_total: f64,
    pub revenue: f64,
    pub profit: f64,
    /// ราคาขั้นต่ำต่อ กก. ที่เท่าทุน
    pub breakeven_price_per_kg: f64,
    pub cost_per_kg: f64,
    pub projected_fcr: Option<f64>,
    pub roi_pct: Option<f64>,
    /// จุดตัวอย่างสำหรับกราฟ (ทุก 7 วัน + วันสุดท้าย)
    pub curve: Vec<ProjectionDay>,
    pub reached_target: bool,
}

pub fn project(i: &ProjectionInput) -> Projection {
    let sp = &i.species;
    let target_w = i.target_weight_g.unwrap_or(sp.market_weight_g);
    let max_days = i.max_days.unwrap_or(400);
    let bag = i.bag_kg.unwrap_or(20.0).max(0.1);
    let growth_scale = if i.growth_scale > 0.0 { i.growth_scale } else { 1.0 };
    let feed_factor = if i.avg_feed_factor > 0.0 { i.avg_feed_factor } else { 1.0 };

    let mut day = i.day;
    let mut w = i.avg_weight_g.max(0.1);
    let mut curve_day = sp.standard_day_for_weight(w);
    let mut alive = i.alive_count.max(0.0);
    let mut feed_cum = 0.0;
    let mut cost_cum = 0.0;
    let mut curve = Vec::new();
    let mut reached = false;
    let mut last_feed_day = 0.0;
    let stop_day = i.target_days.map(|d| i.day + d);

    let push = |curve: &mut Vec<ProjectionDay>, day: u32, w: f64, alive: f64, fd: f64, fc: f64, cc: f64| {
        curve.push(ProjectionDay {
            day,
            avg_weight_g: crate::round(w, 1),
            alive_count: alive.round(),
            biomass_kg: crate::round(alive * w / 1000.0, 1),
            feed_kg_day: crate::round(fd, 2),
            feed_kg_cum: crate::round(fc, 1),
            cost_cum: crate::round(cc, 0),
        })
    };
    push(&mut curve, day, w, alive, 0.0, feed_cum, cost_cum);

    let mut steps = 0u32;
    loop {
        if let Some(sd) = stop_day {
            if day >= sd {
                reached = w >= target_w;
                break;
            }
        } else if w >= target_w {
            reached = true;
            break;
        }
        if steps >= max_days {
            break;
        }
        let pct = sp.feed_pct(w);
        let feed_day = alive * w / 1000.0 * pct / 100.0 * feed_factor;
        feed_cum += feed_day;
        cost_cum += feed_day * i.feed_price_per_kg + i.other_cost_per_day;
        // เดินไปตามเส้นโค้งมาตรฐานโดยตรง (growth_scale = ความเร็วเทียบเกณฑ์) แม่นกว่าบวก ADG ทีละวัน
        curve_day += growth_scale;
        w = sp.standard_weight_at(curve_day).max(w);
        alive -= alive * i.daily_mortality_rate.max(0.0);
        day += 1;
        steps += 1;
        last_feed_day = feed_day;
        if steps % 7 == 0 {
            push(&mut curve, day, w, alive, feed_day, feed_cum, cost_cum);
        }
    }
    if curve.last().map(|c| c.day) != Some(day) {
        push(&mut curve, day, w, alive, last_feed_day, feed_cum, cost_cum);
    }

    let biomass = alive * w / 1000.0;
    let revenue = biomass * i.sell_price_per_kg;
    let cost_total = i.cost_so_far + cost_cum;
    let feed_total = i.feed_kg_so_far + feed_cum;
    let survival = if i.alive_count > 0.0 { alive / i.alive_count * 100.0 } else { 0.0 };
    let gain = biomass - i.alive_count * i.avg_weight_g / 1000.0;
    let projected_fcr = if gain > 0.0 { Some(crate::round(feed_cum / gain, 2)) } else { None };
    let roi = if cost_total > 0.0 { Some(crate::round((revenue - cost_total) / cost_total * 100.0, 1)) } else { None };

    Projection {
        harvest_day: day,
        days_remaining: day - i.day,
        final_avg_weight_g: crate::round(w, 1),
        final_count: alive.round(),
        final_biomass_kg: crate::round(biomass, 1),
        survival_pct: crate::round(survival, 1),
        feed_kg_remaining: crate::round(feed_cum, 1),
        feed_bags_remaining: crate::round(feed_cum / bag, 1),
        feed_kg_total: crate::round(feed_total, 1),
        feed_cost_remaining: crate::round(feed_cum * i.feed_price_per_kg, 0),
        cost_remaining: crate::round(cost_cum, 0),
        cost_total: crate::round(cost_total, 0),
        revenue: crate::round(revenue, 0),
        profit: crate::round(revenue - cost_total, 0),
        breakeven_price_per_kg: if biomass > 0.0 { crate::round(cost_total / biomass, 2) } else { 0.0 },
        cost_per_kg: if biomass > 0.0 { crate::round(cost_total / biomass, 2) } else { 0.0 },
        projected_fcr,
        roi_pct: roi,
        curve,
        reached_target: reached,
    }
}

/// จำลองก่อนลงเลี้ยง: ยังไม่มีข้อมูลจริง ใช้ตารางมาตรฐานทั้งหมด
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationInput {
    pub species: SpeciesProfile,
    pub count: f64,
    pub stock_weight_g: f64,
    pub target_weight_g: Option<f64>,
    pub target_days: Option<u32>,
    /// อัตรารอดที่คาดทั้งรุ่น (%) เช่น 85
    pub expected_survival_pct: f64,
    pub fry_price_each: f64,
    pub feed_price_per_kg: f64,
    pub other_cost_per_day: f64,
    /// ต้นทุนเตรียมบ่อ/ค่าเช่า/อื่น ๆ ครั้งเดียว
    pub fixed_cost: f64,
    pub sell_price_per_kg: f64,
    pub growth_scale: Option<f64>,
    pub avg_feed_factor: Option<f64>,
    pub bag_kg: Option<f64>,
}

pub fn simulate(s: &SimulationInput) -> Projection {
    // แปลงอัตรารอดทั้งรุ่นเป็นอัตราตายต่อวันโดยประมาณ ใช้ระยะเวลามาตรฐานถึงเป้าหมาย
    let target_w = s.target_weight_g.unwrap_or(s.species.market_weight_g);
    let d0 = s.species.standard_day_for_weight(s.stock_weight_g);
    let d1 = s.species.standard_day_for_weight(target_w);
    let est_days = s.target_days.map(|d| d as f64).unwrap_or((d1 - d0).max(1.0));
    let sr = (s.expected_survival_pct / 100.0).clamp(0.01, 1.0);
    let daily_mortality = 1.0 - sr.powf(1.0 / est_days);

    project(&ProjectionInput {
        species: s.species.clone(),
        day: 0,
        avg_weight_g: s.stock_weight_g,
        alive_count: s.count,
        daily_mortality_rate: daily_mortality,
        target_weight_g: Some(target_w),
        target_days: s.target_days,
        growth_scale: s.growth_scale.unwrap_or(1.0),
        avg_feed_factor: s.avg_feed_factor.unwrap_or(1.0),
        feed_price_per_kg: s.feed_price_per_kg,
        other_cost_per_day: s.other_cost_per_day,
        cost_so_far: s.fixed_cost + s.count * s.fry_price_each,
        feed_kg_so_far: 0.0,
        sell_price_per_kg: s.sell_price_per_kg,
        bag_kg: s.bag_kg,
        max_days: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_reaches_market_weight_in_reasonable_time() {
        let p = simulate(&SimulationInput {
            species: SpeciesProfile::nile_tilapia(),
            count: 5000.0,
            stock_weight_g: 30.0,
            target_weight_g: Some(800.0),
            target_days: None,
            expected_survival_pct: 85.0,
            fry_price_each: 2.0,
            feed_price_per_kg: 28.0,
            other_cost_per_day: 100.0,
            fixed_cost: 5000.0,
            sell_price_per_kg: 60.0,
            growth_scale: None,
            avg_feed_factor: None,
            bag_kg: None,
        });
        assert!(p.reached_target);
        assert!(p.harvest_day >= 115 && p.harvest_day <= 125, "day {}", p.harvest_day);
        assert!((p.survival_pct - 85.0).abs() < 1.5, "sr {}", p.survival_pct);
        assert!(p.feed_kg_remaining > 3000.0 && p.feed_kg_remaining < 7000.0, "feed {}", p.feed_kg_remaining);
        assert!(p.projected_fcr.unwrap() > 1.0 && p.projected_fcr.unwrap() < 2.2);
        assert!(!p.curve.is_empty());
    }

    #[test]
    fn projection_with_target_days() {
        let p = project(&ProjectionInput {
            species: SpeciesProfile::nile_tilapia(),
            day: 56,
            avg_weight_g: 300.0,
            alive_count: 4700.0,
            daily_mortality_rate: 0.0005,
            target_weight_g: None,
            target_days: Some(30),
            growth_scale: 1.0,
            avg_feed_factor: 0.95,
            feed_price_per_kg: 28.0,
            other_cost_per_day: 50.0,
            cost_so_far: 60000.0,
            feed_kg_so_far: 1500.0,
            sell_price_per_kg: 60.0,
            bag_kg: None,
            max_days: None,
        });
        assert_eq!(p.days_remaining, 30);
        assert_eq!(p.harvest_day, 86);
        assert!(!p.reached_target);
        assert!(p.final_avg_weight_g > 450.0);
    }
}
