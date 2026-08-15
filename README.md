# ทีเด็ดปลาน้ำจืด (T-Ded)

ผู้ช่วยฟาร์มปลาน้ำจืดประจำวัน สำหรับเกษตรกรและหน่วยส่งเสริม: คำนวณอาหารรายวันตามน้ำหนัก/อากาศ/น้ำ, ติดตามการโตเทียบมาตรฐาน, FCR, ต้นทุน-กำไรคาดการณ์, คะแนนสุขภาพบ่อ, สต๊อกอาหาร, ราคาปลาในพื้นที่, โรคในพื้นที่, หลังบ้านเจ้าหน้าที่, แจ้งเตือน LINE

## โครงสร้าง

| ส่วน | เทคโนโลยี | ที่อยู่ |
|---|---|---|
| กฎการเลี้ยงทั้งหมด (engine) | Rust crate, คอมไพล์เป็น WASM ด้วย | `crates/aqua-engine` |
| API + ฐานข้อมูล + LINE + อากาศ | Rust (Axum, SQLx/SQLite) | `crates/server` |
| แอป (PWA ติดตั้งบนมือถือได้ ออฟไลน์ได้) | Svelte 5 + Vite + engine WASM | `web` |
| โลโก้/ไอคอน | สคริปต์สร้างจากอัตลักษณ์เดิม | `web/scripts/brand.mjs` -> `web/brand/` |

กฎทุกอย่าง (ตารางอัตราให้อาหาร, ตารางการโต, กติกาปรับตามอากาศ, FCR, พยากรณ์, คะแนนสุขภาพ) อยู่ที่ `aqua-engine` ที่เดียว ทั้ง server และแอปใช้ตัวเดียวกัน ผลลัพธ์ตรงกัน

## เริ่มใช้งาน (เครื่องพัฒนา)

ต้องมี Rust (stable), Node 20+, `wasm-pack`, target `wasm32-unknown-unknown`

```bash
cp .env.example .env            # แก้ ADMIN_PHONE / ADMIN_PIN
cargo test -p aqua-engine       # เทสต์กฎ
wasm-pack build crates/aqua-engine --target web --release --out-dir ../../web/src/engine-pkg --out-name aqua_engine -- --features wasm
cd web && npm install && node scripts/brand.mjs && npm run build && cd ..
cargo run --release -p teedet-server   # http://localhost:8787 (เสิร์ฟ web/dist ด้วย)
```

พัฒนาแอปแบบ hot reload: `cd web && npm run dev` (proxy /api ไป 8787)

## ตัวแปรสภาพแวดล้อม

ดู `.env.example` — `DATABASE_URL`, `PORT`, `WEB_DIR`, `ADMIN_PHONE`/`ADMIN_PIN` (สร้างแอดมินคนแรก), `LINE_CHANNEL_SECRET`/`LINE_CHANNEL_ACCESS_TOKEN` (เปิดสรุปเช้า/บันทึกผ่านแชท), `CORS_ORIGIN`

## API หลัก

- สาธารณะ: `POST /api/calc/recommend`, `POST /api/calc/simulate`, `GET /api/species`, `GET /api/weather?lat&lng&date`, `GET /api/prices`, `GET /api/disease-reports`
- ผู้ใช้: `/api/farms/{id}/today` (หน้าแรก), `/api/crops/{id}/today` (สแนปช็อตเต็ม: อาหาร/สุขภาพ/การโต/เงิน/พยากรณ์), `/api/crops/{id}/logs|weighings|expenses|harvests|treatments`, `/api/ponds/{id}/water`, `/api/farms/{id}/stock`, `POST /api/sync` (คิวออฟไลน์)
- หลังบ้าน: `/api/admin/farms`, `/api/admin/rules`, `/api/admin/species/{code}`, `/api/admin/users`, `/api/announcements`
- LINE: `POST /api/line/webhook` (พิมพ์ "สรุป", "บ่อ1 ให้แล้ว 12", "บ่อ2 ตาย 5", "ผูก 123456")

## มือถือ

- PWA: เปิดเว็บ (ต้องเป็น HTTPS) แล้ว "เพิ่มไปยังหน้าจอโฮม" ใช้งานออฟไลน์ได้ ข้อมูลที่บันทึกตอนไม่มีสัญญาณจะส่งให้เมื่อออนไลน์
- แอป store: โครงพร้อมห่อด้วย Tauri 2 (Android จากเครื่อง Windows ได้, iOS ต้อง macOS/Xcode)

## ใบอนุญาตฟอนต์

Prompt และ Sarabun ใช้ตาม SIL Open Font License (`web/public/fonts/OFL.txt`)
