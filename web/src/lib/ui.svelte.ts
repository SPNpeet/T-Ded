// สถานะ UI ร่วม: toast, ออนไลน์/ออฟไลน์, ผู้ใช้, ฟาร์มที่เลือก, เส้นทาง
import { api, getToken, setToken, flushQueue, cachedGet } from './api'

type Toast = { id: number; text: string; kind: 'info' | 'success' | 'error' }
let toastId = 0
export const ui = $state({
  toasts: [] as Toast[],
  online: navigator.onLine,
  queue: 0,
  route: location.hash.replace(/^#/, '') || '/',
})

export function toast(text: string, kind: Toast['kind'] = 'info', ms = 2600) {
  const id = ++toastId
  ui.toasts.push({ id, text, kind })
  setTimeout(() => {
    ui.toasts = ui.toasts.filter((t) => t.id !== id)
  }, ms)
}

window.addEventListener('online', () => (ui.online = true))
window.addEventListener('offline', () => (ui.online = false))
window.addEventListener('teedet:queue', (e: any) => (ui.queue = e.detail))
window.addEventListener('hashchange', () => (ui.route = location.hash.replace(/^#/, '') || '/'))

export function go(path: string) {
  location.hash = path
}

// ---- ผู้ใช้และฟาร์ม ----
export type User = { id: string; name: string; phone: string; role: string; org_id: string; org_name: string; line_linked: number; farms: any[] }
const FARM_KEY = 'teedet.farm'

export const session = $state({
  user: null as User | null,
  loading: true,
  farmId: localStorage.getItem(FARM_KEY) as string | null,
})

export const currentFarm = () => session.user?.farms?.find((f) => f.id === session.farmId) ?? session.user?.farms?.[0] ?? null
export const isStaff = () => session.user?.role === 'officer' || session.user?.role === 'admin'

export async function loadSession() {
  session.loading = true
  if (!getToken()) {
    session.user = null
    session.loading = false
    return
  }
  try {
    const { data } = await cachedGet<User>('/me')
    session.user = data
    if (!session.farmId || !data.farms?.some((f) => f.id === session.farmId)) {
      session.farmId = data.farms?.[0]?.id ?? null
      if (session.farmId) localStorage.setItem(FARM_KEY, session.farmId)
    }
    flushQueue()
  } catch {
    session.user = null
  } finally {
    session.loading = false
  }
}

export function selectFarm(id: string) {
  session.farmId = id
  localStorage.setItem(FARM_KEY, id)
}

export async function logout() {
  try {
    await api.post('/auth/logout')
  } catch {
    /* ignore */
  }
  setToken(null)
  session.user = null
  go('/')
}

window.addEventListener('teedet:unauthorized', () => {
  session.user = null
})
