use serde::{Deserialize, Serialize};

use crate::species::WaterThresholds;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WaterSample {
    pub do_mg_l: Option<f64>,
    pub ph: Option<f64>,
    pub temp_c: Option<f64>,
    /// แอมโมเนียรวม มก./ล.
    pub nh3: Option<f64>,
    pub no2: Option<f64>,
    pub secchi_cm: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricAssessment {
    pub metric: String,
    pub label_th: String,
    pub value: f64,
    pub unit: String,
    /// good | warn | danger
    pub level: String,
    pub message_th: String,
    pub advice_th: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterAssessment {
    pub items: Vec<MetricAssessment>,
    /// good | warn | danger (แย่สุดในชุด)
    pub overall: String,
    pub overall_th: String,
    /// สัดส่วน 0-1 สำหรับใช้ในคะแนนสุขภาพบ่อ (1 = ทุกค่าดี)
    pub score: f64,
}

fn item(metric: &str, label: &str, v: f64, unit: &str, level: &str, msg: String, advice: Option<&str>) -> MetricAssessment {
    MetricAssessment {
        metric: metric.into(),
        label_th: label.into(),
        value: v,
        unit: unit.into(),
        level: level.into(),
        message_th: msg,
        advice_th: advice.map(String::from),
    }
}

pub fn assess_water(s: &WaterSample, t: &WaterThresholds) -> WaterAssessment {
    let mut items = Vec::new();

    if let Some(v) = s.do_mg_l {
        let (lvl, msg, adv) = if v < t.do_min * 0.67 {
            ("danger", format!("ออกซิเจน {:.1} ต่ำมาก ปลาเสี่ยงตาย", v), Some("เปิดเครื่องตีน้ำทันที งดอาหารมื้อนี้ เติมน้ำใหม่ถ้าทำได้"))
        } else if v < t.do_min {
            ("danger", format!("ออกซิเจน {:.1} ต่ำกว่าเกณฑ์ {:.0}", v, t.do_min), Some("เปิดเครื่องตีน้ำ ลดอาหารครึ่งหนึ่ง"))
        } else if v < t.do_ideal {
            ("warn", format!("ออกซิเจน {:.1} พอใช้ (ควร {:.0} ขึ้นไป)", v, t.do_ideal), Some("ตีน้ำช่วงเช้ามืด และอย่าให้อาหารเกิน"))
        } else {
            ("good", format!("ออกซิเจน {:.1} ดี", v), None)
        };
        items.push(item("do", "ออกซิเจนละลายน้ำ", v, "มก./ล.", lvl, msg, adv));
    }
    if let Some(v) = s.ph {
        let (lvl, msg, adv) = if v < t.ph_min - 1.0 || v > t.ph_max + 1.0 {
            ("danger", format!("pH {:.1} ผิดปกติมาก", v), Some(if v < t.ph_min { "โรยปูนขาวปรับ pH และเปลี่ยนถ่ายน้ำบางส่วน" } else { "ลดแสง/แพลงก์ตอน เปลี่ยนถ่ายน้ำ งดปูน" }))
        } else if v < t.ph_min || v > t.ph_max {
            ("warn", format!("pH {:.1} นอกช่วง {:.1}-{:.1}", v, t.ph_min, t.ph_max), Some(if v < t.ph_min { "โรยปูนขาวเล็กน้อยช่วงเย็น" } else { "เฝ้าระวังช่วงบ่าย ลดอาหาร" }))
        } else {
            ("good", format!("pH {:.1} เหมาะสม", v), None)
        };
        items.push(item("ph", "ความเป็นกรด-ด่าง", v, "", lvl, msg, adv));
    }
    if let Some(v) = s.temp_c {
        let (lvl, msg, adv) = if v < t.temp_min || v > t.temp_max {
            ("danger", format!("อุณหภูมิน้ำ {:.1} องศา อันตราย", v), Some("ลดอาหารครึ่งหนึ่ง เพิ่มระดับน้ำให้ลึกขึ้น"))
        } else if v < t.temp_ideal_min || v > t.temp_ideal_max {
            ("warn", format!("อุณหภูมิน้ำ {:.1} องศา นอกช่วงเหมาะสม {:.0}-{:.0}", v, t.temp_ideal_min, t.temp_ideal_max), Some("ปรับอาหารตามตัวปรับอากาศ"))
        } else {
            ("good", format!("อุณหภูมิน้ำ {:.1} องศา เหมาะสม", v), None)
        };
        items.push(item("temp", "อุณหภูมิน้ำ", v, "องศา", lvl, msg, adv));
    }
    if let Some(v) = s.nh3 {
        let (lvl, msg, adv) = if v >= t.nh3_danger {
            ("danger", format!("แอมโมเนีย {:.2} สูงอันตราย", v), Some("เปลี่ยนถ่ายน้ำ 20-30% งดอาหาร 1 มื้อ ตีน้ำเพิ่ม"))
        } else if v >= t.nh3_warn {
            ("warn", format!("แอมโมเนีย {:.2} เริ่มสูง", v), Some("ลดอาหาร 15% ตรวจซ้ำพรุ่งนี้"))
        } else {
            ("good", format!("แอมโมเนีย {:.2} ปกติ", v), None)
        };
        items.push(item("nh3", "แอมโมเนีย", v, "มก./ล.", lvl, msg, adv));
    }
    if let Some(v) = s.no2 {
        let (lvl, msg, adv) = if v >= t.no2_danger {
            ("danger", format!("ไนไตรท์ {:.2} สูงอันตราย", v), Some("เปลี่ยนถ่ายน้ำ เติมเกลือ 0.1-0.3% ลดอาหาร"))
        } else if v >= t.no2_warn {
            ("warn", format!("ไนไตรท์ {:.2} เริ่มสูง", v), Some("ลดอาหาร ตรวจซ้ำ"))
        } else {
            ("good", format!("ไนไตรท์ {:.2} ปกติ", v), None)
        };
        items.push(item("no2", "ไนไตรท์", v, "มก./ล.", lvl, msg, adv));
    }
    if let Some(v) = s.secchi_cm {
        let (lvl, msg, adv) = if v < t.secchi_min * 0.6 {
            ("danger", format!("น้ำขุ่นมาก ({:.0} ซม.) แพลงก์ตอนหนาแน่น เสี่ยงออกซิเจนตกกลางคืน", v), Some("ลดอาหาร เปลี่ยนถ่ายน้ำ ตีน้ำกลางคืน"))
        } else if v < t.secchi_min {
            ("warn", format!("น้ำค่อนข้างขุ่น ({:.0} ซม.)", v), Some("ลดอาหารเล็กน้อย เฝ้าระวังออกซิเจนเช้า"))
        } else if v > t.secchi_max {
            ("warn", format!("น้ำใสเกินไป ({:.0} ซม.) อาหารธรรมชาติน้อย", v), Some("พิจารณาเติมปุ๋ยสร้างสีน้ำตามคำแนะนำ"))
        } else {
            ("good", format!("ความโปร่งใส {:.0} ซม. เหมาะสม", v), None)
        };
        items.push(item("secchi", "ความโปร่งใสน้ำ", v, "ซม.", lvl, msg, adv));
    }

    let (overall, overall_th) = if items.iter().any(|i| i.level == "danger") {
        ("danger", "น้ำมีปัญหา ต้องแก้ทันที")
    } else if items.iter().any(|i| i.level == "warn") {
        ("warn", "น้ำพอใช้ เฝ้าระวัง")
    } else if items.is_empty() {
        ("unknown", "ยังไม่มีข้อมูลน้ำ")
    } else {
        ("good", "น้ำดี")
    };
    let score = if items.is_empty() {
        1.0
    } else {
        let sum: f64 = items
            .iter()
            .map(|i| match i.level.as_str() {
                "good" => 1.0,
                "warn" => 0.6,
                _ => 0.15,
            })
            .sum();
        sum / items.len() as f64
    };

    WaterAssessment { items, overall: overall.into(), overall_th: overall_th.into(), score: crate::round(score, 3) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::species::SpeciesProfile;

    #[test]
    fn danger_when_low_do() {
        let sp = SpeciesProfile::nile_tilapia();
        let a = assess_water(&WaterSample { do_mg_l: Some(1.5), ph: Some(7.5), ..Default::default() }, &sp.water);
        assert_eq!(a.overall, "danger");
        assert_eq!(a.items.len(), 2);
    }
}
