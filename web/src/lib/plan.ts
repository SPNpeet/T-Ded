// โปรแกรมการเลี้ยงที่วางไว้ในหน้าวางโปรแกรม รอผูกกับบ่อตอนปล่อยปลา
const KEY = 'teedet.plan.pending'

export type PendingPlan = {
  code: string
  count: number
  stockW: number
  targetW: number
  days: number | null
  plan: { strategy: string; stages: any[]; days: [number, number, number, number, string][] }
}

export function pendingPlan(): PendingPlan | null {
  try {
    const p = JSON.parse(localStorage.getItem(KEY) || 'null')
    return p && p.plan && Array.isArray(p.plan.days) ? p : null
  } catch {
    return null
  }
}
export function clearPendingPlan() {
  try {
    localStorage.removeItem(KEY)
  } catch {
    /* ignore */
  }
}

/**
 * ปรับโปรแกรมให้ตรงกับที่ปล่อยจริง: จำนวนต่างได้ (อาหารต่อวันแปรตามจำนวนปลา)
 * แต่ชนิดปลาหรือน้ำหนักตอนปล่อยต่าง ต้องวางโปรแกรมใหม่ เพราะเส้นการโตเปลี่ยนทั้งเส้น
 */
export function fitPlan(p: PendingPlan, code: string, count: number, stockW: number): { plan: PendingPlan['plan'] | null; reason: string } {
  if (code !== p.code) return { plan: null, reason: 'ชนิดปลาไม่ตรงกับโปรแกรมที่วางไว้' }
  if (Math.abs(stockW - p.stockW) > Math.max(0.5, p.stockW * 0.1)) return { plan: null, reason: `น้ำหนักตอนปล่อยต่างจากที่วางโปรแกรมไว้ (${p.stockW} ก.)` }
  const k = p.count > 0 && count > 0 ? count / p.count : 1
  const days = p.plan.days.map(([d, w, kg, meals, prod]) => [d, w, Math.round(kg * k * 100) / 100, meals, prod] as [number, number, number, number, string])
  return { plan: { ...p.plan, days }, reason: k !== 1 ? `ปรับปริมาณอาหารตามจำนวนปลาที่ปล่อยจริงแล้ว` : '' }
}

/** โปรแกรมของวันที่ day (เหมือน plan_today ฝั่งเซิร์ฟเวอร์) */
export function planToday(plan: any, day: number) {
  const days = plan?.days
  if (!Array.isArray(days) || !days.length) return null
  const want = Math.max(1, day)
  const row = days.find((r: any) => r[0] === want) ?? days[days.length - 1]
  return {
    day: row[0],
    planned_weight_g: row[1],
    planned_feed_kg: row[2],
    meals: row[3],
    product_code: row[4],
    strategy: plan.strategy ?? null,
    past_end: want > days[days.length - 1][0],
    curve: days.filter((_: any, i: number) => i % 7 === 0).map((r: any) => [r[0], r[1]]),
  }
}
