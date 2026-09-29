// รูปถ่ายบ่อ: ถ่ายจากกล้อง จับพิกัด GPS ระหว่างเปิดกล้อง ประทับวันเวลา/ชื่อบ่อ/พิกัดลงบนรูป
// เก็บลงเครื่องก่อนเสมอ (IndexedDB) แล้วค่อยส่งขึ้นเซิร์ฟเวอร์ สัญญาณหลุดที่บ่อก็ไม่หาย
import { api, ApiError, getApiBase, getToken, newClientId } from './api'
import { isLocalMode, db as localDb, save as localSave } from './local'
import { brand } from './brand'

export type PondPhoto = {
  id: string
  pond_id: string
  taken_at: string
  lat: number | null
  lng: number | null
  accuracy_m: number | null
  kind: string | null
  caption: string | null
  width: number
  height: number
  /** ยังไม่ได้ส่งขึ้นเซิร์ฟเวอร์ */
  pending?: boolean
}
type Stored = PondPhoto & { client_id: string; status: 'local' | 'pending'; image: Blob; thumb: Blob }

export type Fix = { lat: number; lng: number; accuracy_m: number; at: number }

const DB_NAME = 'teedet-photos'
const STORE = 'photos'
/** รูปที่ไฟล์เก่ากว่านี้ถือว่าไม่ได้ถ่ายตอนนี้ จะไม่ใส่พิกัดปัจจุบันให้ */
export const FRESH_MS = 10 * 60 * 1000
const MAX_EDGE = 1600
const THUMB_EDGE = 360

// ---------- IndexedDB ----------
let dbp: Promise<IDBDatabase> | null = null
function idb(): Promise<IDBDatabase> {
  if (!dbp)
    dbp = new Promise((res, rej) => {
      const r = indexedDB.open(DB_NAME, 1)
      r.onupgradeneeded = () => {
        const s = r.result.createObjectStore(STORE, { keyPath: 'id' })
        s.createIndex('pond_id', 'pond_id')
      }
      r.onsuccess = () => res(r.result)
      r.onerror = () => {
        dbp = null
        rej(r.error)
      }
    })
  return dbp
}
async function tx<T>(mode: IDBTransactionMode, fn: (s: IDBObjectStore) => IDBRequest<T>): Promise<T> {
  const d = await idb()
  return new Promise((res, rej) => {
    const t = d.transaction(STORE, mode)
    const r = fn(t.objectStore(STORE))
    t.oncomplete = () => res(r.result)
    t.onerror = () => rej(t.error)
    t.onabort = () => rej(t.error ?? new Error('บันทึกรูปลงเครื่องไม่สำเร็จ (พื้นที่เต็ม?)'))
  })
}
const idbAll = () => tx<Stored[]>('readonly', (s) => s.getAll())
const idbByPond = (pondId: string) => tx<Stored[]>('readonly', (s) => s.index('pond_id').getAll(pondId))
const idbGet = (id: string) => tx<Stored | undefined>('readonly', (s) => s.get(id))
const idbPut = (v: Stored) => tx('readwrite', (s) => s.put(v))
const idbDel = (id: string) => tx('readwrite', (s) => s.delete(id))

const meta = (s: Stored): PondPhoto => {
  const { image: _i, thumb: _t, client_id: _c, status, ...m } = s
  return { ...m, pending: status === 'pending' }
}

// ---------- GPS ----------
/** เริ่มจับพิกัดตั้งแต่กดปุ่มถ่าย ระหว่างที่ผู้ใช้เล็งกล้อง GPS ก็แม่นขึ้นเรื่อย ๆ */
export class Locator {
  best: Fix | null = null
  error: string | null = null
  private watch: number | null = null
  private waiters: (() => void)[] = []

  start() {
    if (!('geolocation' in navigator)) {
      this.error = 'อุปกรณ์นี้ไม่มี GPS'
      return this
    }
    this.watch = navigator.geolocation.watchPosition(
      (p) => {
        const f = { lat: p.coords.latitude, lng: p.coords.longitude, accuracy_m: Math.round(p.coords.accuracy), at: p.timestamp || Date.now() }
        if (!this.best || f.accuracy_m <= this.best.accuracy_m) this.best = f
        this.error = null
        if (this.best.accuracy_m <= 25) this.flush()
      },
      (e) => {
        this.error = e.code === e.PERMISSION_DENIED ? 'ไม่ได้อนุญาตให้ใช้ตำแหน่ง' : 'ยังจับสัญญาณ GPS ไม่ได้'
        if (e.code === e.PERMISSION_DENIED) this.flush()
      },
      { enableHighAccuracy: true, maximumAge: 0, timeout: 30000 },
    )
    return this
  }
  private flush() {
    const w = this.waiters
    this.waiters = []
    w.forEach((f) => f())
  }
  /** รอพิกัดที่แม่นพอ ไม่เกิน ms แล้วคืนค่าที่ดีที่สุดที่มี */
  async wait(ms: number): Promise<Fix | null> {
    if ((this.best && this.best.accuracy_m <= 25) || this.error === 'ไม่ได้อนุญาตให้ใช้ตำแหน่ง' || this.watch == null) return this.best
    await new Promise<void>((res) => {
      const t = setTimeout(res, ms)
      this.waiters.push(() => {
        clearTimeout(t)
        res()
      })
    })
    return this.best
  }
  stop() {
    if (this.watch != null) navigator.geolocation.clearWatch(this.watch)
    this.watch = null
    this.flush()
  }
}

// ---------- ประทับข้อมูลบนรูป ----------
const thDate = (d: Date) => d.toLocaleDateString('th-TH', { day: 'numeric', month: 'short', year: 'numeric' })
const thTime = (d: Date) => d.toLocaleTimeString('th-TH', { hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false })

async function loadBitmap(file: Blob): Promise<{ img: CanvasImageSource; w: number; h: number; close: () => void }> {
  if ('createImageBitmap' in window) {
    try {
      const b = await createImageBitmap(file, { imageOrientation: 'from-image' } as any)
      return { img: b, w: b.width, h: b.height, close: () => b.close() }
    } catch {
      /* ใช้ <img> แทน */
    }
  }
  const url = URL.createObjectURL(file)
  const el = new Image()
  el.decoding = 'async'
  el.src = url
  await el.decode()
  return { img: el, w: el.naturalWidth, h: el.naturalHeight, close: () => URL.revokeObjectURL(url) }
}

function toBlob(c: HTMLCanvasElement, q: number): Promise<Blob> {
  return new Promise((res, rej) => c.toBlob((b) => (b ? res(b) : rej(new Error('แปลงรูปไม่สำเร็จ'))), 'image/jpeg', q))
}

export type Stamp = { pondName: string; farmName: string; takenAt: Date; fix: Fix | null; fresh: boolean }

export function stampLines(s: Stamp): string[] {
  const loc = s.fix ? `พิกัด ${s.fix.lat.toFixed(6)}, ${s.fix.lng.toFixed(6)}  (คลาดเคลื่อน ±${s.fix.accuracy_m} ม.)` : s.fresh ? 'ไม่มีพิกัด GPS' : 'รูปจากเครื่อง ไม่ได้ถ่ายตอนนี้ จึงไม่ใส่พิกัด'
  return [`${thDate(s.takenAt)}  ${thTime(s.takenAt)} น.`, [s.pondName, s.farmName].filter(Boolean).join(' · '), loc]
}

/** ย่อรูป + ประทับแถบข้อมูลด้านล่าง คืนรูปเต็มและรูปย่อ */
export async function stampPhoto(file: Blob, s: Stamp): Promise<{ image: Blob; thumb: Blob; width: number; height: number }> {
  try {
    await (document as any).fonts?.load?.('700 32px Sarabun')
  } catch {
    /* ไม่มีฟอนต์ก็ใช้ฟอนต์ระบบ */
  }
  const bmp = await loadBitmap(file)
  const scale = Math.min(1, MAX_EDGE / Math.max(bmp.w, bmp.h))
  const W = Math.round(bmp.w * scale)
  const H = Math.round(bmp.h * scale)
  const c = document.createElement('canvas')
  c.width = W
  c.height = H
  const g = c.getContext('2d')!
  g.drawImage(bmp.img, 0, 0, W, H)
  bmp.close()

  const lines = stampLines(s)
  const base = Math.max(16, Math.round(Math.min(W, H) * 0.04))
  const pad = Math.round(base * 0.7)
  const lh = Math.round(base * 1.35)
  const band = pad * 2 + lh * lines.length
  g.fillStyle = 'rgba(0,0,0,0.58)'
  g.fillRect(0, H - band, W, band)
  g.textBaseline = 'top'
  g.fillStyle = '#FFFFFF'
  const family = '"Sarabun", "Noto Sans Thai", "Leelawadee UI", system-ui, sans-serif'
  const fit = (text: string, weight: number, size: number, maxW: number) => {
    let sz = size
    g.font = `${weight} ${sz}px ${family}`
    while (g.measureText(text).width > maxW && sz > 10) {
      sz -= 1
      g.font = `${weight} ${sz}px ${family}`
    }
  }
  const mark = brand.shortName
  fit(mark, 700, Math.round(base * 0.8), W * 0.3)
  const markW = g.measureText(mark).width
  g.globalAlpha = 0.85
  g.fillText(mark, W - pad - markW, H - band + pad)
  g.globalAlpha = 1
  lines.forEach((t, i) => {
    fit(t, i === 0 ? 800 : 600, i === 0 ? Math.round(base * 1.15) : base, W - pad * 3 - (i === 0 ? markW : 0))
    g.fillText(t, pad, H - band + pad + i * lh)
  })

  let q = 0.82
  let image = await toBlob(c, q)
  while (image.size > 2.4 * 1024 * 1024 && q > 0.45) {
    q -= 0.12
    image = await toBlob(c, q)
  }
  const ts = Math.min(1, THUMB_EDGE / Math.max(W, H))
  const tc = document.createElement('canvas')
  tc.width = Math.round(W * ts)
  tc.height = Math.round(H * ts)
  tc.getContext('2d')!.drawImage(c, 0, 0, tc.width, tc.height)
  const thumb = await toBlob(tc, 0.72)
  return { image, thumb, width: W, height: H }
}

// ---------- เก็บ/ส่ง ----------
const toDataUrl = (b: Blob) =>
  new Promise<string>((res, rej) => {
    const r = new FileReader()
    r.onload = () => res(String(r.result))
    r.onerror = () => rej(r.error)
    r.readAsDataURL(b)
  })

async function upload(s: Stored): Promise<{ id: string; pond_located?: boolean }> {
  return api.post(`/ponds/${s.pond_id}/photos`, {
    client_id: s.client_id,
    image: await toDataUrl(s.image),
    thumb: await toDataUrl(s.thumb),
    taken_at: s.taken_at,
    lat: s.lat,
    lng: s.lng,
    accuracy_m: s.accuracy_m,
    kind: s.kind,
    caption: s.caption,
    width: s.width,
    height: s.height,
  })
}

/** ใช้พิกัดจากรูปเป็นตำแหน่งบ่อ (โหมดเก็บในเครื่อง ทำแบบเดียวกับเซิร์ฟเวอร์) */
function localAfterAdd(p: PondPhoto): boolean {
  const d = localDb()
  const pond = d.ponds.find((x) => x.id === p.pond_id)
  if (!pond) return false
  let located = false
  if (!pond.cover_photo_id) pond.cover_photo_id = p.id
  if (pond.lat == null && p.lat != null && p.lng != null && (p.accuracy_m == null || p.accuracy_m <= 100)) {
    pond.lat = p.lat
    pond.lng = p.lng
    pond.location_accuracy_m = p.accuracy_m
    located = true
  }
  localSave(d)
  return located
}

export type AddResult = { id: string; queued: boolean; pond_located: boolean }

export async function addPhoto(pondId: string, shot: { image: Blob; thumb: Blob; width: number; height: number }, m: { taken_at: string; fix: Fix | null; kind?: string | null; caption?: string | null }): Promise<AddResult> {
  const local = isLocalMode()
  const rec: Stored = {
    id: newClientId(),
    client_id: newClientId(),
    pond_id: pondId,
    taken_at: m.taken_at,
    lat: m.fix?.lat ?? null,
    lng: m.fix?.lng ?? null,
    accuracy_m: m.fix?.accuracy_m ?? null,
    kind: m.kind ?? null,
    caption: m.caption ?? null,
    width: shot.width,
    height: shot.height,
    status: local ? 'local' : 'pending',
    image: shot.image,
    thumb: shot.thumb,
  }
  await idbPut(rec)
  if (local) return { id: rec.id, queued: false, pond_located: localAfterAdd(rec) }
  try {
    const r = await upload(rec)
    await idbDel(rec.id)
    return { id: r.id, queued: false, pond_located: !!r.pond_located }
  } catch (e) {
    if (e instanceof ApiError && e.status >= 400 && e.status < 500) {
      await idbDel(rec.id)
      throw e
    }
    return { id: rec.id, queued: true, pond_located: false }
  }
}

export async function listPhotos(pondId: string): Promise<PondPhoto[]> {
  const mine = (await idbByPond(pondId).catch(() => [] as Stored[])).map(meta)
  if (isLocalMode()) return mine.sort((a, b) => (a.taken_at < b.taken_at ? 1 : -1))
  // จำรายการล่าสุดไว้ ออฟไลน์ที่บ่อก็ยังเห็นรูปเดิม
  const cacheKey = `teedet.photos:${pondId}`
  let remote: PondPhoto[] = []
  try {
    remote = await api.get(`/ponds/${pondId}/photos`)
    try {
      localStorage.setItem(cacheKey, JSON.stringify(remote))
    } catch {
      /* ที่เก็บเต็ม ไม่เป็นไร */
    }
  } catch (e) {
    const cached = localStorage.getItem(cacheKey)
    if (cached) remote = JSON.parse(cached)
    else if (!mine.length) throw e
  }
  return [...mine, ...remote].sort((a, b) => (a.taken_at < b.taken_at ? 1 : -1))
}

const urls = new Map<string, string>()
/** ที่อยู่รูปสำหรับ <img>: รูปในเครื่องอ่านจาก IndexedDB, รูปบนเซิร์ฟเวอร์ต้องแนบ token จึงโหลดเป็น blob */
export async function photoUrl(id: string, thumb = false): Promise<string | null> {
  const key = id + (thumb ? ':t' : '')
  const hit = urls.get(key)
  if (hit) return hit
  const s = await idbGet(id).catch(() => undefined)
  let blob: Blob | null = s ? (thumb ? s.thumb : s.image) : null
  if (!blob && !isLocalMode()) {
    const t = getToken()
    const r = await fetch(`${getApiBase()}/api/photos/${id}${thumb ? '?thumb=1' : ''}`, { headers: t ? { Authorization: `Bearer ${t}` } : {} }).catch(() => null)
    if (r?.ok) blob = await r.blob()
  }
  if (!blob) return null
  const u = URL.createObjectURL(blob)
  urls.set(key, u)
  return u
}

export async function photoBlob(id: string): Promise<Blob | null> {
  const u = await photoUrl(id)
  return u ? (await fetch(u)).blob() : null
}

export async function removePhoto(p: PondPhoto) {
  const s = await idbGet(p.id).catch(() => undefined)
  if (s) await idbDel(p.id)
  if (!s && !isLocalMode()) await api.del(`/photos/${p.id}`)
  if (isLocalMode()) {
    const d = localDb()
    const pond = d.ponds.find((x) => x.id === p.pond_id)
    if (pond && pond.cover_photo_id === p.id) {
      const rest = (await idbByPond(p.pond_id)).sort((a, b) => (a.taken_at < b.taken_at ? 1 : -1))
      pond.cover_photo_id = rest[0]?.id ?? null
      localSave(d)
    }
  }
  for (const k of [p.id, p.id + ':t']) {
    const u = urls.get(k)
    if (u) URL.revokeObjectURL(u)
    urls.delete(k)
  }
}

export async function setCover(p: PondPhoto) {
  if (isLocalMode() || p.pending) {
    if (!isLocalMode()) throw new Error('รอให้รูปส่งขึ้นระบบก่อน แล้วค่อยตั้งเป็นรูปปก')
    const d = localDb()
    const pond = d.ponds.find((x) => x.id === p.pond_id)
    if (pond) pond.cover_photo_id = p.id
    localSave(d)
    return
  }
  await api.patch(`/photos/${p.id}`, { cover: true })
}

export async function useLocationOf(p: PondPhoto) {
  if (p.lat == null || p.lng == null) throw new Error('รูปนี้ไม่มีพิกัด')
  await api.patch(`/ponds/${p.pond_id}`, { lat: p.lat, lng: p.lng, location_accuracy_m: p.accuracy_m })
}

let flushing = false
/** ส่งรูปที่ค้างในเครื่องขึ้นเซิร์ฟเวอร์ */
export async function flushPhotos(): Promise<number> {
  if (flushing || isLocalMode() || !navigator.onLine || !getToken()) return 0
  flushing = true
  let sent = 0
  try {
    const all = (await idbAll()).filter((s) => s.status === 'pending')
    for (const s of all) {
      try {
        await upload(s)
        await idbDel(s.id)
        sent++
      } catch (e) {
        if (e instanceof ApiError && e.status >= 400 && e.status < 500 && e.status !== 401) await idbDel(s.id)
        else break
      }
    }
  } catch {
    /* IndexedDB ใช้ไม่ได้ ไม่มีอะไรค้าง */
  } finally {
    flushing = false
  }
  if (sent) window.dispatchEvent(new CustomEvent('teedet:photos', { detail: sent }))
  return sent
}
export async function pendingCount(): Promise<number> {
  try {
    return (await idbAll()).filter((s) => s.status === 'pending').length
  } catch {
    return 0
  }
}

window.addEventListener('online', () => {
  flushPhotos()
})

// ---------- สำรอง/นำเข้า (โหมดเก็บในเครื่อง) ----------
export async function exportPhotos(): Promise<any[]> {
  const all = (await idbAll().catch(() => [] as Stored[])).filter((s) => s.status === 'local')
  const out = []
  for (const s of all) out.push({ ...meta(s), client_id: s.client_id, image: await toDataUrl(s.image), thumb: await toDataUrl(s.thumb) })
  return out
}
export async function importPhotos(list: any[]) {
  const old = await idbAll().catch(() => [] as Stored[])
  for (const s of old) if (s.status === 'local') await idbDel(s.id)
  for (const p of list ?? []) {
    if (!p?.id || !p?.image) continue
    const image = await (await fetch(p.image)).blob()
    const thumb = p.thumb ? await (await fetch(p.thumb)).blob() : image
    const { pending: _p, image: _i, thumb: _t, ...m } = p
    await idbPut({ ...m, client_id: p.client_id ?? newClientId(), status: 'local', image, thumb })
  }
}
