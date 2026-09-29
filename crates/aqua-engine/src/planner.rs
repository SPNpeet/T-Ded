//! วางแผนโปรแกรมการเลี้ยงตามเป้าหมาย
//!
//! เกษตรกรบอกว่า ปล่อยปลาขนาดเท่าไร กี่ตัว อยากได้ขนาดเท่าไร ภายในกี่วัน
//! ระบบจำลองรายวันว่าแต่ละช่วงควรใช้อาหารเบอร์ไหน ให้วันละเท่าไร กี่มื้อ ใช้กี่กระสอบ
//! และเทียบหลายโปรแกรม (โตเร็วสุด / ตามความต้องการ / ประหยัด / เลือกเอง) ให้เห็นเป็นตัวเลขว่าโตต่างกันเท่าไร

use serde::{Deserialize, Serialize};

use crate::nutrition::stage_for;
use crate::protein::{growth_factor, requirement_pct, ProteinResponse};
use crate::species::SpeciesProfile;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeedProduct {
    pub code: String,
    pub name_th: String,
    #[serde(default)]
    pub brand: String,
    /// tilapia | catfish | herbivore | carnivore | all
    pub target: String,
    pub weight_from_g: f64,
    pub weight_to_g: f64,
    pub protein_pct: f64,
    #[serde(default)]
    pub pellet_mm: Option<f64>,
    #[serde(default = "default_bag")]
    pub bag_kg: f64,
    /// ราคาต่อกระสอบ (บาท) ถ้ารู้
    #[serde(default)]
    pub price_per_bag: Option<f64>,
    #[serde(default)]
    pub color_th: Option<String>,
}

fn default_bag() -> f64 {
    20.0
}

impl FeedProduct {
    pub fn price_per_kg(&self) -> Option<f64> {
        self.price_per_bag.filter(|p| *p > 0.0).map(|p| p / self.bag_kg.max(0.1))
    }
}

/// กลุ่มอาหารที่ปลาชนิดนี้ใช้ได้ (ตามที่ผู้ผลิตระบุบนกระสอบ)
pub fn feed_groups(species_code: &str) -> &'static [&'static str] {
    match species_code {
        "nile_tilapia" | "red_tilapia" => &["tilapia", "herbivore", "all"],
        "catfish" => &["catfish", "all"],
        "snakehead" => &["carnivore", "all"],
        _ => &["all"],
    }
}

pub fn suits_species(p: &FeedProduct, species_code: &str) -> bool {
    feed_groups(species_code).contains(&p.target.as_str())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    /// เลือกเบอร์โปรตีนสูงสุดที่ใช้กับขนาดนั้นได้
    Fastest,
    /// เลือกเบอร์ที่โปรตีนพอดีความต้องการ (ไม่ต่ำกว่า) ใกล้ที่สุด
    Matched,
    /// เลือกเบอร์ที่ต้นทุนอาหารต่อกิโลเนื้อที่ได้ต่ำสุด
    Economical,
    /// ใช้เบอร์ที่ผู้ใช้เลือกเอง
    Custom,
}

impl Strategy {
    pub fn name_th(self) -> &'static str {
        match self {
            Strategy::Fastest => "โตเร็วสุด",
            Strategy::Matched => "ตามความต้องการของปลา",
            Strategy::Economical => "ประหยัดต้นทุน",
            Strategy::Custom => "เบอร์ที่เลือกเอง",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanRequest {
    pub species: SpeciesProfile,
    pub stock_weight_g: f64,
    pub count: f64,
    /// อัตรารอดที่คาดทั้งรุ่น (%)
    #[serde(default = "default_survival")]
    pub survival_pct: f64,
    /// น้ำหนักเป้าหมาย (ก./ตัว)
    pub target_weight_g: Option<f64>,
    /// ต้องได้ภายในกี่วัน
    pub target_days: Option<u32>,
    pub products: Vec<FeedProduct>,
    /// เบอร์ที่ผู้ใช้เลือกเอง (ใช้เรียงตามขนาด ถ้าเบอร์เดียวใช้ทั้งรุ่น)
    #[serde(default)]
    pub custom_codes: Vec<String>,
    #[serde(default)]
    pub response: Option<ProteinResponse>,
    /// ตัวคูณการโตของฟาร์มนี้เทียบมาตรฐาน (จากผลชั่งจริง) ค่าเริ่มต้น 1
    #[serde(default)]
    pub farm_growth_scale: Option<f64>,
    /// เพดานวันจำลอง (ค่าเริ่มต้น 420)
    #[serde(default)]
    pub max_days: Option<u32>,
}

fn default_survival() -> f64 {
    85.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStage {
    pub product_code: String,
    pub product_name_th: String,
    pub color_th: Option<String>,
    pub protein_pct: f64,
    pub pellet_mm: Option<f64>,
    pub from_day: u32,
    pub to_day: u32,
    pub from_weight_g: f64,
    pub to_weight_g: f64,
    pub feed_kg: f64,
    pub bags: f64,
    pub bag_kg: f64,
    pub cost: Option<f64>,
    pub meals_per_day: u8,
    pub feeding_times: Vec<String>,
    /// ความต้องการโปรตีนเฉลี่ยของปลาในช่วงนี้ และการโตเทียบมาตรฐาน
    pub requirement_pct: f64,
    pub growth_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDay {
    pub day: u32,
    pub weight_g: f64,
    pub alive: f64,
    pub feed_kg: f64,
    pub meals: u8,
    pub per_meal_kg: f64,
    pub product_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanResult {
    pub strategy: Strategy,
    pub strategy_th: String,
    pub stages: Vec<PlanStage>,
    pub days: Vec<PlanDay>,
    pub final_day: u32,
    pub final_weight_g: f64,
    pub final_count: f64,
    pub biomass_kg: f64,
    pub feed_kg_total: f64,
    pub bags_total: f64,
    pub cost_total: Option<f64>,
    pub fcr: Option<f64>,
    pub reached_target: bool,
    /// วันที่ถึงน้ำหนักเป้าหมาย (ถ้าถึง)
    pub day_reached: Option<u32>,
    /// ขาดจากเป้าหมายกี่กรัม ณ วันครบกำหนด
    pub shortfall_g: f64,
    pub warnings_th: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanComparison {
    pub plans: Vec<PlanResult>,
    /// ตำแหน่งแผนที่แนะนำใน plans
    pub recommended: usize,
    pub message_th: String,
    /// ขนาดสูงสุดที่เส้นการโตไปถึงได้ (ใช้บอกว่าเป้าเกินความเป็นไปได้)
    pub max_reachable_weight_g: f64,
}

/// เม็ดใหญ่สุดที่ปลาขนาดนี้กินได้ (มม.) = ขนาดเม็ดตามตาราง + 1 มม.
/// เม็ดเล็กกว่านี้กินได้เสมอ ส่วนช่วงน้ำหนักบนกระสอบใช้เป็นช่วงที่ผู้ผลิตแนะนำ ไม่ใช่ข้อห้าม
pub fn pellet_cap_mm(sp: &SpeciesProfile, w: f64) -> f64 {
    sp.pellet_mm(w) + 1.0
}

fn pellet_ok(sp: &SpeciesProfile, p: &FeedProduct, w: f64) -> bool {
    p.pellet_mm.map(|mm| mm <= pellet_cap_mm(sp, w) + 1e-9).unwrap_or(true)
}

/// เบอร์ที่ใช้กับปลาขนาดนี้ได้จริง: เม็ดไม่ใหญ่เกินปาก, ไม่เล็กจนเป็นผง/เม็ดจิ๋วสำหรับลูกปลา,
/// และปลาไม่โตเกินช่วงบนกระสอบไปมาก (เกินได้ถึง 3 เท่า) กันไม่ให้ใช้อาหารลูกปลาถุงเล็กเลี้ยงปลาใหญ่ทั้งรุ่น
fn usable(sp: &SpeciesProfile, p: &FeedProduct, w: f64) -> bool {
    let not_tiny = p.pellet_mm.map(|mm| mm + 1e-9 >= sp.pellet_mm(w) * 0.5).unwrap_or(true);
    pellet_ok(sp, p, w) && not_tiny && w < p.weight_to_g * 3.0
}

fn in_range(p: &FeedProduct, w: f64) -> bool {
    w >= p.weight_from_g && w < p.weight_to_g
}

/// เลือกเบอร์อาหารที่ใช้วันนี้ตามกลยุทธ์
fn pick<'a>(req: &'a PlanRequest, r: &ProteinResponse, strategy: Strategy, w: f64) -> Option<&'a FeedProduct> {
    let sp = &req.species;
    let code = sp.code.as_str();
    let need = requirement_pct(code, w);
    if strategy == Strategy::Custom {
        let custom: Vec<&FeedProduct> = req
            .custom_codes
            .iter()
            .filter_map(|c| req.products.iter().find(|p| &p.code == c))
            .collect();
        if custom.is_empty() {
            return None;
        }
        // ใช้ตามลำดับที่ผู้ใช้เลือก: เบอร์ที่ช่วงแนะนำตรงก่อน, ถัดมาเบอร์ที่เม็ดพอดีปาก, สุดท้ายเบอร์ที่ช่วงใกล้ที่สุด
        return custom
            .iter()
            .copied()
            .find(|p| in_range(p, w) && pellet_ok(sp, p, w))
            .or_else(|| custom.iter().copied().find(|p| pellet_ok(sp, p, w)))
            .or_else(|| custom.iter().copied().min_by(|a, b| dist(a, w).total_cmp(&dist(b, w))));
    }
    let suits: Vec<&FeedProduct> = req.products.iter().filter(|p| suits_species(p, code)).collect();
    let mut cands: Vec<&FeedProduct> = suits.iter().copied().filter(|p| usable(sp, p, w)).collect();
    if cands.is_empty() {
        // ปลายังเล็กเกินทุกเบอร์ ใช้เบอร์เม็ดเล็กสุดของชนิดนี้ (จะมีคำเตือนเรื่องเม็ดใหญ่เกิน)
        let smallest = suits.iter().copied().min_by(|a, b| a.pellet_mm.unwrap_or(0.0).total_cmp(&b.pellet_mm.unwrap_or(0.0)))?;
        cands.push(smallest);
    }
    let ranged: Vec<&FeedProduct> = cands.iter().copied().filter(|p| in_range(p, w)).collect();
    let pool = if ranged.is_empty() { cands.clone() } else { ranged.clone() };
    match strategy {
        // โปรตีนสูงสุดที่ปลาขนาดนี้กินได้ (ไม่จำกัดแค่ช่วงแนะนำ)
        Strategy::Fastest => cands.into_iter().max_by(|a, b| {
            a.protein_pct
                .total_cmp(&b.protein_pct)
                .then_with(|| in_range(a, w).cmp(&in_range(b, w)))
                .then_with(|| b.price_per_kg().unwrap_or(f64::MAX).total_cmp(&a.price_per_kg().unwrap_or(f64::MAX)))
        }),
        // โปรตีนถึงความต้องการด้วยเบอร์ที่โปรตีนต่ำสุด ไม่ถึงก็ใช้สูงสุดที่มี
        Strategy::Matched => {
            let enough = |v: &Vec<&'a FeedProduct>| -> Option<&'a FeedProduct> {
                v.iter().copied().filter(|p| p.protein_pct + 0.01 >= need).min_by(|a, b| a.protein_pct.total_cmp(&b.protein_pct))
            };
            enough(&pool).or_else(|| enough(&cands)).or_else(|| cands.iter().copied().max_by(|a, b| a.protein_pct.total_cmp(&b.protein_pct)))
        }
        // มีราคา: ต้นทุนต่อการโตต่ำสุด / ไม่มีราคา: เบอร์โปรตีนต่ำสุดในช่วงที่ผู้ผลิตแนะนำ (สูตรประหยัดของผู้ผลิต)
        Strategy::Economical => {
            if pool.iter().all(|p| p.price_per_kg().is_some()) {
                pool.into_iter().min_by(|a, b| cost_per_gain(a, r, need).total_cmp(&cost_per_gain(b, r, need)))
            } else {
                pool.into_iter().min_by(|a, b| a.protein_pct.total_cmp(&b.protein_pct))
            }
        }
        Strategy::Custom => unreachable!(),
    }
}

fn dist(p: &FeedProduct, w: f64) -> f64 {
    if w < p.weight_from_g {
        p.weight_from_g - w
    } else if w >= p.weight_to_g {
        w - p.weight_to_g
    } else {
        0.0
    }
}

/// ต้นทุนอาหารต่อหน่วยการโต (ยิ่งน้อยยิ่งคุ้ม) ถ้าไม่รู้ราคาใช้ความพอดีของโปรตีนแทน
fn cost_per_gain(p: &FeedProduct, r: &ProteinResponse, need: f64) -> f64 {
    let gf = growth_factor(r, p.protein_pct, need).max(0.01);
    match p.price_per_kg() {
        Some(price) => price / gf,
        None => (p.protein_pct - need).abs() + if p.protein_pct < need { 100.0 } else { 0.0 },
    }
}

fn run(req: &PlanRequest, r: &ProteinResponse, strategy: Strategy) -> Option<PlanResult> {
    let sp = &req.species;
    let code = sp.code.as_str();
    let target_w = req.target_weight_g.unwrap_or(sp.market_weight_g);
    let max_days = req.max_days.unwrap_or(420);
    let horizon = req.target_days.unwrap_or(max_days).min(max_days).max(1);
    let farm_scale = req.farm_growth_scale.filter(|x| *x > 0.0).unwrap_or(1.0);
    let survival = (req.survival_pct / 100.0).clamp(0.05, 1.0);
    // อัตราตายต่อวันคิดจากระยะเวลาที่คาดว่าจะเลี้ยง
    let expected_days = req.target_days.map(|d| d as f64).unwrap_or_else(|| {
        (sp.standard_day_for_weight(target_w) - sp.standard_day_for_weight(req.stock_weight_g)).max(30.0)
    });
    let daily_mort = 1.0 - survival.powf(1.0 / expected_days.max(1.0));

    let mut w = req.stock_weight_g.max(0.1);
    let mut curve_day = sp.standard_day_for_weight(w);
    let mut alive = req.count.max(0.0);
    let mut days: Vec<PlanDay> = Vec::new();
    let mut stages: Vec<PlanStage> = Vec::new();
    let mut feed_total = 0.0;
    let mut cost_total = 0.0;
    let mut cost_known = true;
    let mut day_reached: Option<u32> = None;
    let mut warnings: Vec<String> = Vec::new();
    let mut oversize_from: Option<(u32, String, f64, f64)> = None;

    // ถึงเป้าเมื่อไรก็จับขายเมื่อนั้น (ไม่ให้อาหารต่อจนครบกำหนด) จึงเทียบปริมาณอาหารระหว่างแผนได้ยุติธรรม
    let stop_at_target = true;
    let mut day: u32 = 0;
    while day < horizon {
        let product = pick(req, r, strategy, w)?;
        if !pellet_ok(sp, product, w) && oversize_from.is_none() {
            oversize_from = Some((day + 1, product.code.clone(), product.pellet_mm.unwrap_or(0.0), w));
        }
        let need = requirement_pct(code, w);
        let gf = growth_factor(r, product.protein_pct, need);
        let stage_info = stage_for(code, w);
        let pct = sp.feed_pct(w);
        let feed = alive * w / 1000.0 * pct / 100.0;
        let meals = stage_info.meals_per_day.max(1);

        days.push(PlanDay {
            day: day + 1,
            weight_g: crate::round(w, 1),
            alive: alive.round(),
            feed_kg: crate::round(feed, 2),
            meals,
            per_meal_kg: crate::round(feed / meals as f64, 2),
            product_code: product.code.clone(),
        });
        feed_total += feed;
        match product.price_per_kg() {
            Some(p) => cost_total += feed * p,
            None => cost_known = false,
        }

        let same = stages.last().map(|s| s.product_code == product.code && s.meals_per_day == meals).unwrap_or(false);
        if same {
            let s = stages.last_mut().unwrap();
            s.to_day = day + 1;
            s.feed_kg += feed;
            // ถ่วงความต้องการโปรตีนและการโตด้วยปริมาณอาหาร
            let n = (s.to_day - s.from_day + 1) as f64;
            s.requirement_pct += (need - s.requirement_pct) / n;
            s.growth_factor += (gf - s.growth_factor) / n;
        } else {
            stages.push(PlanStage {
                product_code: product.code.clone(),
                product_name_th: product.name_th.clone(),
                color_th: product.color_th.clone(),
                protein_pct: product.protein_pct,
                pellet_mm: product.pellet_mm,
                from_day: day + 1,
                to_day: day + 1,
                from_weight_g: w,
                to_weight_g: w,
                feed_kg: feed,
                bags: 0.0,
                bag_kg: product.bag_kg,
                cost: None,
                meals_per_day: meals,
                feeding_times: stage_info.feeding_times.clone(),
                requirement_pct: need,
                growth_factor: gf,
            });
        }

        // โต 1 วัน: เดินบนเส้นมาตรฐานด้วยความเร็ว = ตัวคูณโปรตีน x ตัวคูณฟาร์ม
        curve_day += gf * farm_scale;
        w = sp.standard_weight_at(curve_day).max(w);
        alive -= alive * daily_mort;
        if let Some(s) = stages.last_mut() {
            s.to_weight_g = w;
        }
        day += 1;
        if day_reached.is_none() && w >= target_w {
            day_reached = Some(day);
            if stop_at_target {
                break;
            }
        }
    }

    for s in &mut stages {
        s.bags = crate::round(s.feed_kg / s.bag_kg.max(0.1), 1);
        s.cost = req
            .products
            .iter()
            .find(|p| p.code == s.product_code)
            .and_then(|p| p.price_per_kg())
            .map(|p| crate::round(s.feed_kg * p, 0));
        s.feed_kg = crate::round(s.feed_kg, 1);
        s.from_weight_g = crate::round(s.from_weight_g, 1);
        s.to_weight_g = crate::round(s.to_weight_g, 1);
        s.requirement_pct = crate::round(s.requirement_pct, 1);
        s.growth_factor = crate::round(s.growth_factor, 3);
        if s.growth_factor < 0.9 {
            warnings.push(format!(
                "ช่วงวันที่ {}-{} ใช้ {} โปรตีน {:.1}% ต่ำกว่าที่ปลาต้องการ ({:.0}%) ปลาจะโตช้ากว่ามาตรฐานประมาณ {:.0}%",
                s.from_day,
                s.to_day,
                s.product_code,
                s.protein_pct,
                s.requirement_pct,
                (1.0 - s.growth_factor) * 100.0
            ));
        }
    }
    if let Some((d, code_, mm, wt)) = &oversize_from {
        warnings.push(format!(
            "วันที่ {} ปลาหนัก {:.0} ก. แต่ {} เม็ด {:.1} มม. ใหญ่เกินกว่าที่ปลากินได้ (ไม่ควรเกิน {:.1} มม.) ปลาจะกินไม่ได้ ควรใช้เม็ดเล็กกว่านี้ก่อน",
            d, wt, code_, mm, pellet_cap_mm(sp, *wt)
        ));
    }
    if strategy == Strategy::Custom {
        let wrong: Vec<String> = req
            .custom_codes
            .iter()
            .filter_map(|c| req.products.iter().find(|p| &p.code == c))
            .filter(|p| !suits_species(p, code))
            .map(|p| p.code.clone())
            .collect();
        if !wrong.is_empty() {
            warnings.push(format!("{} ไม่ได้ผลิตมาสำหรับ{} ระบบคำนวณให้ตามโปรตีนจริงของอาหาร", wrong.join(", "), sp.name_th));
        }
    }

    let biomass = alive * w / 1000.0;
    let gain = biomass - req.count * req.stock_weight_g / 1000.0;
    let final_day = day;
    let reached = day_reached.is_some();
    Some(PlanResult {
        strategy,
        strategy_th: strategy.name_th().into(),
        bags_total: crate::round(stages.iter().map(|s| s.bags).sum(), 1),
        stages,
        days,
        final_day,
        final_weight_g: crate::round(w, 1),
        final_count: alive.round(),
        biomass_kg: crate::round(biomass, 1),
        feed_kg_total: crate::round(feed_total, 1),
        cost_total: if cost_known { Some(crate::round(cost_total, 0)) } else { None },
        fcr: if gain > 0.0 { Some(crate::round(feed_total / gain, 2)) } else { None },
        reached_target: reached,
        day_reached,
        shortfall_g: if reached { 0.0 } else { crate::round((target_w - w).max(0.0), 1) },
        warnings_th: warnings,
    })
}

pub fn plan(req: &PlanRequest) -> PlanComparison {
    let r = req.response.unwrap_or_else(|| ProteinResponse::for_species(&req.species.code));
    let target_w = req.target_weight_g.unwrap_or(req.species.market_weight_g);
    let mut strategies = vec![Strategy::Matched, Strategy::Fastest, Strategy::Economical];
    if !req.custom_codes.is_empty() {
        strategies.insert(0, Strategy::Custom);
    }
    let mut plans: Vec<PlanResult> = Vec::new();
    for s in strategies {
        if let Some(p) = run(req, &r, s) {
            // กลยุทธ์ที่เลือกเบอร์ออกมาเหมือนกันทุกช่วง ไม่ต้องแสดงซ้ำ
            let dup = plans.iter().any(|q| {
                q.stages.len() == p.stages.len()
                    && q.stages.iter().zip(&p.stages).all(|(a, b)| (a.protein_pct - b.protein_pct).abs() < 0.01 && a.from_day == b.from_day)
            });
            if !dup || s == Strategy::Custom {
                plans.push(p);
            }
        }
    }
    let max_w = req.species.max_standard_weight();
    if plans.is_empty() {
        return PlanComparison {
            plans,
            recommended: 0,
            message_th: format!("ยังไม่มีอาหารในรายการที่ใช้กับ{}ได้", req.species.name_th),
            max_reachable_weight_g: crate::round(max_w, 0),
        };
    }

    // แนะนำ: แผนที่ถึงเป้าตามกำหนด และต้นทุนต่ำสุด (ไม่รู้ราคาใช้อาหารน้อยสุด) ถ้าไม่มีแผนไหนถึง เลือกที่ได้ตัวใหญ่สุด
    let score_cost = |p: &PlanResult| p.cost_total.unwrap_or(p.feed_kg_total * 1000.0);
    let reaching: Vec<usize> = (0..plans.len()).filter(|&i| plans[i].reached_target).collect();
    let recommended = if !reaching.is_empty() {
        *reaching.iter().min_by(|&&a, &&b| score_cost(&plans[a]).total_cmp(&score_cost(&plans[b]))).unwrap()
    } else {
        (0..plans.len()).max_by(|&a, &b| plans[a].final_weight_g.total_cmp(&plans[b].final_weight_g)).unwrap()
    };

    let best = &plans[recommended];
    let priced = plans.iter().all(|p| p.cost_total.is_some());
    let why = if !best.reached_target {
        "ได้ตัวใหญ่ที่สุดในเวลาที่กำหนด"
    } else if priced {
        "ถึงเป้าด้วยค่าอาหารต่ำสุด"
    } else {
        "ถึงเป้าโดยใช้อาหารน้อยที่สุด (ใส่ราคากระสอบเพื่อเทียบเป็นเงิน)"
    };
    let message_th = match (req.target_days, best.reached_target) {
        (Some(d), true) => format!(
            "ทำได้: แนะนำโปรแกรม \"{}\" ปลาถึง {:.0} ก. ในวันที่ {} (กำหนด {} วัน) ใช้อาหาร {:.1} กระสอบ · เหตุผล: {}",
            best.strategy_th,
            target_w,
            best.day_reached.unwrap_or(best.final_day),
            d,
            best.bags_total,
            why
        ),
        (Some(d), false) => {
            let fastest = plans.iter().map(|p| p.final_weight_g).fold(0.0, f64::max);
            if target_w > max_w {
                format!("เป้า {:.0} ก. เกินขนาดที่{}โตถึงได้ตามมาตรฐาน (ประมาณ {:.0} ก.) ลองลดเป้าหมาย", target_w, req.species.name_th, max_w)
            } else {
                format!(
                    "ภายใน {} วัน ทำได้สูงสุดประมาณ {:.0} ก. (ขาดจากเป้า {:.0} ก.) ต้องเลี้ยงนานขึ้น หรือลดเป้าหมาย",
                    d,
                    fastest,
                    (target_w - fastest).max(0.0)
                )
            }
        }
        (None, true) => format!(
            "แนะนำโปรแกรม \"{}\" ปลาถึง {:.0} ก. ในวันที่ {} ใช้อาหาร {:.1} กระสอบ · เหตุผล: {}",
            best.strategy_th,
            target_w,
            best.day_reached.unwrap_or(best.final_day),
            best.bags_total,
            why
        ),
        (None, false) => format!("ภายในระยะที่จำลอง ปลายังไม่ถึง {:.0} ก. (ได้ประมาณ {:.0} ก.)", target_w, best.final_weight_g),
    };
    PlanComparison { plans, recommended, message_th, max_reachable_weight_g: crate::round(max_w, 0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nb_like_catalog() -> Vec<FeedProduct> {
        let p = |code: &str, target: &str, from: f64, to: f64, prot: f64, mm: f64, price: Option<f64>| FeedProduct {
            code: code.into(),
            name_th: code.into(),
            brand: "NB".into(),
            target: target.into(),
            weight_from_g: from,
            weight_to_g: to,
            protein_pct: prot,
            pellet_mm: Some(mm),
            bag_kg: 20.0,
            price_per_bag: price,
            color_th: None,
        };
        vec![
            p("GF 411 SM", "herbivore", 0.0, 300.0, 32.0, 3.0, Some(620.0)),
            p("NB 202", "herbivore", 300.0, 100000.0, 15.5, 4.0, Some(380.0)),
            p("GZ 202 L", "herbivore", 300.0, 100000.0, 15.5, 4.0, Some(370.0)),
            p("NB PP 888", "carnivore", 400.0, 100000.0, 40.0, 5.0, Some(800.0)),
        ]
    }

    fn req(target_days: Option<u32>, target: f64, custom: Vec<&str>) -> PlanRequest {
        PlanRequest {
            species: SpeciesProfile::nile_tilapia(),
            stock_weight_g: 30.0,
            count: 5000.0,
            survival_pct: 85.0,
            target_weight_g: Some(target),
            target_days,
            products: nb_like_catalog(),
            custom_codes: custom.into_iter().map(String::from).collect(),
            response: None,
            farm_growth_scale: None,
            max_days: None,
        }
    }

    #[test]
    fn low_protein_grows_slower_than_matched() {
        // ใช้ 15.5% ทั้งรุ่นกับปลานิล ต้องโตช้ากว่าโปรแกรมที่เปลี่ยนเบอร์ตามความต้องการ
        let c = plan(&req(Some(150), 800.0, vec!["NB 202"]));
        let custom = c.plans.iter().find(|p| p.strategy == Strategy::Custom).unwrap();
        let matched = c.plans.iter().find(|p| p.strategy == Strategy::Matched).unwrap();
        assert!(custom.final_weight_g < matched.final_weight_g, "custom {} matched {}", custom.final_weight_g, matched.final_weight_g);
        assert!(!custom.warnings_th.is_empty());
        // FCR ของอาหารโปรตีนต่ำต้องแย่กว่า
        assert!(custom.fcr.unwrap() > matched.fcr.unwrap());
    }

    #[test]
    fn stages_switch_product_by_size_and_count_bags() {
        let c = plan(&req(None, 800.0, vec![]));
        let m = &c.plans[c.recommended];
        assert!(m.reached_target);
        // ลูกปลากิน 3 มื้อ ปลาใหญ่ 2 มื้อ ต้องแยกเป็นคนละช่วงในแผน
        assert!(m.stages.len() >= 2, "ต้องแยกช่วงเมื่อจำนวนมื้อเปลี่ยน");
        assert!(m.stages.windows(2).all(|w| w[0].product_code != w[1].product_code || w[0].meals_per_day != w[1].meals_per_day));
        // เบอร์โปรตีนเท่ากันต่างยี่ห้อ ไม่ต้องแสดงเป็นอีกโปรแกรม
        for (i, a) in c.plans.iter().enumerate() {
            for b in c.plans.iter().skip(i + 1) {
                let same = a.stages.len() == b.stages.len() && a.stages.iter().zip(&b.stages).all(|(x, y)| x.protein_pct == y.protein_pct && x.from_day == y.from_day);
                assert!(!same, "{} ซ้ำกับ {}", a.strategy_th, b.strategy_th);
            }
        }
        let bags: f64 = m.stages.iter().map(|s| s.bags).sum();
        assert!((bags - m.bags_total).abs() < 0.5);
        // ตารางรายวันต้องต่อเนื่องไม่ขาด
        for (i, d) in m.days.iter().enumerate() {
            assert_eq!(d.day as usize, i + 1);
        }
    }

    #[test]
    fn impossible_target_is_reported_honestly() {
        let c = plan(&req(Some(60), 800.0, vec![]));
        assert!(c.plans.iter().all(|p| !p.reached_target));
        assert!(c.message_th.contains("ขาดจากเป้า"));
    }

    #[test]
    fn carnivore_feed_not_offered_for_tilapia_unless_chosen() {
        let c = plan(&req(None, 800.0, vec![]));
        for p in &c.plans {
            assert!(p.stages.iter().all(|s| s.product_code != "NB PP 888"));
        }
    }

    #[test]
    fn fry_feed_not_used_for_grow_out() {
        let mut req = req(None, 800.0, vec![]);
        req.products.push(FeedProduct {
            code: "FRY42".into(),
            name_th: "อาหารลูกปลา".into(),
            brand: "x".into(),
            target: "all".into(),
            weight_from_g: 0.0,
            weight_to_g: 10.0,
            protein_pct: 42.0,
            pellet_mm: Some(1.2),
            bag_kg: 2.0,
            price_per_bag: None,
            color_th: None,
        });
        let out = plan(&req);
        for p in &out.plans {
            for st in &p.stages {
                assert_ne!(st.product_code, "FRY42", "{:?} ใช้อาหารลูกปลากับปลา {} ก.", p.strategy, st.from_weight_g);
            }
        }
    }
}
