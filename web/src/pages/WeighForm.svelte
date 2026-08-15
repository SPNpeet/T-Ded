<script lang="ts">
  import { onMount } from 'svelte'
  import { cachedGet, submit } from '../lib/api'
  import { go, toast } from '../lib/ui.svelte'
  import { todayISO, thDate, n, daysBetween } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'
  import { growthLocal, speciesByCode } from '../lib/engine'

  let { cropId }: { cropId: string } = $props()
  let s: any = $state(null)
  let date = $state(todayISO())
  let mode = $state<'avg' | 'total'>('total')
  let totalKg = $state('')
  let count = $state('')
  let avg = $state('')
  let note = $state('')
  let busy = $state(false)
  let preview: any = $state(null)

  onMount(async () => {
    try {
      s = (await cachedGet(`/crops/${cropId}/today?forecast=0&weather=0`)).data
    } catch (e: any) {
      toast(e.message, 'error')
    }
  })
  const avgValue = $derived(mode === 'avg' ? parseFloat(avg) : (parseFloat(totalKg) * 1000) / Math.max(1, parseInt(count || '0')))
  $effect(() => {
    if (!s || !isFinite(avgValue) || avgValue <= 0) {
      preview = null
      return
    }
    const day = daysBetween(s.crop.stocked_at, date)
    speciesByCode(s.species.code).then((sp) => growthLocal(sp, s.crop.stock_weight_g, day, avgValue, { day: daysBetween(s.crop.stocked_at, s.last_weighed.date), w: s.last_weighed.avg_weight_g }, s.crop.target_weight_g ?? sp.market_weight_g).then((g) => (preview = g)).catch(() => {}))
  })
  async function save() {
    if (!isFinite(avgValue) || avgValue <= 0) return toast('กรอกน้ำหนักให้ถูกต้อง', 'error')
    busy = true
    try {
      const r = await submit('weighing', cropId, { weigh_date: date, avg_weight_g: Math.round(avgValue * 10) / 10, sample_count: mode === 'total' ? parseInt(count) : null, method: 'sample', note: note || null }, 'ชั่งน้ำหนัก')
      if (!r.queued) toast('บันทึกการชั่งเรียบร้อย อาหารวันนี้จะปรับตามน้ำหนักใหม่', 'success')
      go(`/pond/${cropId}/growth`)
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      busy = false
    }
  }
</script>

<TopBar title="สุ่มชั่งน้ำหนัก" sub={s ? s.crop.pond_name : ''} back="/pond/{cropId}/growth" />
<main class="page">
  {#if s}
    <div class="card tint-cyan small">ล่าสุดชั่งเมื่อ {thDate(s.last_weighed.date)} ได้ {n(s.last_weighed.avg_weight_g)} ก./ตัว · วิธีที่ดี: สุ่มปลา 30-50 ตัว ชั่งน้ำหนักรวม แล้วกรอกด้านล่าง</div>
    <label>วันที่ชั่ง</label>
    <input type="date" bind:value={date} max={todayISO()} />
    <label>วิธีกรอก</label>
    <div class="segment">
      <button type="button" class:active={mode === 'total'} onclick={() => (mode = 'total')}>น้ำหนักรวม + จำนวนตัว</button>
      <button type="button" class:active={mode === 'avg'} onclick={() => (mode = 'avg')}>น้ำหนักเฉลี่ยต่อตัว</button>
    </div>
    {#if mode === 'total'}
      <div class="grid2">
        <div><label for="tk">น้ำหนักรวม (กก.)</label><input id="tk" type="number" inputmode="decimal" step="0.01" bind:value={totalKg} placeholder="เช่น 12.5" /></div>
        <div><label for="ct">จำนวนตัวที่ชั่ง</label><input id="ct" type="number" inputmode="numeric" bind:value={count} placeholder="เช่น 30" /></div>
      </div>
    {:else}
      <label for="av">น้ำหนักเฉลี่ย (กรัม/ตัว)</label>
      <input id="av" type="number" inputmode="decimal" bind:value={avg} placeholder="เช่น 350" />
    {/if}
    {#if isFinite(avgValue) && avgValue > 0}
      <div class="card mt">
        <div class="big-number">{n(avgValue)}<small>กรัม/ตัว</small></div>
        {#if preview}
          <div class="mt"><span class="pill {preview.status === 'on_track' || preview.status === 'ahead' ? 'good' : preview.status === 'behind' ? 'warn' : 'danger'}">{preview.status_th}</span> <span class="small muted">มาตรฐานวันที่ {preview.day} ควรหนัก {n(preview.expected_g)} ก. ({preview.deviation_pct > 0 ? '+' : ''}{preview.deviation_pct}%)</span></div>
          {#if preview.actual_adg_recent != null}<div class="small mt">โตวันละ {preview.actual_adg_recent} ก. (มาตรฐาน {preview.standard_adg} ก.){preview.days_to_target != null ? ` · ถึงขนาดจับอีกประมาณ ${preview.days_to_target} วัน` : ''}</div>{/if}
          {#each preview.advice_th as a}<div class="reason small"><span>{a}</span></div>{/each}
        {/if}
      </div>
    {/if}
    <label for="nt">หมายเหตุ</label>
    <input id="nt" bind:value={note} placeholder="เช่น ปลาขนาดไม่สม่ำเสมอ" />
    <button class="btn success mt2" onclick={save} disabled={busy}>{busy ? 'กำลังบันทึก...' : 'บันทึกการชั่ง'}</button>
  {:else}
    <div class="skeleton"></div>
  {/if}
</main>
