//! แพ็กเกจการใช้งานและการเก็บค่าบริการ
//! ฟาร์มใหม่ได้ทดลองใช้ฟรี 60 วัน หมดแล้วยังดูข้อมูลเดิมได้ แต่เพิ่มบ่อ/ฟาร์มใหม่ไม่ได้จนกว่าจะต่ออายุ

use axum::{extract::{Path, State}, Json};
use serde_json::{json, Value};
use sqlx::Row;

use crate::{
    auth::AuthUser,
    db::{new_id, now_iso, row_to_json, rows_to_json, today_bkk},
    error::{ApiResult, AppError},
    AppState,
};

pub const TRIAL_DAYS: i64 = 60;

/// แพ็กเกจมาตรฐาน (แก้ราคาได้ภายหลังโดยไม่ต้องแก้โค้ดผู้ใช้)
pub fn plan_defaults(plan: &str) -> (i64, i64, i64, f64) {
    // (max_farms, max_ponds, max_members, price_per_month)
    match plan {
        "free" => (1, 2, 2, 0.0),
        "trial" => (1, 5, 3, 0.0),
        "basic" => (1, 10, 5, 199.0),
        "pro" => (5, 50, 20, 590.0),
        "unlimited" => (999, 9999, 999, 0.0),
        _ => (1, 5, 3, 0.0),
    }
}

pub fn plan_name_th(plan: &str) -> &'static str {
    match plan {
        "free" => "ฟรี",
        "trial" => "ทดลองใช้",
        "basic" => "มาตรฐาน",
        "pro" => "มืออาชีพ",
        "unlimited" => "ไม่จำกัด (หน่วยงาน)",
        _ => "ทดลองใช้",
    }
}

/// สร้างสิทธิ์ทดลองใช้ให้องค์กรใหม่
pub async fn start_trial(st: &AppState, org_id: &str) -> ApiResult<()> {
    let (f, p, m, price) = plan_defaults("trial");
    let expires = chrono::Utc::now() + chrono::Duration::days(TRIAL_DAYS);
    sqlx::query("INSERT OR IGNORE INTO subscriptions (org_id, plan, status, max_farms, max_ponds, max_members, price_per_month, started_at, expires_at, updated_at) VALUES (?, 'trial', 'active', ?, ?, ?, ?, ?, ?, ?)")
        .bind(org_id)
        .bind(f)
        .bind(p)
        .bind(m)
        .bind(price)
        .bind(now_iso())
        .bind(expires.format("%Y-%m-%d").to_string())
        .bind(now_iso())
        .execute(&st.db)
        .await?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Limits {
    pub plan: String,
    pub active: bool,
    pub max_farms: i64,
    pub max_ponds: i64,
    pub max_members: i64,
    pub expires_at: Option<String>,
    pub days_left: Option<i64>,
}

pub async fn limits_for(st: &AppState, org_id: &str) -> ApiResult<Limits> {
    let row = sqlx::query("SELECT * FROM subscriptions WHERE org_id = ?").bind(org_id).fetch_optional(&st.db).await?;
    let Some(row) = row else {
        // องค์กรเก่าที่ยังไม่มีระเบียน ให้เริ่มทดลองใช้แล้วอ่านใหม่
        start_trial(st, org_id).await?;
        return Box::pin(limits_for(st, org_id)).await;
    };
    let plan: String = row.get("plan");
    let status: String = row.get("status");
    let expires: Option<String> = row.get("expires_at");
    let today = today_bkk();
    let days_left = expires.as_ref().and_then(|e| {
        let a = chrono::NaiveDate::parse_from_str(&today, "%Y-%m-%d").ok()?;
        let b = chrono::NaiveDate::parse_from_str(e, "%Y-%m-%d").ok()?;
        Some((b - a).num_days())
    });
    let not_expired = days_left.map(|d| d >= 0).unwrap_or(true);
    Ok(Limits {
        plan,
        active: status == "active" && not_expired,
        max_farms: row.get("max_farms"),
        max_ponds: row.get("max_ponds"),
        max_members: row.get("max_members"),
        expires_at: expires,
        days_left,
    })
}

/// ตรวจก่อนสร้างของใหม่ ถ้าเกินสิทธิ์จะบอกชัดว่าต้องทำอะไร
pub async fn check_can_add(st: &AppState, org_id: &str, what: &str) -> ApiResult<()> {
    let l = limits_for(st, org_id).await?;
    if !l.active {
        return Err(AppError::BadRequest(format!(
            "แพ็กเกจ{}หมดอายุแล้ว ข้อมูลเดิมยังดูได้ทั้งหมด แต่ต้องต่ออายุก่อนจึงจะเพิ่ม{}ใหม่ได้",
            plan_name_th(&l.plan),
            what
        )));
    }
    let (used, max, label) = match what {
        "ฟาร์ม" => {
            let n: i64 = sqlx::query("SELECT COUNT(*) AS n FROM farms WHERE org_id = ?").bind(org_id).fetch_one(&st.db).await?.get("n");
            (n, l.max_farms, "ฟาร์ม")
        }
        "สมาชิก" => {
            let n: i64 = sqlx::query("SELECT COUNT(*) AS n FROM users WHERE org_id = ?").bind(org_id).fetch_one(&st.db).await?.get("n");
            (n, l.max_members, "ผู้ใช้")
        }
        _ => {
            let n: i64 = sqlx::query("SELECT COUNT(*) AS n FROM ponds p JOIN farms f ON f.id = p.farm_id WHERE f.org_id = ? AND p.active = 1")
                .bind(org_id)
                .fetch_one(&st.db)
                .await?
                .get("n");
            (n, l.max_ponds, "บ่อ")
        }
    };
    if used >= max {
        return Err(AppError::BadRequest(format!(
            "แพ็กเกจ{}ใช้ได้ {} {} ตอนนี้ใช้ครบแล้ว หากต้องการเพิ่มกรุณาอัปเกรดแพ็กเกจ",
            plan_name_th(&l.plan),
            max,
            label
        )));
    }
    Ok(())
}

/// ผู้ใช้ดูแพ็กเกจของตัวเอง
pub async fn my_subscription(State(st): State<AppState>, user: AuthUser) -> ApiResult<Json<Value>> {
    let l = limits_for(&st, &user.org_id).await?;
    let farms: i64 = sqlx::query("SELECT COUNT(*) AS n FROM farms WHERE org_id = ?").bind(&user.org_id).fetch_one(&st.db).await?.get("n");
    let ponds: i64 = sqlx::query("SELECT COUNT(*) AS n FROM ponds p JOIN farms f ON f.id = p.farm_id WHERE f.org_id = ? AND p.active = 1")
        .bind(&user.org_id)
        .fetch_one(&st.db)
        .await?
        .get("n");
    let members: i64 = sqlx::query("SELECT COUNT(*) AS n FROM users WHERE org_id = ?").bind(&user.org_id).fetch_one(&st.db).await?.get("n");
    let payments = sqlx::query("SELECT amount, paid_at, period_from, period_to, method FROM payments WHERE org_id = ? ORDER BY paid_at DESC LIMIT 12")
        .bind(&user.org_id)
        .fetch_all(&st.db)
        .await?;
    Ok(Json(json!({
        "plan": l.plan,
        "plan_name_th": plan_name_th(&l.plan),
        "active": l.active,
        "expires_at": l.expires_at,
        "days_left": l.days_left,
        "usage": { "farms": farms, "ponds": ponds, "members": members },
        "limits": { "farms": l.max_farms, "ponds": l.max_ponds, "members": l.max_members },
        "payments": rows_to_json(&payments),
        "plans": [
            { "code": "trial", "name_th": "ทดลองใช้", "price": 0, "days": TRIAL_DAYS, "detail_th": "ใช้ฟรี 60 วัน 1 ฟาร์ม 5 บ่อ" },
            { "code": "basic", "name_th": "มาตรฐาน", "price": 199, "detail_th": "1 ฟาร์ม 10 บ่อ ผู้ใช้ 5 คน" },
            { "code": "pro", "name_th": "มืออาชีพ", "price": 590, "detail_th": "5 ฟาร์ม 50 บ่อ ผู้ใช้ 20 คน เหมาะกับหน่วยส่งเสริม" },
        ],
    })))
}

/// แอดมินดูรายชื่อองค์กรทั้งหมดพร้อมสถานะแพ็กเกจ (ใช้ติดตามการเก็บเงิน)
pub async fn list_subscriptions(State(st): State<AppState>, user: AuthUser) -> ApiResult<Json<Value>> {
    if user.role != "admin" {
        return Err(AppError::Forbidden);
    }
    let rows = sqlx::query(
        "SELECT o.id AS org_id, o.name AS org_name, s.plan, s.status, s.expires_at, s.price_per_month,
                (SELECT COUNT(*) FROM farms f WHERE f.org_id = o.id) AS farms,
                (SELECT COUNT(*) FROM users u WHERE u.org_id = o.id) AS members,
                (SELECT u.phone FROM users u WHERE u.org_id = o.id ORDER BY u.created_at LIMIT 1) AS contact_phone,
                (SELECT COALESCE(SUM(p.amount),0) FROM payments p WHERE p.org_id = o.id) AS paid_total
         FROM orgs o LEFT JOIN subscriptions s ON s.org_id = o.id
         ORDER BY o.created_at DESC",
    )
    .fetch_all(&st.db)
    .await?;
    Ok(Json(json!(rows_to_json(&rows))))
}

/// แอดมินเปลี่ยนแพ็กเกจ/ต่ออายุให้ลูกค้า
pub async fn set_subscription(State(st): State<AppState>, user: AuthUser, Path(org_id): Path<String>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    if user.role != "admin" {
        return Err(AppError::Forbidden);
    }
    let plan = b.get("plan").and_then(|v| v.as_str()).unwrap_or("basic").to_string();
    let (df, dp, dm, dprice) = plan_defaults(&plan);
    let months = b.get("months").and_then(|v| v.as_i64()).unwrap_or(1).clamp(0, 60);
    let expires = match b.get("expires_at").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => {
            let base = chrono::NaiveDate::parse_from_str(&today_bkk(), "%Y-%m-%d").unwrap_or_else(|_| chrono::Utc::now().date_naive());
            (base + chrono::Duration::days(months * 30)).format("%Y-%m-%d").to_string()
        }
    };
    let g = |k: &str, d: i64| b.get(k).and_then(|v| v.as_i64()).unwrap_or(d);
    sqlx::query("INSERT INTO subscriptions (org_id, plan, status, max_farms, max_ponds, max_members, price_per_month, started_at, expires_at, note, updated_by, updated_at) VALUES (?, ?, 'active', ?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(org_id) DO UPDATE SET plan = excluded.plan, status = 'active', max_farms = excluded.max_farms, max_ponds = excluded.max_ponds, max_members = excluded.max_members, price_per_month = excluded.price_per_month, expires_at = excluded.expires_at, note = excluded.note, updated_by = excluded.updated_by, updated_at = excluded.updated_at")
        .bind(&org_id)
        .bind(&plan)
        .bind(g("max_farms", df))
        .bind(g("max_ponds", dp))
        .bind(g("max_members", dm))
        .bind(b.get("price_per_month").and_then(|v| v.as_f64()).unwrap_or(dprice))
        .bind(now_iso())
        .bind(&expires)
        .bind(b.get("note").and_then(|v| v.as_str()))
        .bind(&user.id)
        .bind(now_iso())
        .execute(&st.db)
        .await?;
    let _ = sqlx::query("INSERT INTO audit_log (user_id, action, entity, entity_id, detail_json, at) VALUES (?, 'set_plan', 'subscription', ?, ?, ?)")
        .bind(&user.id)
        .bind(&org_id)
        .bind(b.to_string())
        .bind(now_iso())
        .execute(&st.db)
        .await;
    Ok(Json(json!({ "ok": true, "plan": plan, "expires_at": expires })))
}

/// แอดมินบันทึกการชำระเงิน (โอน/พร้อมเพย์/เงินสด) และต่ออายุให้อัตโนมัติ
pub async fn record_payment(State(st): State<AppState>, user: AuthUser, Path(org_id): Path<String>, Json(b): Json<Value>) -> ApiResult<Json<Value>> {
    if user.role != "admin" {
        return Err(AppError::Forbidden);
    }
    let amount = b.get("amount").and_then(|v| v.as_f64()).ok_or_else(|| AppError::BadRequest("กรอกจำนวนเงิน".into()))?;
    let months = b.get("months").and_then(|v| v.as_i64()).unwrap_or(1).clamp(1, 60);
    let cur = limits_for(&st, &org_id).await?;
    let base = cur
        .expires_at
        .as_deref()
        .and_then(|e| chrono::NaiveDate::parse_from_str(e, "%Y-%m-%d").ok())
        .filter(|d| *d >= chrono::NaiveDate::parse_from_str(&today_bkk(), "%Y-%m-%d").unwrap_or_else(|_| chrono::Utc::now().date_naive()))
        .unwrap_or_else(|| chrono::NaiveDate::parse_from_str(&today_bkk(), "%Y-%m-%d").unwrap_or_else(|_| chrono::Utc::now().date_naive()));
    let new_expiry = (base + chrono::Duration::days(months * 30)).format("%Y-%m-%d").to_string();
    sqlx::query("INSERT INTO payments (id, org_id, amount, method, reference, period_from, period_to, paid_at, recorded_by, note, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(new_id())
        .bind(&org_id)
        .bind(amount)
        .bind(b.get("method").and_then(|v| v.as_str()).unwrap_or("transfer"))
        .bind(b.get("reference").and_then(|v| v.as_str()))
        .bind(base.format("%Y-%m-%d").to_string())
        .bind(&new_expiry)
        .bind(b.get("paid_at").and_then(|v| v.as_str()).map(String::from).unwrap_or_else(today_bkk))
        .bind(&user.id)
        .bind(b.get("note").and_then(|v| v.as_str()))
        .bind(now_iso())
        .execute(&st.db)
        .await?;
    sqlx::query("UPDATE subscriptions SET expires_at = ?, status = 'active', updated_by = ?, updated_at = ? WHERE org_id = ?")
        .bind(&new_expiry)
        .bind(&user.id)
        .bind(now_iso())
        .bind(&org_id)
        .execute(&st.db)
        .await?;
    Ok(Json(json!({ "ok": true, "expires_at": new_expiry })))
}

/// สรุปรายได้สำหรับเจ้าของระบบ
pub async fn revenue_summary(State(st): State<AppState>, user: AuthUser) -> ApiResult<Json<Value>> {
    if user.role != "admin" {
        return Err(AppError::Forbidden);
    }
    let row = sqlx::query(
        "SELECT COALESCE(SUM(amount),0) AS total,
                COALESCE(SUM(CASE WHEN paid_at >= date('now','start of month') THEN amount END),0) AS this_month,
                COUNT(*) AS payments
         FROM payments",
    )
    .fetch_one(&st.db)
    .await?;
    let active: i64 = sqlx::query("SELECT COUNT(*) AS n FROM subscriptions WHERE status = 'active' AND (expires_at IS NULL OR expires_at >= date('now'))").fetch_one(&st.db).await?.get("n");
    let paying: i64 = sqlx::query("SELECT COUNT(*) AS n FROM subscriptions WHERE price_per_month > 0 AND status = 'active' AND (expires_at IS NULL OR expires_at >= date('now'))").fetch_one(&st.db).await?.get("n");
    let mrr: f64 = sqlx::query("SELECT COALESCE(SUM(price_per_month),0) AS s FROM subscriptions WHERE status = 'active' AND (expires_at IS NULL OR expires_at >= date('now'))").fetch_one(&st.db).await?.get("s");
    let mut out = row_to_json(&row);
    out["active_orgs"] = json!(active);
    out["paying_orgs"] = json!(paying);
    out["mrr"] = json!(mrr);
    Ok(Json(out))
}
