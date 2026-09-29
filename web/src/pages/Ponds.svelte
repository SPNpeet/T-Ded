<script lang="ts">
  import { cachedGet } from '../lib/api'
  import { currentFarm, session, toast } from '../lib/ui.svelte'
  import { thDate, n } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'
  import Icon from '../lib/Icon.svelte'
  import PhotoImg from '../lib/PhotoImg.svelte'
  import { pondSummary, speciesTh } from '../lib/pond'
  import { pendingPlan, clearPendingPlan } from '../lib/plan'

  let farm: any = $state(null)
  let crops: any[] = $state([])
  let closed: any[] = $state([])
  let plan = $state(pendingPlan())

  async function load() {
    const f = currentFarm()
    if (!f) return
    try {
      farm = (await cachedGet(`/farms/${f.id}`)).data
      const all = (await cachedGet(`/farms/${f.id}/crops?status=all`)).data as any[]
      crops = all.filter((c) => c.status === 'active')
      closed = all.filter((c) => c.status !== 'active')
    } catch (e: any) {
      toast(e.message, 'error')
    }
  }
  $effect(() => {
    session.farmId
    load()
  })
  const activeCrop = (pondId: string) => crops.find((c) => c.pond_id === pondId)
  const emptyPonds = $derived((farm?.ponds ?? []).filter((p: any) => !activeCrop(p.id)))
  function dropPlan() {
    clearPendingPlan()
    plan = null
  }
</script>

<TopBar title="บ่อและรุ่นการเลี้ยง" sub={farm?.name ?? ''} back="/" />
<main class="page">
  {#if farm}
    {#if plan}
      <div class="card tint-cyan">
        <div class="card-title"><h3>โปรแกรมการเลี้ยงรอใช้</h3><button class="btn link" onclick={dropPlan}>ไม่ใช้</button></div>
        <div class="small">{speciesTh(plan.code)} {n(plan.count)} ตัว ปล่อย {plan.stockW} ก. เป้า {n(plan.targetW)} ก.</div>
        <div class="small bold mt">{emptyPonds.length ? 'เลือกบ่อว่างที่จะปล่อยปลาตามโปรแกรมนี้' : 'ยังไม่มีบ่อว่าง เพิ่มบ่อใหม่ก่อน'}</div>
        {#if !emptyPonds.length}<a class="btn primary mt" href="#/pond-edit/new?next=crop">เพิ่มบ่อแล้วปล่อยปลา</a>{/if}
      </div>
    {/if}

    <a class="btn primary" class:mt={!!plan} href="#/pond-edit/new{plan ? '?next=crop' : ''}"><Icon name="plus" size={22} /> เพิ่มบ่อใหม่</a>

    {#if !farm.ponds.length}
      <div class="card mt center">
        <Icon name="pond" size={40} />
        <p class="bold">ยังไม่มีบ่อ</p>
        <p class="small muted">เพิ่มบ่อ ใส่ขนาด แล้วถ่ายรูปบ่อ ระบบจะรู้ตำแหน่งบ่อและคำนวณน้ำในบ่อให้</p>
      </div>
    {/if}

    {#each farm.ponds as p (p.id)}
      {@const c = activeCrop(p.id)}
      <div class="card mt pond">
        <a class="thumb" href="#/pond-info/{p.id}" aria-label="ข้อมูลและรูปของ {p.name}">
          {#if p.cover_photo_id}<PhotoImg id={p.cover_photo_id} thumb alt="" />{:else}<span class="nothumb"><Icon name="camera" size={26} /><small>ถ่ายรูป</small></span>{/if}
        </a>
        <div class="body">
          <div class="row" style="justify-content:space-between;align-items:flex-start;gap:6px">
            <h3>{p.name}</h3>
            {#if p.lat != null}<span class="loc" title="ปักหมุดตำแหน่งแล้ว"><Icon name="pin" size={16} /></span>{/if}
          </div>
          <div class="small muted">{pondSummary(p)}</div>
          {#if c}
            <div class="small mt">{speciesTh(c.species_code)} ปล่อย {thDate(c.stocked_at)} · {n(c.stocked_count)} ตัว</div>
          {:else}
            <div class="small mt muted">บ่อว่าง</div>
          {/if}
        </div>
        <div class="acts">
          {#if c}
            <a class="btn primary" href="#/pond/{c.id}">เปิดบ่อนี้</a>
          {:else}
            <a class="btn primary" href="#/new-crop/{p.id}">{plan ? 'ใช้โปรแกรมกับบ่อนี้' : 'ปล่อยปลารุ่นใหม่'}</a>
          {/if}
          <a class="btn ghost" href="#/pond-info/{p.id}">ข้อมูลบ่อ/รูป</a>
        </div>
      </div>
    {/each}

    {#if closed.length}
      <details class="mt2"><summary>รุ่นที่ปิดแล้ว ({closed.length})</summary>
        {#each closed as c}<div class="list-item"><div class="main"><div class="title">{c.pond_name} · ปล่อย {thDate(c.stocked_at)} ปิด {thDate(c.closed_at)}</div><div class="sub">{n(c.stocked_count)} ตัว</div></div><a class="btn link" href="#/pond/{c.id}/money">ดูสรุป</a></div>{/each}
      </details>
    {/if}
  {:else}
    <div class="skeleton"></div>
  {/if}
</main>

<style>
  .pond { display: grid; grid-template-columns: 96px 1fr; gap: 12px; }
  .thumb { width: 96px; height: 96px; border-radius: 12px; overflow: hidden; background: #eef2f7; display: block; }
  .nothumb { width: 100%; height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; color: var(--muted); gap: 2px; }
  .nothumb small { font-weight: 700; font-size: 0.75rem; }
  .body { min-width: 0; }
  .body h3 { margin: 0; }
  .loc { color: var(--cyan-deep); flex-shrink: 0; }
  .acts { grid-column: 1 / -1; display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  .acts :global(.btn) { margin: 0; }
</style>
