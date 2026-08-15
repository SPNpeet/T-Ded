# การขึ้นใช้งานจริง (โฮสต์) และการตั้งค่า LINE

## 1. หน้าเครื่องคำนวณสาธารณะ — ฟรีถาวร ใช้ได้แล้ว

**https://spnpeet.github.io/T-Ded/**

หน้านี้ทำงานด้วย engine ที่คอมไพล์เป็น WASM ในเครื่องผู้ใช้ จึงใช้ได้โดยไม่ต้องมีเซิร์ฟเวอร์:

- คำนวณอาหารรายวัน (ปรับตามอากาศได้เมื่อมีเน็ต)
- จำลองรุ่นเลี้ยงก่อนลงทุน
- ตารางอาหารตามช่วง / ยี่ห้อในไทย / ผสมอาหารเอง / Pearson square
- ติดตั้งเป็นแอปบนมือถือได้ (Add to Home Screen) และเปิดใช้ออฟไลน์ได้

deploy อัตโนมัติทุกครั้งที่ push ผ่าน `.github/workflows/pages.yml`

ส่วนที่ต้องมีเซิร์ฟเวอร์ (บัญชีผู้ใช้ ฟาร์ม บ่อ บันทึกประจำวัน สต๊อก การเงิน หลังบ้าน LINE) ต้อง deploy API ตามข้อ 2

## 2. ระบบเต็ม (API + ฐานข้อมูล)

| ทาง | ค่าใช้จ่าย | ข้อมูลถาวร | หมายเหตุ |
|---|---|---|---|
| Fly.io | ฟรีระดับเครื่องเล็ก (ต้องผูกบัตรเพื่อยืนยันตัวตน) | มี (volume 1 GB ที่ /data) | แนะนำ ใกล้ไทย (สิงคโปร์) หลับ/ตื่นอัตโนมัติ |
| Render | ฟรี (หลับหลังไม่มีคนใช้ 15 นาที ตื่นเองเมื่อมีคนเข้า) | ต้องซื้อดิสก์เพิ่ม | เหมาะทดลองสั้น ๆ |
| VPS ไทย | ~150-300 บาท/เดือน | มี | เร็วสุดสำหรับผู้ใช้ในไทย |

ไฟล์พร้อมแล้วในโปรเจกต์: `Dockerfile`, `fly.toml`, `render.yaml`

### ขึ้น Fly.io (ทำครั้งเดียว ~10 นาที)

```bash
fly auth login
fly launch --no-deploy --copy-config --name teedet-pla
fly volumes create teedet_data --size 1 --region sin
fly secrets set ADMIN_PHONE=08xxxxxxxx ADMIN_PIN=123456 PUBLIC_BASE_URL=https://teedet-pla.fly.dev
fly deploy
```

ได้ลิงก์ `https://teedet-pla.fly.dev` ใช้ได้ทั้งระบบ

**ให้ deploy อัตโนมัติทุกครั้งที่แก้โค้ด:** `fly tokens create deploy` แล้วเอา token ไปใส่ที่ GitHub repo > Settings > Secrets and variables > Actions > New repository secret ชื่อ `FLY_API_TOKEN` จากนั้นสั่งรัน workflow "Deploy API (Fly.io)" ได้จากแท็บ Actions

**ให้หน้า GitHub Pages คุยกับ API ตัวนี้:** ที่ repo > Settings > Secrets and variables > Actions > แท็บ Variables > New variable ชื่อ `VITE_API_BASE` ค่า `https://teedet-pla.fly.dev` แล้ว push ใหม่หนึ่งครั้ง

### ทดสอบชั่วคราวจากเครื่องตัวเอง (ไม่ต้องสมัครอะไร)

ดับเบิลคลิก `start-mobile.cmd` จะเปิดเซิร์ฟเวอร์และสร้างลิงก์ HTTPS ชั่วคราวให้ทดสอบบนมือถือ (ลิงก์เปลี่ยนทุกครั้งที่เปิดใหม่ และต้องเปิดเครื่องไว้)

## 3. LINE OA — ตั้งค่าจากในแอป ไม่ต้องแก้ไฟล์

เข้าแอปด้วยบัญชีแอดมิน > เมนู **ฟาร์มทั้งหมด** > แท็บ **LINE**

1. สร้าง LINE Official Account ฟรีที่ https://manager.line.biz แล้วเปิดใช้ Messaging API
2. เข้า https://developers.line.biz/console/ เลือก channel ของ OA นั้น
   - แท็บ **Basic settings** คัดลอก **Channel secret**
   - แท็บ **Messaging API** กด Issue เพื่อสร้าง **Channel access token (long-lived)**
3. วางทั้งสองค่าในแท็บ LINE ของแอป แล้วกดบันทึก
4. คัดลอก **Webhook URL** ที่แอปแสดง ไปวางในหน้า Messaging API ของ LINE เปิด "Use webhook" และปิด "Auto-reply messages"
5. กดปุ่ม "ส่งข้อความทดสอบหาตัวเอง" ในแอปเพื่อยืนยัน (ต้องผูก LINE กับบัญชีตัวเองก่อนที่ ตั้งค่า > เชื่อม LINE)

เมื่อเปิดใช้แล้ว เกษตรกรจะ:

- ได้สรุปอาหารทุกบ่อทุกเช้า 06:00 น. อัตโนมัติ
- พิมพ์ "สรุป" ดูอาหารวันนี้
- พิมพ์ "บ่อ1 ให้แล้ว 12" บันทึกการให้อาหารโดยไม่ต้องเปิดแอป
- พิมพ์ "บ่อ2 ตาย 5" หรือ "บ่อ1 ลอยหัว" บันทึกเหตุการณ์

ค่า token เก็บในฐานข้อมูล ไม่ต้องแก้ `.env` และหน้าจอแสดงเพียงบางส่วนของ token
