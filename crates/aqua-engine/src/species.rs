use serde::{Deserialize, Serialize};

/// หลังพ้นตารางการโต ADG ลดลงครึ่งหนึ่งทุกกี่วัน
pub const TAIL_HALF_LIFE_DAYS: f64 = 60.0;

/// แถวตารางอัตราให้อาหาร: ปลาหนัก weight_g กรัม ให้ pct % ของน้ำหนักตัวต่อวัน ใช้เม็ด pellet_mm
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FeedRateRow {
    pub weight_g: f64,
    pub pct: f64,
    pub pellet_mm: f64,
}

/// แถวตารางการเจริญเติบโตมาตรฐาน: ช่วงวันที่ day_from..day_to น้ำหนักต้นช่วง weight_g และ ADG (ก./วัน)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GrowthRow {
    pub day_from: u32,
    pub day_to: u32,
    pub weight_g: f64,
    pub adg: f64,
}

/// เกณฑ์คุณภาพน้ำต่อชนิดปลา (ค่าต่ำสุด/สูงสุดที่ยอมรับได้ และช่วงเหมาะสม)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WaterThresholds {
    pub do_min: f64,
    pub do_ideal: f64,
    pub ph_min: f64,
    pub ph_max: f64,
    pub temp_min: f64,
    pub temp_max: f64,
    pub temp_ideal_min: f64,
    pub temp_ideal_max: f64,
    /// แอมโมเนียรวม (TAN) มก./ล. ที่เริ่มเตือน และที่อันตราย
    pub nh3_warn: f64,
    pub nh3_danger: f64,
    pub no2_warn: f64,
    pub no2_danger: f64,
    /// ความโปร่งใส (Secchi) ซม. ช่วงเหมาะสม
    pub secchi_min: f64,
    pub secchi_max: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeciesProfile {
    pub code: String,
    pub name_th: String,
    pub feed_table: Vec<FeedRateRow>,
    /// ตารางโตระยะเลี้ยง (วันที่ 1 = วันปล่อยที่น้ำหนักแถวแรก)
    pub growth: Vec<GrowthRow>,
    /// ตารางโตระยะลูกปลา (ทางเลือก)
    pub growth_fry: Vec<GrowthRow>,
    pub meals_per_day: u8,
    pub water: WaterThresholds,
    /// น้ำหนักที่ตลาดนิยม (ก.) ใช้เป็นค่าเริ่มต้นเป้าหมายจับ
    pub market_weight_g: f64,
    /// true = ตารางเป็นค่าเริ่มต้นโดยประมาณ ควรปรับตามฟาร์ม
    pub approximate: bool,
}

impl SpeciesProfile {
    /// ปลานิล — ตารางจากคู่มือหน่วยส่งเสริม โชคอนันต์ฟาร์ม (ยืนยันแล้ว)
    pub fn nile_tilapia() -> Self {
        // ต่ำกว่า 30 ก. ใช้ตารางของ NBDC (nbdcthailand.com 31 พ.ค. 2564 อ้างอิง NRC 1993): 1-5 ก. 6-10%, 5-20 ก. 4-6%
        let feed_table = vec![
            (1.0, 8.0, 0.5),
            (5.0, 6.0, 1.0),
            (20.0, 5.0, 1.5),
            (30.0, 4.5, 2.0),
            (50.0, 4.0, 2.0),
            (75.0, 3.5, 2.0),
            (100.0, 3.5, 3.0),
            (150.0, 3.5, 3.0),
            (200.0, 3.5, 3.0),
            (250.0, 3.5, 3.0),
            (300.0, 3.0, 3.0),
            (400.0, 3.0, 3.0),
            (500.0, 2.5, 3.0),
            (600.0, 2.0, 3.0),
            (700.0, 2.0, 3.0),
            (800.0, 1.8, 3.0),
            (900.0, 1.5, 3.0),
            (950.0, 1.5, 3.0),
        ]
        .into_iter()
        .map(|(w, p, m)| FeedRateRow { weight_g: w, pct: p, pellet_mm: m })
        .collect();

        let growth = vec![
            (1, 7, 30.0, 2.5),
            (7, 14, 70.0, 3.0),
            (14, 21, 90.0, 3.5),
            (21, 28, 120.0, 4.0),
            (28, 35, 150.0, 4.5),
            (35, 42, 190.0, 5.0),
            (42, 49, 230.0, 5.3),
            (49, 56, 260.0, 5.5),
            (56, 63, 300.0, 6.0),
            (63, 70, 350.0, 6.3),
            (70, 77, 400.0, 6.5),
            (77, 84, 440.0, 6.8),
            (84, 91, 500.0, 7.0),
            (91, 98, 540.0, 7.5),
            (98, 105, 600.0, 8.3),
            (105, 112, 660.0, 8.5),
            (112, 119, 730.0, 9.0),
            (119, 126, 790.0, 9.5),
            (126, 133, 860.0, 9.5),
            (133, 140, 960.0, 10.0),
            (140, 147, 1000.0, 10.0),
        ]
        .into_iter()
        .map(|(a, b, w, g)| GrowthRow { day_from: a, day_to: b, weight_g: w, adg: g })
        .collect();

        let growth_fry = vec![
            (1, 7, 1.0, 0.15),
            (7, 15, 4.0, 0.30),
            (15, 23, 7.0, 0.45),
            (23, 30, 12.0, 0.60),
            (30, 37, 17.0, 0.75),
            (37, 45, 24.0, 0.90),
            (45, 53, 32.0, 1.00),
            (53, 60, 40.0, 1.20),
        ]
        .into_iter()
        .map(|(a, b, w, g)| GrowthRow { day_from: a, day_to: b, weight_g: w, adg: g })
        .collect();

        SpeciesProfile {
            code: "nile_tilapia".into(),
            name_th: "ปลานิล".into(),
            feed_table,
            growth,
            growth_fry,
            meals_per_day: 2,
            water: WaterThresholds {
                do_min: 3.0,
                do_ideal: 5.0,
                ph_min: 6.5,
                ph_max: 8.5,
                temp_min: 20.0,
                temp_max: 35.0,
                temp_ideal_min: 28.0,
                temp_ideal_max: 32.0,
                nh3_warn: 0.5,
                nh3_danger: 1.0,
                no2_warn: 0.3,
                no2_danger: 1.0,
                secchi_min: 30.0,
                secchi_max: 60.0,
            },
            market_weight_g: 800.0,
            approximate: false,
        }
    }

    /// ปลาทับทิม — สรีระใกล้เคียงปลานิล ใช้ตารางเดียวกันเป็นค่าเริ่มต้น
    pub fn red_tilapia() -> Self {
        let mut p = Self::nile_tilapia();
        p.code = "red_tilapia".into();
        p.name_th = "ปลาทับทิม".into();
        p.approximate = true;
        p
    }

    /// ปลาดุก — ค่าเริ่มต้นโดยประมาณจากแนวปฏิบัติทั่วไป ต้องปรับตามฟาร์ม
    pub fn catfish() -> Self {
        // อัตราให้อาหารตามแผนเลี้ยงของกรมประมง (สัปดาห์แรก 10-15%, สัปดาห์ 2 6-8%, 3-4 สัปดาห์ 4-6%,
        // 1-2.5 เดือน 3.2-4%, 2.5 เดือนถึงจับ 3-3.2%) แปลงเป็นน้ำหนักตามเส้นการโตด้านล่าง
        let feed_table = vec![
            (1.0, 12.0, 0.5),
            (3.0, 7.0, 1.0),
            (10.0, 5.0, 1.5),
            (30.0, 3.6, 2.0),
            (80.0, 3.1, 3.0),
            (200.0, 3.0, 3.0),
            (400.0, 2.8, 4.0),
        ]
        .into_iter()
        .map(|(w, p, m)| FeedRateRow { weight_g: w, pct: p, pellet_mm: m })
        .collect();
        // กรมประมง: 3-4 เดือนได้ 200-400 ก. / เกษตรศาสตร์-เบทาโกร 2567: ตอนนี้ 220 ก. ต้องใช้ 140-150 วัน
        // ใช้ค่ากลางราว 255 ก. ที่ 120 วัน
        let growth = vec![
            (1, 15, 5.0, 1.0),
            (15, 30, 20.0, 2.0),
            (30, 45, 50.0, 2.7),
            (45, 60, 90.0, 3.0),
            (60, 75, 135.0, 3.0),
            (75, 90, 180.0, 2.7),
            (90, 105, 220.0, 2.3),
            (105, 120, 255.0, 2.0),
        ]
        .into_iter()
        .map(|(a, b, w, g)| GrowthRow { day_from: a, day_to: b, weight_g: w, adg: g })
        .collect();
        SpeciesProfile {
            code: "catfish".into(),
            name_th: "ปลาดุก".into(),
            feed_table,
            growth,
            growth_fry: vec![],
            meals_per_day: 2,
            water: WaterThresholds {
                do_min: 2.0,
                do_ideal: 4.0,
                ph_min: 6.5,
                ph_max: 8.5,
                temp_min: 22.0,
                temp_max: 35.0,
                temp_ideal_min: 27.0,
                temp_ideal_max: 32.0,
                nh3_warn: 1.0,
                nh3_danger: 2.0,
                no2_warn: 0.5,
                no2_danger: 1.5,
                secchi_min: 20.0,
                secchi_max: 50.0,
            },
            market_weight_g: 250.0,
            approximate: true,
        }
    }

    /// ปลาช่อน — ค่าประมาณจากเอกสารกรมประมง (ปล่อย 27-28 ตัว/กก. เลี้ยงอาหารเม็ดลอยราว 5 เดือนได้ประมาณ 700 ก./ตัว, FCR 1.77)
    /// อัตราให้อาหารตอนปล่อย ~4% คำนวณจากตัวอย่างบ่อของกรมประมง ลูกปลาเล็ก 5-6% จากงานวิจัยต่างประเทศ ต้องปรับตามฟาร์ม
    pub fn snakehead() -> Self {
        let feed_table = vec![(10.0, 6.0, 1.5), (36.0, 4.0, 2.0), (100.0, 3.5, 3.0), (250.0, 3.0, 4.0), (500.0, 2.5, 5.0), (700.0, 2.0, 5.0)]
            .into_iter()
            .map(|(w, p, m)| FeedRateRow { weight_g: w, pct: p, pellet_mm: m })
            .collect();
        let growth = vec![(1, 30, 36.0, 2.1), (30, 60, 100.0, 3.3), (60, 90, 200.0, 4.7), (90, 120, 340.0, 5.7), (120, 150, 510.0, 6.3)]
            .into_iter()
            .map(|(a, b, w, g)| GrowthRow { day_from: a, day_to: b, weight_g: w, adg: g })
            .collect();
        SpeciesProfile {
            code: "snakehead".into(),
            name_th: "ปลาช่อน".into(),
            feed_table,
            growth,
            growth_fry: vec![],
            meals_per_day: 2,
            water: WaterThresholds {
                // ปลาช่อนหายใจจากอากาศได้ ทนออกซิเจนต่ำกว่าปลานิล
                do_min: 2.0,
                do_ideal: 4.0,
                ph_min: 6.5,
                ph_max: 8.5,
                temp_min: 22.0,
                temp_max: 34.0,
                temp_ideal_min: 26.0,
                temp_ideal_max: 32.0,
                nh3_warn: 1.0,
                nh3_danger: 2.0,
                no2_warn: 0.5,
                no2_danger: 1.5,
                secchi_min: 20.0,
                secchi_max: 50.0,
            },
            market_weight_g: 600.0,
            approximate: true,
        }
    }

    pub fn defaults() -> Vec<SpeciesProfile> {
        vec![Self::nile_tilapia(), Self::red_tilapia(), Self::catfish(), Self::snakehead()]
    }

    pub fn by_code(code: &str) -> Option<SpeciesProfile> {
        Self::defaults().into_iter().find(|s| s.code == code)
    }

    /// % อาหารต่อวันที่น้ำหนัก w (interpolate เชิงเส้น นอกช่วงใช้ค่าปลาย)
    pub fn feed_pct(&self, w: f64) -> f64 {
        let t = &self.feed_table;
        if t.is_empty() {
            return 0.0;
        }
        if w <= t[0].weight_g {
            return t[0].pct;
        }
        let last = t[t.len() - 1];
        if w >= last.weight_g {
            return last.pct;
        }
        for pair in t.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if w >= a.weight_g && w <= b.weight_g {
                let f = (w - a.weight_g) / (b.weight_g - a.weight_g);
                return a.pct + f * (b.pct - a.pct);
            }
        }
        last.pct
    }

    /// ขนาดเม็ดอาหารที่น้ำหนัก w: ใช้แถวแรกที่ weight_g >= w
    pub fn pellet_mm(&self, w: f64) -> f64 {
        for r in &self.feed_table {
            if w <= r.weight_g {
                return r.pellet_mm;
            }
        }
        self.feed_table.last().map(|r| r.pellet_mm).unwrap_or(0.0)
    }

    /// จุดอ้างอิงเส้นโค้งมาตรฐาน (วัน, น้ำหนัก) จากตารางโต
    pub fn growth_points(&self) -> Vec<(f64, f64)> {
        let mut pts: Vec<(f64, f64)> = Vec::new();
        if let (Some(first), false) = (self.growth.first(), self.growth_fry.is_empty()) {
            // หาวันบนเส้นลูกปลาที่ปลาหนักเท่าจุดเริ่มของตารางเลี้ยง แล้วเลื่อนเส้นลูกปลาไปต่อหน้าวันที่ 1
            let fry: Vec<(f64, f64)> = self.growth_fry.iter().map(|r| (r.day_from as f64, r.weight_g)).collect();
            let join = fry
                .windows(2)
                .find(|w| first.weight_g >= w[0].1 && first.weight_g <= w[1].1)
                .map(|w| w[0].0 + (first.weight_g - w[0].1) / (w[1].1 - w[0].1).max(1e-9) * (w[1].0 - w[0].0));
            if let Some(join) = join {
                let shift = first.day_from as f64 - join;
                pts.extend(fry.iter().filter(|(_, wt)| *wt < first.weight_g).map(|(d, wt)| (d + shift, *wt)));
            }
        }
        pts.extend(self.growth.iter().map(|r| (r.day_from as f64, r.weight_g)));
        if let Some(last) = self.growth.last() {
            let span = (last.day_to - last.day_from) as f64;
            pts.push((last.day_to as f64, last.weight_g + last.adg * span));
        }
        pts
    }

    /// น้ำหนักมาตรฐานที่วันที่ day บนเส้นโค้ง (นอกช่วงต่อเส้นด้วย ADG ปลาย)
    pub fn standard_weight_at(&self, day: f64) -> f64 {
        let pts = self.growth_points();
        if pts.is_empty() {
            return 0.0;
        }
        if day <= pts[0].0 {
            let adg = if pts.len() >= 2 { (pts[1].1 - pts[0].1) / (pts[1].0 - pts[0].0).max(1e-9) } else { self.growth[0].adg };
            return (pts[0].1 - (pts[0].0 - day) * adg).max(0.01);
        }
        for pair in pts.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if day >= a.0 && day <= b.0 {
                let f = if b.0 > a.0 { (day - a.0) / (b.0 - a.0) } else { 0.0 };
                return a.1 + f * (b.1 - a.1);
            }
        }
        // เลยตารางไปแล้ว ปลาโตช้าลงเรื่อย ๆ (ADG ลดครึ่งทุก TAIL_HALF_LIFE_DAYS วัน) ไม่ต่อเส้นตรงไปไม่สิ้นสุด
        let last = pts[pts.len() - 1];
        let adg = self.growth.last().map(|r| r.adg).unwrap_or(0.0);
        let k = std::f64::consts::LN_2 / TAIL_HALF_LIFE_DAYS;
        last.1 + adg * (1.0 - (-k * (day - last.0)).exp()) / k
    }

    /// วันบนเส้นโค้งมาตรฐานที่ปลาจะหนัก w (inverse ของ standard_weight_at)
    pub fn standard_day_for_weight(&self, w: f64) -> f64 {
        let pts = self.growth_points();
        if pts.is_empty() {
            return 0.0;
        }
        if w <= pts[0].1 {
            let adg = if pts.len() >= 2 { (pts[1].1 - pts[0].1) / (pts[1].0 - pts[0].0).max(1e-9) } else { self.growth[0].adg };
            return pts[0].0 - (pts[0].1 - w) / adg.max(1e-9);
        }
        for pair in pts.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if w >= a.1 && w <= b.1 {
                let f = if b.1 > a.1 { (w - a.1) / (b.1 - a.1) } else { 0.0 };
                return a.0 + f * (b.0 - a.0);
            }
        }
        let last = pts[pts.len() - 1];
        let adg = self.growth.last().map(|r| r.adg).unwrap_or(1.0).max(1e-9);
        let k = std::f64::consts::LN_2 / TAIL_HALF_LIFE_DAYS;
        let x = 1.0 - (w - last.1) * k / adg;
        if x <= 1e-6 {
            // เกินขนาดสูงสุดที่เส้นโค้งไปถึงได้
            return last.0 + TAIL_HALF_LIFE_DAYS * 20.0;
        }
        last.0 - x.ln() / k
    }

    /// น้ำหนักสูงสุดที่เส้นโค้งมาตรฐานเข้าใกล้ (ใช้บอกว่าเป้าหมายเกินความเป็นไปได้)
    pub fn max_standard_weight(&self) -> f64 {
        let pts = self.growth_points();
        let Some(last) = pts.last() else { return 0.0 };
        let adg = self.growth.last().map(|r| r.adg).unwrap_or(0.0);
        last.1 + adg * TAIL_HALF_LIFE_DAYS / std::f64::consts::LN_2
    }

    /// ADG มาตรฐาน ณ น้ำหนัก w (ความชันของเส้นโค้งตรงจุดนั้น)
    pub fn standard_adg_at_weight(&self, w: f64) -> f64 {
        let d = self.standard_day_for_weight(w);
        self.standard_weight_at(d + 1.0) - self.standard_weight_at(d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn pct_interpolates_like_prototype() {
        let s = SpeciesProfile::nile_tilapia();
        assert_abs_diff_eq!(s.feed_pct(300.0), 3.0);
        assert_abs_diff_eq!(s.feed_pct(65.0), 3.7, epsilon = 1e-9);
        assert_abs_diff_eq!(s.feed_pct(10.0), 6.0 - 5.0 / 15.0 * 1.0, epsilon = 1e-9);
        assert_abs_diff_eq!(s.feed_pct(0.5), 8.0);
        assert_abs_diff_eq!(s.feed_pct(2000.0), 1.5);
        assert_abs_diff_eq!(s.pellet_mm(75.0), 2.0);
        assert_abs_diff_eq!(s.pellet_mm(76.0), 3.0);
    }

    #[test]
    fn growth_curve_roundtrip() {
        let s = SpeciesProfile::nile_tilapia();
        assert_abs_diff_eq!(s.standard_weight_at(56.0), 300.0);
        assert_abs_diff_eq!(s.standard_day_for_weight(300.0), 56.0);
        let d = s.standard_day_for_weight(415.0);
        assert_abs_diff_eq!(s.standard_weight_at(d), 415.0, epsilon = 1e-9);
        assert!(s.standard_weight_at(200.0) > 1000.0);
        // เลยตารางแล้วต้องไม่โตเป็นเส้นตรงไม่สิ้นสุด และกลับด้านได้
        assert!(s.standard_weight_at(400.0) < s.max_standard_weight());
        // ลูกปลา 1 ก. ต้องใช้เวลาหลายสิบวันกว่าจะถึง 30 ก. ไม่ใช่โตวันละ 2.5 ก.
        let fry_days = s.standard_day_for_weight(30.0) - s.standard_day_for_weight(1.0);
        assert!(fry_days > 35.0 && fry_days < 55.0, "fry_days {fry_days}");
        let d2 = s.standard_day_for_weight(1500.0);
        assert_abs_diff_eq!(s.standard_weight_at(d2), 1500.0, epsilon = 1e-6);
    }
}
