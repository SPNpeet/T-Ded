//! โภชนาการอาหาร: โปรตีนตามช่วงน้ำหนัก ชนิดเม็ด มื้อ/เวลาให้ เทียบอาหารในมือ ผสมอาหารเอง (Pearson square)

use serde::{Deserialize, Serialize};

/// ช่วงการเลี้ยงกับสเปกอาหารที่ควรใช้
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeedStage {
    pub name_th: String,
    pub weight_from_g: f64,
    pub weight_to_g: f64,
    pub protein_min: f64,
    pub protein_max: f64,
    pub fat_min: f64,
    pub pellet_mm: f64,
    /// powder | crumble | floating | sinking
    pub form: String,
    pub form_th: String,
    pub meals_per_day: u8,
    /// เวลาให้ที่แนะนำ (นาฬิกา 24 ชม.)
    pub feeding_times: Vec<String>,
    pub note_th: String,
}

pub fn stages_for(species_code: &str) -> Vec<FeedStage> {
    let st = |name: &str, a: f64, b: f64, p1: f64, p2: f64, fat: f64, mm: f64, form: &str, form_th: &str, meals: u8, times: &[&str], note: &str| FeedStage {
        name_th: name.into(),
        weight_from_g: a,
        weight_to_g: b,
        protein_min: p1,
        protein_max: p2,
        fat_min: fat,
        pellet_mm: mm,
        form: form.into(),
        form_th: form_th.into(),
        meals_per_day: meals,
        feeding_times: times.iter().map(|s| s.to_string()).collect(),
        note_th: note.into(),
    };
    match species_code {
        "catfish" => vec![
            st("ลูกปลา", 0.0, 5.0, 38.0, 42.0, 6.0, 1.0, "crumble", "เม็ดเล็ก/ป่น", 4, &["07:00", "11:00", "15:00", "18:00"], "โปรตีนสูง ให้บ่อยครั้งละน้อย"),
            st("ปลาเล็ก", 5.0, 50.0, 32.0, 35.0, 5.0, 2.0, "floating", "เม็ดลอยน้ำ", 3, &["07:30", "12:00", "17:00"], "เริ่มฝึกกินเม็ดลอย ดูให้กินหมดใน 15 นาที"),
            st("ปลารุ่น", 50.0, 150.0, 30.0, 32.0, 5.0, 3.0, "floating", "เม็ดลอยน้ำ", 2, &["08:00", "17:00"], "ช่วงโตเร็ว คุมปริมาณตามตาราง"),
            st("ปลาใหญ่/ขุน", 150.0, 100000.0, 28.0, 30.0, 4.0, 4.0, "floating", "เม็ดลอยน้ำ", 2, &["08:00", "17:00"], "ก่อนจับ 1-2 วันงดอาหารเพื่อล้างท้อง"),
        ],
        // ปลานิล / ปลาทับทิม
        _ => vec![
            st("ลูกปลา", 0.0, 10.0, 38.0, 40.0, 6.0, 0.8, "powder", "อาหารผง/เม็ดจิ๋ว", 4, &["07:00", "10:30", "14:00", "17:00"], "โปรตีนสูง ให้ครั้งละน้อยแต่บ่อย"),
            st("ปลานิ้ว", 10.0, 30.0, 35.0, 38.0, 6.0, 1.5, "crumble", "เม็ดเล็ก", 3, &["07:30", "12:00", "17:00"], "หว่านให้ทั่วบ่อ สังเกตตัวเล็กได้กินด้วย"),
            st("ปลาเล็ก", 30.0, 75.0, 32.0, 35.0, 5.0, 2.0, "floating", "เม็ดลอยน้ำ", 3, &["07:30", "12:00", "17:00"], "เริ่มใช้เม็ด 2 มม. ตามตารางอัตราให้อาหาร"),
            st("ปลารุ่น", 75.0, 300.0, 30.0, 32.0, 5.0, 3.0, "floating", "เม็ดลอยน้ำ", 2, &["08:00", "17:00"], "ช่วงกินจุที่สุด คุมตามตาราง อย่าให้เกิน"),
            st("ปลาใหญ่", 300.0, 600.0, 28.0, 30.0, 4.0, 3.0, "floating", "เม็ดลอยน้ำ", 2, &["08:00", "17:00"], "ลด % ต่อวันตามตาราง เน้นน้ำดี"),
            st("ขุนก่อนจับ", 600.0, 100000.0, 25.0, 28.0, 4.0, 3.0, "floating", "เม็ดลอยน้ำ", 2, &["08:00", "17:00"], "โปรตีนต่ำลงได้ ประหยัดต้นทุน งดอาหาร 1 วันก่อนจับ"),
        ],
    }
}

pub fn stage_for(species_code: &str, weight_g: f64) -> FeedStage {
    let stages = stages_for(species_code);
    stages
        .iter()
        .find(|s| weight_g >= s.weight_from_g && weight_g < s.weight_to_g)
        .cloned()
        .unwrap_or_else(|| stages.last().cloned().unwrap())
}

/// อาหารที่มีอยู่ (จากสต๊อกหรือที่ผู้ใช้บอก)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeedOnHand {
    pub brand: Option<String>,
    pub protein_pct: Option<f64>,
    pub pellet_mm: Option<f64>,
    pub price_per_kg: Option<f64>,
    /// floating | sinking
    pub form: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NutritionAdvice {
    pub stage: FeedStage,
    /// ok | protein_low | protein_high | pellet_mismatch | unknown
    pub status: String,
    pub status_th: String,
    pub messages_th: Vec<String>,
    /// ราคาต่อ 1 กก. โปรตีน (บาท) ถ้ารู้ราคาและโปรตีน
    pub price_per_kg_protein: Option<f64>,
    /// โปรตีนที่ปลาได้รับต่อวัน (กก.) = อาหาร × โปรตีน%
    pub protein_intake_kg_day: Option<f64>,
}

pub fn advise(species_code: &str, weight_g: f64, feed_kg_day: f64, on_hand: &FeedOnHand) -> NutritionAdvice {
    let stage = stage_for(species_code, weight_g);
    let mut msgs = Vec::new();
    let mut status = "unknown";
    let mut status_th = "ยังไม่ทราบสเปกอาหารในมือ";
    if let Some(p) = on_hand.protein_pct {
        if p < stage.protein_min - 0.5 {
            status = "protein_low";
            status_th = "โปรตีนต่ำกว่าที่แนะนำ";
            msgs.push(format!(
                "อาหารที่ใช้โปรตีน {:.0}% ต่ำกว่าช่วงแนะนำ {:.0}-{:.0}% สำหรับ{} ปลาอาจโตช้ากว่าเกณฑ์ พิจารณาเปลี่ยนสูตรหรือยอมรับว่าจะใช้เวลานานขึ้น",
                p, stage.protein_min, stage.protein_max, stage.name_th
            ));
        } else if p > stage.protein_max + 3.0 {
            status = "protein_high";
            status_th = "โปรตีนสูงเกินจำเป็น";
            msgs.push(format!(
                "อาหารโปรตีน {:.0}% สูงกว่าช่วงแนะนำ {:.0}-{:.0}% สำหรับ{} เสียเงินเกินจำเป็นและของเสีย/แอมโมเนียในน้ำมากขึ้น ใช้สูตรถูกลงได้",
                p, stage.protein_min, stage.protein_max, stage.name_th
            ));
        } else {
            status = "ok";
            status_th = "โปรตีนเหมาะสม";
        }
    }
    if let (Some(mm), true) = (on_hand.pellet_mm, status != "unknown") {
        if (mm - stage.pellet_mm).abs() >= 1.0 {
            if status == "ok" {
                status = "pellet_mismatch";
                status_th = "ขนาดเม็ดไม่ตรงช่วง";
            }
            msgs.push(format!("ขนาดเม็ด {:.1} มม. ไม่ตรงกับที่แนะนำ {:.1} มม. สำหรับ{} เม็ดใหญ่ไปปลาเล็กกินไม่ได้ เม็ดเล็กไปสิ้นเปลืองและปลาใหญ่กินไม่ทัน", mm, stage.pellet_mm, stage.name_th));
        }
    }
    if status == "unknown" {
        msgs.push(format!("ระบุโปรตีนของอาหารตอนรับเข้าสต๊อก จะเทียบให้ว่าเหมาะกับ{} (แนะนำ {:.0}-{:.0}%) หรือไม่", stage.name_th, stage.protein_min, stage.protein_max));
    }
    let ppp = match (on_hand.price_per_kg, on_hand.protein_pct) {
        (Some(pr), Some(p)) if p > 0.0 => Some(crate::round(pr / (p / 100.0), 2)),
        _ => None,
    };
    let intake = on_hand.protein_pct.map(|p| crate::round(feed_kg_day * p / 100.0, 3));
    NutritionAdvice { stage, status: status.into(), status_th: status_th.into(), messages_th: msgs, price_per_kg_protein: ppp, protein_intake_kg_day: intake }
}

/// วัตถุดิบผสมอาหาร (ค่ามาตรฐานโดยประมาณ ผู้ใช้แก้ได้)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ingredient {
    pub name_th: String,
    pub protein_pct: f64,
    pub price_per_kg: f64,
    /// สัดส่วนในสูตร (%)
    pub share_pct: f64,
    /// เพดานที่แนะนำ (%) เช่น รำ ไม่ควรเกิน 30
    pub max_share_pct: Option<f64>,
}

pub fn default_ingredients() -> Vec<Ingredient> {
    let mk = |n: &str, p: f64, pr: f64, s: f64, m: Option<f64>| Ingredient { name_th: n.into(), protein_pct: p, price_per_kg: pr, share_pct: s, max_share_pct: m };
    vec![
        mk("ปลาป่น (โปรตีน 55-60%)", 58.0, 45.0, 15.0, Some(30.0)),
        mk("กากถั่วเหลือง", 44.0, 22.0, 30.0, Some(45.0)),
        mk("รำละเอียด", 12.0, 9.0, 25.0, Some(30.0)),
        mk("ปลายข้าว/ข้าวโพดบด", 8.0, 10.0, 25.0, Some(40.0)),
        mk("พรีมิกซ์วิตามิน-แร่ธาตุ", 0.0, 120.0, 2.0, Some(3.0)),
        mk("น้ำมันปลา/น้ำมันพืช", 0.0, 40.0, 3.0, Some(6.0)),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixResult {
    pub total_share_pct: f64,
    pub protein_pct: f64,
    pub cost_per_kg: f64,
    pub warnings_th: Vec<String>,
    /// น้ำหนักวัตถุดิบต่อ batch (กก.) ถ้าระบุ batch_kg
    pub per_batch_kg: Vec<(String, f64)>,
}

/// รวมโปรตีนและต้นทุนของสูตรผสม
pub fn mix(ingredients: &[Ingredient], batch_kg: Option<f64>) -> MixResult {
    let total: f64 = ingredients.iter().map(|i| i.share_pct).sum();
    let mut warnings = Vec::new();
    if (total - 100.0).abs() > 0.5 {
        warnings.push(format!("สัดส่วนรวม {:.1}% ควรเท่ากับ 100% (ระบบคิดตามสัดส่วนที่ให้)", total));
    }
    let t = if total > 0.0 { total } else { 1.0 };
    let protein = ingredients.iter().map(|i| i.protein_pct * i.share_pct / t).sum::<f64>();
    let cost = ingredients.iter().map(|i| i.price_per_kg * i.share_pct / t).sum::<f64>();
    for i in ingredients {
        if let Some(m) = i.max_share_pct {
            if i.share_pct > m + 0.01 {
                warnings.push(format!("{} ใส่ {:.0}% เกินเพดานแนะนำ {:.0}% อาจย่อยยากหรือเม็ดไม่จับตัว", i.name_th, i.share_pct, m));
            }
        }
    }
    let per_batch = match batch_kg {
        Some(b) => ingredients.iter().map(|i| (i.name_th.clone(), crate::round(b * i.share_pct / t, 2))).collect(),
        None => vec![],
    };
    MixResult { total_share_pct: crate::round(total, 1), protein_pct: crate::round(protein, 1), cost_per_kg: crate::round(cost, 2), warnings_th: warnings, per_batch_kg: per_batch }
}

/// Pearson square: หาสัดส่วนวัตถุดิบโปรตีนสูง (a) และต่ำ (b) ให้ได้โปรตีนเป้าหมาย
/// คืน (share_a_pct, share_b_pct) หรือ None ถ้าเป้าหมายอยู่นอกช่วง
pub fn pearson_square(protein_a: f64, protein_b: f64, target: f64) -> Option<(f64, f64)> {
    let (hi, lo, swapped) = if protein_a >= protein_b { (protein_a, protein_b, false) } else { (protein_b, protein_a, true) };
    if target > hi || target < lo || (hi - lo).abs() < 1e-9 {
        return None;
    }
    let share_hi = (target - lo) / (hi - lo) * 100.0;
    let share_lo = 100.0 - share_hi;
    Some(if swapped { (crate::round(share_lo, 1), crate::round(share_hi, 1)) } else { (crate::round(share_hi, 1), crate::round(share_lo, 1)) })
}

/// เคล็ดลับเก็บรักษาอาหารและการให้ (ข้อความสำหรับแสดงในแอป)
pub fn feed_tips() -> Vec<(&'static str, &'static str)> {
    vec![
        ("การเก็บอาหาร", "วางกระสอบบนพาเลท ไม่ติดพื้น/ผนัง ที่ร่ม แห้ง อากาศถ่ายเท ใช้ให้หมดภายใน 1-2 เดือนหลังผลิต อาหารชื้น/ขึ้นรา ห้ามให้เด็ดขาด (สารพิษอะฟลาทอกซิน)"),
        ("เวลาให้", "ให้เมื่อแดดออกและออกซิเจนสูงพอ (หลัง 07:30) มื้อเย็นก่อน 17:30 ห้ามให้ตอนมืดหรือเช้ามืดที่ออกซิเจนต่ำ"),
        ("วิธีให้", "หว่านให้ทั่วจุดเดิมทุกวัน สังเกต 15-20 นาที ถ้าเหลือลอยให้ลดมื้อถัดไป 10-20% ถ้ากินหมดเร็วมากและปลายังวนหา ค่อยเพิ่มไม่เกิน 10%"),
        ("โปรตีนกับต้นทุน", "โปรตีนสูงเกินความจำเป็นไม่ทำให้โตเร็วขึ้นแต่แพงและทำให้แอมโมเนียในน้ำสูง เลือกตามช่วงน้ำหนัก และเทียบราคาต่อ 1 กก. โปรตีน"),
        ("เสริมช่วงเครียด", "อากาศเปลี่ยน/หลังย้ายบ่อ/พบโรค เสริมวิตามินซี 1-2 กรัมต่ออาหาร 1 กก. คลุกน้ำมันพืชเล็กน้อย 5-7 วัน (ปรึกษาเจ้าหน้าที่ประมงก่อนใช้ยา)"),
        ("ก่อนจับ", "งดอาหาร 1 วัน (ปลาดุก 1-2 วัน) เพื่อล้างท้อง เนื้อสะอาด ขนส่งไม่ตาย และไม่เสียอาหารเปล่า"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_and_advice() {
        let s = stage_for("nile_tilapia", 300.0);
        assert_eq!(s.name_th, "ปลาใหญ่");
        let a = advise("nile_tilapia", 300.0, 10.0, &FeedOnHand { protein_pct: Some(25.0), pellet_mm: Some(3.0), price_per_kg: Some(28.0), ..Default::default() });
        assert_eq!(a.status, "protein_low");
        assert_eq!(a.price_per_kg_protein, Some(112.0));
        assert_eq!(a.protein_intake_kg_day, Some(2.5));
        let ok = advise("nile_tilapia", 300.0, 10.0, &FeedOnHand { protein_pct: Some(30.0), pellet_mm: Some(3.0), ..Default::default() });
        assert_eq!(ok.status, "ok");
    }

    #[test]
    fn mix_and_pearson() {
        let m = mix(&default_ingredients(), Some(100.0));
        assert!(m.protein_pct > 20.0 && m.protein_pct < 35.0, "{}", m.protein_pct);
        assert_eq!(m.per_batch_kg.len(), 6);
        let (a, b) = pearson_square(58.0, 12.0, 30.0).unwrap();
        assert!((a - 39.1).abs() < 0.2 && (b - 60.9).abs() < 0.2);
        assert!(pearson_square(58.0, 12.0, 70.0).is_none());
    }
}
