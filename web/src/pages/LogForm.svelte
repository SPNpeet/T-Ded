<script lang="ts">
  import { onMount } from 'svelte'
  import { cachedGet, submit } from '../lib/api'
  import { go, toast, ui } from '../lib/ui.svelte'
  import { todayISO, thDate, n2, FEEDING_RESPONSE, portionText } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'
  import { recommendLocal, speciesByCode } from '../lib/engine'

  let { cropId }: { cropId: string } = $props()
  const qs = new URLSearchParams(ui.route.split('?')[1] || '')
  let date = $state(qs.get('date') || todayISO())
  let s: any = $state(null)
  let fed = $state('')
  let mortality = $state('0')
  let response = $state(0)
  let note = $state('')
  let doVal = $state('')
  let tempVal = $state('')
  let busy = $state(false)
  let localRec: any = $state(null)

  onMount(async () => {
    try {
      const r = await cachedGet(`/crops/${cropId}/today?date=${date}&forecast=0`)
      s = r.data
      const rec = s.recommendation
      if (s.today_log) {
        fed = s.today_log.fed_kg != null ? String(s.today_log.fed_kg) : ''
        mortality = String(s.today_log.mortality ?? 0)
        response = s.today_log.feeding_response ?? 0
        note = s.today_log.note ?? ''
      }
      if (!fed) fed = String(rec.final_kg)
    } catch (e: any) {
      toast(e.message, 'error')
    }
  })

  // คำนวณใหม่ในเครื่องเมื่อผู้ใช้เปลี่ยน "การกิน" หรือใส่ค่า DO เพื่อให้เห็นผลทันทีแม้ออฟไลน์
  $effect(() => {
    if (!s) return
    const env = { ...(s.env_used ?? {}), stress: ['normal', 'slow_eating', 'gasping'][response] }
    if (doVal) env.do_morning = parseFloat(doVal)
    speciesByCode(s.species.code).then((sp) =>
      recommendLocal({ species: sp, avg_weight_g: s.avg_weight_g, count: s.alive_count, env, meals_per_day: s.recommendation.meals_per_day, farm_factor: s.crop.farm_factor })
        .then((r) => (localRec = r))
        .catch(() => {}),
    )
  })
  function useSuggested() {
    if (localRec) fed = String(localRec.final_kg)
  }

  async function save() {
    busy = true
    try {
      const body: any = {
        log_date: date,
        fed_kg: fed === '' ? null : parseFloat(fed),
        recommended_kg: localRec?.final_kg ?? s?.recommendation?.final_kg,
        factor: localRec?.factor ?? s?.recommendation?.factor,
        mortality: parseInt(mortality || '0'),
        feeding_response: response,
        weather: s?.weather ?? null,
        reasons: (localRec ?? s?.recommendation)?.reasons ?? null,
        note: note || null,
      }
      if (doVal || tempVal) body.water = { measured_at: date === todayISO() ? new Date().toISOString() : `${date}T07:00:00+07:00`, do_mg_l: doVal ? parseFloat(doVal) : null, temp_c: tempVal ? parseFloat(tempVal) : null }
      const r = await submit('log', cropId, body, `บันทึก ${s?.crop?.pond_name ?? 'บ่อ'} ${thDate(date)}`)
      if (!r.queued) toast('บันทึกเรียบร้อย', 'success')
      go(`/pond/${cropId}`)
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      busy = false
    }
  }
</script>

<TopBar title={s ? `บันทึก ${s.crop.pond_name}` : 'บันทึกประจำวัน'} sub={thDate(date)} back="/pond/{cropId}" />
<main class="page">
  {#if !s}
    <div class="skeleton" style="min-height:200px"></div>
  {:else}
    {@const rec = localRec ?? s.recommendation}
    <div class="card tint-cyan">
      <div class="small muted">แนะนำวันนี้</div>
      <div class="big-number">{n2(rec.final_kg)}<small>กก. · มื้อละ {n2(rec.per_meal_kg)}</small></div>
      <div class="small">มื้อละ{portionText(rec.per_meal_kg).replace('ประมาณ', ' ประมาณ')}</div>
      <div class="small">{rec.headline_th}</div>
      {#if rec.reasons?.length}<details class="mt"><summary>ดูเหตุผล</summary>{#each rec.reasons as r}<div class="reason"><b>× {r.factor.toFixed(2)}</b><span>{r.text_th}</span></div>{/each}</details>{/if}
    </div>

    <label>วันที่</label>
    <input type="date" bind:value={date} max={todayISO()} />

    <label>ปลากินเป็นอย่างไร <span class="hint">(สังเกตจากบ่อ ระบบจะปรับอาหารให้)</span></label>
    <div class="segment">
      {#each FEEDING_RESPONSE as f}
        <button type="button" class:active={response === f.v} class={f.v === 1 ? 'warn' : f.v === 2 ? 'danger' : ''} onclick={() => (response = f.v)}>{f.label}</button>
      {/each}
    </div>

    <label for="fed">ให้อาหารจริง (กก.) <span class="hint">รวมทุกมื้อ</span></label>
    <div class="row">
      <input id="fed" type="number" inputmode="decimal" step="0.1" bind:value={fed} style="font-size:1.4rem;font-weight:700" />
      <button type="button" class="btn ghost sm" onclick={useSuggested} style="white-space:nowrap">ใช้ค่าแนะนำ</button>
    </div>

    <label for="dead">ปลาตายวันนี้ (ตัว)</label>
    <input id="dead" type="number" inputmode="numeric" bind:value={mortality} />

    <details class="mt">
      <summary>จดค่าน้ำเช้านี้ด้วย (ถ้ามีเครื่องวัด)</summary>
      <div class="grid2">
        <div><label for="do">ออกซิเจน (มก./ล.)</label><input id="do" type="number" inputmode="decimal" step="0.1" bind:value={doVal} placeholder="เช่น 4.5" /></div>
        <div><label for="tw">อุณหภูมิน้ำ (องศา)</label><input id="tw" type="number" inputmode="decimal" step="0.5" bind:value={tempVal} placeholder="เช่น 29" /></div>
      </div>
    </details>

    <label for="note">หมายเหตุ</label>
    <textarea id="note" bind:value={note} placeholder="เช่น น้ำเขียวเข้ม, ฝนตกตอนบ่าย"></textarea>

    <button class="btn success mt2" onclick={save} disabled={busy}>{busy ? 'กำลังบันทึก...' : 'บันทึก'}</button>
  {/if}
</main>
