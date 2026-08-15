use serde::{Deserialize, Serialize};

use crate::species::SpeciesProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthCompare {
    /// วันที่เลี้ยงมาแล้ว
    pub day: u32,
    /// น้ำหนักที่ควรเป็นตามมาตรฐาน ณ วันนี้ (เริ่มนับจากน้ำหนักปล่อยจริง)
    pub expected_g: f64,
    pub actual_g: f64,
    /// ส่วนต่างเป็น % (บวก = โตเร็วกว่าเกณฑ์)
    pub deviation_pct: f64,
    /// ahead | on_track | behind | far_behind
    pub status: String,
    pub status_th: String,
    /// ADG มาตรฐาน ณ ขนาดนี้ (ก./วัน)
    pub standard_adg: f64,
    /// ADG จริงตั้งแต่ปล่อย (ก./วัน)
    pub actual_adg_overall: f64,
    /// ADG จริงช่วงล่าสุด (ถ้ามีข้อมูลชั่งครั้งก่อน)
    pub actual_adg_recent: Option<f64>,
    /// วันมาตรฐานที่จะถึงน้ำหนักเป้าหมาย นับจากวันนี้ (ตามอัตราจริงล่าสุด)
    pub days_to_target: Option<u32>,
    pub advice_th: Vec<String>,
}

/// เทียบน้ำหนักจริงกับมาตรฐาน
/// - stock_weight_g: น้ำหนักตอนปล่อย
/// - day: วันที่เลี้ยง (วันปล่อย = 0)
/// - actual_g: น้ำหนักเฉลี่ยจากการสุ่มชั่งวันนี้
/// - prev: (วันที่ชั่งครั้งก่อน, น้ำหนักครั้งก่อน)
/// - target_g: น้ำหนักเป้าหมายจับ
pub fn compare_growth(
    sp: &SpeciesProfile,
    stock_weight_g: f64,
    day: u32,
    actual_g: f64,
    prev: Option<(u32, f64)>,
    target_g: Option<f64>,
) -> GrowthCompare {
    let day0 = sp.standard_day_for_weight(stock_weight_g);
    let expected = sp.standard_weight_at(day0 + day as f64);
    let deviation = if expected > 0.0 { (actual_g - expected) / expected * 100.0 } else { 0.0 };
    let (status, status_th) = if deviation >= 5.0 {
        ("ahead", "โตเร็วกว่าเกณฑ์")
    } else if deviation >= -5.0 {
        ("on_track", "โตตามเกณฑ์")
    } else if deviation >= -15.0 {
        ("behind", "โตช้ากว่าเกณฑ์")
    } else {
        ("far_behind", "โตช้ากว่าเกณฑ์มาก")
    };

    let standard_adg = sp.standard_adg_at_weight(actual_g);
    let overall = if day > 0 { (actual_g - stock_weight_g) / day as f64 } else { 0.0 };
    let recent = prev.and_then(|(pd, pw)| {
        if day > pd {
            Some(crate::round((actual_g - pw) / (day - pd) as f64, 2))
        } else {
            None
        }
    });

    let rate = recent.filter(|r| *r > 0.0).unwrap_or(if overall > 0.0 { overall } else { standard_adg });
    let days_to_target = target_g.and_then(|t| {
        if t <= actual_g {
            Some(0)
        } else if rate > 0.0 {
            Some(((t - actual_g) / rate).ceil() as u32)
        } else {
            None
        }
    });

    let mut advice = Vec::new();
    match status {
        "far_behind" => {
            advice.push("ตรวจคุณภาพน้ำ (ออกซิเจนเช้า แอมโมเนีย) และดูว่าอาหารเหลือหรือไม่".into());
            advice.push("นับจำนวนปลาใหม่ อาจมีปลาตายที่ไม่เห็น ทำให้คำนวณอาหารเกิน".into());
            advice.push("พิจารณาคัดขนาดหรือลดความหนาแน่น".into());
        }
        "behind" => {
            advice.push("เพิ่มมื้ออาหารเป็น 3 มื้อในวันที่อากาศดี และสังเกตการกิน".into());
            advice.push("ตรวจโปรตีนอาหารให้เหมาะกับขนาดปลา".into());
        }
        "ahead" => {
            advice.push("โตดี รักษาคุณภาพน้ำ อย่าเพิ่มอาหารเกินตาราง".into());
        }
        _ => {
            advice.push("โตตามเกณฑ์ ชั่งซ้ำทุก 1-2 สัปดาห์เพื่อปรับปริมาณอาหาร".into());
        }
    }

    GrowthCompare {
        day,
        expected_g: crate::round(expected, 1),
        actual_g,
        deviation_pct: crate::round(deviation, 1),
        status: status.into(),
        status_th: status_th.into(),
        standard_adg: crate::round(standard_adg, 2),
        actual_adg_overall: crate::round(overall, 2),
        actual_adg_recent: recent,
        days_to_target,
        advice_th: advice,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_track_when_matches_curve() {
        let sp = SpeciesProfile::nile_tilapia();
        let g = compare_growth(&sp, 30.0, 55, 300.0, None, Some(800.0));
        assert_eq!(g.status, "on_track");
        assert!(g.days_to_target.unwrap() > 0);
    }

    #[test]
    fn behind_detected() {
        let sp = SpeciesProfile::nile_tilapia();
        let g = compare_growth(&sp, 30.0, 55, 240.0, Some((41, 200.0)), None);
        assert!(g.status == "behind" || g.status == "far_behind");
        assert!(g.actual_adg_recent.is_some());
    }
}
