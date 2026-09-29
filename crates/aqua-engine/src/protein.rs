//! โปรตีนในอาหารมีผลต่อการโตอย่างไร
//!
//! หลักการ (broken-line response ที่ใช้ทั่วไปในงานโภชนาการสัตว์น้ำ):
//! - ปลากินอาหารตามตารางในปริมาณเท่าเดิม ไม่ว่าโปรตีนจะกี่เปอร์เซ็นต์ (อิ่มเท่ากัน)
//! - ถ้าโปรตีนต่ำกว่าความต้องการของปลาขนาดนั้น การโตลดลงตามสัดส่วน
//! - ถ้าโปรตีนถึงความต้องการแล้ว การโตแทบไม่เพิ่ม (มีส่วนเพิ่มเล็กน้อยแล้วอิ่มตัว)
//! ผลคืออาหารน้ำหนักเท่ากันแต่โปรตีนต่างกัน ได้เนื้อต่างกัน และอัตราแลกเนื้อ (FCR) ต่างกันเอง
//!
//! ค่าตั้งต้นทุกตัวอยู่ใน [`ProteinResponse`] ปรับได้เมื่อมีผลทดลองเลี้ยงจริงของบริษัท

use serde::{Deserialize, Serialize};

use crate::nutrition::stage_for;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ProteinResponse {
    /// โปรตีน (%) ที่ต่ำจนปลาแทบไม่โต ใช้เป็นจุดศูนย์ของเส้นตอบสนอง
    pub p_floor: f64,
    /// ความโค้งช่วงโปรตีนไม่พอ (1 = เส้นตรง, น้อยกว่า 1 = ลดช้ากว่าเส้นตรง)
    pub shape: f64,
    /// การโตเพิ่มได้สูงสุดเมื่อโปรตีนเกินความต้องการ (สัดส่วน เช่น 0.06 = เพิ่ม 6%)
    pub surplus_gain_max: f64,
    /// โปรตีนเกินความต้องการกี่เปอร์เซ็นต์จึงได้ส่วนเพิ่มเต็ม
    pub surplus_span: f64,
    /// การโตต่ำสุดที่ยอมให้ (กันค่าเข้าใกล้ศูนย์จนพยากรณ์วันจับไม่มีที่สิ้นสุด)
    pub min_factor: f64,
}

impl Default for ProteinResponse {
    fn default() -> Self {
        ProteinResponse { p_floor: 8.0, shape: 0.85, surplus_gain_max: 0.05, surplus_span: 10.0, min_factor: 0.15 }
    }
}

impl ProteinResponse {
    /// ค่าที่ปรับให้ตรงผลทดลองเลี้ยงจริงของแต่ละชนิด (ดู research-protein-response.json ของลูกค้า)
    /// - ปลานิล shape 0.47: โปรตีน 25% เทียบ 35% น้ำหนักเพิ่ม 10.52 ต่อ 13.12 ก. และ FCR 2.25 ต่อ 1.81
    ///   ได้ค่าเดียวกันจากสองการทดลอง; 35% -> 45% FCR ดีขึ้นแค่ 5% (1.81 -> 1.72)
    /// - ปลาดุกอุย shape 1.10: โปรตีน 25% เทียบ 40% น้ำหนักเพิ่ม 11.0 ต่อ 22.1 ก. (Hien 2018)
    /// - ปลาช่อน shape 1.09: โปรตีน 33.6% เทียบ 42% น้ำหนักเพิ่ม 6.91 ต่อ 9.41 ก.
    pub fn for_species(code: &str) -> Self {
        let base = ProteinResponse::default();
        match code {
            "nile_tilapia" | "red_tilapia" => ProteinResponse { shape: 0.47, ..base },
            "catfish" => ProteinResponse { shape: 1.10, ..base },
            "snakehead" => ProteinResponse { shape: 1.09, ..base },
            _ => base,
        }
    }
}

/// ความต้องการโปรตีน (%) ของปลาชนิดนี้ที่ขนาดนี้ = ค่าต่ำสุดของช่วงแนะนำ
pub fn requirement_pct(species_code: &str, weight_g: f64) -> f64 {
    stage_for(species_code, weight_g).protein_min
}

/// ตัวคูณการโตเทียบกับการโตมาตรฐาน (1.0 = โตตามตารางมาตรฐาน)
pub fn growth_factor(r: &ProteinResponse, protein_pct: f64, requirement_pct: f64) -> f64 {
    if protein_pct >= requirement_pct {
        let surplus = ((protein_pct - requirement_pct) / r.surplus_span.max(0.1)).min(1.0);
        return 1.0 + r.surplus_gain_max * surplus;
    }
    let span = (requirement_pct - r.p_floor).max(0.1);
    let frac = ((protein_pct - r.p_floor) / span).clamp(0.0, 1.0);
    frac.powf(r.shape).max(r.min_factor)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProteinFit {
    pub protein_pct: f64,
    pub requirement_pct: f64,
    pub growth_factor: f64,
    /// ok | low | very_low | high
    pub level: String,
    pub message_th: String,
}

/// อธิบายเป็นภาษาคนว่าอาหารนี้เหมาะกับปลาขนาดนี้ไหม
pub fn fit(r: &ProteinResponse, species_code: &str, weight_g: f64, protein_pct: f64) -> ProteinFit {
    let _ = r;
    let r = &ProteinResponse::for_species(species_code);
    let req = requirement_pct(species_code, weight_g);
    let gf = growth_factor(r, protein_pct, req);
    let pct = ((gf - 1.0) * 100.0).round();
    let (level, msg) = if protein_pct + 0.01 < req {
        let lvl = if gf < 0.7 { "very_low" } else { "low" };
        (lvl, format!("โปรตีน {:.1}% ต่ำกว่าที่ปลาขนาดนี้ต้องการ ({:.0}%) คาดว่าโตช้ากว่ามาตรฐานประมาณ {:.0}%", protein_pct, req, -pct))
    } else if protein_pct >= req + 5.0 {
        ("high", format!("โปรตีน {:.1}% สูงกว่าความต้องการ ({:.0}%) โตเร็วขึ้นเล็กน้อยประมาณ {:.0}% แต่ต้นทุนต่อกิโลอาจสูงขึ้น", protein_pct, req, pct))
    } else {
        ("ok", format!("โปรตีน {:.1}% เหมาะกับปลาขนาดนี้ (ต้องการ {:.0}%)", protein_pct, req))
    };
    ProteinFit { protein_pct, requirement_pct: req, growth_factor: crate::round(gf, 3), level: level.into(), message_th: msg }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_intake_more_protein_grows_more_until_requirement() {
        let r = ProteinResponse::default();
        let low = growth_factor(&r, 15.5, 30.0);
        let mid = growth_factor(&r, 25.5, 30.0);
        let ok = growth_factor(&r, 30.0, 30.0);
        let high = growth_factor(&r, 40.0, 30.0);
        assert!(low < mid && mid < ok && ok < high);
        assert!((ok - 1.0).abs() < 1e-9);
        // เกินความต้องการ 10% ได้เพิ่มแค่ส่วนเพิ่มสูงสุด ไม่ใช่โตเป็นเท่าตัว
        assert!((high - 1.05).abs() < 1e-9);
    }

    #[test]
    fn floor_never_zero() {
        let r = ProteinResponse::default();
        assert!(growth_factor(&r, 5.0, 40.0) >= r.min_factor);
    }
}

#[cfg(test)]
mod calibration {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn matches_published_trials() {
        // ปลานิล 4 ก.: 25% ได้ 80% ของ 35%
        let t = ProteinResponse::for_species("nile_tilapia");
        assert!(close(growth_factor(&t, 25.0, 35.0), 10.52 / 13.12, 0.02));
        // ปลาดุก: 25% ได้ครึ่งหนึ่งของ 40%
        let c = ProteinResponse::for_species("catfish");
        assert!(close(growth_factor(&c, 25.0, 40.0), 11.0 / 22.1, 0.02));
        // ปลาช่อน: 33.6% ได้ 73% ของ 42%
        let s = ProteinResponse::for_species("snakehead");
        assert!(close(growth_factor(&s, 33.6, 42.0), 6.91 / 9.41, 0.02));
    }
}
