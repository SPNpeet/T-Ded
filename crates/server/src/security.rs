//! มาตรการความปลอดภัยเชิงเทคนิค อ้างอิง ISO/IEC 27001:2022 Annex A
//! A.5.15 ควบคุมการเข้าถึง, A.5.17 ข้อมูลยืนยันตัวตน, A.8.5 การยืนยันตัวตนที่ปลอดภัย,
//! A.8.15 การบันทึกล็อก, A.8.16 การเฝ้าระวัง, A.8.23 การกรองเว็บ, A.8.24 การเข้ารหัส

use std::{collections::HashMap, net::IpAddr, sync::Mutex, time::{Duration, Instant}};

use axum::{extract::Request, http::{header, HeaderValue}, middleware::Next, response::Response};

/// จำนวนครั้งที่ยอมให้ล็อกอินผิดก่อนล็อก และระยะเวลาล็อก
const MAX_ATTEMPTS: u32 = 5;
const LOCK_DURATION: Duration = Duration::from_secs(15 * 60);
const ATTEMPT_WINDOW: Duration = Duration::from_secs(15 * 60);
/// อายุ session ถ้าไม่มีการใช้งาน (A.8.5) และอายุสูงสุด
pub const SESSION_IDLE_DAYS: i64 = 30;
pub const SESSION_MAX_DAYS: i64 = 90;

#[derive(Default)]
pub struct LoginGuard {
    attempts: Mutex<HashMap<String, (u32, Instant)>>,
}

impl LoginGuard {
    pub fn new() -> Self {
        Self::default()
    }

    /// คืน Some(วินาทีที่ต้องรอ) ถ้าถูกล็อกอยู่
    pub fn locked_for(&self, key: &str) -> Option<u64> {
        let mut map = self.attempts.lock().ok()?;
        let (count, at) = *map.get(key)?;
        if count >= MAX_ATTEMPTS {
            let elapsed = at.elapsed();
            if elapsed < LOCK_DURATION {
                return Some((LOCK_DURATION - elapsed).as_secs());
            }
            map.remove(key);
        } else if at.elapsed() > ATTEMPT_WINDOW {
            map.remove(key);
        }
        None
    }

    pub fn record_failure(&self, key: &str) {
        if let Ok(mut map) = self.attempts.lock() {
            let e = map.entry(key.to_string()).or_insert((0, Instant::now()));
            if e.1.elapsed() > ATTEMPT_WINDOW {
                *e = (0, Instant::now());
            }
            e.0 += 1;
            e.1 = Instant::now();
        }
    }

    pub fn record_success(&self, key: &str) {
        if let Ok(mut map) = self.attempts.lock() {
            map.remove(key);
        }
    }

    /// ล้างรายการเก่าเพื่อไม่ให้หน่วยความจำโต
    pub fn sweep(&self) {
        if let Ok(mut map) = self.attempts.lock() {
            map.retain(|_, (count, at)| {
                if *count >= MAX_ATTEMPTS {
                    at.elapsed() < LOCK_DURATION
                } else {
                    at.elapsed() < ATTEMPT_WINDOW
                }
            });
        }
    }
}

/// PIN ที่เดาง่าย ห้ามใช้ (A.5.17)
pub fn is_weak_pin(pin: &str) -> bool {
    const COMMON: [&str; 14] = ["0000", "1111", "1234", "4321", "1212", "2222", "3333", "4444", "5555", "6666", "7777", "8888", "9999", "123456"];
    if COMMON.contains(&pin) {
        return true;
    }
    let bytes = pin.as_bytes();
    // เลขซ้ำทั้งหมด
    if bytes.windows(2).all(|w| w[0] == w[1]) {
        return true;
    }
    // เรียงขึ้นหรือลงทั้งหมด
    let asc = bytes.windows(2).all(|w| w[1] == w[0] + 1);
    let desc = bytes.windows(2).all(|w| w[0] == w[1] + 1);
    asc || desc
}

/// ดึง IP ผู้เรียกจาก header ของ proxy (Cloudflare/nginx) หรือ socket
pub fn client_ip(req: &Request) -> String {
    let h = req.headers();
    for name in ["cf-connecting-ip", "x-real-ip"] {
        if let Some(v) = h.get(name).and_then(|v| v.to_str().ok()) {
            if v.parse::<IpAddr>().is_ok() {
                return v.to_string();
            }
        }
    }
    if let Some(v) = h.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        if let Some(first) = v.split(',').next() {
            let first = first.trim();
            if first.parse::<IpAddr>().is_ok() {
                return first.to_string();
            }
        }
    }
    "unknown".into()
}

/// ส่วนหัวความปลอดภัยของเว็บ (A.8.23 / OWASP)
pub async fn security_headers(req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;
    let h = res.headers_mut();
    let set = |h: &mut axum::http::HeaderMap, k: header::HeaderName, v: &'static str| {
        h.insert(k, HeaderValue::from_static(v));
    };
    set(h, header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    set(h, header::X_FRAME_OPTIONS, "DENY");
    set(h, header::REFERRER_POLICY, "strict-origin-when-cross-origin");
    set(h, header::STRICT_TRANSPORT_SECURITY, "max-age=31536000; includeSubDomains");
    h.insert(header::HeaderName::from_static("permissions-policy"), HeaderValue::from_static("geolocation=(self), camera=(), microphone=(), payment=()"));
    h.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data: blob:; connect-src 'self' https://api.open-meteo.com https://archive-api.open-meteo.com; frame-ancestors 'none'; base-uri 'self'; form-action 'self'",
        ),
    );
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weak_pins_rejected() {
        assert!(is_weak_pin("1234"));
        assert!(is_weak_pin("0000"));
        assert!(is_weak_pin("4321"));
        assert!(is_weak_pin("111111"));
        assert!(!is_weak_pin("8317"));
        assert!(!is_weak_pin("290471"));
    }

    #[test]
    fn lockout_after_five_failures() {
        let g = LoginGuard::new();
        assert!(g.locked_for("k").is_none());
        for _ in 0..5 {
            g.record_failure("k");
        }
        assert!(g.locked_for("k").is_some());
        g.record_success("k");
        assert!(g.locked_for("k").is_none());
    }
}
