// โหลด aqua-engine (WASM) ตัวเดียวกับ server เพื่อคำนวณออฟไลน์ได้
import init, * as wasm from '../engine-pkg/aqua_engine.js'
import wasmUrl from '../engine-pkg/aqua_engine_bg.wasm?url'

let ready: Promise<typeof wasm> | null = null

export function engine(): Promise<typeof wasm> {
  if (!ready) {
    ready = init({ module_or_path: wasmUrl }).then(() => wasm)
  }
  return ready
}

let speciesCache: any[] | null = null
export async function speciesList(): Promise<any[]> {
  if (speciesCache) return speciesCache
  const e = await engine()
  speciesCache = e.species_defaults() as any[]
  return speciesCache
}
export async function speciesByCode(code: string) {
  const list = await speciesList()
  return list.find((s) => s.code === code) ?? list[0]
}

export async function recommendLocal(input: { species: any; avg_weight_g: number; count: number; env?: any; meals_per_day?: number; farm_factor?: number }) {
  const e = await engine()
  return e.feed_recommend({ rules: [], env: null, meals_per_day: null, farm_factor: null, ...input })
}

export async function simulateLocal(input: any) {
  const e = await engine()
  return e.forecast_simulate({ target_weight_g: null, target_days: null, growth_scale: null, avg_feed_factor: null, bag_kg: null, ...input })
}

export async function growthLocal(species: any, stock_weight_g: number, day: number, actual_g: number, prev?: { day: number; w: number }, target?: number) {
  const e = await engine()
  return e.growth_compare(species, stock_weight_g, day, actual_g, prev?.day, prev?.w, target)
}

export async function waterLocal(sample: any, species: any) {
  const e = await engine()
  return e.water_assess({ do_mg_l: null, ph: null, temp_c: null, nh3: null, no2: null, secchi_cm: null, ...sample }, species)
}

export async function healthLocal(input: any, species: any) {
  const e = await engine()
  return e.pond_health({ water: {}, mortality_7d_pct: null, feeding_response: null, growth_status: null, days_since_last_log: null, previous_score: null, ...input }, species)
}
