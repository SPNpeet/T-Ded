use serde::{Deserialize, Serialize};

/// ค่าที่ใช้ตัดสินใจปรับอาหาร: อากาศ + คุณภาพน้ำ + สิ่งที่คนเลี้ยงสังเกตเห็น
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EnvInput {
    pub tmax_c: Option<f64>,
    pub tmin_c: Option<f64>,
    pub rain_mm: Option<f64>,
    pub cloud_pct: Option<f64>,
    /// DO ช่วงเช้ามืด (มก./ล.) ถ้ามี
    pub do_morning: Option<f64>,
    /// แอมโมเนียรวม (มก./ล.) ถ้ามี
    pub nh3: Option<f64>,
    #[serde(default)]
    pub stress: StressLevel,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StressLevel {
    #[default]
    Normal,
    /// เริ่มกินช้า / กินไม่หมด
    SlowEating,
    /// ลอยหัว / กินน้อยมาก
    Gasping,
}

impl StressLevel {
    fn as_f64(self) -> f64 {
        match self {
            StressLevel::Normal => 0.0,
            StressLevel::SlowEating => 1.0,
            StressLevel::Gasping => 2.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Metric {
    Tmax,
    Tmin,
    Rain,
    Cloud,
    DoMorning,
    Nh3,
    Stress,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Op {
    /// value > a
    Gt,
    /// value >= a
    Gte,
    /// value < a
    Lt,
    /// a <= value <= b
    Between,
    /// value == a (ใช้กับ stress)
    Eq,
}

/// กลุ่มกติกา: ในกลุ่มเดียวกันใช้กติกาแรกที่ตรงเพียงข้อเดียว (เรียงจากรุนแรงมากไปน้อย)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleGroup {
    Heat,
    Cold,
    Rain,
    Cloud,
    Oxygen,
    Ammonia,
    Observation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdjustRule {
    pub group: RuleGroup,
    pub metric: Metric,
    pub op: Op,
    pub a: f64,
    #[serde(default)]
    pub b: f64,
    pub factor: f64,
    /// ข้อความอธิบายให้ผู้ใช้ (ภาษาไทย ไม่มีอิโมจิ) รองรับ {v} แทนค่าที่วัดได้
    pub label_th: String,
}

impl AdjustRule {
    fn new(group: RuleGroup, metric: Metric, op: Op, a: f64, b: f64, factor: f64, label: &str) -> Self {
        AdjustRule { group, metric, op, a, b, factor, label_th: label.to_string() }
    }
}

/// กติกาเริ่มต้น — ชุดเดียวกับคู่มือโชคอนันต์ฟาร์ม + เพิ่ม DO/แอมโมเนียเมื่อมีค่าวัด
pub fn default_rules() -> Vec<AdjustRule> {
    use Metric as M;
    use Op::*;
    use RuleGroup as G;
    vec![
        AdjustRule::new(G::Heat, M::Tmax, Gt, 35.0, 0.0, 0.70, "ร้อนจัดช่วงบ่าย สูงสุด {v} องศา ปลาเครียดและออกซิเจนในน้ำลด"),
        AdjustRule::new(G::Heat, M::Tmax, Gte, 33.0, 0.0, 0.85, "ร้อนช่วงบ่าย สูงสุด {v} องศา ปลาเริ่มเครียด"),
        AdjustRule::new(G::Cold, M::Tmin, Lt, 20.0, 0.0, 0.50, "หนาวจัดช่วงเช้า ต่ำสุด {v} องศา ปลาเกือบหยุดกิน"),
        AdjustRule::new(G::Cold, M::Tmin, Lt, 24.0, 0.0, 0.70, "หนาวช่วงเช้า ต่ำสุด {v} องศา ความอยากอาหารลด"),
        AdjustRule::new(G::Cold, M::Tmin, Lt, 28.0, 0.0, 0.90, "เย็นช่วงเช้า ต่ำสุด {v} องศา กินช้าลงเล็กน้อย"),
        AdjustRule::new(G::Rain, M::Rain, Gt, 30.0, 0.0, 0.80, "ฝนตกหนัก {v} มม. ความกดอากาศต่ำ ปลากินน้อย"),
        AdjustRule::new(G::Rain, M::Rain, Gt, 10.0, 0.0, 0.92, "ฝนปานกลาง {v} มม. เฝ้าระวังการกิน"),
        AdjustRule::new(G::Cloud, M::Cloud, Gte, 75.0, 0.0, 0.90, "ฟ้าครึ้ม เมฆ {v}% สังเคราะห์แสงน้อย ออกซิเจนต่ำ"),
        AdjustRule::new(G::Oxygen, M::DoMorning, Lt, 2.0, 0.0, 0.50, "ออกซิเจนเช้า {v} มก./ล. ต่ำมาก งดหรือลดอาหารและเปิดเครื่องตีน้ำ"),
        AdjustRule::new(G::Oxygen, M::DoMorning, Lt, 3.0, 0.0, 0.75, "ออกซิเจนเช้า {v} มก./ล. ต่ำ ลดอาหารและเพิ่มการตีน้ำ"),
        AdjustRule::new(G::Oxygen, M::DoMorning, Lt, 4.0, 0.0, 0.90, "ออกซิเจนเช้า {v} มก./ล. ค่อนข้างต่ำ"),
        AdjustRule::new(G::Ammonia, M::Nh3, Gt, 1.0, 0.0, 0.70, "แอมโมเนีย {v} มก./ล. สูง ลดอาหารและเปลี่ยนถ่ายน้ำ"),
        AdjustRule::new(G::Ammonia, M::Nh3, Gt, 0.5, 0.0, 0.85, "แอมโมเนีย {v} มก./ล. เริ่มสูง"),
        AdjustRule::new(G::Observation, M::Stress, Eq, 2.0, 0.0, 0.50, "สังเกตปลาลอยหัวหรือกินน้อยมาก ลดทันทีและตรวจออกซิเจน/แอมโมเนีย"),
        AdjustRule::new(G::Observation, M::Stress, Eq, 1.0, 0.0, 0.80, "สังเกตปลาเริ่มกินช้า ลดเพื่อไม่ให้อาหารเหลือ"),
    ]
}

/// ตัวคูณต่ำสุดหลังคูณทุกกติกา (กันปลาอดอาหาร)
pub const FACTOR_FLOOR: f64 = 0.40;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reason {
    pub group: RuleGroup,
    pub factor: f64,
    pub text_th: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Adjustment {
    pub factor: f64,
    pub reasons: Vec<Reason>,
    /// normal | down | cut
    pub band: String,
    /// true ถ้าอุณหภูมิทั้งวันอยู่ในช่วงเหมาะสม (ข้อความบวก)
    pub temp_optimal: bool,
}

fn value_of(env: &EnvInput, m: Metric) -> Option<f64> {
    match m {
        Metric::Tmax => env.tmax_c,
        Metric::Tmin => env.tmin_c,
        Metric::Rain => env.rain_mm,
        Metric::Cloud => env.cloud_pct,
        Metric::DoMorning => env.do_morning,
        Metric::Nh3 => env.nh3,
        Metric::Stress => Some(env.stress.as_f64()),
    }
}

fn matches(rule: &AdjustRule, v: f64) -> bool {
    match rule.op {
        Op::Gt => v > rule.a,
        Op::Gte => v >= rule.a,
        Op::Lt => v < rule.a,
        Op::Between => v >= rule.a && v <= rule.b,
        Op::Eq => (v - rule.a).abs() < 1e-9,
    }
}

fn fmt_value(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 {
        format!("{}", v.round() as i64)
    } else {
        format!("{:.1}", v)
    }
}

/// คำนวณตัวปรับรวมจากกติกา: แต่ละกลุ่มใช้กติกาแรกที่ตรง แล้วคูณทุกกลุ่มเข้าด้วยกัน
pub fn compute_adjustment(env: &EnvInput, rules: &[AdjustRule], temp_ideal: (f64, f64)) -> Adjustment {
    let mut factor = 1.0;
    let mut reasons = Vec::new();
    let mut done_groups: Vec<RuleGroup> = Vec::new();

    for rule in rules {
        if done_groups.contains(&rule.group) {
            continue;
        }
        let Some(v) = value_of(env, rule.metric) else { continue };
        if matches(rule, v) {
            if rule.metric == Metric::Stress && rule.factor >= 1.0 {
                continue;
            }
            factor *= rule.factor;
            reasons.push(Reason {
                group: rule.group,
                factor: rule.factor,
                text_th: rule.label_th.replace("{v}", &fmt_value(v)),
            });
            done_groups.push(rule.group);
        }
    }

    let temp_optimal = match (env.tmin_c, env.tmax_c) {
        (Some(lo), Some(hi)) => lo >= temp_ideal.0 && hi <= temp_ideal.1,
        _ => false,
    };

    if factor < FACTOR_FLOOR {
        factor = FACTOR_FLOOR;
    }
    let factor = crate::round(factor, 3);
    let band = if factor >= 0.95 {
        "normal"
    } else if factor >= 0.75 {
        "down"
    } else {
        "cut"
    };
    Adjustment { factor, reasons, band: band.to_string(), temp_optimal }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn prototype_case_cold_morning() {
        let env = EnvInput { tmax_c: Some(30.0), tmin_c: Some(22.0), rain_mm: Some(0.0), cloud_pct: Some(40.0), ..Default::default() };
        let a = compute_adjustment(&env, &default_rules(), (28.0, 32.0));
        assert_abs_diff_eq!(a.factor, 0.70);
        assert_eq!(a.reasons.len(), 1);
        assert_eq!(a.band, "cut");
    }

    #[test]
    fn prototype_case_everything_bad_hits_floor() {
        let env = EnvInput {
            tmax_c: Some(36.0),
            tmin_c: Some(19.0),
            rain_mm: Some(40.0),
            cloud_pct: Some(90.0),
            stress: StressLevel::Gasping,
            ..Default::default()
        };
        let a = compute_adjustment(&env, &default_rules(), (28.0, 32.0));
        assert_abs_diff_eq!(a.factor, 0.40);
        assert_eq!(a.reasons.len(), 5);
    }

    #[test]
    fn optimal_day_no_change() {
        let env = EnvInput { tmax_c: Some(31.0), tmin_c: Some(28.0), rain_mm: Some(0.0), cloud_pct: Some(20.0), ..Default::default() };
        let a = compute_adjustment(&env, &default_rules(), (28.0, 32.0));
        assert_abs_diff_eq!(a.factor, 1.0);
        assert!(a.temp_optimal);
        assert_eq!(a.band, "normal");
    }

    #[test]
    fn low_do_reduces() {
        let env = EnvInput { do_morning: Some(2.5), ..Default::default() };
        let a = compute_adjustment(&env, &default_rules(), (28.0, 32.0));
        assert_abs_diff_eq!(a.factor, 0.75);
    }
}
