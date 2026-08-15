<script lang="ts">
  import { onMount } from 'svelte'
  import { cachedGet } from '../lib/api'
  import { toast, session } from '../lib/ui.svelte'
  import { thDate, n, baht } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'
  import Icon from '../lib/Icon.svelte'

  let s: any = $state(null)
  onMount(async () => {
    try {
      s = (await cachedGet('/subscription')).data
    } catch (e: any) {
      toast(e.message, 'error')
    }
  })
  const bar = (used: number, max: number) => Math.min(100, Math.round((used / Math.max(1, max)) * 100))
</script>

<TopBar title="แพ็กเกจการใช้งาน" sub={session.user?.org_name ?? ''} back="/menu" />
<main class="page">
  {#if !s}
    <div class="skeleton" style="min-height:160px"></div>
  {:else}
    <section class="hero">
      <div class="muted small">แพ็กเกจปัจจุบัน</div>
      <div class="big-number" style="font-size:2rem">{s.plan_name_th}</div>
      {#if s.days_left != null}
        <div class="mt">
          {#if s.days_left >= 0}
            เหลืออีก <b>{s.days_left} วัน</b> (ถึง {thDate(s.expires_at)})
          {:else}
            <span style="color:#FFB4A8">หมดอายุแล้วเมื่อ {thDate(s.expires_at)}</span>
          {/if}
        </div>
      {/if}
    </section>

    {#if !s.active}
      <div class="alert warn mt"><Icon name="alert" /><div><b>แพ็กเกจหมดอายุ</b><div class="small">ข้อมูลเดิมยังดูได้ครบทุกอย่าง แต่เพิ่มบ่อหรือฟาร์มใหม่ไม่ได้จนกว่าจะต่ออายุ</div></div></div>
    {:else if s.days_left != null && s.days_left <= 14}
      <div class="alert info mt">เหลืออีก {s.days_left} วันจะหมดอายุ ติดต่อผู้ดูแลระบบเพื่อต่ออายุได้ล่วงหน้า</div>
    {/if}

    <div class="card mt">
      <h3>การใช้งานตอนนี้</h3>
      {#each [['ฟาร์ม', s.usage.farms, s.limits.farms], ['บ่อ', s.usage.ponds, s.limits.ponds], ['ผู้ใช้', s.usage.members, s.limits.members]] as [label, used, max]}
        <div class="mt">
          <div class="row" style="justify-content:space-between"><b>{label}</b><span class="num">{n(used)} / {n(max)}</span></div>
          <div class="progress {used >= max ? 'amber' : ''}"><div style="width:{bar(used, max)}%"></div></div>
        </div>
      {/each}
    </div>

    <h3 class="mt2">แพ็กเกจทั้งหมด</h3>
    {#each s.plans as p}
      <div class="card mt {p.code === s.plan ? 'tint-cyan' : ''}">
        <div class="row" style="justify-content:space-between;align-items:flex-start">
          <div>
            <b style="font-size:1.1rem">{p.name_th}</b>
            <div class="small muted">{p.detail_th}</div>
          </div>
          <div class="right">
            <div class="big-number" style="font-size:1.4rem">{p.price === 0 ? 'ฟรี' : baht(p.price)}</div>
            {#if p.price > 0}<div class="tiny muted">ต่อเดือน</div>{/if}
          </div>
        </div>
        {#if p.code === s.plan}<div class="pill good mt">กำลังใช้อยู่</div>{/if}
      </div>
    {/each}

    <div class="card mt2">
      <h3>ต้องการต่ออายุหรืออัปเกรด</h3>
      <p class="mt">ติดต่อผู้ดูแลระบบเพื่อแจ้งต่ออายุ เมื่อชำระเงินแล้วผู้ดูแลจะกดต่ออายุให้ทันที และสิทธิ์จะเพิ่มให้เองโดยไม่ต้องติดตั้งอะไรใหม่</p>
      {#if s.payments?.length}
        <h3 class="mt2">ประวัติการชำระเงิน</h3>
        <div class="list">
          {#each s.payments as p}
            <div class="list-item"><div class="main"><div class="title">{baht(p.amount)}</div><div class="sub">{thDate(p.paid_at)} · ครอบคลุมถึง {thDate(p.period_to)}</div></div></div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</main>
