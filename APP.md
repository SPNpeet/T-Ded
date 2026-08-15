# ปล่อยเป็นแอปติดตั้ง (iOS / Android / เว็บ)

โค้ดชุดเดียวออกได้ 3 แบบ ใช้ Capacitor ครอบเว็บแอปเดิม

| แบบ | ได้วันนี้ไหม | ค่าใช้จ่าย | ใครติดตั้งได้ |
|---|---|---|---|
| **PWA (เว็บติดตั้งได้)** | **ได้แล้ว** | 0 บาท | ทุกคน เปิด https://spnpeet.github.io/T-Ded/ แล้วกด "เพิ่มไปยังหน้าจอโฮม" |
| **Android APK** | **ได้วันนี้** | 0 บาท | ส่งไฟล์ .apk ให้ติดตั้งเองได้เลย ไม่ต้องผ่าน Play Store |
| **Android บน Play Store** | ไม่ทัน (รีวิว 1-7 วัน) | 25 USD ครั้งเดียว | ทุกคนโหลดจาก Play Store |
| **iOS ลงเครื่องตัวเอง** | **ได้วันนี้ (ต้องใช้ Mac)** | 0 บาท | เฉพาะเครื่องที่เสียบกับ Mac อายุ 7 วันต่อครั้ง |
| **iOS ผ่าน TestFlight / App Store** | ไม่ทันวันนี้ | 99 USD/ปี | ทุกคน (TestFlight รีวิว ~1 วัน, App Store 1-3 วัน) |

## 1. Android APK — พร้อมติดตั้งแล้ว

**ลิงก์ดาวน์โหลดตรง (เปิดบนมือถือ Android ได้เลย ไม่ต้องล็อกอิน GitHub):**

https://github.com/SPNpeet/T-Ded/releases/latest

ไฟล์ `teedet-pla.apk` ขนาด 6.5 MB


ไฟล์ APK ถูก build อัตโนมัติทุกครั้งที่ push ผ่าน `.github/workflows/android.yml`

**ดาวน์โหลด:** ไปที่แท็บ Actions ของ repo > เลือกงาน "Build Android APK" ล่าสุด > หัวข้อ Artifacts > `teedet-pla-apk`

**ถ้าอยากได้ลิงก์แจกตรง:** ไปแท็บ Actions > Build Android APK > Run workflow (กดเอง) ระบบจะสร้าง Release พร้อมไฟล์ `teedet-pla.apk` ให้แจกลิงก์ได้เลย

**วิธีติดตั้งบนมือถือ Android:** เปิดไฟล์ .apk > ถ้าขึ้นเตือน ให้กด "ตั้งค่า" แล้วอนุญาต "ติดตั้งแอปที่ไม่รู้จัก" สำหรับเบราว์เซอร์/ไฟล์แมเนเจอร์ที่ใช้ > ติดตั้ง

## 2. iOS บน Mac (ทำได้วันนี้)

บนเครื่อง Mac:

```bash
git clone https://github.com/SPNpeet/T-Ded.git
cd T-Ded
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
cargo install wasm-pack
wasm-pack build crates/aqua-engine --target web --release --out-dir ../../web/src/engine-pkg --out-name aqua_engine -- --features wasm
cd web
npm ci
node scripts/brand.mjs
npm run build
npx cap sync ios
npx cap open ios
```

จากนั้นใน Xcode:

1. เลือกโปรเจกต์ **App** ที่แถบซ้าย > แท็บ **Signing & Capabilities**
2. ติ๊ก **Automatically manage signing** แล้วเลือก Team เป็น Apple ID ของพี่ (ใช้ Apple ID ธรรมดาได้ ฟรี)
3. แก้ **Bundle Identifier** ให้ไม่ซ้ำใคร เช่น `com.teedetpla.app.peet`
4. เสียบ iPhone กับ Mac เลือกเครื่องที่แถบบน แล้วกด **Run** (ปุ่มสามเหลี่ยม)
5. ที่ iPhone: ตั้งค่า > ทั่วไป > VPN และการจัดการอุปกรณ์ > เชื่อถือ Apple ID ของพี่

**ข้อจำกัดของ Apple ID ฟรี:** แอปใช้ได้ 7 วันแล้วต้อง Run ใหม่ · ถ้าจะแจกให้คนอื่นต้องสมัคร Apple Developer Program 99 USD/ปี แล้วส่งขึ้น TestFlight

## 3. หลังติดตั้งแอปแล้ว ต้องทำอะไรต่อ

แอปมีหน้าจอครบในตัว แต่ **ระบบบันทึกข้อมูลต้องมีเซิร์ฟเวอร์** ครั้งแรกที่เปิดแอป:

1. กด "ตั้งที่อยู่เซิร์ฟเวอร์"
2. วางลิงก์เซิร์ฟเวอร์ของฟาร์ม (ได้จาก `start-mobile.cmd` หรือโฮสต์ถาวรตาม DEPLOY.md)
3. กดเชื่อมต่อ แล้วเข้าสู่ระบบได้เลย

ส่วนเครื่องคำนวณอาหาร จำลองรุ่น ตารางอาหาร และยี่ห้ออาหาร **ใช้ได้ทันทีโดยไม่ต้องมีเซิร์ฟเวอร์** เพราะคำนวณในเครื่องด้วย WASM

## 4. อัปเดตแอปในอนาคต

- **PWA:** push โค้ดแล้วผู้ใช้ได้ของใหม่อัตโนมัติรอบถัดไปที่เปิดแอป
- **Android:** push แล้วโหลด APK ใหม่จาก Actions
- **iOS:** บน Mac รัน `npm run build && npx cap sync ios` แล้ว Run ใหม่ใน Xcode
