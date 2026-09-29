// ข้อมูลบ่อ: ชนิด รูปทรง ขนาด หน่วยไทย (ไร่ งาน ตารางวา) ใช้ร่วมกันทั้งหน้าจอและโหมดเก็บในเครื่อง

export const POND_TYPES = [
  { v: 'earthen', th: 'บ่อดิน' },
  { v: 'concrete', th: 'บ่อปูน' },
  { v: 'liner', th: 'บ่อผ้าใบ' },
  { v: 'cage', th: 'กระชัง' },
]
export const SHAPES = [
  { v: 'rect', th: 'สี่เหลี่ยม' },
  { v: 'round', th: 'วงกลม' },
  { v: 'free', th: 'รูปร่างอื่น (กรอกพื้นที่)' },
]
export const WATER_SOURCES = [
  { v: 'well', th: 'บ่อบาดาล' },
  { v: 'canal', th: 'คลอง/แม่น้ำ' },
  { v: 'reservoir', th: 'อ่าง/สระ' },
  { v: 'rain', th: 'น้ำฝน' },
  { v: 'tap', th: 'ประปา' },
]
export const AERATION = [
  { v: 'none', th: 'ไม่มี' },
  { v: 'paddle', th: 'ใบพัดตีน้ำ' },
  { v: 'air', th: 'ปั๊มลม/หัวทราย' },
  { v: 'both', th: 'มีทั้งสองแบบ' },
]
export const PHOTO_KINDS = [
  { v: 'overview', th: 'ภาพรวมบ่อ' },
  { v: 'water', th: 'สีน้ำ' },
  { v: 'fish', th: 'ตัวปลา' },
  { v: 'equipment', th: 'อุปกรณ์' },
  { v: 'problem', th: 'ปัญหาที่พบ' },
]

const label = (list: { v: string; th: string }[], v: string | null | undefined) => list.find((x) => x.v === v)?.th ?? ''
export const pondTypeTh = (v?: string | null) => label(POND_TYPES, v) || 'บ่อ'
export const waterSourceTh = (v?: string | null) => label(WATER_SOURCES, v)
export const aerationTh = (v?: string | null) => label(AERATION, v)
export const photoKindTh = (v?: string | null) => label(PHOTO_KINDS, v)

export const SQM_PER_RAI = 1600
export const SQM_PER_NGAN = 400
export const SQM_PER_WA = 4

const num = (v: any): number | null => {
  const x = typeof v === 'string' ? parseFloat(v) : v
  return typeof x === 'number' && Number.isFinite(x) && x > 0 ? x : null
}

/** พื้นที่ผิวน้ำ (ตร.ม.) จากรูปทรงและขนาด หรือจากพื้นที่ที่กรอกเอง */
export function areaM2(p: any): number | null {
  if (p.shape === 'rect') {
    const w = num(p.width_m)
    const l = num(p.length_m)
    if (w && l) return w * l
  }
  if (p.shape === 'round') {
    const d = num(p.diameter_m)
    if (d) return Math.PI * (d / 2) ** 2
  }
  return num(p.area_m2) ?? (num(p.area_rai) ? num(p.area_rai)! * SQM_PER_RAI : null)
}

export function toRaiNganWa(m2: number) {
  const rai = Math.floor(m2 / SQM_PER_RAI)
  const ngan = Math.floor((m2 - rai * SQM_PER_RAI) / SQM_PER_NGAN)
  const wa = Math.round((m2 - rai * SQM_PER_RAI - ngan * SQM_PER_NGAN) / SQM_PER_WA)
  return wa === 100 ? { rai, ngan: ngan + 1, wa: 0 } : { rai, ngan, wa }
}
export const fromRaiNganWa = (rai: any, ngan: any, wa: any) => (num(rai) ?? 0) * SQM_PER_RAI + (num(ngan) ?? 0) * SQM_PER_NGAN + (num(wa) ?? 0) * SQM_PER_WA

const fmt = (x: number, d = 0) => x.toLocaleString('th-TH', { maximumFractionDigits: d })

/** "1 ไร่ 2 งาน 50 ตร.วา" แบบที่ชาวบ้านพูดกัน */
export function areaThai(m2: number | null): string {
  if (!m2) return ''
  if (m2 < SQM_PER_NGAN) return `${fmt(m2 / SQM_PER_WA)} ตร.วา`
  const { rai, ngan, wa } = toRaiNganWa(m2)
  return [rai ? `${rai} ไร่` : '', ngan ? `${ngan} งาน` : '', wa ? `${wa} ตร.วา` : ''].filter(Boolean).join(' ')
}

/** ค่าที่ส่งเก็บ: เติม area_m2/area_rai ให้ตรงกับขนาด */
export function withArea<T extends Record<string, any>>(p: T): T & { area_m2: number | null; area_rai: number | null } {
  const m2 = areaM2(p)
  return { ...p, area_m2: m2 == null ? null : Math.round(m2 * 10) / 10, area_rai: m2 == null ? null : Math.round((m2 / SQM_PER_RAI) * 1000) / 1000 }
}

export function volumeM3(p: any): number | null {
  const a = areaM2(p)
  const d = num(p.depth_m)
  return a && d ? a * d : null
}

export function sizeText(p: any): string {
  const a = areaM2(p)
  const dims = p.shape === 'rect' && num(p.width_m) && num(p.length_m) ? `${fmt(p.width_m, 1)}×${fmt(p.length_m, 1)} ม.` : p.shape === 'round' && num(p.diameter_m) ? `กว้าง ${fmt(p.diameter_m, 1)} ม.` : ''
  return [a ? areaThai(a) : '', dims ? `(${dims})` : ''].filter(Boolean).join(' ')
}

export function pondSummary(p: any): string {
  const v = volumeM3(p)
  return [pondTypeTh(p.pond_type), sizeText(p), num(p.depth_m) ? `ลึก ${fmt(p.depth_m, 1)} ม.` : '', v ? `น้ำ ~${fmt(v)} ลบ.ม.` : ''].filter(Boolean).join(' · ')
}

export const mapUrl = (lat: number, lng: number) => `https://www.google.com/maps/search/?api=1&query=${lat.toFixed(6)},${lng.toFixed(6)}`

export const SPECIES_TH: Record<string, string> = { nile_tilapia: 'ปลานิล', red_tilapia: 'ปลาทับทิม', catfish: 'ปลาดุก', snakehead: 'ปลาช่อน' }
export const speciesTh = (code: string) => SPECIES_TH[code] ?? code
