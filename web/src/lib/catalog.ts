// แคตตาล็อกอาหาร: ดึงจากเซิร์ฟเวอร์ ถ้าไม่มีใช้ไฟล์ในแอป แล้วกรองตามแบรนด์ที่ build
import { cachedGet } from './api'
import { brand } from './brand'

export type Product = {
  id: string
  brand: string
  brand_group?: string
  product_code: string
  name_th: string
  target: string
  stage_th?: string
  weight_from_g: number
  weight_to_g: number
  protein_pct: number
  fat_pct?: number | null
  pellet_mm?: number | null
  form?: string | null
  bag_kg: number
  price_ref?: number | null
  color_th?: string | null
  sack_label_th?: string | null
  verified?: number
  note?: string | null
  source_url?: string | null
}

let cache: Product[] | null = null

export async function loadProducts(): Promise<Product[]> {
  if (cache) return cache
  let list: Product[] = []
  try {
    list = (await cachedGet<Product[]>('/feed-products')).data
  } catch {
    try {
      list = await fetch(import.meta.env.BASE_URL + 'feed-products.json').then((r) => r.json())
    } catch {
      list = []
    }
  }
  // บางเบอร์ไม่มีรหัสสินค้า ต้องมีรหัสไม่ซ้ำ ไม่งั้นการเลือกเบอร์/ผลความเหมาะของโปรตีนจะทับกัน
  const seen = new Set<string>()
  list = list.map((p) => {
    let code = (p.product_code || '').trim() || p.name_th
    if (seen.has(code)) code = `${code} (${p.id})`
    seen.add(code)
    return code === p.product_code ? p : { ...p, product_code: code }
  })
  cache = brand.productGroup ? list.filter((p) => p.brand_group === brand.productGroup) : list
  return cache
}

/** แปลงเป็นรูปแบบที่ engine ใช้วางแผน */
export function toEngineProduct(p: Product, priceOverride?: Record<string, number>) {
  const price = priceOverride?.[p.product_code] ?? (p.price_ref && p.price_ref > 0 ? p.price_ref : null)
  return {
    code: p.product_code,
    name_th: p.name_th,
    brand: p.brand,
    target: p.target,
    weight_from_g: p.weight_from_g ?? 0,
    weight_to_g: p.weight_to_g ?? 100000,
    protein_pct: p.protein_pct ?? 0,
    pellet_mm: p.pellet_mm ?? null,
    bag_kg: p.bag_kg || 20,
    price_per_bag: price,
    color_th: p.color_th ?? null,
  }
}

/** กลุ่มอาหารที่ปลาชนิดนี้ใช้ได้ ต้องตรงกับ feed_groups ใน engine */
export function suits(p: Product, speciesCode: string) {
  const g: Record<string, string[]> = {
    nile_tilapia: ['tilapia', 'herbivore', 'all'],
    red_tilapia: ['tilapia', 'herbivore', 'all'],
    catfish: ['catfish', 'all'],
    snakehead: ['carnivore', 'all'],
  }
  return (g[speciesCode] ?? ['all']).includes(p.target)
}

/** สีกระสอบเป็นรหัสสีสำหรับแสดงผล */
export function sackColor(th?: string | null): string | null {
  const map: Record<string, string> = { ส้ม: '#F28C28', น้ำเงิน: '#1F4FA3', เหลือง: '#F2C200', เขียว: '#2E9D5B', แดง: '#D0342C', ม่วง: '#7A3FB0', ฟ้า: '#2BA6DE' }
  return th ? map[th] ?? null : null
}
