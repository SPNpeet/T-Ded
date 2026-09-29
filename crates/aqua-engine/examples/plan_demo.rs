use aqua_engine::{plan, FeedProduct, PlanRequest, SpeciesProfile};

fn main() {
    let p = |code: &str, target: &str, from: f64, to: f64, prot: f64, mm: f64| FeedProduct {
        code: code.into(), name_th: code.into(), brand: "NB".into(), target: target.into(),
        weight_from_g: from, weight_to_g: to, protein_pct: prot, pellet_mm: Some(mm), bag_kg: 20.0, price_per_bag: None, color_th: None,
    };
    let products = vec![
        p("GF 411 SM", "herbivore", 0.0, 300.0, 32.0, 3.0),
        p("GZ 401 M", "herbivore", 0.0, 300.0, 32.0, 3.0),
        p("NB 202", "herbivore", 300.0, 100000.0, 15.5, 4.0),
        p("GZ 202 L", "herbivore", 300.0, 100000.0, 15.5, 4.0),
    ];
    for (label, days, target, custom) in [
        ("ปลานิล 30ก.->800ก. ไม่กำหนดวัน", None, 800.0, vec![]),
        ("ปลานิล 8 เดือน (240 วัน) เป้า 1000 ก.", Some(240), 1000.0, vec![]),
        ("ปลานิล ภายใน 2 เดือน เป้า 800 ก.", Some(60), 800.0, vec![]),
        ("ใช้ NB 202 (15.5%) ตลอดรุ่น 150 วัน", Some(150), 800.0, vec!["NB 202"]),
    ] {
        let r = plan(&PlanRequest {
            species: SpeciesProfile::nile_tilapia(), stock_weight_g: 30.0, count: 5000.0, survival_pct: 85.0,
            target_weight_g: Some(target), target_days: days, products: products.clone(),
            custom_codes: custom.into_iter().map(String::from).collect(), response: None, farm_growth_scale: None, max_days: None,
        });
        println!("\n=== {label}\n{}", r.message_th);
        for (i, pl) in r.plans.iter().enumerate() {
            println!("  [{}]{} {} | วันสุดท้าย {} | {:.0} ก. | ถึงเป้า {:?} | อาหาร {:.0} กก. = {:.1} กระสอบ | FCR {:?}",
                if i == r.recommended { "*" } else { " " }, pl.strategy_th, "", pl.final_day, pl.final_weight_g, pl.day_reached, pl.feed_kg_total, pl.bags_total, pl.fcr);
            for s in &pl.stages {
                println!("       วัน {:>3}-{:<3} {:<10} โปรตีน {:>4}% ต้องการ {:>4}% โต x{:.2} | {:.0}->{:.0} ก. | {:.1} กระสอบ | {} มื้อ",
                    s.from_day, s.to_day, s.product_code, s.protein_pct, s.requirement_pct, s.growth_factor, s.from_weight_g, s.to_weight_g, s.bags, s.meals_per_day);
            }
            for w in &pl.warnings_th { println!("       ! {w}"); }
        }
    }
}
