//! LINE Messaging API: ผูกบัญชีด้วยรหัส, บันทึกผ่านข้อความ ("บ่อ1 ให้แล้ว 12"), สรุปเช้าอัตโนมัติ
//! ต้องตั้ง LINE_CHANNEL_SECRET และ LINE_CHANNEL_ACCESS_TOKEN ใน env ถึงจะทำงาน

use axum::{body::Bytes, extract::State, http::HeaderMap, Json};
use base64::Engine;
use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::Sha256;
use sqlx::Row;

use crate::{
    auth::AuthUser,
    db::{new_id, now_iso, today_bkk},
    error::{ApiResult, AppError},
    AppState,
};

fn verify_signature(secret: &str, body: &[u8], signature: &str) -> bool {
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else { return false };
    mac.update(body);
    let expected = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    expected == signature
}

pub async fn push_text(st: &AppState, line_user_id: &str, text: &str) -> ApiResult<()> {
    let Some(token) = st.cfg.line_token.as_ref() else { return Ok(()) };
    st.http
        .post("https://api.line.me/v2/bot/message/push")
        .bearer_auth(token)
        .json(&json!({ "to": line_user_id, "messages": [{ "type": "text", "text": text }] }))
        .send()
        .await?;
    Ok(())
}

async fn reply_text(st: &AppState, reply_token: &str, text: &str) -> ApiResult<()> {
    let Some(token) = st.cfg.line_token.as_ref() else { return Ok(()) };
    st.http
        .post("https://api.line.me/v2/bot/message/reply")
        .bearer_auth(token)
        .json(&json!({ "replyToken": reply_token, "messages": [{ "type": "text", "text": text }] }))
        .send()
        .await?;
    Ok(())
}

/// ผู้ใช้ขอรหัสผูก LINE (6 หลัก) แล้วพิมพ์ "ผูก 123456" ในแชท
pub async fn link_code(State(st): State<AppState>, user: AuthUser) -> ApiResult<Json<Value>> {
    let code = format!("{:06}", rand::random::<u32>() % 1_000_000);
    sqlx::query("UPDATE users SET line_link_code = ? WHERE id = ?").bind(&code).bind(&user.id).execute(&st.db).await?;
    Ok(Json(json!({ "code": code, "bot_configured": st.cfg.line_token.is_some(), "add_friend_url": st.cfg.line_add_friend_url })))
}

pub async fn unlink(State(st): State<AppState>, user: AuthUser) -> ApiResult<Json<Value>> {
    sqlx::query("UPDATE users SET line_user_id = NULL WHERE id = ?").bind(&user.id).execute(&st.db).await?;
    Ok(Json(json!({ "ok": true })))
}

pub async fn webhook(State(st): State<AppState>, headers: HeaderMap, body: Bytes) -> ApiResult<Json<Value>> {
    let Some(secret) = st.cfg.line_secret.as_ref() else { return Err(AppError::BadRequest("LINE ยังไม่ได้ตั้งค่า".into())) };
    let sig = headers.get("x-line-signature").and_then(|v| v.to_str().ok()).unwrap_or("");
    if !verify_signature(secret, &body, sig) {
        return Err(AppError::Forbidden);
    }
    let payload: Value = serde_json::from_slice(&body)?;
    if let Some(events) = payload.get("events").and_then(|e| e.as_array()) {
        for ev in events {
            if ev["type"] != "message" || ev["message"]["type"] != "text" {
                continue;
            }
            let uid = ev["source"]["userId"].as_str().unwrap_or("").to_string();
            let reply_token = ev["replyToken"].as_str().unwrap_or("").to_string();
            let text = ev["message"]["text"].as_str().unwrap_or("").trim().to_string();
            let answer = handle_text(&st, &uid, &text).await.unwrap_or_else(|e| format!("ขออภัย ทำรายการไม่สำเร็จ: {e}"));
            let _ = reply_text(&st, &reply_token, &answer).await;
        }
    }
    Ok(Json(json!({ "ok": true })))
}

fn thai_digits_to_arabic(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '๐'..='๙' => char::from_u32('0' as u32 + (c as u32 - '๐' as u32)).unwrap_or(c),
            _ => c,
        })
        .collect()
}

fn numbers_in(s: &str) -> Vec<f64> {
    let s = thai_digits_to_arabic(s);
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in s.chars() {
        if c.is_ascii_digit() || (c == '.' && !cur.is_empty()) {
            cur.push(c);
        } else if !cur.is_empty() {
            if let Ok(v) = cur.parse() { out.push(v); }
            cur.clear();
        }
    }
    if let Ok(v) = cur.parse() { out.push(v); }
    out
}

async fn handle_text(st: &AppState, line_uid: &str, text: &str) -> ApiResult<String> {
    // ผูกบัญชี
    if text.starts_with("ผูก") || text.to_lowercase().starts_with("link") {
        let nums = numbers_in(text);
        let Some(code) = nums.first().map(|c| format!("{:06}", *c as u64)) else { return Ok("พิมพ์ ผูก ตามด้วยรหัส 6 หลักจากแอป เช่น ผูก 123456".into()) };
        let r = sqlx::query("SELECT id, name FROM users WHERE line_link_code = ?").bind(&code).fetch_optional(&st.db).await?;
        let Some(r) = r else { return Ok("รหัสไม่ถูกต้องหรือหมดอายุ ขอรหัสใหม่ในแอปที่เมนู ตั้งค่า > เชื่อม LINE".into()) };
        let id: String = r.get("id");
        let name: String = r.get("name");
        sqlx::query("UPDATE users SET line_user_id = ?, line_link_code = NULL WHERE id = ?").bind(line_uid).bind(&id).execute(&st.db).await?;
        return Ok(format!("เชื่อม LINE กับบัญชีของ {} เรียบร้อย ทุกเช้าจะส่งสรุปอาหารให้ และพิมพ์ \"บ่อ1 ให้แล้ว 12\" เพื่อบันทึกได้เลย", name));
    }

    let user = sqlx::query("SELECT id, org_id, role, name FROM users WHERE line_user_id = ?").bind(line_uid).fetch_optional(&st.db).await?;
    let Some(user) = user else { return Ok("ยังไม่ได้เชื่อมบัญชี เปิดแอปทีเด็ดปลาน้ำจืด > ตั้งค่า > เชื่อม LINE แล้วพิมพ์ ผูก ตามด้วยรหัส 6 หลัก".into()) };
    let auth = AuthUser { id: user.get("id"), org_id: user.get("org_id"), role: user.get("role"), name: user.get("name") };

    let farms = sqlx::query("SELECT f.id, f.name FROM farms f JOIN farm_members m ON m.farm_id = f.id WHERE m.user_id = ? ORDER BY f.created_at").bind(&auth.id).fetch_all(&st.db).await?;
    let Some(farm) = farms.first() else { return Ok("บัญชีนี้ยังไม่มีฟาร์ม สร้างฟาร์มในแอปก่อน".into()) };
    let farm_id: String = farm.get("id");

    let lower = text.to_lowercase();
    if lower.contains("สรุป") || lower.contains("วันนี้") || lower == "อาหาร" {
        return crate::snapshot::morning_summary_text(st, &farm_id, &auth.org_id).await;
    }

    // "บ่อ1 ให้แล้ว 12" / "บ่อ 2 ตาย 5" / "บ่อ1 ให้ 12.5 ตาย 3"
    if text.contains("บ่อ") {
        let nums = numbers_in(text);
        let Some(pond_no) = nums.first() else { return Ok("ระบุหมายเลขบ่อ เช่น บ่อ1 ให้แล้ว 12".into()) };
        let pond_name_candidates = [format!("บ่อ{}", *pond_no as i64), format!("บ่อ {}", *pond_no as i64), format!("{}", *pond_no as i64)];
        let ponds = sqlx::query("SELECT p.id, p.name, c.id AS crop_id FROM ponds p JOIN crops c ON c.pond_id = p.id AND c.status = 'active' WHERE p.farm_id = ? ORDER BY p.sort_order, p.name").bind(&farm_id).fetch_all(&st.db).await?;
        let mut target: Option<(String, String)> = None;
        for (idx, p) in ponds.iter().enumerate() {
            let name: String = p.get("name");
            if pond_name_candidates.iter().any(|c| name.replace(' ', "") == c.replace(' ', "")) || (idx + 1) as f64 == *pond_no {
                target = Some((p.get("crop_id"), name));
                break;
            }
        }
        let Some((crop_id, pond_name)) = target else { return Ok(format!("ไม่พบบ่อหมายเลข {} ที่กำลังเลี้ยงอยู่", *pond_no as i64)) };

        let mut body = json!({ "log_date": today_bkk(), "client_id": format!("line-{}", new_id()) });
        let mut parts = Vec::new();
        // หา "ให้" ตามด้วยตัวเลข และ "ตาย" ตามด้วยตัวเลข
        let arabic = thai_digits_to_arabic(text);
        if let Some(pos) = arabic.find("ให้") {
            let n = numbers_in(&arabic[pos..]);
            if let Some(kg) = n.first() {
                body["fed_kg"] = json!(kg);
                parts.push(format!("ให้อาหาร {} กก.", kg));
            }
        }
        if let Some(pos) = arabic.find("ตาย") {
            let n = numbers_in(&arabic[pos..]);
            if let Some(d) = n.first() {
                body["mortality"] = json!(*d as i64);
                parts.push(format!("ตาย {} ตัว", *d as i64));
            }
        }
        if arabic.contains("กินช้า") || arabic.contains("กินไม่หมด") {
            body["feeding_response"] = json!(1);
            parts.push("ปลากินช้า".into());
        }
        if arabic.contains("ลอยหัว") {
            body["feeding_response"] = json!(2);
            parts.push("ปลาลอยหัว".into());
        }
        if parts.is_empty() {
            return Ok("บอกได้ว่า ให้แล้วกี่กก. หรือ ตายกี่ตัว เช่น บ่อ1 ให้แล้ว 12 ตาย 3".into());
        }
        crate::api::insert_log(st, &auth, &crop_id, &farm_id, &body).await?;
        return Ok(format!("บันทึก {} วันนี้: {} เรียบร้อย", pond_name, parts.join(", ")));
    }

    Ok("พิมพ์ได้: \"สรุป\" ดูอาหารวันนี้ทุกบ่อ / \"บ่อ1 ให้แล้ว 12\" บันทึกอาหาร / \"บ่อ2 ตาย 5\" บันทึกปลาตาย".into())
}

/// ส่งสรุปเช้าให้ทุกคนที่ผูก LINE (เรียกจาก scheduler เวลา 06:00 น. หรือ endpoint แอดมิน)
pub async fn send_morning_summaries(st: &AppState) -> usize {
    let users = match sqlx::query("SELECT u.id, u.org_id, u.line_user_id FROM users u WHERE u.line_user_id IS NOT NULL").fetch_all(&st.db).await {
        Ok(u) => u,
        Err(_) => return 0,
    };
    let mut sent = 0;
    for u in users {
        let uid: String = u.get("id");
        let org: String = u.get("org_id");
        let lid: String = u.get("line_user_id");
        let farms = sqlx::query("SELECT f.id FROM farms f JOIN farm_members m ON m.farm_id = f.id WHERE m.user_id = ?").bind(&uid).fetch_all(&st.db).await.unwrap_or_default();
        for f in farms {
            let fid: String = f.get("id");
            if let Ok(text) = crate::snapshot::morning_summary_text(st, &fid, &org).await {
                if push_text(st, &lid, &text).await.is_ok() {
                    sent += 1;
                    let _ = sqlx::query("INSERT INTO notifications (id, user_id, channel, title, body, sent_at, created_at) VALUES (?, ?, 'line', 'สรุปเช้า', ?, ?, ?)")
                        .bind(new_id())
                        .bind(&uid)
                        .bind(&text)
                        .bind(now_iso())
                        .bind(now_iso())
                        .execute(&st.db)
                        .await;
                }
            }
        }
    }
    sent
}

pub async fn trigger_morning(State(st): State<AppState>, user: AuthUser) -> ApiResult<Json<Value>> {
    if !user.is_staff() {
        return Err(AppError::Forbidden);
    }
    let n = send_morning_summaries(&st).await;
    Ok(Json(json!({ "sent": n })))
}

/// ตัวจับเวลา: ทุก 06:00 น. เวลาไทย ส่งสรุปเช้า
pub fn spawn_scheduler(st: AppState) {
    tokio::spawn(async move {
        let tz = chrono::FixedOffset::east_opt(7 * 3600).unwrap();
        let mut last_sent_date = String::new();
        loop {
            let now = chrono::Utc::now().with_timezone(&tz);
            let date = now.format("%Y-%m-%d").to_string();
            let hour = now.format("%H").to_string();
            if hour == "06" && last_sent_date != date {
                last_sent_date = date.clone();
                let n = send_morning_summaries(&st).await;
                tracing::info!(sent = n, "morning summaries sent");
            }
            tokio::time::sleep(std::time::Duration::from_secs(300)).await;
        }
    });
}
