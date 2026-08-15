<script lang="ts">
  import { onMount } from 'svelte'
  import { cachedGet } from '../lib/api'
  import { toast } from '../lib/ui.svelte'
  import { thDate, n, n1, baht, pct } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'
  import ScoreRing from '../lib/ScoreRing.svelte'

  let { cropId }: { cropId: string } = $props()
  let s: any = $state(null)
  onMount(async () => {
    try {
      s = (await cachedGet(`/crops/${cropId}/today`)).data
    } catch (e: any) {
      toast(e.message, 'error')
    }
  })
  async function share() {
    const text = `${s.crop.farm_name} · ${s.crop.pond_name} (${s.species.name_th})\nวันที่ ${s.day} ของรุ่น · น้ำหนักเฉลี่ย ${n(s.avg_weight_g)} ก.\nอัตรารอด ${pct(s.performance.survival_pct, 1)} · FCR ${s.performance.fcr ?? '-'}\nคะแนนสุขภาพบ่อ ${s.health.score}/100\nบันทึกด้วยแอปทีเด็ดปลาน้ำจืด`
    if (navigator.share) {
      try {
        await navigator.share({ title: 'สรุปบ่อ', text })
      } catch {}
    } else {
      await navigator.clipboard.writeText(text)
      toast('คัดลอกข้อความแล้ว วางใน LINE ได้เลย', 'success')
    }
  }
</script>

<TopBar title="การ์ดสรุป" back="/pond/{cropId}/money" />
<main class="page">
  {#if s}
    <div class="hero" style="padding:24px">
      <div class="row" style="justify-content:space-between"><div class="brand" style="font-weight:800">ทีเด็ดปลาน้ำจืด</div><span class="small muted">{thDate(s.date)}</span></div>
      <h1 class="mt">{s.crop.farm_name}</h1>
      <div class="muted">{s.crop.pond_name} · {s.species.name_th} · วันที่ {s.day} ของรุ่น</div>
      <div class="row mt2" style="gap:18px;align-items:center">
        <ScoreRing score={s.health.score} label="สุขภาพบ่อ" size={100} />
        <div class="kpi" style="flex:1">
          <div class="k" style="background:rgba(255,255,255,0.12)"><div class="lbl" style="color:rgba(255,255,255,0.75)">น้ำหนักเฉลี่ย</div><div class="val" style="color:#fff">{n(s.avg_weight_g)} ก.</div></div>
          <div class="k" style="background:rgba(255,255,255,0.12)"><div class="lbl" style="color:rgba(255,255,255,0.75)">อัตรารอด</div><div class="val" style="color:#fff">{pct(s.performance.survival_pct, 1)}</div></div>
          <div class="k" style="background:rgba(255,255,255,0.12)"><div class="lbl" style="color:rgba(255,255,255,0.75)">FCR</div><div class="val" style="color:#fff">{s.performance.fcr ?? '-'}</div></div>
          <div class="k" style="background:rgba(255,255,255,0.12)"><div class="lbl" style="color:rgba(255,255,255,0.75)">ชีวมวล</div><div class="val" style="color:#fff">{n(s.performance.biomass_kg)} กก.</div></div>
        </div>
      </div>
      {#if s.performance.stock_value}<div class="mt small">มูลค่าปลาในบ่อวันนี้ {baht(s.performance.stock_value)} · ต้นทุนสะสม {baht(s.totals.cost_total)}</div>{/if}
      {#if s.projection}<div class="small">คาดจับอีก {s.projection.days_remaining} วัน · กำไรคาด {baht(s.projection.profit)}</div>{/if}
      <div class="tiny mt" style="opacity:0.7">อาหารสะสม {n1(s.totals.fed_kg)} กก. · โต {s.growth.status_th}</div>
    </div>
    <button class="btn primary mt" onclick={share}>แชร์ / คัดลอกข้อความ</button>
    <p class="tiny muted center mt">แคปหน้าจอการ์ดด้านบนเพื่อส่งเป็นรูปได้</p>
  {:else}
    <div class="skeleton" style="min-height:200px"></div>
  {/if}
</main>
