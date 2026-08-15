//! รวมข้อมูลจริงของรุ่นการเลี้ยงแล้วส่งเข้า aqua-engine: อาหารวันนี้ ผลงาน การโต คะแนนสุขภาพ พยากรณ์

use aqua_engine::{
    compare_growth, health_score, performance, project, recommend, AdjustRule, EnvInput, FeedInput, HealthInput, PerfInput,
    ProjectionInput, SpeciesProfile, StressLevel, WaterSample,
};
use axum::{extract::{Path, Query, State}, Json};
use serde_json::{json, Value};
use sqlx::Row;

use crate::{
    api::{assert_farm_access, farm_of_crop},
    auth::AuthUser,
    db::{row_to_json, rows_to_json, today_bkk},
    error::{ApiResult, AppError},
    weather::fetch_daily,
    AppState,
};

pub async fn species_for(st: &AppState, org_id: &str, code: &str) -> ApiResult<SpeciesProfile> {
    if let Some(r) = sqlx::query("SELECT profile_json FROM species_overrides WHERE org_id = ? AND code = ?").bind(org_id).bind(code).fetch_optional(&st.db).await? {
        let s: String = r.get("profile_json");
        if let Ok(p) = serde_json::from_str::<SpeciesProfile>(&s) {
            return Ok(p);
        }
    }
    SpeciesProfile::by_code(code).ok_or_else(|| AppError::BadRequest("ไม่รู้จักชนิดปลา".into()))
}

pub async fn rules_for(st: &AppState, org_id: &str) -> ApiResult<Vec<AdjustRule>> {
    if let Some(r) = sqlx::query("SELECT rules_json FROM rule_overrides WHERE org_id = ?").bind(org_id).fetch_optional(&st.db).await? {
        let s: String = r.get("rules_json");
        if let Ok(v) = serde_json::from_str::<Vec<AdjustRule>>(&s) {
            return Ok(v);
        }
    }
    Ok(aqua_engine::env::default_rules())
}

fn days_between(a: &str, b: &str) -> i64 {
    let pa = chrono::NaiveDate::parse_from_str(a, "%Y-%m-%d");
    let pb = chrono::NaiveDate::parse_from_str(b, "%Y-%m-%d");
    match (pa, pb) {
        (Ok(x), Ok(y)) => (y - x).num_days(),
        _ => 0,
    }
}

fn stress_from(v: i64) -> StressLevel {
    match v {
        1 => StressLevel::SlowEating,
        2 => StressLevel::Gasping,
        _ => StressLevel::Normal,
    }
}

/// สแนปช็อตเต็มของรุ่น ณ วันที่กำหนด (ค่าเริ่มต้นวันนี้)
pub async fn crop_snapshot(st: &AppState, crop_id: &str, date: &str, org_id: &str, opts: &SnapshotOpts) -> ApiResult<Value> {
    let crop_row = sqlx::query("SELECT c.*, p.name AS pond_name, p.area_rai, f.name AS farm_name, f.lat, f.lng, f.province, f.meals_per_day, f.farm_factor, f.bag_kg FROM crops c JOIN ponds p ON p.id = c.pond_id JOIN farms f ON f.id = c.farm_id WHERE c.id = ?")
        .bind(crop_id)
        .fetch_optional(&st.db)
        .await?
        .ok_or(AppError::NotFound)?;
    let crop = row_to_json(&crop_row);
    let species_code = crop["species_code"].as_str().unwrap_or("nile_tilapia").to_string();
    let sp = species_for(st, org_id, &species_code).await?;
    let rules = rules_for(st, org_id).await?;

    let stocked_at = crop["stocked_at"].as_str().unwrap_or(date).to_string();
    let day = days_between(&stocked_at, date).max(0) as u32;
    let stocked_count = crop["stocked_count"].as_f64().unwrap_or(0.0);
    let stock_w = crop["stock_weight_g"].as_f64().unwrap_or(30.0);
    let target_w = crop["target_weight_g"].as_f64().unwrap_or(sp.market_weight_g);

    // น้ำหนักล่าสุดจากการชั่ง + ประมาณค่าถึงวันนี้ด้วยอัตราการโตจริงหรือมาตรฐาน
    let weighings = sqlx::query("SELECT weigh_date, avg_weight_g, sample_count FROM weighings WHERE crop_id = ? AND weigh_date <= ? ORDER BY weigh_date, created_at").bind(crop_id).bind(date).fetch_all(&st.db).await?;
    let wlist: Vec<(String, f64)> = weighings.iter().map(|r| (r.get::<String, _>("weigh_date"), r.get::<f64, _>("avg_weight_g"))).collect();
    let (last_date, last_w) = wlist.last().cloned().unwrap_or((stocked_at.clone(), stock_w));
    let prev = if wlist.len() >= 2 { Some(wlist[wlist.len() - 2].clone()) } else { None };
    let last_day = days_between(&stocked_at, &last_date).max(0) as u32;
    let recent_adg = prev.as_ref().and_then(|(pd, pw)| {
        let d = days_between(pd, &last_date);
        if d > 0 { Some((last_w - pw) / d as f64) } else { None }
    });
    let growth_scale = recent_adg
        .map(|a| {
            let std = sp.standard_adg_at_weight(last_w).max(0.01);
            (a / std).clamp(0.3, 1.5)
        })
        .unwrap_or(1.0);
    let days_since_weigh = day.saturating_sub(last_day);
    let est_w = if days_since_weigh == 0 {
        last_w
    } else {
        let d0 = sp.standard_day_for_weight(last_w);
        sp.standard_weight_at(d0 + days_since_weigh as f64 * growth_scale).max(last_w)
    };

    // ตัวเลขสะสม
    let sums = sqlx::query("SELECT COALESCE(SUM(mortality),0) AS dead, COALESCE(SUM(fed_kg),0.0) AS fed, COUNT(*) AS n, MAX(log_date) AS last_log FROM daily_logs WHERE crop_id = ? AND log_date <= ?").bind(crop_id).bind(date).fetch_one(&st.db).await?;
    let dead: f64 = sums.get::<i64, _>("dead") as f64;
    let fed_total: f64 = sums.get("fed");
    let last_log: Option<String> = sums.get("last_log");
    let harv = sqlx::query("SELECT COALESCE(SUM(count),0) AS c, COALESCE(SUM(kg),0.0) AS kg, COALESCE(SUM(kg * COALESCE(price_per_kg,0.0)),0.0) AS revenue FROM harvests WHERE crop_id = ? AND harvest_date <= ?").bind(crop_id).bind(date).fetch_one(&st.db).await?;
    let harvested_count: f64 = harv.get::<i64, _>("c") as f64;
    let harvested_kg: f64 = harv.get("kg");
    let revenue: f64 = harv.get("revenue");
    let expenses: f64 = sqlx::query("SELECT COALESCE(SUM(amount),0.0) AS a FROM expenses WHERE crop_id = ? AND expense_date <= ?").bind(crop_id).bind(date).fetch_one(&st.db).await?.get("a");
    let stock = crate::api::stock_summary_json(st, crop["farm_id"].as_str().unwrap_or("")).await?;
    let feed_price = opts.feed_price.or(stock["avg_price_per_kg"].as_f64().filter(|p| *p > 0.0)).unwrap_or(0.0);
    let feed_cost = fed_total * feed_price;
    let cost_total = expenses + feed_cost;
    let alive = (stocked_count - dead - harvested_count).max(0.0);

    // ราคาขาย: ที่ส่งมา > ราคาล่าสุดในจังหวัด > ราคาล่าสุดทั้งระบบ
    let province = crop["province"].as_str().map(|s| s.to_string());
    let market_price: Option<f64> = match opts.sell_price {
        Some(p) => Some(p),
        None => {
            let r = match &province {
                Some(p) => sqlx::query("SELECT price_per_kg FROM market_prices WHERE species_code = ? AND (province = ? OR province IS NULL) ORDER BY (province = ?) DESC, price_date DESC LIMIT 1").bind(&species_code).bind(p).bind(p).fetch_optional(&st.db).await?,
                None => sqlx::query("SELECT price_per_kg FROM market_prices WHERE species_code = ? ORDER BY price_date DESC LIMIT 1").bind(&species_code).fetch_optional(&st.db).await?,
            };
            r.map(|r| r.get::<f64, _>("price_per_kg"))
        }
    };

    // อากาศวันนี้ (ถ้าฟาร์มมีพิกัด)
    let weather = match (crop["lat"].as_f64(), crop["lng"].as_f64()) {
        (Some(lat), Some(lng)) if opts.with_weather => fetch_daily(st, lat, lng, date).await.ok(),
        _ => None,
    };

    // น้ำล่าสุด (ภายใน 36 ชั่วโมงก่อนสิ้นวันที่ดู)
    let pond_id = crop["pond_id"].as_str().unwrap_or("").to_string();
    let water_row = sqlx::query("SELECT * FROM water_quality WHERE pond_id = ? AND measured_at <= ? || 'T23:59:59' ORDER BY measured_at DESC LIMIT 1").bind(&pond_id).bind(date).fetch_optional(&st.db).await?;
    let water_json = water_row.as_ref().map(row_to_json);
    let water_recent = water_json.as_ref().filter(|w| {
        w["measured_at"].as_str().map(|m| days_between(&m[..10.min(m.len())], date) <= 1).unwrap_or(false)
    });
    let water_sample = water_recent
        .map(|w| WaterSample {
            do_mg_l: w["do_mg_l"].as_f64(),
            ph: w["ph"].as_f64(),
            temp_c: w["temp_c"].as_f64(),
            nh3: w["nh3"].as_f64(),
            no2: w["no2"].as_f64(),
            secchi_cm: w["secchi_cm"].as_f64(),
        })
        .unwrap_or_default();

    // log ของวันนี้ (การกิน/ตาย) และของเมื่อวาน (สำหรับ stress ถ้าวันนี้ยังไม่บันทึก)
    let today_log = sqlx::query("SELECT * FROM daily_logs WHERE crop_id = ? AND log_date = ?").bind(crop_id).bind(date).fetch_optional(&st.db).await?.map(|r| row_to_json(&r));
    let last_response = match &today_log {
        Some(l) => l["feeding_response"].as_i64().unwrap_or(0),
        None => sqlx::query("SELECT feeding_response FROM daily_logs WHERE crop_id = ? AND log_date < ? ORDER BY log_date DESC LIMIT 1")
            .bind(crop_id)
            .bind(date)
            .fetch_optional(&st.db)
            .await?
            .map(|r| r.get::<i64, _>("feeding_response"))
            .unwrap_or(0),
    };
    let stress = opts.stress.map(stress_from).unwrap_or(stress_from(last_response));

    let env = EnvInput {
        tmax_c: opts.tmax.or(weather.as_ref().and_then(|w| w["tmax_c"].as_f64())),
        tmin_c: opts.tmin.or(weather.as_ref().and_then(|w| w["tmin_c"].as_f64())),
        rain_mm: opts.rain.or(weather.as_ref().and_then(|w| w["rain_mm"].as_f64())),
        cloud_pct: opts.cloud.or(weather.as_ref().and_then(|w| w["cloud_pct"].as_f64())),
        do_morning: water_sample.do_mg_l,
        nh3: water_sample.nh3,
        stress,
    };
    let has_env = env.tmax_c.is_some() || env.tmin_c.is_some() || env.do_morning.is_some() || env.nh3.is_some() || stress != StressLevel::Normal;

    let rec = recommend(&FeedInput {
        species: sp.clone(),
        avg_weight_g: est_w,
        count: alive,
        env: if has_env { Some(env.clone()) } else { None },
        rules: rules.clone(),
        meals_per_day: crop["meals_per_day"].as_i64().map(|m| m as u8),
        farm_factor: crop["farm_factor"].as_f64(),
    });

    let perf = performance(&PerfInput {
        stocked_count,
        stock_weight_g: stock_w,
        dead_count: dead,
        harvested_count,
        harvested_kg,
        avg_weight_g: est_w,
        feed_kg_total: fed_total,
        cost_total,
        feed_cost_total: feed_cost,
        revenue_total: revenue,
        day,
        price_per_kg: market_price,
    });

    let growth = compare_growth(&sp, stock_w, last_day, last_w, prev.as_ref().map(|(d, w)| (days_between(&stocked_at, d).max(0) as u32, *w)), Some(target_w));

    // ตาย 7 วัน
    let dead7: f64 = sqlx::query("SELECT COALESCE(SUM(mortality),0) AS d FROM daily_logs WHERE crop_id = ? AND log_date > date(?, '-7 days') AND log_date <= ?").bind(crop_id).bind(date).bind(date).fetch_one(&st.db).await?.get::<i64, _>("d") as f64;
    let mortality7_pct = if alive + dead7 > 0.0 { dead7 / (alive + dead7) * 100.0 } else { 0.0 };
    let prev_score: Option<f64> = sqlx::query("SELECT score FROM health_history WHERE crop_id = ? AND score_date < ? ORDER BY score_date DESC LIMIT 1").bind(crop_id).bind(date).fetch_optional(&st.db).await?.map(|r| r.get::<i64, _>("score") as f64);
    let days_since_log = last_log.as_ref().map(|d| days_between(d, date).max(0) as u32).unwrap_or(day);
    let health = health_score(
        &HealthInput {
            water: water_sample.clone(),
            mortality_7d_pct: Some(mortality7_pct),
            feeding_response: Some(last_response as u8),
            growth_status: if day > 7 { Some(growth.status.clone()) } else { None },
            days_since_last_log: Some(days_since_log),
            previous_score: prev_score,
        },
        &sp.water,
    );
    if date == today_bkk() {
        let _ = sqlx::query("INSERT INTO health_history (crop_id, score_date, score) VALUES (?, ?, ?) ON CONFLICT(crop_id, score_date) DO UPDATE SET score = excluded.score")
            .bind(crop_id)
            .bind(date)
            .bind(health.score as i64)
            .execute(&st.db)
            .await;
    }

    let projection = if opts.with_forecast {
        Some(project(&ProjectionInput {
            species: sp.clone(),
            day,
            avg_weight_g: est_w,
            alive_count: alive,
            daily_mortality_rate: perf.daily_mortality_rate,
            target_weight_g: opts.target_weight.or(Some(target_w)),
            target_days: opts.target_days,
            growth_scale,
            avg_feed_factor: opts.avg_feed_factor.unwrap_or(0.95),
            feed_price_per_kg: feed_price,
            other_cost_per_day: opts.other_cost_per_day.unwrap_or(0.0),
            cost_so_far: cost_total,
            feed_kg_so_far: fed_total,
            sell_price_per_kg: market_price.unwrap_or(0.0),
            bag_kg: crop["bag_kg"].as_f64(),
            max_days: None,
        }))
    } else {
        None
    };

    // ยาที่ยังอยู่ในระยะหยุดยา
    let withdrawal = sqlx::query("SELECT product, COALESCE(end_date, start_date) AS end_d, withdrawal_days FROM treatments WHERE crop_id = ? AND withdrawal_days > 0").bind(crop_id).fetch_all(&st.db).await?;
    let mut withdrawal_until: Option<String> = None;
    for r in withdrawal {
        let end: String = r.get("end_d");
        let wd: i64 = r.get("withdrawal_days");
        if let Ok(d) = chrono::NaiveDate::parse_from_str(&end, "%Y-%m-%d") {
            let until = (d + chrono::Duration::days(wd)).format("%Y-%m-%d").to_string();
            if until.as_str() > date && withdrawal_until.as_deref().map(|u| until.as_str() > u).unwrap_or(true) {
                withdrawal_until = Some(until);
            }
        }
    }

    let mut alerts: Vec<Value> = Vec::new();
    for a in &health.alerts_th {
        alerts.push(json!({ "level": "warn", "text": a }));
    }
    if stock["low"].as_bool().unwrap_or(false) {
        alerts.push(json!({ "level": "info", "text": format!("อาหารในสต๊อกเหลือประมาณ {} วัน", stock["days_left"].as_f64().unwrap_or(0.0)) }));
    }
    if days_since_weigh >= 14 && day >= 14 {
        alerts.push(json!({ "level": "info", "text": format!("ไม่ได้ชั่งน้ำหนักมา {} วัน ควรสุ่มชั่งเพื่อปรับอาหาร", days_since_weigh) }));
    }
    if let Some(u) = &withdrawal_until {
        alerts.push(json!({ "level": "warn", "text": format!("อยู่ในระยะหยุดยา ห้ามจับขายก่อนวันที่ {}", u) }));
    }

    Ok(json!({
        "date": date,
        "crop": crop,
        "species": { "code": sp.code, "name_th": sp.name_th, "market_weight_g": sp.market_weight_g, "approximate": sp.approximate },
        "day": day,
        "alive_count": alive,
        "avg_weight_g": (est_w * 10.0).round() / 10.0,
        "avg_weight_source": if days_since_weigh == 0 { "weighed" } else { "estimated" },
        "last_weighed": { "date": last_date, "avg_weight_g": last_w, "days_ago": days_since_weigh },
        "growth_scale": (growth_scale * 100.0).round() / 100.0,
        "weather": weather,
        "water": water_json,
        "env_used": env,
        "recommendation": rec,
        "performance": perf,
        "growth": growth,
        "health": health,
        "projection": projection,
        "stock": { "balance_kg": stock["balance_kg"], "balance_bags": stock["balance_bags"], "days_left": stock["days_left"], "low": stock["low"], "avg_price_per_kg": stock["avg_price_per_kg"] },
        "market_price_per_kg": market_price,
        "today_log": today_log,
        "withdrawal_until": withdrawal_until,
        "alerts": alerts,
        "totals": { "fed_kg": fed_total, "dead": dead, "expenses": expenses, "feed_cost": feed_cost, "cost_total": cost_total, "revenue": revenue, "harvested_kg": harvested_kg },
    }))
}

#[derive(Default, Clone, Copy)]
pub struct SnapshotOpts {
    pub with_weather: bool,
    pub with_forecast: bool,
    pub sell_price: Option<f64>,
    pub feed_price: Option<f64>,
    pub target_weight: Option<f64>,
    pub target_days: Option<u32>,
    pub other_cost_per_day: Option<f64>,
    pub avg_feed_factor: Option<f64>,
    pub tmax: Option<f64>,
    pub tmin: Option<f64>,
    pub rain: Option<f64>,
    pub cloud: Option<f64>,
    pub stress: Option<i64>,
}

fn qf(q: &Value, k: &str) -> Option<f64> {
    q.get(k).and_then(|v| v.as_str()).and_then(|s| s.parse().ok())
}

pub async fn crop_today(State(st): State<AppState>, user: AuthUser, Path(crop_id): Path<String>, Query(q): Query<Value>) -> ApiResult<Json<Value>> {
    let farm_id = farm_of_crop(&st, &crop_id).await?;
    assert_farm_access(&st, &user, &farm_id).await?;
    let date = q.get("date").and_then(|v| v.as_str()).map(|s| s.to_string()).unwrap_or_else(today_bkk);
    let opts = SnapshotOpts {
        with_weather: q.get("weather").and_then(|v| v.as_str()) != Some("0"),
        with_forecast: q.get("forecast").and_then(|v| v.as_str()) != Some("0"),
        sell_price: qf(&q, "sell_price"),
        feed_price: qf(&q, "feed_price"),
        target_weight: qf(&q, "target_weight"),
        target_days: qf(&q, "target_days").map(|d| d as u32),
        other_cost_per_day: qf(&q, "other_cost_per_day"),
        avg_feed_factor: qf(&q, "avg_feed_factor"),
        tmax: qf(&q, "tmax"),
        tmin: qf(&q, "tmin"),
        rain: qf(&q, "rain"),
        cloud: qf(&q, "cloud"),
        stress: qf(&q, "stress").map(|s| s as i64),
    };
    Ok(Json(crop_snapshot(&st, &crop_id, &date, &user.org_id, &opts).await?))
}

/// หน้าแรกของฟาร์ม: ทุกบ่อที่เลี้ยงอยู่ + อาหารแนะนำ + คะแนนสุขภาพ + เตือน
pub async fn farm_today(State(st): State<AppState>, user: AuthUser, Path(farm_id): Path<String>) -> ApiResult<Json<Value>> {
    assert_farm_access(&st, &user, &farm_id).await?;
    let date = today_bkk();
    let farm = sqlx::query("SELECT * FROM farms WHERE id = ?").bind(&farm_id).fetch_one(&st.db).await?;
    let farm = row_to_json(&farm);
    let crops = sqlx::query("SELECT c.id FROM crops c JOIN ponds p ON p.id = c.pond_id WHERE c.farm_id = ? AND c.status = 'active' ORDER BY p.sort_order, p.name").bind(&farm_id).fetch_all(&st.db).await?;
    let opts = SnapshotOpts { with_weather: true, with_forecast: false, ..Default::default() };
    let mut items = Vec::new();
    let mut total_feed = 0.0;
    let mut total_value = 0.0;
    for r in crops {
        let id: String = r.get("id");
        let snap = crop_snapshot(&st, &id, &date, &user.org_id, &opts).await?;
        total_feed += snap["recommendation"]["final_kg"].as_f64().unwrap_or(0.0);
        total_value += snap["performance"]["stock_value"].as_f64().unwrap_or(0.0);
        items.push(json!({
            "crop_id": id,
            "pond_id": snap["crop"]["pond_id"],
            "pond_name": snap["crop"]["pond_name"],
            "species": snap["species"],
            "day": snap["day"],
            "alive_count": snap["alive_count"],
            "avg_weight_g": snap["avg_weight_g"],
            "avg_weight_source": snap["avg_weight_source"],
            "recommendation": snap["recommendation"],
            "health": { "score": snap["health"]["score"], "grade": snap["health"]["grade"], "grade_th": snap["health"]["grade_th"], "trend": snap["health"]["trend"] },
            "growth": { "status": snap["growth"]["status"], "status_th": snap["growth"]["status_th"], "deviation_pct": snap["growth"]["deviation_pct"] },
            "performance": { "fcr": snap["performance"]["fcr"], "survival_pct": snap["performance"]["survival_pct"], "stock_value": snap["performance"]["stock_value"], "biomass_kg": snap["performance"]["biomass_kg"] },
            "today_log": snap["today_log"],
            "alerts": snap["alerts"],
            "weather": snap["weather"],
        }));
    }
    let empty_ponds = sqlx::query("SELECT p.id, p.name FROM ponds p WHERE p.farm_id = ? AND p.active = 1 AND NOT EXISTS (SELECT 1 FROM crops c WHERE c.pond_id = p.id AND c.status = 'active') ORDER BY p.sort_order, p.name").bind(&farm_id).fetch_all(&st.db).await?;
    let stock = crate::api::stock_summary_json(&st, &farm_id).await?;
    let weather = match (farm["lat"].as_f64(), farm["lng"].as_f64()) {
        (Some(lat), Some(lng)) => fetch_daily(&st, lat, lng, &date).await.ok(),
        _ => None,
    };
    // streak: จำนวนวันติดต่อกันที่ฟาร์มมีบันทึกอย่างน้อย 1 รายการ
    let dates = sqlx::query("SELECT DISTINCT l.log_date FROM daily_logs l JOIN crops c ON c.id = l.crop_id WHERE c.farm_id = ? ORDER BY l.log_date DESC LIMIT 400").bind(&farm_id).fetch_all(&st.db).await?;
    let mut streak = 0i64;
    let mut cursor = date.clone();
    for r in dates {
        let d: String = r.get("log_date");
        if d == cursor {
            streak += 1;
            cursor = chrono::NaiveDate::parse_from_str(&cursor, "%Y-%m-%d").map(|x| (x - chrono::Duration::days(1)).format("%Y-%m-%d").to_string()).unwrap_or_default();
        } else if streak == 0 && days_between(&d, &date) == 1 {
            // วันนี้ยังไม่บันทึกแต่เมื่อวานบันทึก นับต่อจากเมื่อวาน
            streak = 1;
            cursor = chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").map(|x| (x - chrono::Duration::days(1)).format("%Y-%m-%d").to_string()).unwrap_or_default();
        } else {
            break;
        }
    }
    let announcements = sqlx::query("SELECT id, title, body, created_at FROM announcements WHERE org_id = ? ORDER BY created_at DESC LIMIT 3").bind(&user.org_id).fetch_all(&st.db).await?;
    Ok(Json(json!({
        "date": date,
        "farm": farm,
        "weather": weather,
        "ponds": items,
        "empty_ponds": rows_to_json(&empty_ponds),
        "stock": { "balance_kg": stock["balance_kg"], "balance_bags": stock["balance_bags"], "days_left": stock["days_left"], "low": stock["low"] },
        "totals": { "feed_today_kg": (total_feed * 100.0).round() / 100.0, "stock_value": total_value.round() },
        "streak_days": streak,
        "announcements": rows_to_json(&announcements),
    })))
}

/// สรุปข้อความเช้าสำหรับ LINE/แจ้งเตือน
pub async fn morning_summary_text(st: &AppState, farm_id: &str, org_id: &str) -> ApiResult<String> {
    let date = today_bkk();
    let farm = sqlx::query("SELECT name FROM farms WHERE id = ?").bind(farm_id).fetch_one(&st.db).await?;
    let name: String = farm.get("name");
    let crops = sqlx::query("SELECT c.id FROM crops c JOIN ponds p ON p.id = c.pond_id WHERE c.farm_id = ? AND c.status = 'active' ORDER BY p.sort_order, p.name").bind(farm_id).fetch_all(&st.db).await?;
    let opts = SnapshotOpts { with_weather: true, with_forecast: false, ..Default::default() };
    let mut lines = vec![format!("ทีเด็ดปลาน้ำจืด - {} - สรุปเช้านี้", name)];
    let mut total = 0.0;
    for r in crops {
        let id: String = r.get("id");
        let s = crop_snapshot(st, &id, &date, org_id, &opts).await?;
        let kg = s["recommendation"]["final_kg"].as_f64().unwrap_or(0.0);
        total += kg;
        let pond = s["crop"]["pond_name"].as_str().unwrap_or("-");
        let per_meal = s["recommendation"]["per_meal_kg"].as_f64().unwrap_or(0.0);
        let band = s["recommendation"]["band"].as_str().unwrap_or("normal");
        let note = match band { "cut" => " (ลดมาก)", "down" => " (ลด)", _ => "" };
        lines.push(format!("{}: {:.1} กก. มื้อละ {:.1}{}", pond, kg, per_meal, note));
        if let Some(a) = s["alerts"].as_array().and_then(|a| a.first()) {
            lines.push(format!("  - {}", a["text"].as_str().unwrap_or("")));
        }
    }
    lines.push(format!("รวมวันนี้ {:.1} กก.", total));
    Ok(lines.join("\n"))
}
