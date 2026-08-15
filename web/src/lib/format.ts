const TH_MONTHS = ['ม.ค.', 'ก.พ.', 'มี.ค.', 'เม.ย.', 'พ.ค.', 'มิ.ย.', 'ก.ค.', 'ส.ค.', 'ก.ย.', 'ต.ค.', 'พ.ย.', 'ธ.ค.']

export function todayISO(): string {
  const now = new Date()
  const bkk = new Date(now.getTime() + (now.getTimezoneOffset() + 420) * 60000)
  return `${bkk.getFullYear()}-${String(bkk.getMonth() + 1).padStart(2, '0')}-${String(bkk.getDate()).padStart(2, '0')}`
}

export function nowISO(): string {
  return new Date().toISOString()
}

export function thDate(iso?: string | null, withYear = true): string {
  if (!iso) return '-'
  const [y, m, d] = iso.slice(0, 10).split('-').map(Number)
  if (!y || !m || !d) return iso
  return `${d} ${TH_MONTHS[m - 1]}${withYear ? ' ' + (y + 543) : ''}`
}

export function thDateTime(iso?: string | null): string {
  if (!iso) return '-'
  const d = new Date(iso)
  if (isNaN(d.getTime())) return thDate(iso)
  const hh = String(d.getHours()).padStart(2, '0')
  const mm = String(d.getMinutes()).padStart(2, '0')
  return `${thDate(iso, false)} ${hh}:${mm} น.`
}

export function n(v: number | null | undefined, digits = 0): string {
  if (v === null || v === undefined || isNaN(v as number)) return '-'
  return Number(v).toLocaleString('th-TH', { minimumFractionDigits: digits, maximumFractionDigits: digits })
}
export function n1(v: number | null | undefined) {
  return n(v, 1)
}
export function n2(v: number | null | undefined) {
  return n(v, 2)
}
export function baht(v: number | null | undefined) {
  if (v === null || v === undefined || isNaN(v as number)) return '-'
  return n(v, 0) + ' บาท'
}
export function pct(v: number | null | undefined, digits = 0) {
  if (v === null || v === undefined || isNaN(v as number)) return '-'
  return n(v, digits) + '%'
}
export function daysBetween(a: string, b: string): number {
  const x = new Date(a.slice(0, 10) + 'T00:00:00')
  const y = new Date(b.slice(0, 10) + 'T00:00:00')
  return Math.round((y.getTime() - x.getTime()) / 86400000)
}
export function addDays(iso: string, d: number): string {
  const x = new Date(iso.slice(0, 10) + 'T00:00:00')
  x.setDate(x.getDate() + d)
  return `${x.getFullYear()}-${String(x.getMonth() + 1).padStart(2, '0')}-${String(x.getDate()).padStart(2, '0')}`
}
export function healthColor(score: number): string {
  if (score >= 85) return 'var(--green)'
  if (score >= 70) return 'var(--cyan-deep)'
  if (score >= 50) return 'var(--amber)'
  return 'var(--red)'
}
export function bandPill(band: string): { cls: string; text: string } {
  if (band === 'cut') return { cls: 'danger', text: 'ลดมาก' }
  if (band === 'down') return { cls: 'warn', text: 'ลดลง' }
  return { cls: 'good', text: 'ปกติ' }
}
export const FEEDING_RESPONSE = [
  { v: 0, label: 'กินดี ปกติ' },
  { v: 1, label: 'กินช้า / เหลือ' },
  { v: 2, label: 'ลอยหัว / ไม่กิน' },
]
export const EXPENSE_CATEGORIES = [
  ['fry', 'ลูกปลา'],
  ['feed', 'อาหาร (ซื้อนอกสต๊อก)'],
  ['medicine', 'ยา / สารเคมี'],
  ['electric', 'ค่าไฟ / น้ำมัน'],
  ['labor', 'ค่าแรง'],
  ['pond', 'เตรียมบ่อ / ปูน / ปุ๋ย'],
  ['other', 'อื่น ๆ'],
] as const
export function expenseLabel(cat: string) {
  return EXPENSE_CATEGORIES.find((c) => c[0] === cat)?.[1] ?? cat
}
export const PROVINCES = ['เชียงราย', 'เชียงใหม่', 'พะเยา', 'ลำปาง', 'ลำพูน', 'แพร่', 'น่าน', 'อุตรดิตถ์', 'พิษณุโลก', 'นครสวรรค์', 'ขอนแก่น', 'อุดรธานี', 'นครราชสีมา', 'สกลนคร', 'อุบลราชธานี', 'สุพรรณบุรี', 'นครปฐม', 'ราชบุรี', 'ฉะเชิงเทรา', 'ปราจีนบุรี', 'ชลบุรี', 'สมุทรปราการ', 'กรุงเทพมหานคร', 'นครศรีธรรมราช', 'สงขลา', 'สุราษฎร์ธานี', 'อื่น ๆ']
