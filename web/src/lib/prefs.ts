// ค่าแสดงผลที่ผู้ใช้เลือกเอง: ขนาดตัวอักษร และโหมดคอนทราสต์สูง (จำในเครื่อง)
const FS_KEY = 'teedet.fs'
const HC_KEY = 'teedet.hc'
export const FONT_SIZES = [
  { key: 'normal', label: 'ปกติ', px: 18 },
  { key: 'large', label: 'ใหญ่', px: 20 },
  { key: 'xlarge', label: 'ใหญ่มาก', px: 23 },
]
export function getFontSize(): string {
  return localStorage.getItem(FS_KEY) || 'normal'
}
export function getHighContrast(): boolean {
  return localStorage.getItem(HC_KEY) === '1'
}
export function setFontSize(key: string) {
  localStorage.setItem(FS_KEY, key)
  applyDisplayPrefs()
}
export function setHighContrast(on: boolean) {
  localStorage.setItem(HC_KEY, on ? '1' : '0')
  applyDisplayPrefs()
}
export function applyDisplayPrefs() {
  const fs = FONT_SIZES.find((f) => f.key === getFontSize()) ?? FONT_SIZES[0]
  document.documentElement.style.setProperty('--fs', fs.px + 'px')
  document.body.classList.toggle('hc', getHighContrast())
}
