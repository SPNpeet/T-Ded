import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'
import { applyDisplayPrefs } from './lib/prefs'
import { applyBrandTheme } from './lib/brand'

applyDisplayPrefs()

// มีเวอร์ชันใหม่: โหลดหน้าใหม่หนึ่งครั้งเพื่อไม่ให้ผู้ใช้ค้างอยู่กับแอปเวอร์ชันเก่า
if ('serviceWorker' in navigator) {
  let reloaded = false
  navigator.serviceWorker.addEventListener('controllerchange', () => {
    if (reloaded) return
    reloaded = true
    location.reload()
  })
}

applyBrandTheme()

const app = mount(App, {
  target: document.getElementById('app')!,
})

export default app
