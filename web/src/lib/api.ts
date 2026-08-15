// ตัวกลางเรียก API + คิวออฟไลน์ (เก็บใน localStorage) + cache หน้าอ่าน
import { toast } from './ui.svelte'

const TOKEN_KEY = 'teedet.token'
const QUEUE_KEY = 'teedet.queue'
const CACHE_PREFIX = 'teedet.cache:'
const API_KEY = 'teedet.api'

/** true เมื่อเปิดจากที่เก็บไฟล์นิ่ง (GitHub Pages) ซึ่งไม่มีเซิร์ฟเวอร์ในตัว */
export const IS_STATIC_HOST = /github\.io$|netlify\.app$|pages\.dev$/.test(location.hostname)

function normalizeBase(v: string): string {
  let s = v.trim().replace(/\/+$/, '')
  if (!s) return ''
  if (!/^https?:\/\//i.test(s)) s = 'https://' + s
  return s.replace(/\/api$/, '')
}

/** ที่อยู่เซิร์ฟเวอร์: ที่ผู้ใช้ตั้งเอง > ค่าตอน build > โดเมนเดียวกับหน้าเว็บ */
export function getApiBase(): string {
  const saved = localStorage.getItem(API_KEY)
  if (saved) return saved
  const built = (import.meta.env.VITE_API_BASE as string | undefined) ?? ''
  return built ? normalizeBase(built) : ''
}
export function setApiBase(v: string | null) {
  if (v) localStorage.setItem(API_KEY, normalizeBase(v))
  else localStorage.removeItem(API_KEY)
}
/** ต้องให้ผู้ใช้ตั้งที่อยู่เซิร์ฟเวอร์ก่อนหรือยัง */
export function needsApiSetup(): boolean {
  return IS_STATIC_HOST && !getApiBase()
}
/** ทดสอบว่าที่อยู่นี้เป็นเซิร์ฟเวอร์ทีเด็ดปลาน้ำจืดจริง */
export async function testApiBase(v: string): Promise<boolean> {
  const base = normalizeBase(v)
  if (!base) return false
  try {
    const ctl = new AbortController()
    const timer = setTimeout(() => ctl.abort(), 8000)
    const res = await fetch(base + '/api/health', { signal: ctl.signal })
    clearTimeout(timer)
    return res.ok && (await res.text()).trim() === 'ok'
  } catch {
    return false
  }
}

export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY)
}
export function setToken(t: string | null) {
  if (t) localStorage.setItem(TOKEN_KEY, t)
  else localStorage.removeItem(TOKEN_KEY)
}

export class ApiError extends Error {
  status: number
  constructor(status: number, message: string) {
    super(message)
    this.status = status
  }
}

async function request<T = any>(method: string, path: string, body?: unknown, opts: { raw?: boolean } = {}): Promise<T> {
  const headers: Record<string, string> = {}
  if (body !== undefined) headers['Content-Type'] = 'application/json'
  const t = getToken()
  if (t) headers['Authorization'] = `Bearer ${t}`
  if (needsApiSetup()) {
    throw new ApiError(0, 'ยังไม่ได้ตั้งที่อยู่เซิร์ฟเวอร์ของฟาร์ม — ไปที่ "ตั้งค่าเซิร์ฟเวอร์" เพื่อใส่ที่อยู่ก่อน')
  }
  const res = await fetch(getApiBase() + '/api' + path, { method, headers, body: body === undefined ? undefined : JSON.stringify(body) })
  if (opts.raw) return res as unknown as T
  const text = await res.text()
  let data: any = null
  try {
    data = text ? JSON.parse(text) : null
  } catch {
    data = { error: text }
  }
  if (!res.ok) {
    if (res.status === 401) {
      setToken(null)
      window.dispatchEvent(new CustomEvent('teedet:unauthorized'))
    }
    throw new ApiError(res.status, data?.error || `HTTP ${res.status}`)
  }
  return data as T
}

export const api = {
  get: <T = any>(path: string) => request<T>('GET', path),
  post: <T = any>(path: string, body?: unknown) => request<T>('POST', path, body ?? {}),
  patch: <T = any>(path: string, body?: unknown) => request<T>('PATCH', path, body ?? {}),
  put: <T = any>(path: string, body?: unknown) => request<T>('PUT', path, body ?? {}),
  del: <T = any>(path: string) => request<T>('DELETE', path),
}

// ---- cache สำหรับหน้าอ่าน: ใช้ค่าล่าสุดเมื่อออฟไลน์ ----
export async function cachedGet<T = any>(path: string): Promise<{ data: T; fromCache: boolean }> {
  try {
    const data = await api.get<T>(path)
    try {
      localStorage.setItem(CACHE_PREFIX + path, JSON.stringify({ at: Date.now(), data }))
    } catch {
      /* storage full: ignore */
    }
    return { data, fromCache: false }
  } catch (e) {
    const raw = localStorage.getItem(CACHE_PREFIX + path)
    if (raw) {
      const parsed = JSON.parse(raw)
      return { data: parsed.data as T, fromCache: true }
    }
    throw e
  }
}

// ---- คิวออฟไลน์ ----
export type QueuedOp = { op: string; target_id: string; body: any; client_id: string; label: string; at: number }

export function readQueue(): QueuedOp[] {
  try {
    return JSON.parse(localStorage.getItem(QUEUE_KEY) || '[]')
  } catch {
    return []
  }
}
function writeQueue(q: QueuedOp[]) {
  localStorage.setItem(QUEUE_KEY, JSON.stringify(q))
  window.dispatchEvent(new CustomEvent('teedet:queue', { detail: q.length }))
}
export function queueLength() {
  return readQueue().length
}
export function newClientId() {
  return (crypto?.randomUUID?.() ?? `${Date.now()}-${Math.random().toString(16).slice(2)}`)
}

/// บันทึกข้อมูล: ลองส่งทันที ถ้าออฟไลน์/ล้มเหลวจากเครือข่าย ให้เข้าคิวและบอกผู้ใช้
export async function submit(op: string, target_id: string, body: any, label: string): Promise<{ queued: boolean; id?: string }> {
  const client_id = body.client_id ?? newClientId()
  const payload = { ...body, client_id }
  const path = pathFor(op, target_id)
  if (navigator.onLine) {
    try {
      const r = await api.post(path, payload)
      return { queued: false, id: r?.id }
    } catch (e: any) {
      if (e instanceof ApiError && e.status >= 400 && e.status < 500) throw e
      // network / server error: เข้าคิว
    }
  }
  const q = readQueue()
  q.push({ op, target_id, body: payload, client_id, label, at: Date.now() })
  writeQueue(q)
  toast(`ออฟไลน์อยู่ บันทึก "${label}" ไว้ในเครื่องแล้ว จะส่งให้อัตโนมัติเมื่อมีสัญญาณ`, 'info', 4000)
  return { queued: true }
}

function pathFor(op: string, id: string): string {
  switch (op) {
    case 'log':
      return `/crops/${id}/logs`
    case 'weighing':
      return `/crops/${id}/weighings`
    case 'water':
      return `/ponds/${id}/water`
    case 'stock':
      return `/farms/${id}/stock`
    case 'expense':
      return `/crops/${id}/expenses`
    case 'harvest':
      return `/crops/${id}/harvests`
    case 'treatment':
      return `/crops/${id}/treatments`
    default:
      throw new Error('unknown op ' + op)
  }
}

let flushing = false
export async function flushQueue(): Promise<number> {
  if (flushing || !navigator.onLine || !getToken()) return 0
  const q = readQueue()
  if (!q.length) return 0
  flushing = true
  try {
    const r = await api.post('/sync', q.map(({ op, target_id, body, client_id }) => ({ op, target_id, body, client_id })))
    const results: any[] = r.results ?? []
    const failed = new Set(results.filter((x) => x.status === 'error').map((x) => x.client_id))
    const remain = q.filter((x) => failed.has(x.client_id))
    writeQueue(remain)
    const sent = q.length - remain.length
    if (sent > 0) toast(`ส่งข้อมูลที่ค้างไว้ ${sent} รายการเรียบร้อย`, 'success')
    if (remain.length) toast(`มี ${remain.length} รายการส่งไม่สำเร็จ ตรวจในหน้าตั้งค่า`, 'error', 5000)
    return sent
  } catch {
    return 0
  } finally {
    flushing = false
  }
}

export function dropQueued(client_id: string) {
  writeQueue(readQueue().filter((x) => x.client_id !== client_id))
}

window.addEventListener('online', () => {
  flushQueue()
})
