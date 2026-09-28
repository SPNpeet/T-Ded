// โหมดเก็บในเครื่อง: ใช้แอปได้ครบโดยไม่ต้องมีเซิร์ฟเวอร์
// กติกาการคำนวณทั้งหมดยังมาจาก aqua-engine (WASM) ตัวเดียวกับฝั่งเซิร์ฟเวอร์ ที่นี่ทำแค่ประกอบข้อมูลป้อนให้ engine
import { engine, speciesByCode } from './engine'
import { todayISO } from './format'

const KEY = 'teedet.local.db'
const MODE_KEY = 'teedet.local.on'

export type LocalDb = {
  v: number
  user: { id: string; name: string; phone: string; role: string; org_id: string; org_name: string }
  farms: any[]
  ponds: any[]
  crops: any[]
  logs: any[]
  weighings: any[]
  water: any[]
  stock: any[]
  expenses: any[]
  harvests: any[]
  treatments: any[]
  prices: any[]
}

export const isLocalMode = () => localStorage.getItem(MODE_KEY) === '1'
export function setLocalMode(on: boolean) {
  if (on) localStorage.setItem(MODE_KEY, '1')
  else localStorage.removeItem(MODE_KEY)
}

const uid = () => (crypto?.randomUUID?.() ?? `${Date.now()}-${Math.random().toString(16).slice(2)}`)

function blank(): LocalDb {
  return {
    v: 1,
    user: { id: 'local-user', name: 'เจ้าของฟาร์ม', phone: '', role: 'owner', org_id: 'local-org', org_name: 'ฟาร์มของฉัน' },
    farms: [],
    ponds: [],
    crops: [],
    logs: [],
    weighings: [],
    water: [],
    stock: [],
    expenses: [],
    harvests: [],
    treatments: [],
    prices: [],
  }
}

export function db(): LocalDb {
  try {
    const raw = localStorage.getItem(KEY)
    if (raw) return { ...blank(), ...JSON.parse(raw) }
  } catch {
    /* ข้อมูลเสีย ใช้ของว่างแทน ไม่ให้แอปพัง */
  }
  return blank()
}
export function save(d: LocalDb) {
  localStorage.setItem(KEY, JSON.stringify(d))
}
export function exportJson(): string {
  return JSON.stringify(db(), null, 1)
}
export function importJson(text: string) {
  const d = JSON.parse(text)
  if (!d || typeof d !== 'object' || !Array.isArray(d.farms)) throw new Error('ไฟล์ไม่ถูกต้อง')
  save({ ...blank(), ...d })
}

/** เริ่มใช้งานครั้งแรก: สร้างฟาร์มให้เลย */
export function initFarm(farmName: string, ownerName: string, province?: string, lat?: number, lng?: number) {
  const d = blank()
  d.user.name = ownerName || 'เจ้าของฟาร์ม'
  d.user.org_name = farmName
  d.farms = [{ id: uid(), org_id: 'local-org', name: farmName, province: province ?? null, lat: lat ?? null, lng: lng ?? null, meals_per_day: 2, farm_factor: 1, bag_kg: 20, created_at: new Date().toISOString() }]
  save(d)
  return d
}

const days = (a: string, b: string) => Math.round((new Date(b.slice(0, 10) + 'T00:00:00').getTime() - new Date(a.slice(0, 10) + 'T00:00:00').getTime()) / 86400000)

async function weatherFor(lat?: number | null, lng?: number | null, date?: string) {
  if (lat == null || lng == null) return null
  const d = date || todayISO()
  const key = `teedet.wx:${lat.toFixed(2)},${lng.toFixed(2)},${d}`
  try {
    const cached = localStorage.getItem(key)
    if (cached) {
      const c = JSON.parse(cached)
      if (Date.now() - c.at < 3 * 3600 * 1000 || d < todayISO()) return c.data
    }
  } catch {
    /* ignore */
  }
  const past = d < todayISO()
  const base = past ? 'https://archive-api.open-meteo.com/v1/archive' : 'https://api.open-meteo.com/v1/forecast'
  const url = `${base}?latitude=${lat}&longitude=${lng}&daily=temperature_2m_max,temperature_2m_min,precipitation_sum,cloud_cover_mean&timezone=Asia%2FBangkok&start_date=${d}&end_date=${d}`
  try {
    const r = await fetch(url)
    if (!r.ok) return null
    const j = await r.json()
    const first = (k: string) => j?.daily?.[k]?.[0] ?? null
    const out = { date: d, lat, lng, source: past ? 'open-meteo-archive' : 'open-meteo-forecast', tmax_c: first('temperature_2m_max'), tmin_c: first('temperature_2m_min'), rain_mm: first('precipitation_sum'), cloud_pct: first('cloud_cover_mean') }
    try {
      localStorage.setItem(key, JSON.stringify({ at: Date.now(), data: out }))
    } catch {
      /* ignore */
    }
    return out
  } catch {
    return null
  }
}

export function stockSummary(d: LocalDb, farmId: string) {
  const moves = d.stock.filter((m) => m.farm_id === farmId)
  const bagKg = d.farms.find((f) => f.id === farmId)?.bag_kg ?? 20
  const balance = moves.reduce((a, m) => a + (m.kind === 'out' ? -m.kg : m.kg), 0)
  const inKg = moves.filter((m) => m.kind === 'in').reduce((a, m) => a + m.kg, 0)
  const inPrice = moves.filter((m) => m.kind === 'in').reduce((a, m) => a + (m.price_total || 0), 0)
  const since = new Date(Date.now() - 7 * 86400000).toISOString().slice(0, 10)
  const used7 = moves.filter((m) => m.kind === 'out' && m.move_date >= since).reduce((a, m) => a + m.kg, 0)
  const perDay = used7 / 7
  const lastIn = [...moves].filter((m) => m.kind === 'in').sort((a, b) => (a.move_date < b.move_date ? 1 : -1))[0]
  return {
    balance_kg: Math.round(balance * 10) / 10,
    balance_bags: Math.round((balance / bagKg) * 10) / 10,
    bag_kg: bagKg,
    avg_price_per_kg: inKg > 0 ? Math.round((inPrice / inKg) * 100) / 100 : 0,
    used_per_day_7d: Math.round(perDay * 100) / 100,
    days_left: perDay > 0 ? Math.floor(balance / perDay) : null,
    low: perDay > 0 ? balance / perDay <= 5 : false,
    moves: [...moves].sort((a, b) => (a.move_date < b.move_date ? 1 : -1)).slice(0, 30),
    current_feed: lastIn ? { brand: lastIn.brand ?? null, protein_pct: lastIn.protein_pct ?? null, pellet_mm: lastIn.pellet_mm ?? null, form: lastIn.form ?? null, price_per_kg: lastIn.price_total && lastIn.kg ? Math.round((lastIn.price_total / lastIn.kg) * 100) / 100 : null } : null,
  }
}

/** ประกอบสแนปช็อตของรุ่นเลี้ยง ให้ผลเหมือน /crops/:id/today ของเซิร์ฟเวอร์ */
export async function cropSnapshot(cropId: string, opts: { date?: string; withWeather?: boolean; withForecast?: boolean; sellPrice?: number; feedPrice?: number; otherCostPerDay?: number } = {}) {
  const d = db()
  const crop = d.crops.find((c) => c.id === cropId)
  if (!crop) throw new Error('ไม่พบรุ่นการเลี้ยง')
  const pond = d.ponds.find((p) => p.id === crop.pond_id)
  const farm = d.farms.find((f) => f.id === crop.farm_id)
  const date = opts.date || todayISO()
  const sp = await speciesByCode(crop.species_code)
  const e = await engine()

  const day = Math.max(0, days(crop.stocked_at, date))
  const ws = d.weighings.filter((w) => w.crop_id === cropId && w.weigh_date <= date).sort((a, b) => (a.weigh_date < b.weigh_date ? -1 : 1))
  const last = ws[ws.length - 1] ?? { weigh_date: crop.stocked_at, avg_weight_g: crop.stock_weight_g }
  const prev = ws.length >= 2 ? ws[ws.length - 2] : null
  const lastDay = Math.max(0, days(crop.stocked_at, last.weigh_date))
  const recentAdg = prev && days(prev.weigh_date, last.weigh_date) > 0 ? (last.avg_weight_g - prev.avg_weight_g) / days(prev.weigh_date, last.weigh_date) : null

  // ประมาณน้ำหนักวันนี้จากเส้นโค้งมาตรฐาน ต่อจากการชั่งล่าสุด
  const growthPts: [number, number][] = sp.growth.map((g: any) => [g.day_from, g.weight_g])
  const lastRow = sp.growth[sp.growth.length - 1]
  growthPts.push([lastRow.day_to, lastRow.weight_g + lastRow.adg * (lastRow.day_to - lastRow.day_from)])
  const wAt = (x: number) => {
    if (x <= growthPts[0][0]) return growthPts[0][1]
    for (let i = 0; i < growthPts.length - 1; i++) if (x >= growthPts[i][0] && x <= growthPts[i + 1][0]) return growthPts[i][1] + ((x - growthPts[i][0]) / (growthPts[i + 1][0] - growthPts[i][0])) * (growthPts[i + 1][1] - growthPts[i][1])
    return growthPts[growthPts.length - 1][1] + (x - growthPts[growthPts.length - 1][0]) * lastRow.adg
  }
  const dAt = (w: number) => {
    if (w <= growthPts[0][1]) return growthPts[0][0]
    for (let i = 0; i < growthPts.length - 1; i++) if (w >= growthPts[i][1] && w <= growthPts[i + 1][1]) return growthPts[i][0] + ((w - growthPts[i][1]) / (growthPts[i + 1][1] - growthPts[i][1])) * (growthPts[i + 1][0] - growthPts[i][0])
    return growthPts[growthPts.length - 1][0] + (w - growthPts[growthPts.length - 1][1]) / lastRow.adg
  }
  const stdAdgAt = (w: number) => wAt(dAt(w) + 1) - wAt(dAt(w))
  const growthScale = recentAdg ? Math.min(1.5, Math.max(0.3, recentAdg / Math.max(0.01, stdAdgAt(last.avg_weight_g)))) : 1
  const sinceWeigh = day - lastDay
  const estW = sinceWeigh <= 0 ? last.avg_weight_g : Math.max(last.avg_weight_g, wAt(dAt(last.avg_weight_g) + sinceWeigh * growthScale))

  const logs = d.logs.filter((l) => l.crop_id === cropId && l.log_date <= date)
  const dead = logs.reduce((a, l) => a + (l.mortality || 0), 0)
  const fedTotal = logs.reduce((a, l) => a + (l.fed_kg || 0), 0)
  const harv = d.harvests.filter((h) => h.crop_id === cropId && h.harvest_date <= date)
  const harvestedCount = harv.reduce((a, h) => a + (h.count || 0), 0)
  const harvestedKg = harv.reduce((a, h) => a + (h.kg || 0), 0)
  const revenue = harv.reduce((a, h) => a + (h.kg || 0) * (h.price_per_kg || 0), 0)
  const expenses = d.expenses.filter((x) => x.crop_id === cropId && x.expense_date <= date).reduce((a, x) => a + (x.amount || 0), 0)
  const stock = stockSummary(d, crop.farm_id)
  const feedPrice = opts.feedPrice ?? (stock.avg_price_per_kg > 0 ? stock.avg_price_per_kg : 0)
  const feedCost = fedTotal * feedPrice
  const costTotal = expenses + feedCost
  const alive = Math.max(0, crop.stocked_count - dead - harvestedCount)
  const marketPrice = opts.sellPrice ?? crop.sell_price ?? d.prices.filter((p) => p.species_code === crop.species_code).sort((a, b) => (a.price_date < b.price_date ? 1 : -1))[0]?.price_per_kg ?? null

  const weather = opts.withWeather === false ? null : await weatherFor(farm?.lat, farm?.lng, date)
  const waterRow = d.water.filter((w) => w.pond_id === crop.pond_id && w.measured_at.slice(0, 10) <= date).sort((a, b) => (a.measured_at < b.measured_at ? 1 : -1))[0] ?? null
  const waterRecent = waterRow && days(waterRow.measured_at.slice(0, 10), date) <= 1 ? waterRow : null
  const sample = { do_mg_l: waterRecent?.do_mg_l ?? null, ph: waterRecent?.ph ?? null, temp_c: waterRecent?.temp_c ?? null, nh3: waterRecent?.nh3 ?? null, no2: waterRecent?.no2 ?? null, secchi_cm: waterRecent?.secchi_cm ?? null }

  const todayLog = logs.find((l) => l.log_date === date) ?? null
  const lastResponse = todayLog ? todayLog.feeding_response ?? 0 : [...logs].sort((a, b) => (a.log_date < b.log_date ? 1 : -1))[0]?.feeding_response ?? 0
  const env: any = {
    tmax_c: weather?.tmax_c ?? null,
    tmin_c: weather?.tmin_c ?? null,
    rain_mm: weather?.rain_mm ?? null,
    cloud_pct: weather?.cloud_pct ?? null,
    do_morning: sample.do_mg_l,
    nh3: sample.nh3,
    stress: ['normal', 'slow_eating', 'gasping'][lastResponse] ?? 'normal',
  }
  const hasEnv = env.tmax_c != null || env.tmin_c != null || env.do_morning != null || env.nh3 != null || lastResponse > 0

  const rec = e.feed_recommend({ species: sp, avg_weight_g: estW, count: alive, env: hasEnv ? env : null, rules: [], meals_per_day: farm?.meals_per_day ?? null, farm_factor: farm?.farm_factor ?? null })
  const perf = e.perf_calc({
    stocked_count: crop.stocked_count,
    stock_weight_g: crop.stock_weight_g,
    dead_count: dead,
    harvested_count: harvestedCount,
    harvested_kg: harvestedKg,
    avg_weight_g: estW,
    feed_kg_total: fedTotal,
    cost_total: costTotal,
    feed_cost_total: feedCost,
    revenue_total: revenue,
    day,
    price_per_kg: marketPrice,
  })
  const targetW = crop.target_weight_g ?? sp.market_weight_g
  const growth = e.growth_compare(sp, crop.stock_weight_g, lastDay, last.avg_weight_g, prev ? Math.max(0, days(crop.stocked_at, prev.weigh_date)) : undefined, prev ? prev.avg_weight_g : undefined, targetW)

  const since7 = new Date(new Date(date).getTime() - 7 * 86400000).toISOString().slice(0, 10)
  const dead7 = logs.filter((l) => l.log_date > since7).reduce((a, l) => a + (l.mortality || 0), 0)
  const mortality7 = alive + dead7 > 0 ? (dead7 / (alive + dead7)) * 100 : 0
  const lastLogDate = [...logs].sort((a, b) => (a.log_date < b.log_date ? 1 : -1))[0]?.log_date
  const health = e.pond_health(
    {
      water: sample,
      mortality_7d_pct: mortality7,
      feeding_response: lastResponse,
      growth_status: day > 7 ? growth.status : null,
      days_since_last_log: lastLogDate ? Math.max(0, days(lastLogDate, date)) : day,
      previous_score: null,
    },
    sp,
  )

  const projection =
    opts.withForecast === false
      ? null
      : e.forecast_project({
          species: sp,
          day,
          avg_weight_g: estW,
          alive_count: alive,
          daily_mortality_rate: perf.daily_mortality_rate,
          target_weight_g: targetW,
          target_days: null,
          growth_scale: growthScale,
          avg_feed_factor: 0.95,
          feed_price_per_kg: feedPrice,
          other_cost_per_day: opts.otherCostPerDay ?? 0,
          cost_so_far: costTotal,
          feed_kg_so_far: fedTotal,
          sell_price_per_kg: marketPrice ?? 0,
          bag_kg: farm?.bag_kg ?? 20,
          max_days: null,
        })

  const nutrition = e.nutrition_advise(crop.species_code, estW, rec.final_kg, {
    brand: stock.current_feed?.brand ?? null,
    protein_pct: stock.current_feed?.protein_pct ?? null,
    pellet_mm: stock.current_feed?.pellet_mm ?? null,
    price_per_kg: stock.current_feed?.price_per_kg ?? (feedPrice > 0 ? feedPrice : null),
    form: stock.current_feed?.form ?? null,
  })

  const alerts: any[] = health.alerts_th.map((t: string) => ({ level: 'warn', text: t }))
  if (stock.low) alerts.push({ level: 'info', text: `อาหารในสต๊อกเหลือประมาณ ${stock.days_left} วัน` })
  if (sinceWeigh >= 14 && day >= 14) alerts.push({ level: 'info', text: `ไม่ได้ชั่งน้ำหนักมา ${sinceWeigh} วัน ควรสุ่มชั่งเพื่อปรับอาหาร` })
  if (nutrition.status === 'protein_low' || nutrition.status === 'pellet_mismatch') if (nutrition.messages_th[0]) alerts.push({ level: 'info', text: nutrition.messages_th[0] })

  return {
    date,
    crop: { ...crop, pond_name: pond?.name ?? '-', area_rai: pond?.area_rai ?? null, farm_name: farm?.name ?? '', lat: farm?.lat, lng: farm?.lng, province: farm?.province, meals_per_day: farm?.meals_per_day, farm_factor: farm?.farm_factor, bag_kg: farm?.bag_kg },
    species: { code: sp.code, name_th: sp.name_th, market_weight_g: sp.market_weight_g, approximate: sp.approximate },
    day,
    alive_count: alive,
    avg_weight_g: Math.round(estW * 10) / 10,
    avg_weight_source: sinceWeigh <= 0 ? 'weighed' : 'estimated',
    last_weighed: { date: last.weigh_date, avg_weight_g: last.avg_weight_g, days_ago: sinceWeigh },
    growth_scale: Math.round(growthScale * 100) / 100,
    weather,
    water: waterRow,
    env_used: env,
    recommendation: rec,
    nutrition,
    feed_on_hand: stock.current_feed ?? {},
    performance: perf,
    growth,
    health,
    projection,
    stock: { balance_kg: stock.balance_kg, balance_bags: stock.balance_bags, days_left: stock.days_left, low: stock.low, avg_price_per_kg: stock.avg_price_per_kg },
    market_price_per_kg: marketPrice,
    today_log: todayLog,
    withdrawal_until: null,
    alerts,
    totals: { fed_kg: fedTotal, dead, expenses, feed_cost: feedCost, cost_total: costTotal, revenue, harvested_kg: harvestedKg },
  }
}

export async function farmToday(farmId: string) {
  const d = db()
  const farm = d.farms.find((f) => f.id === farmId)
  if (!farm) throw new Error('ไม่พบฟาร์ม')
  const date = todayISO()
  const crops = d.crops.filter((c) => c.farm_id === farmId && c.status === 'active')
  const items: any[] = []
  let totalFeed = 0
  let totalValue = 0
  for (const c of crops) {
    const s = await cropSnapshot(c.id, { date, withForecast: false })
    totalFeed += s.recommendation.final_kg || 0
    totalValue += s.performance.stock_value || 0
    items.push({
      crop_id: c.id,
      pond_id: c.pond_id,
      pond_name: s.crop.pond_name,
      species: s.species,
      day: s.day,
      alive_count: s.alive_count,
      avg_weight_g: s.avg_weight_g,
      avg_weight_source: s.avg_weight_source,
      recommendation: s.recommendation,
      health: { score: s.health.score, grade: s.health.grade, grade_th: s.health.grade_th, trend: s.health.trend },
      growth: { status: s.growth.status, status_th: s.growth.status_th, deviation_pct: s.growth.deviation_pct },
      performance: { fcr: s.performance.fcr, survival_pct: s.performance.survival_pct, stock_value: s.performance.stock_value, biomass_kg: s.performance.biomass_kg },
      today_log: s.today_log,
      alerts: s.alerts,
      weather: s.weather,
    })
  }
  const emptyPonds = d.ponds.filter((p) => p.farm_id === farmId && p.active !== 0 && !crops.some((c) => c.pond_id === p.id))
  const stock = stockSummary(d, farmId)
  const weather = await weatherFor(farm.lat, farm.lng, date)
  // นับวันบันทึกต่อเนื่อง
  const dates = [...new Set(d.logs.filter((l) => crops.some((c) => c.id === l.crop_id) || d.crops.some((c) => c.farm_id === farmId && c.id === l.crop_id)).map((l) => l.log_date))].sort().reverse()
  let streak = 0
  let cursor = date
  for (const dt of dates) {
    if (dt === cursor) {
      streak++
      cursor = new Date(new Date(cursor).getTime() - 86400000).toISOString().slice(0, 10)
    } else if (streak === 0 && days(dt, date) === 1) {
      streak = 1
      cursor = new Date(new Date(dt).getTime() - 86400000).toISOString().slice(0, 10)
    } else break
  }
  return {
    date,
    farm,
    weather,
    ponds: items,
    empty_ponds: emptyPonds,
    stock: { balance_kg: stock.balance_kg, balance_bags: stock.balance_bags, days_left: stock.days_left, low: stock.low },
    totals: { feed_today_kg: Math.round(totalFeed * 100) / 100, stock_value: Math.round(totalValue) },
    streak_days: streak,
    announcements: [],
  }
}

/** จัดการคำขอแบบเดียวกับ API ของเซิร์ฟเวอร์ */
export async function handle(method: string, path: string, body?: any): Promise<any> {
  const [p, qs] = path.split('?')
  const q = new URLSearchParams(qs || '')
  const seg = p.split('/').filter(Boolean)
  const d = db()
  const now = new Date().toISOString()
  const stamp = (extra: any) => ({ id: uid(), created_at: now, ...extra })

  const M = (m: string, ...parts: (string | RegExp)[]) =>
    method === m && seg.length === parts.length && parts.every((x, i) => (typeof x === 'string' ? x === seg[i] : x.test(seg[i])))
  const ANY = /.+/

  if (M('GET', 'me')) return { ...d.user, farms: d.farms }
  if (M('PATCH', 'me')) {
    d.user.name = String(body?.name || d.user.name).trim()
    save(d)
    return { ...d.user, farms: d.farms }
  }
  if (M('PATCH', 'org')) {
    d.user.org_name = String(body?.name || d.user.org_name).trim()
    save(d)
    return { ...d.user, farms: d.farms }
  }
  if (M('GET', 'subscription'))
    return {
      plan: 'local',
      plan_name_th: 'เก็บในเครื่อง (ไม่จำกัด)',
      active: true,
      expires_at: null,
      days_left: null,
      usage: { farms: d.farms.length, ponds: d.ponds.length, members: 1 },
      limits: { farms: 99, ponds: 999, members: 1 },
      payments: [],
      plans: [{ code: 'local', name_th: 'เก็บในเครื่อง', price: 0, detail_th: 'ข้อมูลอยู่ในมือถือของคุณ ใช้ได้ไม่จำกัด ไม่มีค่าบริการ' }],
    }

  if (M('GET', 'farms')) return d.farms
  if (M('POST', 'farms')) {
    const f = stamp({ org_id: 'local-org', name: body.name, province: body.province ?? null, lat: body.lat ?? null, lng: body.lng ?? null, meals_per_day: 2, farm_factor: 1, bag_kg: 20 })
    d.farms.push(f)
    save(d)
    return { id: f.id }
  }
  if (M('GET', 'farms', ANY)) {
    const f = d.farms.find((x) => x.id === seg[1])
    return { ...f, ponds: d.ponds.filter((p) => p.farm_id === seg[1] && p.active !== 0), members: [{ id: d.user.id, name: d.user.name, phone: d.user.phone, role: 'owner' }] }
  }
  if (M('PATCH', 'farms', ANY)) {
    const f = d.farms.find((x) => x.id === seg[1])
    if (f) Object.assign(f, Object.fromEntries(Object.entries(body).filter(([, v]) => v !== null && v !== undefined)))
    save(d)
    return { ok: true }
  }
  if (M('GET', 'farms', ANY, 'today')) return farmToday(seg[1])
  if (M('POST', 'farms', ANY, 'ponds')) {
    const order = d.ponds.filter((p) => p.farm_id === seg[1]).length + 1
    const p2 = stamp({ farm_id: seg[1], name: body.name, pond_type: body.pond_type ?? 'earthen', area_rai: body.area_rai ?? null, area_m2: body.area_m2 ?? (body.area_rai ? body.area_rai * 1600 : null), depth_m: body.depth_m ?? null, sort_order: order, active: 1 })
    d.ponds.push(p2)
    save(d)
    return { id: p2.id }
  }
  if (M('GET', 'farms', ANY, 'crops')) {
    const status = q.get('status') || 'active'
    return d.crops
      .filter((c) => c.farm_id === seg[1] && (status === 'all' || c.status === status))
      .map((c) => ({ ...c, pond_name: d.ponds.find((p) => p.id === c.pond_id)?.name ?? '-' }))
  }
  if (M('GET', 'farms', ANY, 'stock')) return stockSummary(d, seg[1])
  if (M('POST', 'farms', ANY, 'stock')) {
    const bagKg = body.bag_kg ?? d.farms.find((f) => f.id === seg[1])?.bag_kg ?? 20
    const kg = body.kg ?? (body.bags ? body.bags * bagKg : 0)
    const m = stamp({ farm_id: seg[1], move_date: body.move_date || todayISO(), kind: body.kind || 'in', brand: body.brand ?? null, pellet_mm: body.pellet_mm ?? null, protein_pct: body.protein_pct ?? null, form: body.form ?? null, bags: body.bags ?? null, kg, price_total: body.price_total ?? null, product_id: body.product_id ?? null, note: body.note ?? null })
    d.stock.push(m)
    save(d)
    return { id: m.id }
  }

  if (M('PATCH', 'ponds', ANY)) {
    const p2 = d.ponds.find((x) => x.id === seg[1])
    if (p2) Object.assign(p2, Object.fromEntries(Object.entries(body).filter(([, v]) => v !== null && v !== undefined)))
    save(d)
    return { ok: true }
  }
  if (M('POST', 'ponds', ANY, 'crops')) {
    const pond = d.ponds.find((x) => x.id === seg[1])
    if (d.crops.some((c) => c.pond_id === seg[1] && c.status === 'active')) throw new Error('บ่อนี้มีรุ่นที่เลี้ยงอยู่ ปิดรุ่นเดิมก่อน')
    const c = stamp({ pond_id: seg[1], farm_id: pond.farm_id, species_code: body.species_code || 'nile_tilapia', stocked_at: body.stocked_at || todayISO(), stocked_count: Number(body.stocked_count), stock_weight_g: Number(body.stock_weight_g), fry_price_each: Number(body.fry_price_each || 0), target_weight_g: body.target_weight_g ?? null, note: body.note ?? null, status: 'active' })
    d.crops.push(c)
    d.weighings.push(stamp({ crop_id: c.id, weigh_date: c.stocked_at, avg_weight_g: c.stock_weight_g, method: 'stocking', note: 'น้ำหนักตอนปล่อย' }))
    if (c.fry_price_each > 0) d.expenses.push(stamp({ crop_id: c.id, expense_date: c.stocked_at, category: 'fry', amount: c.fry_price_each * c.stocked_count, note: 'ค่าลูกปลา' }))
    save(d)
    return { id: c.id }
  }
  if (M('GET', 'ponds', ANY, 'water')) return d.water.filter((w) => w.pond_id === seg[1]).sort((a, b) => (a.measured_at < b.measured_at ? 1 : -1)).slice(0, Number(q.get('limit') || 90))
  if (M('POST', 'ponds', ANY, 'water')) {
    const w = stamp({ pond_id: seg[1], measured_at: body.measured_at || now, do_mg_l: body.do_mg_l ?? null, ph: body.ph ?? null, temp_c: body.temp_c ?? null, nh3: body.nh3 ?? null, no2: body.no2 ?? null, secchi_cm: body.secchi_cm ?? null, color: body.color ?? null, note: body.note ?? null })
    d.water.push(w)
    save(d)
    return { id: w.id }
  }

  if (M('GET', 'crops', ANY, 'today'))
    return cropSnapshot(seg[1], {
      date: q.get('date') || undefined,
      withWeather: q.get('weather') !== '0',
      withForecast: q.get('forecast') !== '0',
      sellPrice: q.get('sell_price') ? Number(q.get('sell_price')) : undefined,
      feedPrice: q.get('feed_price') ? Number(q.get('feed_price')) : undefined,
      otherCostPerDay: q.get('other_cost_per_day') ? Number(q.get('other_cost_per_day')) : undefined,
    })
  if (M('PATCH', 'crops', ANY)) {
    const c = d.crops.find((x) => x.id === seg[1])
    if (c) Object.assign(c, Object.fromEntries(Object.entries(body).filter(([, v]) => v !== null && v !== undefined)))
    save(d)
    return { ok: true }
  }
  if (M('POST', 'crops', ANY, 'close')) {
    const c = d.crops.find((x) => x.id === seg[1])
    if (c) {
      c.status = 'closed'
      c.closed_at = todayISO()
    }
    save(d)
    return { ok: true }
  }
  if (M('GET', 'crops', ANY, 'logs')) return d.logs.filter((l) => l.crop_id === seg[1]).sort((a, b) => (a.log_date < b.log_date ? 1 : -1)).slice(0, Number(q.get('limit') || 60))
  if (M('POST', 'crops', ANY, 'logs')) {
    const date = body.log_date || todayISO()
    const crop = d.crops.find((c) => c.id === seg[1])
    let l = d.logs.find((x) => x.crop_id === seg[1] && x.log_date === date)
    if (!l) {
      l = stamp({ crop_id: seg[1], log_date: date, mortality: 0, feeding_response: 0 })
      d.logs.push(l)
    }
    for (const k of ['fed_kg', 'recommended_kg', 'factor', 'mortality', 'feeding_response', 'note', 'photo_url']) if (body[k] !== undefined && body[k] !== null) (l as any)[k] = body[k]
    l.updated_at = now
    // ตัดสต๊อกตามอาหารที่ให้
    d.stock = d.stock.filter((m) => m.ref_log_id !== l!.id)
    if (body.fed_kg > 0) d.stock.push(stamp({ farm_id: crop.farm_id, move_date: date, kind: 'out', kg: Number(body.fed_kg), crop_id: seg[1], ref_log_id: l.id, note: 'ให้อาหารตามบันทึกประจำวัน' }))
    if (body.water && typeof body.water === 'object' && Object.values(body.water).some((v) => v !== null && v !== undefined)) {
      d.water.push(stamp({ pond_id: crop.pond_id, measured_at: body.water.measured_at || now, do_mg_l: body.water.do_mg_l ?? null, temp_c: body.water.temp_c ?? null, ph: null, nh3: null, no2: null, secchi_cm: null }))
    }
    save(d)
    return { id: l.id }
  }
  if (M('GET', 'crops', ANY, 'weighings')) return d.weighings.filter((w) => w.crop_id === seg[1]).sort((a, b) => (a.weigh_date < b.weigh_date ? -1 : 1))
  if (M('POST', 'crops', ANY, 'weighings')) {
    const w = stamp({ crop_id: seg[1], weigh_date: body.weigh_date || todayISO(), avg_weight_g: Number(body.avg_weight_g), sample_count: body.sample_count ?? null, method: body.method || 'sample', note: body.note ?? null })
    d.weighings.push(w)
    save(d)
    return { id: w.id }
  }
  if (M('GET', 'crops', ANY, 'expenses')) return d.expenses.filter((x) => x.crop_id === seg[1]).sort((a, b) => (a.expense_date < b.expense_date ? 1 : -1))
  if (M('POST', 'crops', ANY, 'expenses')) {
    const x = stamp({ crop_id: seg[1], expense_date: body.expense_date || todayISO(), category: body.category || 'other', amount: Number(body.amount), note: body.note ?? null })
    d.expenses.push(x)
    save(d)
    return { id: x.id }
  }
  if (M('GET', 'crops', ANY, 'harvests')) return d.harvests.filter((h) => h.crop_id === seg[1]).sort((a, b) => (a.harvest_date < b.harvest_date ? 1 : -1))
  if (M('POST', 'crops', ANY, 'harvests')) {
    const h = stamp({ crop_id: seg[1], harvest_date: body.harvest_date || todayISO(), kg: Number(body.kg), count: body.count ?? null, price_per_kg: body.price_per_kg ?? null, buyer: body.buyer ?? null, note: body.note ?? null })
    d.harvests.push(h)
    if (body.price_per_kg) {
      const c = d.crops.find((x) => x.id === seg[1])
      d.prices.push(stamp({ species_code: c?.species_code ?? 'nile_tilapia', province: null, price_per_kg: Number(body.price_per_kg), source: 'harvest', price_date: h.harvest_date }))
    }
    save(d)
    return { id: h.id }
  }
  if (M('GET', 'crops', ANY, 'treatments')) return d.treatments.filter((t) => t.crop_id === seg[1]).sort((a, b) => (a.start_date < b.start_date ? 1 : -1))
  if (M('POST', 'crops', ANY, 'treatments')) {
    const t = stamp({ crop_id: seg[1], start_date: body.start_date || todayISO(), end_date: body.end_date ?? null, product: body.product, dose: body.dose ?? null, withdrawal_days: Number(body.withdrawal_days || 0), symptom: body.symptom ?? null, note: body.note ?? null })
    d.treatments.push(t)
    save(d)
    return { id: t.id }
  }

  if (M('GET', 'prices')) {
    const code = q.get('species') || 'nile_tilapia'
    const hist = d.prices.filter((p) => p.species_code === code).sort((a, b) => (a.price_date < b.price_date ? 1 : -1))
    const avg = hist.length ? hist.reduce((a, p) => a + p.price_per_kg, 0) / hist.length : null
    return { latest: hist[0] ?? null, avg_30d: avg, history: hist }
  }
  if (M('POST', 'prices')) {
    const p2 = stamp({ species_code: body.species_code || 'nile_tilapia', province: body.province ?? null, price_per_kg: Number(body.price_per_kg), size_note: body.size_note ?? null, source: 'farmer', price_date: body.price_date || todayISO() })
    d.prices.push(p2)
    save(d)
    return { id: p2.id }
  }
  if (M('GET', 'disease-reports')) return []
  if (M('GET', 'announcements')) return []
  if (M('GET', 'benchmark')) return { n_crops: d.crops.filter((c) => c.status === 'active').length, fcr_avg: null, survival_avg: null, health_avg: null }
  if (M('POST', 'sync')) return { results: [] }

  throw new Error('โหมดเก็บในเครื่องยังไม่รองรับรายการนี้ (ต้องใช้เซิร์ฟเวอร์)')
}
