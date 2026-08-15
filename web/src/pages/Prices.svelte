<script lang="ts">
  import { onMount } from 'svelte'
  import { api, cachedGet } from '../lib/api'
  import { currentFarm, toast, session } from '../lib/ui.svelte'
  import { thDate, n1, todayISO, PROVINCES } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'
  import LineChart from '../lib/LineChart.svelte'
  import { speciesList } from '../lib/engine'

  let species: any[] = $state([])
  let code = $state('nile_tilapia')
  let province = $state(currentFarm()?.province ?? '')
  let data: any = $state(null)
  let show = $state(false)
  let price = $state('')
  let size = $state('')
  let busy = $state(false)
  async function load() {
    try {
      data = (await cachedGet(`/prices?species=${code}${province ? '&province=' + encodeURIComponent(province) : ''}`)).data
    } catch (e: any) {
      toast(e.message, 'error')
    }
  }
  onMount(async () => {
    species = await speciesList()
    load()
  })
  $effect(() => {
    code
    province
    load()
  })
  async function save() {
    if (!price || !(parseFloat(price) > 0)) return toast('กรอกราคาบาท/กก. ก่อนครับ', 'error')
    busy = true
    try {
      await api.post('/prices', { species_code: code, province: province || null, price_per_kg: parseFloat(price), size_note: size || null, price_date: todayISO() })
      toast('ขอบคุณ ราคานี้ช่วยเพื่อนเกษตรกรวางแผนขาย', 'success')
      show = false
      price = ''
      load()
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      busy = false
    }
  }
  const series = $derived.by(() => {
    if (!data?.history?.length) return []
    const pts = [...data.history].reverse()
    const base = new Date(pts[0].price_date).getTime()
    return [{ name: 'บาท/กก.', color: '#0e8ea7', points: pts.map((p: any) => ({ x: Math.round((new Date(p.price_date).getTime() - base) / 86400000), y: p.price_per_kg })) }]
  })
</script>

<TopBar title="ราคาปลาในพื้นที่" sub="รายงานโดยเกษตรกรและเจ้าหน้าที่" back="/" />
<main class="page">
  <div class="grid2">
    <div><label for="sp">ชนิดปลา</label><select id="sp" bind:value={code}>{#each species as s}<option value={s.code}>{s.name_th}</option>{/each}</select></div>
    <div><label for="pv">จังหวัด</label><select id="pv" bind:value={province}><option value="">ทุกจังหวัด</option>{#each PROVINCES as p}<option value={p}>{p}</option>{/each}</select></div>
  </div>
  {#if data}
    <section class="hero mt">
      <div class="muted small">ราคาล่าสุด {data.latest ? thDate(data.latest.price_date) : ''}</div>
      <div class="big-number">{data.latest ? n1(data.latest.price_per_kg) : '-'}<small>บาท/กก.</small></div>
      <div class="small mt">เฉลี่ย 30 วัน {data.avg_30d ? n1(data.avg_30d) : '-'} บาท/กก.{data.latest?.size_note ? ` · ${data.latest.size_note}` : ''}{data.latest?.source === 'harvest' ? ' · จากการขายจริง' : ''}</div>
    </section>
    {#if series.length && series[0].points.length > 1}<div class="card mt"><LineChart {series} height={170} xLabel={(x) => `+${Math.round(x)} วัน`} yMin={0} /></div>{/if}
    {#if session.user}
      <button class="btn primary mt" onclick={() => (show = !show)}>{show ? 'ยกเลิก' : 'รายงานราคาที่คุณเห็นวันนี้'}</button>
      {#if show}
        <div class="card mt">
          <label for="pr">ราคา (บาท/กก.)</label><input id="pr" type="number" inputmode="decimal" bind:value={price} />
          <label for="sz">ขนาด/หมายเหตุ</label><input id="sz" bind:value={size} placeholder="เช่น ไซส์ 3-4 ตัว/กก. หน้าฟาร์ม" />
          <button class="btn success mt" onclick={save} disabled={busy}>ส่งราคา</button>
        </div>
      {/if}
    {/if}
    <div class="card mt">
      <h3>ประวัติราคา</h3>
      {#if !data.history.length}<p class="muted">ยังไม่มีข้อมูลราคาในพื้นที่นี้ เป็นคนแรกที่รายงานได้เลย</p>{/if}
      <div class="list">
        {#each data.history.slice(0, 30) as p}
          <div class="list-item"><div class="main"><div class="title">{n1(p.price_per_kg)} บาท/กก.</div><div class="sub">{thDate(p.price_date)}{p.province ? ' · ' + p.province : ''}{p.size_note ? ' · ' + p.size_note : ''} · {p.source === 'harvest' ? 'ขายจริง' : p.source === 'officer' ? 'เจ้าหน้าที่' : 'เกษตรกร'}</div></div></div>
        {/each}
      </div>
    </div>
  {/if}
</main>
