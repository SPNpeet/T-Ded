<script lang="ts">
  import { onMount } from 'svelte'
  import { cachedGet } from '../lib/api'
  import { currentFarm, session, toast } from '../lib/ui.svelte'
  import { n, n1, baht, bahtShort, pct } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'

  let items: any[] = $state([])
  let bench: any = $state(null)
  let loading = $state(true)
  async function load() {
    const farm = currentFarm()
    if (!farm) return
    loading = true
    try {
      const crops = (await cachedGet(`/farms/${farm.id}/crops?status=active`)).data as any[]
      const out = []
      for (const c of crops) {
        const s = (await cachedGet(`/crops/${c.id}/today?weather=0`)).data
        out.push(s)
      }
      items = out
      try {
        bench = (await cachedGet('/benchmark')).data
      } catch {}
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      loading = false
    }
  }
  onMount(load)
  $effect(() => {
    session.farmId
    load()
  })
  const sum = (k: (s: any) => number) => items.reduce((a, s) => a + (k(s) || 0), 0)
  const totalCost = $derived(sum((s) => s.totals.cost_total))
  const totalValue = $derived(sum((s) => s.performance.stock_value ?? 0))
  const totalRevenue = $derived(sum((s) => s.totals.revenue))
  const projProfit = $derived(sum((s) => s.projection?.profit ?? 0))
</script>

<TopBar title="การเงินฟาร์ม" sub={currentFarm()?.name ?? ''} back="/" />
<main class="page">
  {#if loading && !items.length}
    <div class="skeleton" style="min-height:160px"></div>
  {:else if !items.length}
    <div class="card center"><p class="muted">ยังไม่มีรุ่นที่เลี้ยงอยู่</p><a class="btn primary mt" href="#/ponds">จัดการบ่อ</a></div>
  {:else}
    <section class="hero">
      <div class="muted small">ภาพรวมทุกบ่อที่เลี้ยงอยู่</div>
      <div class="kpi mt" style="--paper:rgba(255,255,255,0.1)">
        <div class="k" style="background:rgba(255,255,255,0.12)"><div class="lbl" style="color:rgba(255,255,255,0.85)">มูลค่าปลาในบ่อ (บาท)</div><div class="val" style="color:#fff">{bahtShort(totalValue)}</div></div>
        <div class="k" style="background:rgba(255,255,255,0.12)"><div class="lbl" style="color:rgba(255,255,255,0.85)">ต้นทุนสะสม (บาท)</div><div class="val" style="color:#fff">{bahtShort(totalCost)}</div></div>
        <div class="k" style="background:rgba(255,255,255,0.12)"><div class="lbl" style="color:rgba(255,255,255,0.85)">กำไรคาดเมื่อจับ (บาท)</div><div class="val" style="color:#fff">{bahtShort(projProfit)}</div></div>
        <div class="k" style="background:rgba(255,255,255,0.12)"><div class="lbl" style="color:rgba(255,255,255,0.85)">ขายแล้ว (บาท)</div><div class="val" style="color:#fff">{bahtShort(totalRevenue)}</div></div>
      </div>
    </section>
    {#if bench?.n_crops > 1}
      <div class="card tint-cyan mt small"><b>เทียบกับกลุ่ม ({bench.n_crops} บ่อ):</b> FCR เฉลี่ย {bench.fcr_avg ?? '-'} · อัตรารอดเฉลี่ย {bench.survival_avg ?? '-'}% · คะแนนสุขภาพเฉลี่ย {bench.health_avg ?? '-'}</div>
    {/if}
    {#each items as s}
      {@const p = s.performance}
      <a class="card" href="#/pond/{s.crop.id}/money" style="display:block;text-decoration:none;color:inherit;margin-top:12px">
        <div class="row" style="justify-content:space-between"><h3>{s.crop.pond_name}</h3><span class="pill neutral">วันที่ {s.day}</span></div>
        <div class="kpi mt">
          <div class="k"><div class="lbl">FCR</div><div class="val">{p.fcr ?? 'ยังไม่มี'}</div><div class="tiny muted">{p.fcr_grade_th ?? ''}{bench?.fcr_avg && p.fcr ? (p.fcr <= bench.fcr_avg ? ' · ดีกว่ากลุ่ม' : ' · แย่กว่ากลุ่ม') : ''}</div></div>
          <div class="k"><div class="lbl">รอด</div><div class="val">{pct(p.survival_pct, 1)}</div></div>
          <div class="k"><div class="lbl">ต้นทุน/กก. (บาท)</div><div class="val">{p.cost_per_kg ? n1(p.cost_per_kg) : 'ยังไม่มีข้อมูล'}</div></div>
          <div class="k"><div class="lbl">มูลค่าในบ่อ (บาท)</div><div class="val">{p.stock_value != null ? bahtShort(p.stock_value) : 'ใส่ราคาขายก่อน'}</div></div>
          {#if s.projection}<div class="k"><div class="lbl">กำไรคาด (บาท)</div><div class="val" style="color:{s.projection.profit >= 0 ? 'var(--green)' : 'var(--red)'}">{s.market_price_per_kg || s.projection.revenue > 0 ? bahtShort(s.projection.profit) : 'ใส่ราคาขายก่อน'}</div><div class="tiny muted">อีก {s.projection.days_remaining} วัน</div></div>{/if}
        </div>
      </a>
    {/each}
  {/if}
</main>
