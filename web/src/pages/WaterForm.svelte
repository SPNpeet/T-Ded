<script lang="ts">
  import { submit } from '../lib/api'
  import { go, toast, currentFarm } from '../lib/ui.svelte'
  import TopBar from '../lib/TopBar.svelte'
  import { waterLocal, speciesByCode } from '../lib/engine'

  let { pondId }: { pondId: string } = $props()
  let v = $state({ do_mg_l: '', ph: '', temp_c: '', nh3: '', no2: '', secchi_cm: '', color: '', note: '' })
  let busy = $state(false)
  let assess: any = $state(null)
  const speciesCode = 'nile_tilapia'
  const num = (x: string) => (x === '' ? null : parseFloat(x))

  $effect(() => {
    const sample = { do_mg_l: num(v.do_mg_l), ph: num(v.ph), temp_c: num(v.temp_c), nh3: num(v.nh3), no2: num(v.no2), secchi_cm: num(v.secchi_cm) }
    if (Object.values(sample).every((x) => x === null)) {
      assess = null
      return
    }
    speciesByCode(speciesCode).then((sp) => waterLocal(sample, sp).then((a) => (assess = a)).catch(() => {}))
  })
  async function save() {
    busy = true
    try {
      const r = await submit('water', pondId, { measured_at: new Date().toISOString(), do_mg_l: num(v.do_mg_l), ph: num(v.ph), temp_c: num(v.temp_c), nh3: num(v.nh3), no2: num(v.no2), secchi_cm: num(v.secchi_cm), color: v.color || null, note: v.note || null }, 'ค่าน้ำ')
      if (!r.queued) toast('บันทึกค่าน้ำแล้ว', 'success')
      history.back()
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      busy = false
    }
  }
</script>

<TopBar title="จดค่าน้ำ" sub={currentFarm()?.name ?? ''} />
<main class="page">
  <p class="small muted">กรอกเฉพาะค่าที่วัดได้ ค่าที่สำคัญที่สุดคือออกซิเจนตอนเช้ามืด (05:00-06:00)</p>
  <div class="grid2">
    <div><label for="do">ออกซิเจน DO <span class="hint">มก./ล.</span></label><input id="do" type="number" inputmode="decimal" step="0.1" bind:value={v.do_mg_l} placeholder="เช่น 4.5" /></div>
    <div><label for="ph">pH</label><input id="ph" type="number" inputmode="decimal" step="0.1" bind:value={v.ph} placeholder="เช่น 7.8" /></div>
    <div><label for="t">อุณหภูมิน้ำ <span class="hint">องศา</span></label><input id="t" type="number" inputmode="decimal" step="0.5" bind:value={v.temp_c} placeholder="เช่น 29" /></div>
    <div><label for="nh3">แอมโมเนีย <span class="hint">มก./ล.</span></label><input id="nh3" type="number" inputmode="decimal" step="0.05" bind:value={v.nh3} placeholder="เช่น 0.25" /></div>
    <div><label for="no2">ไนไตรท์ <span class="hint">มก./ล.</span></label><input id="no2" type="number" inputmode="decimal" step="0.05" bind:value={v.no2} /></div>
    <div><label for="sec">ความใส (Secchi) <span class="hint">ซม.</span></label><input id="sec" type="number" inputmode="numeric" bind:value={v.secchi_cm} placeholder="เช่น 35" /></div>
  </div>
  <label for="col">สีน้ำ</label>
  <select id="col" bind:value={v.color}>
    <option value="">-- เลือก --</option>
    <option>เขียวอ่อน (ดี)</option><option>เขียวเข้ม</option><option>น้ำตาล/ขุ่นดิน</option><option>ใสมาก</option><option>เขียวเข้มมีฟอง/กลิ่น</option>
  </select>
  {#if assess}
    <div class="card mt {assess.overall === 'danger' ? 'tint-red' : assess.overall === 'warn' ? 'tint-amber' : 'tint-green'}">
      <b>{assess.overall_th}</b>
      {#each assess.items as it}
        <div class="reason"><span class="pill {it.level === 'good' ? 'good' : it.level === 'warn' ? 'warn' : 'danger'}">{it.label_th}</span><span>{it.message_th}{it.advice_th ? ` — ${it.advice_th}` : ''}</span></div>
      {/each}
    </div>
  {/if}
  <label for="note">หมายเหตุ</label>
  <input id="note" bind:value={v.note} />
  <button class="btn success mt2" onclick={save} disabled={busy}>บันทึกค่าน้ำ</button>
</main>
