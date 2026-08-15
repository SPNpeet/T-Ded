<script lang="ts">
  import { onMount } from 'svelte'
  import { api, cachedGet } from '../lib/api'
  import { currentFarm, toast, session } from '../lib/ui.svelte'
  import { thDate, todayISO, PROVINCES } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'

  let province = $state(currentFarm()?.province ?? '')
  let list: any[] = $state([])
  let show = $state(false)
  let symptom = $state('')
  let severity = $state('medium')
  let note = $state('')
  let busy = $state(false)
  const SYMPTOMS = ['ตัวแดง/จุดเลือดออก', 'เหงือกซีด/เน่า', 'ตาโปน/ท้องบวม', 'ว่ายเอียง/หมุน', 'แผลตามตัว/ครีบกร่อน', 'ลอยหัวตายเช้ามืด', 'ปรสิต/เห็บปลา', 'ตายไม่ทราบสาเหตุ']
  async function load() {
    try {
      list = (await cachedGet(`/disease-reports${province ? '?province=' + encodeURIComponent(province) : ''}`)).data
    } catch (e: any) {
      toast(e.message, 'error')
    }
  }
  onMount(load)
  $effect(() => {
    province
    load()
  })
  async function save() {
    if (!symptom) return toast('เลือกอาการที่พบก่อนครับ', 'error')
    busy = true
    try {
      await api.post('/disease-reports', { farm_id: currentFarm()?.id ?? null, province: currentFarm()?.province ?? province ?? null, symptom, severity, note: note || null, report_date: todayISO(), species_code: 'nile_tilapia' })
      toast('แจ้งแล้ว ขอบคุณที่ช่วยเตือนเพื่อนบ้าน', 'success')
      show = false
      load()
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      busy = false
    }
  }
  const sevPill = (s: string) => (s === 'high' ? 'danger' : s === 'low' ? 'info' : 'warn')
  const sevText = (s: string) => (s === 'high' ? 'รุนแรง' : s === 'low' ? 'เล็กน้อย' : 'ปานกลาง')
</script>

<TopBar title="โรคปลาในพื้นที่" sub="รายงาน 30 วันล่าสุด ไม่ระบุชื่อฟาร์ม" back="/" />
<main class="page">
  <label for="pv">จังหวัด</label>
  <select id="pv" bind:value={province}><option value="">ทุกจังหวัด</option>{#each PROVINCES as p}<option value={p}>{p}</option>{/each}</select>
  {#if list.length}
    <div class="alert {list.some((r) => r.severity === 'high') ? 'danger' : 'warn'} mt">พบรายงาน {list.length} ครั้งใน 30 วัน{province ? ` ที่ ${province}` : ''} เฝ้าระวังคุณภาพน้ำ งดนำน้ำ/ปลาจากแหล่งไม่ทราบที่มาเข้าบ่อ</div>
  {:else}
    <div class="alert good mt">ยังไม่มีรายงานโรคในพื้นที่ช่วง 30 วัน</div>
  {/if}
  {#if session.user}
    <button class="btn primary mt" onclick={() => (show = !show)}>{show ? 'ยกเลิก' : 'แจ้งพบอาการผิดปกติในบ่อของฉัน'}</button>
    {#if show}
      <div class="card mt">
        <label for="sy">อาการ</label>
        <select id="sy" bind:value={symptom}><option value="">-- เลือก --</option>{#each SYMPTOMS as s}<option>{s}</option>{/each}</select>
        <label>ความรุนแรง</label>
        <div class="segment"><button class:active={severity === 'low'} onclick={() => (severity = 'low')}>เล็กน้อย</button><button class:active={severity === 'medium'} class="warn" onclick={() => (severity = 'medium')}>ปานกลาง</button><button class:active={severity === 'high'} class="danger" onclick={() => (severity = 'high')}>รุนแรง</button></div>
        <label for="nt">รายละเอียด</label><input id="nt" bind:value={note} placeholder="เช่น ตายวันละ 20-30 ตัว 3 วันแล้ว" />
        <button class="btn success mt" onclick={save} disabled={busy}>ส่งรายงาน</button>
        <p class="tiny muted mt">ระบบแสดงเฉพาะอำเภอ/จังหวัดและพิกัดหยาบ (ประมาณ 10 กม.) ไม่แสดงชื่อฟาร์ม</p>
      </div>
    {/if}
  {/if}
  <div class="card mt">
    <div class="list">
      {#each list as r}
        <div class="list-item"><div class="main"><div class="title">{r.symptom}</div><div class="sub">{thDate(r.report_date)} · {r.district ? r.district + ' ' : ''}{r.province ?? ''}</div></div><span class="pill {sevPill(r.severity)}">{sevText(r.severity)}</span></div>
      {/each}
    </div>
  </div>
  <div class="card mt">
    <h3>ป้องกันเบื้องต้นเมื่อมีโรคระบาดใกล้เคียง</h3>
    <div class="reason"><span>ตรวจออกซิเจนเช้ามืดและแอมโมเนียทุกวัน ลดอาหาร 20-30% ชั่วคราว</span></div>
    <div class="reason"><span>ไม่ใช้อุปกรณ์ร่วมกับบ่ออื่น ฆ่าเชื้อสวิง/ถัง ด้วยด่างทับทิมหรือคลอรีนเจือจาง</span></div>
    <div class="reason"><span>เก็บปลาตายออกทันที ฝังกลบโรยปูนขาว ห้ามทิ้งลงแหล่งน้ำ</span></div>
    <div class="reason"><span>ปรึกษาเจ้าหน้าที่ประมงอำเภอก่อนใช้ยา และบันทึกยาที่ใช้ในแอปเพื่อคุมระยะหยุดยา</span></div>
  </div>
</main>
