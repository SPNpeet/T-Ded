use serde::{Deserialize, Serialize};

use crate::species::WaterThresholds;
use crate::water::{assess_water, WaterSample};

/// ข้อมูลที่ใช้ให้คะแนนสุขภาพบ่อ ทุกอย่างเป็นทางเลือก ยิ่งกรอกมากคะแนนยิ่งแม่น
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthInput {
    pub water: WaterSample,
    /// อัตราตาย 7 วันล่าสุด (% ของปลาที่มีชีวิต)
    pub mortality_7d_pct: Option<f64>,
    /// การกิน 0 = ปกติ, 1 = กินช้า, 2 = ลอยหัว/ไม่กิน
    pub feeding_response: Option<u8>,
    /// สถานะการโต: ahead | on_track | behind | far_behind
    pub growth_status: Option<String>,
    /// จำนวนวันที่ไม่ได้บันทึกอะไรเลยติดต่อกัน (ยิ่งนานยิ่งไม่รู้ว่าบ่อเป็นอย่างไร)
    pub days_since_last_log: Option<u32>,
    /// คะแนนครั้งก่อน (ถ้ามี) เพื่อบอกแนวโน้ม
    pub previous_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthComponent {
    pub key: String,
    pub label_th: String,
    /// 0-1
    pub score: f64,
    pub weight: f64,
    pub note_th: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScore {
    /// 0-100
    pub score: u32,
    /// excellent | good | fair | poor | critical
    pub grade: String,
    pub grade_th: String,
    /// up | down | flat | none
    pub trend: String,
    pub components: Vec<HealthComponent>,
    /// การแจ้งเตือนสำคัญวันนี้ (เรียงจากด่วนสุด)
    pub alerts_th: Vec<String>,
    /// ความครบถ้วนของข้อมูล 0-100
    pub data_completeness: u32,
}

pub fn health_score(i: &HealthInput, t: &WaterThresholds) -> HealthScore {
    let mut comps: Vec<HealthComponent> = Vec::new();
    let mut alerts: Vec<(u8, String)> = Vec::new();

    let wa = assess_water(&i.water, t);
    if !wa.items.is_empty() {
        comps.push(HealthComponent {
            key: "water".into(),
            label_th: "คุณภาพน้ำ".into(),
            score: wa.score,
            weight: 0.40,
            note_th: wa.overall_th.clone(),
        });
        for it in &wa.items {
            match it.level.as_str() {
                "danger" => alerts.push((0, it.message_th.clone())),
                "warn" => alerts.push((2, it.message_th.clone())),
                _ => {}
            }
        }
    }

    if let Some(m) = i.mortality_7d_pct {
        let (s, note) = if m <= 0.5 {
            (1.0, "ตายน้อย ปกติ".to_string())
        } else if m <= 1.5 {
            (0.7, format!("ตาย {:.1}% ใน 7 วัน เริ่มสูง", m))
        } else if m <= 3.0 {
            (0.35, format!("ตาย {:.1}% ใน 7 วัน สูง", m))
        } else {
            (0.05, format!("ตาย {:.1}% ใน 7 วัน สูงมาก", m))
        };
        if s < 0.7 {
            alerts.push((if s < 0.35 { 0 } else { 1 }, format!("ปลาตายสะสม {:.1}% ใน 7 วัน ตรวจน้ำและอาการโรค", m)));
        }
        comps.push(HealthComponent { key: "mortality".into(), label_th: "อัตราตาย 7 วัน".into(), score: s, weight: 0.25, note_th: note });
    }

    if let Some(f) = i.feeding_response {
        let (s, note) = match f {
            0 => (1.0, "กินดี"),
            1 => (0.55, "กินช้า"),
            _ => (0.1, "ไม่กิน/ลอยหัว"),
        };
        if f == 2 {
            alerts.push((0, "ปลาลอยหัวหรือไม่กินอาหาร ตรวจออกซิเจนทันที".into()));
        } else if f == 1 {
            alerts.push((2, "ปลากินช้า ลดอาหารและสังเกตต่อ".into()));
        }
        comps.push(HealthComponent { key: "feeding".into(), label_th: "การกินอาหาร".into(), score: s, weight: 0.20, note_th: note.into() });
    }

    if let Some(g) = &i.growth_status {
        let (s, note) = match g.as_str() {
            "ahead" => (1.0, "โตเร็วกว่าเกณฑ์"),
            "on_track" => (0.9, "โตตามเกณฑ์"),
            "behind" => (0.55, "โตช้ากว่าเกณฑ์"),
            _ => (0.25, "โตช้ากว่าเกณฑ์มาก"),
        };
        if s < 0.6 {
            alerts.push((2, "ปลาโตช้ากว่ามาตรฐาน ดูคำแนะนำในหน้าชั่งน้ำหนัก".into()));
        }
        comps.push(HealthComponent { key: "growth".into(), label_th: "การเจริญเติบโต".into(), score: s, weight: 0.15, note_th: note.into() });
    }

    // ความสม่ำเสมอของการบันทึก: ไม่คิดคะแนนตรง ๆ แต่ลดความมั่นใจ
    let mut completeness = (comps.len() as f64 / 4.0 * 100.0).round() as u32;
    if let Some(d) = i.days_since_last_log {
        if d >= 3 {
            alerts.push((3, format!("ไม่ได้บันทึกมา {} วัน คะแนนอาจไม่ตรงกับสภาพจริง", d)));
            completeness = completeness.saturating_sub((d.min(10) * 5) as u32);
        }
    }

    let score = if comps.is_empty() {
        50.0
    } else {
        let wsum: f64 = comps.iter().map(|c| c.weight).sum();
        comps.iter().map(|c| c.score * c.weight).sum::<f64>() / wsum * 100.0
    };
    let score_u = score.round().clamp(0.0, 100.0) as u32;

    let (grade, grade_th) = if comps.is_empty() {
        ("unknown", "ยังไม่มีข้อมูล")
    } else if score_u >= 85 {
        ("excellent", "ดีมาก")
    } else if score_u >= 70 {
        ("good", "ดี")
    } else if score_u >= 50 {
        ("fair", "พอใช้")
    } else if score_u >= 30 {
        ("poor", "แย่")
    } else {
        ("critical", "วิกฤต")
    };

    let trend = match i.previous_score {
        Some(p) if score - p >= 3.0 => "up",
        Some(p) if p - score >= 3.0 => "down",
        Some(_) => "flat",
        None => "none",
    };

    alerts.sort_by_key(|a| a.0);
    let alerts_th = alerts.into_iter().map(|a| a.1).collect();

    HealthScore {
        score: score_u,
        grade: grade.into(),
        grade_th: grade_th.into(),
        trend: trend.into(),
        components: comps,
        alerts_th,
        data_completeness: completeness,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::species::SpeciesProfile;

    #[test]
    fn healthy_pond_scores_high() {
        let sp = SpeciesProfile::nile_tilapia();
        let h = health_score(
            &HealthInput {
                water: WaterSample { do_mg_l: Some(5.5), ph: Some(7.8), temp_c: Some(29.0), ..Default::default() },
                mortality_7d_pct: Some(0.2),
                feeding_response: Some(0),
                growth_status: Some("on_track".into()),
                days_since_last_log: Some(0),
                previous_score: Some(80.0),
            },
            &sp.water,
        );
        assert!(h.score >= 90, "score {}", h.score);
        assert_eq!(h.grade, "excellent");
        assert_eq!(h.trend, "up");
        assert!(h.alerts_th.is_empty());
    }

    #[test]
    fn critical_pond() {
        let sp = SpeciesProfile::nile_tilapia();
        let h = health_score(
            &HealthInput {
                water: WaterSample { do_mg_l: Some(1.2), ..Default::default() },
                mortality_7d_pct: Some(4.0),
                feeding_response: Some(2),
                ..Default::default()
            },
            &sp.water,
        );
        assert!(h.score < 30, "score {}", h.score);
        assert!(!h.alerts_th.is_empty());
    }
}
