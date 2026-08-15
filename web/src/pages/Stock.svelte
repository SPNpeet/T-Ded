<script lang="ts">
  import { onMount } from 'svelte'
  import { cachedGet, submit } from '../lib/api'
  import { currentFarm, toast } from '../lib/ui.svelte'
  import { thDate, n, n1, baht, todayISO } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'

  let data: any = $state(null)
  let show = $state(false)
  let f = $state({ move_date: todayISO(), bags: '', bag_kg: '', brand: '', pellet_mm: '3', protein_pct: '', form: 'floating', price_total: '', kind: 'in', note: '' })
  let busy = $state(false)
  let products: any[] = $state([])
  let productId = $state('')
  const farm = () => currentFarm()
  function pickProduct(id: string) {
    productId = id
    const p = products.find((x) => x.id === id)
    if (!p) return
    f.brand = p.name_th
    if (p.protein_pct != null) f.protein_pct = String(p.protein_pct)
    if (p.pellet_mm != null) f.pellet_mm = String(p.pellet_mm)
    if (p.form) f.form = p.form
    if (p.bag_kg) f.bag_kg = String(p.bag_kg)
    if (p.price_ref && f.bags) f.price_total = String(Math.round(p.price_ref * parseFloat(f.bags)))
  }

  async function load() {
    if (!farm()) return
    try {
      data = (await cachedGet(`/farms/${farm()!.id}/stock`)).data
      if (!f.bag_kg) f.bag_kg = String(data.bag_kg)
      try { products = (await cachedGet('/feed-products')).data } catch {}
    } catch (e: any) {
      toast(e.message, 'error')
    }
  }
  onMount(load)
  async function save() {
    busy = true
    try {
      const body: any = { move_date: f.move_date, kind: f.kind, product_id: productId || null, brand: f.brand || null, pellet_mm: f.pellet_mm ? parseFloat(f.pellet_mm) : null, protein_pct: f.protein_pct ? parseFloat(f.protein_pct) : null, form: f.form || null, note: f.note || null }
      if (f.bags) {
        body.bags = parseFloat(f.bags)
        body.bag_kg = parseFloat(f.bag_kg || String(data?.bag_kg ?? 20))
      }
      if (f.price_total) body.price_total = parseFloat(f.price_total)
      const r = await submit('stock', farm()!.id, body, 'รับอาหารเข้าสต๊อก')
      if (!r.queued) toast('บันทึกสต๊อกแล้ว', 'success')
      show = false
      f.bags = ''
      f.price_total = ''
      load()
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      busy = false
    }
  }
</script>

<TopBar title="สต๊อกอาหาร" sub={farm()?.name ?? ''} back="/" />
<main class="page">
  {#if data}
    <section class="hero">
      <div class="muted small">อาหารคงเหลือ</div>
      <div class="big-number">{n1(data.balance_bags)}<small>กระสอบ ({n(data.balance_kg)} กก.)</small></div>
      <div class="row wrap small mt" style="gap:14px">
        <span>ใช้เฉลี่ย {n1(data.used_per_day_7d)} กก./วัน</span>
        {#if data.days_left != null}<span class="pill {data.low ? 'warn' : 'good'}">พอใช้อีกประมาณ {n(data.days_left)} วัน</span>{/if}
        {#if data.avg_price_per_kg}<span>ราคาเฉลี่ย {n1(data.avg_price_per_kg)} บาท/กก.</span>{/if}
        {#if data.current_feed?.protein_pct}<span class="pill info">ล่าสุด: {data.current_feed.brand ?? 'อาหาร'} โปรตีน {n(data.current_feed.protein_pct)}%</span>{/if}
      </div>
    </section>
    <button class="btn primary mt" onclick={() => (show = !show)}>{show ? 'ยกเลิก' : 'รับอาหารเข้า / ปรับยอด'}</button>
    {#if show}
      <div class="card mt">
        <div class="segment"><button class:active={f.kind === 'in'} onclick={() => (f.kind = 'in')}>ซื้อเข้า</button><button class:active={f.kind === 'adjust'} onclick={() => (f.kind = 'adjust')}>ปรับยอด (+/-)</button></div>
        <label for="prod">เลือกจากยี่ห้อที่ขายในไทย <span class="hint">(ระบบใส่โปรตีน/เม็ด/ราคาให้)</span></label>
        <select id="prod" value={productId} onchange={(e) => pickProduct((e.target as HTMLSelectElement).value)}>
          <option value="">-- พิมพ์เองด้านล่าง หรือเลือกยี่ห้อ --</option>
          {#each products as p}<option value={p.id}>{p.name_th} · โปรตีน {p.protein_pct ?? "?"}% · เม็ด {p.pellet_mm ?? "?"} มม.{p.verified ? "" : " (ตรวจฉลาก)"}</option>{/each}
        </select>
        <label>วันที่</label><input type="date" bind:value={f.move_date} />
        <div class="grid2">
          <div><label for="bg">จำนวนกระสอบ</label><input id="bg" type="number" inputmode="decimal" bind:value={f.bags} placeholder={f.kind === 'adjust' ? 'ติดลบได้' : 'เช่น 20'} /></div>
          <div><label for="bk">กก./กระสอบ</label><input id="bk" type="number" inputmode="decimal" bind:value={f.bag_kg} /></div>
          <div><label for="br">ยี่ห้อ/สูตร</label><input id="br" bind:value={f.brand} placeholder="เช่น โปรตีน 30%" /></div>
          <div><label for="pm">ขนาดเม็ด (มม.)</label><input id="pm" type="number" inputmode="decimal" bind:value={f.pellet_mm} /></div>
          <div><label for="pp">โปรตีน (%) <span class="hint">ดูข้างถุง</span></label><input id="pp" type="number" inputmode="decimal" bind:value={f.protein_pct} placeholder="เช่น 30" /></div>
          <div><label for="fm">ชนิดเม็ด</label><select id="fm" bind:value={f.form}><option value="floating">ลอยน้ำ</option><option value="sinking">จมน้ำ</option><option value="crumble">เม็ดเล็ก/ป่น</option></select></div>
        </div>
        {#if f.kind === 'in'}<label for="pt">ราคารวม (บาท) <span class="hint">ใช้คิดต้นทุนอาหาร</span></label><input id="pt" type="number" inputmode="decimal" bind:value={f.price_total} />{/if}
        <label for="nt">หมายเหตุ</label><input id="nt" bind:value={f.note} />
        <button class="btn success mt" onclick={save} disabled={busy}>บันทึก</button>
      </div>
    {/if}
    <div class="card mt">
      <h3>รายการล่าสุด</h3>
      <div class="list">
        {#each data.moves as m}
          <div class="list-item">
            <div class="main"><div class="title">{m.kind === 'in' ? 'รับเข้า' : m.kind === 'out' ? 'ให้อาหาร' : 'ปรับยอด'} {n1(m.kg)} กก.{m.bags ? ` (${n1(m.bags)} กระสอบ)` : ''}</div><div class="sub">{thDate(m.move_date)}{m.brand ? ' · ' + m.brand : ''}{m.protein_pct ? ` · โปรตีน ${m.protein_pct}%` : ''}{m.note ? ' · ' + m.note : ''}</div></div>
            {#if m.price_total}<b>{baht(m.price_total)}</b>{/if}
          </div>
        {/each}
        {#if !data.moves.length}<p class="muted">ยังไม่มีรายการ เริ่มจากรับอาหารเข้าสต๊อก ระบบจะตัดยอดให้อัตโนมัติทุกครั้งที่บันทึกให้อาหาร</p>{/if}
      </div>
    </div>
  {:else}
    <div class="skeleton" style="min-height:160px"></div>
  {/if}
</main>
