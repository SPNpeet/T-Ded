// แก้ชื่อที่เพี้ยนเป็น ???? จากการทดสอบผ่าน shell (ใช้ HTTP API ล้วน ไม่แตะไฟล์ DB)
const BASE = process.env.API_BASE || 'http://127.0.0.1:8787'
const ADMIN_PHONE = process.env.ADMIN_PHONE || '0800000000'
const ADMIN_PIN = process.env.ADMIN_PIN || '123456'

const post = async (path, body, token) =>
  (await fetch(BASE + '/api' + path, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', ...(token ? { Authorization: 'Bearer ' + token } : {}) },
    body: JSON.stringify(body),
  })).json()

const { token, error } = await post('/auth/login', { phone: ADMIN_PHONE, pin: ADMIN_PIN })
if (!token) throw new Error('ล็อกอินแอดมินไม่ได้: ' + error)

const users = await (await fetch(BASE + '/api/admin/users', { headers: { Authorization: 'Bearer ' + token } })).json()
const broken = users.filter((u) => /\?/.test(u.name || ''))
console.log('ผู้ใช้ที่ชื่อเพี้ยน:', broken.map((u) => u.phone + ' -> ' + u.name).join(', ') || 'ไม่มี')
console.log('หมายเหตุ: ยังไม่มี endpoint แก้ชื่อผู้ใช้ ให้แก้จากหน้าตั้งค่าของผู้ใช้เอง หรือเพิ่ม endpoint ภายหลัง')
