<script lang="ts">
  import { onMount } from 'svelte'
  import { api, readQueue, dropQueued, flushQueue } from '../lib/api'
  import { session, currentFarm, toast, loadSession, ui } from '../lib/ui.svelte'
  import { PROVINCES, thDateTime } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'

  let farm: any = $state(null)
  let f = $state({ name: '', province: '', district: '', lat: '', lng: '', meals_per_day: '2', farm_factor: '1', bag_kg: '20' })
  let busy = $state(false)
  let lineCode: any = $state(null)
  let oldPin = $state('')
  let newPin = $state('')
  let newFarmName = $state('')
  let workerName = $state('')
  let workerPhone = $state('')
  let workerPin = $state('')
  let queue = $state(readQueue())

  onMount(async () => {
    const cf = currentFarm()
    if (!cf) return
    farm = (await api.get(`/farms/${cf.id}`))
    f = { name: farm.name, province: farm.province ?? '', district: farm.district ?? '', lat: farm.lat ?? '', lng: farm.lng ?? '', meals_per_day: String(farm.meals_per_day), farm_factor: String(farm.farm_factor), bag_kg: String(farm.bag_kg) }
  })
  async function saveFarm() {
    busy = true
    try {
      await api.patch(`/farms/${farm.id}`, { name: f.name, province: f.province || null, district: f.district || null, lat: f.lat ? parseFloat(f.lat) : null, lng: f.lng ? parseFloat(f.lng) : null, meals_per_day: parseInt(f.meals_per_day), farm_factor: parseFloat(f.farm_factor), bag_kg: parseFloat(f.bag_kg) })
      toast('บันทึกแล้ว', 'success')
      loadSession()
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      busy = false
    }
  }
  function gps() {
    navigator.geolocation?.getCurrentPosition((p) => { f.lat = p.coords.latitude.toFixed(4); f.lng = p.coords.longitude.toFixed(4) }, () => toast('จับพิกัดไม่ได้', 'error'), { enableHighAccuracy: true, timeout: 10000 })
  }
  async function getLineCode() {
    lineCode = await api.post('/line/link-code')
  }
  async function unlinkLine() {
    await api.post('/line/unlink')
    toast('ยกเลิกการเชื่อม LINE แล้ว')
    loadSession()
  }
  async function changePin() {
    if (!oldPin || !newPin) return toast('กรอก PIN เดิมและ PIN ใหม่ก่อน', 'error')
    try {
      await api.post('/auth/pin', { old_pin: oldPin, new_pin: newPin })
      toast('เปลี่ยน PIN แล้ว', 'success')
      oldPin = newPin = ''
    } catch (e: any) {
      toast(e.message, 'error')
    }
  }
  async function addFarm() {
    if (!newFarmName.trim()) return toast('ตั้งชื่อฟาร์มใหม่ก่อน', 'error')
    try {
      await api.post('/farms', { name: newFarmName })
      toast('เพิ่มฟาร์มแล้ว สลับฟาร์มได้ที่หน้าวันนี้', 'success')
      newFarmName = ''
      loadSession()
    } catch (e: any) {
      toast(e.message, 'error')
    }
  }
  async function addWorker() {
    if (!workerName || !workerPhone || !workerPin) return toast('กรอกชื่อ เบอร์โทร และ PIN ให้ครบ', 'error')
    try {
      await api.post('/users', { name: workerName, phone: workerPhone, pin: workerPin, role: 'worker', farm_id: farm.id })
      toast('เพิ่มคนงานแล้ว ให้เข้าสู่ระบบด้วยเบอร์และ PIN ที่ตั้ง', 'success')
      workerName = workerPhone = workerPin = ''
    } catch (e: any) {
      toast(e.message, 'error')
    }
  }
  async function retry() {
    await flushQueue()
    queue = readQueue()
  }
</script>

<TopBar title="ตั้งค่า" back="/menu" />
<main class="page">
  {#if farm}
    <div class="card">
      <h3>ฟาร์ม</h3>
      <label for="fn">ชื่อฟาร์ม</label><input id="fn" bind:value={f.name} />
      <div class="grid2">
        <div><label for="pv">จังหวัด</label><select id="pv" bind:value={f.province}><option value="">-</option>{#each PROVINCES as p}<option value={p}>{p}</option>{/each}</select></div>
        <div><label for="ds">อำเภอ</label><input id="ds" bind:value={f.district} /></div>
      </div>
      <label>พิกัด <span class="hint">(ใช้ดึงอากาศอัตโนมัติทุกเช้า)</span></label>
      <div class="row"><input bind:value={f.lat} placeholder="ละติจูด" /><input bind:value={f.lng} placeholder="ลองจิจูด" /><button class="btn ghost sm" onclick={gps} style="white-space:nowrap">ตำแหน่งฉัน</button></div>
      <div class="grid3">
        <div><label for="mp">มื้อ/วัน</label><input id="mp" type="number" inputmode="numeric" bind:value={f.meals_per_day} /></div>
        <div><label for="ff">ตัวคูณฟาร์ม <span class="hint">1 = ปกติ</span></label><input id="ff" type="number" inputmode="decimal" step="0.05" bind:value={f.farm_factor} /></div>
        <div><label for="bk">กก./กระสอบ</label><input id="bk" type="number" inputmode="decimal" bind:value={f.bag_kg} /></div>
      </div>
      <button class="btn primary mt" onclick={saveFarm} disabled={busy}>บันทึกการตั้งค่าฟาร์ม</button>
    </div>
  {/if}

  <div class="card mt">
    <h3>เชื่อม LINE รับสรุปทุกเช้า</h3>
    <p class="small muted">เมื่อเชื่อมแล้ว ทุกเช้า 06:00 น. จะได้รับอาหารที่ต้องให้ทุกบ่อทาง LINE และพิมพ์ "บ่อ1 ให้แล้ว 12" เพื่อบันทึกโดยไม่ต้องเปิดแอป</p>
    {#if session.user?.line_linked}
      <div class="alert good mt">เชื่อม LINE แล้ว</div>
      <button class="btn ghost mt" onclick={unlinkLine}>ยกเลิกการเชื่อม</button>
    {:else if lineCode}
      {#if !lineCode.bot_configured}<div class="alert warn mt">ระบบยังไม่ได้ตั้งค่า LINE OA (ผู้ดูแลต้องใส่ token ที่ server) รหัสด้านล่างจะใช้ได้เมื่อตั้งค่าแล้ว</div>{/if}
      <div class="card tint-cyan mt center"><div class="small">1. เพิ่มเพื่อน LINE OA ของทีเด็ดปลาน้ำจืด{#if lineCode.add_friend_url} <a href={lineCode.add_friend_url} target="_blank" rel="noopener">กดที่นี่</a>{/if}</div><div class="small mt">2. พิมพ์ในแชท:</div><div class="big-number" style="font-size:1.6rem">ผูก {lineCode.code}</div></div>
    {:else}
      <button class="btn primary mt" onclick={getLineCode}>ขอรหัสเชื่อม LINE</button>
    {/if}
  </div>

  <div class="card mt">
    <h3>เปลี่ยนรหัส PIN</h3>
    <div class="grid2"><div><label for="op">PIN เดิม</label><input id="op" type="password" inputmode="numeric" bind:value={oldPin} /></div><div><label for="np">PIN ใหม่</label><input id="np" type="password" inputmode="numeric" bind:value={newPin} /></div></div>
    <button class="btn ghost mt" onclick={changePin} >เปลี่ยน PIN</button>
  </div>

  {#if farm && (session.user?.role === 'owner' || session.user?.role === 'admin' || session.user?.role === 'officer')}
    <div class="card mt">
      <h3>เพิ่มคนงาน/ผู้ช่วยให้เข้าบันทึกฟาร์มนี้</h3>
      <div class="grid3"><div><label for="wn">ชื่อ</label><input id="wn" bind:value={workerName} /></div><div><label for="wp">เบอร์โทร</label><input id="wp" type="tel" bind:value={workerPhone} /></div><div><label for="wpin">PIN</label><input id="wpin" inputmode="numeric" bind:value={workerPin} /></div></div>
      <button class="btn ghost mt" onclick={addWorker} >เพิ่มคนงาน</button>
    </div>
    <div class="card mt">
      <h3>เพิ่มฟาร์มอีกแห่ง</h3>
      <div class="row"><input bind:value={newFarmName} placeholder="ชื่อฟาร์มใหม่" /><button class="btn ghost sm" onclick={addFarm} >เพิ่ม</button></div>
    </div>
  {/if}

  <div class="card mt">
    <h3>รายการรอส่ง (ออฟไลน์)</h3>
    {#if !queue.length}<p class="muted small">ไม่มีรายการค้าง</p>{:else}
      {#each queue as q}<div class="list-item"><div class="main"><div class="title">{q.label}</div><div class="sub">{thDateTime(new Date(q.at).toISOString())}</div></div><button class="btn link" style="color:var(--red)" onclick={() => { dropQueued(q.client_id); queue = readQueue() }}>ลบ</button></div>{/each}
      <button class="btn primary mt" onclick={retry} disabled={!ui.online}>ส่งตอนนี้</button>
    {/if}
  </div>
</main>
