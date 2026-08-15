<script lang="ts">
  const BASE = import.meta.env.BASE_URL
  import { api, setToken, needsApiSetup, getApiBase, IS_STATIC_HOST } from '../lib/api'
  import { loadSession, toast, go } from '../lib/ui.svelte'
  import { PROVINCES } from '../lib/format'

  let { mode = 'login' }: { mode?: string } = $props()
  let tab = $state(mode === 'register' ? 'register' : 'login')
  let phone = $state('')
  let pin = $state('')
  let name = $state('')
  let farmName = $state('')
  let province = $state('')
  let orgCode = $state('')
  let busy = $state(false)
  let useGps = $state(false)
  let coords: { lat: number; lng: number } | null = $state(null)

  async function doLogin() {
    if (needsApiSetup()) return go('/server')
    busy = true
    try {
      const r = await api.post('/auth/login', { phone, pin, device: navigator.userAgent.slice(0, 80) })
      setToken(r.token)
      await loadSession()
      go('/')
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      busy = false
    }
  }
  async function doRegister() {
    if (needsApiSetup()) return go('/server')
    busy = true
    try {
      const r = await api.post('/auth/register', { phone, pin, name, farm_name: farmName, province: province || null, org_code: orgCode || null, lat: coords?.lat ?? null, lng: coords?.lng ?? null })
      setToken(r.token)
      await loadSession()
      toast('สร้างฟาร์มเรียบร้อย เพิ่มบ่อแรกได้เลย', 'success')
      go('/ponds')
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      busy = false
    }
  }
  function grabGps() {
    if (!navigator.geolocation) return toast('อุปกรณ์นี้ไม่รองรับ GPS', 'error')
    useGps = true
    navigator.geolocation.getCurrentPosition(
      (p) => {
        coords = { lat: +p.coords.latitude.toFixed(4), lng: +p.coords.longitude.toFixed(4) }
        useGps = false
      },
      () => {
        useGps = false
        toast('จับพิกัดไม่ได้ ตั้งค่าภายหลังได้ในหน้าตั้งค่า', 'error')
      },
      { enableHighAccuracy: true, timeout: 10000 },
    )
  }
</script>

<div class="auth-wrap">
  <div class="logo-lockup">
    <div style="background:#fff;border-radius:22px;padding:14px 18px;box-shadow:var(--shadow)"><img src="{BASE}logo-full.png" alt="ทีเด็ดปลาน้ำจืด" style="width:260px;height:auto;display:block" /></div>
  </div>
  <div class="card auth-card">
    <div class="tabs">
      <button class:active={tab === 'login'} onclick={() => (tab = 'login')}>เข้าสู่ระบบ</button>
      <button class:active={tab === 'register'} onclick={() => (tab = 'register')}>สมัครฟาร์มใหม่</button>
    </div>
    {#if tab === 'login'}
      <form onsubmit={(e) => { e.preventDefault(); doLogin() }}>
        <label for="phone">เบอร์โทรศัพท์</label>
        <input id="phone" type="tel" inputmode="numeric" bind:value={phone} placeholder="08x xxx xxxx" autocomplete="tel" required />
        <label for="pin">รหัส PIN <span class="hint">(ตัวเลข 4-8 หลัก)</span></label>
        <input id="pin" type="password" inputmode="numeric" bind:value={pin} placeholder="เช่น 1234" autocomplete="current-password" required />
        <button class="btn primary mt2" type="submit" disabled={busy}>{busy ? 'กำลังเข้า...' : 'เข้าสู่ระบบ'}</button>
      </form>
    {:else}
      <form onsubmit={(e) => { e.preventDefault(); doRegister() }}>
        <label for="fname">ชื่อฟาร์ม</label>
        <input id="fname" bind:value={farmName} placeholder="เช่น ฟาร์มลุงสมชาย" required />
        <label for="name">ชื่อของคุณ</label>
        <input id="name" bind:value={name} placeholder="ชื่อ-นามสกุล หรือชื่อเล่น" required />
        <label for="rphone">เบอร์โทรศัพท์ <span class="hint">(ใช้เข้าสู่ระบบ)</span></label>
        <input id="rphone" type="tel" inputmode="numeric" bind:value={phone} required />
        <label for="rpin">ตั้งรหัส PIN <span class="hint">(ตัวเลข 4-8 หลัก จำง่าย ๆ)</span></label>
        <input id="rpin" type="password" inputmode="numeric" bind:value={pin} required />
        <label for="prov">จังหวัด</label>
        <select id="prov" bind:value={province}>
          <option value="">-- เลือกจังหวัด --</option>
          {#each PROVINCES as p}<option value={p}>{p}</option>{/each}
        </select>
        <label>ตำแหน่งฟาร์ม <span class="hint">(ใช้ดึงพยากรณ์อากาศให้อัตโนมัติ)</span></label>
        <button type="button" class="btn ghost" onclick={grabGps} disabled={useGps}>
          {coords ? `ได้พิกัดแล้ว ${coords.lat}, ${coords.lng}` : useGps ? 'กำลังจับพิกัด...' : 'ใช้ตำแหน่งปัจจุบันของฉัน'}
        </button>
        <label for="org">รหัสหน่วยส่งเสริม <span class="hint">(ถ้ามี เจ้าหน้าที่จะให้มา)</span></label>
        <input id="org" bind:value={orgCode} placeholder="ไม่บังคับ" />
        <button class="btn primary mt2" type="submit" disabled={busy}>{busy ? 'กำลังสร้าง...' : 'สร้างฟาร์มและเริ่มใช้งาน'}</button>
      </form>
    {/if}
    <div class="divider"></div>
    {#if IS_STATIC_HOST}
      <div class="alert {getApiBase() ? 'good' : 'warn'} mt" style="font-size:0.92rem">
        {#if getApiBase()}
          เชื่อมกับเซิร์ฟเวอร์: {getApiBase().replace(/^https?:\/\//, '')} <a href="#/server">เปลี่ยน</a>
        {:else}
          หน้านี้เป็นเว็บฟรี ยังไม่ได้ต่อกับระบบบันทึกของฟาร์ม — <a href="#/server">ตั้งที่อยู่เซิร์ฟเวอร์ก่อน</a> ถึงจะเข้าสู่ระบบได้
        {/if}
      </div>
    {/if}
    <p class="center small muted">ยังไม่พร้อมสมัคร? ลองเครื่องคำนวณก่อนได้</p>
    <div class="grid2 mt">
      <a class="btn ghost" href="#/calc">คำนวณอาหารปลา</a>
      <a class="btn ghost" href="#/simulate">จำลองรุ่นเลี้ยง</a>
    </div>
  </div>
</div>
