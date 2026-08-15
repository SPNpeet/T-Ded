<script lang="ts">
  import { onMount } from 'svelte'
  import { api, cachedGet } from '../lib/api'
  import { currentFarm, session, toast, go } from '../lib/ui.svelte'
  import { thDate, n } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'

  let farm: any = $state(null)
  let crops: any[] = $state([])
  let closed: any[] = $state([])
  let show = $state(false)
  let name = $state('')
  let areaRai = $state('')
  let depth = $state('1.5')
  let ptype = $state('earthen')
  let busy = $state(false)
  let editing: any = $state(null)

  async function load() {
    const f = currentFarm()
    if (!f) return
    try {
      farm = (await cachedGet(`/farms/${f.id}`)).data
      const all = (await cachedGet(`/farms/${f.id}/crops?status=all`)).data as any[]
      crops = all.filter((c) => c.status === 'active')
      closed = all.filter((c) => c.status !== 'active')
      if (!name && !farm.ponds.length) name = 'บ่อ 1'
    } catch (e: any) {
      toast(e.message, 'error')
    }
  }
  onMount(load)
  $effect(() => {
    session.farmId
    load()
  })
  const activeCrop = (pondId: string) => crops.find((c) => c.pond_id === pondId)
  async function addPond() {
    busy = true
    try {
      if (editing) {
        await api.patch(`/ponds/${editing.id}`, { name, area_rai: areaRai ? parseFloat(areaRai) : null, depth_m: depth ? parseFloat(depth) : null, pond_type: ptype })
        toast('แก้ไขบ่อแล้ว', 'success')
      } else {
        await api.post(`/farms/${farm.id}/ponds`, { name, area_rai: areaRai ? parseFloat(areaRai) : null, depth_m: depth ? parseFloat(depth) : null, pond_type: ptype })
        toast('เพิ่มบ่อแล้ว ต่อไปปล่อยปลาเพื่อเริ่มรุ่น', 'success')
      }
      show = false
      editing = null
      name = ''
      areaRai = ''
      await load()
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      busy = false
    }
  }
  function startEdit(p: any) {
    editing = p
    name = p.name
    areaRai = p.area_rai ?? ''
    depth = p.depth_m ?? ''
    ptype = p.pond_type
    show = true
  }
  async function hidePond(p: any) {
    if (!confirm(`ซ่อนบ่อ "${p.name}"? ข้อมูลเก่ายังอยู่ แต่จะไม่แสดงในหน้าวันนี้`)) return
    await api.patch(`/ponds/${p.id}`, { active: 0 })
    load()
  }
</script>

<TopBar title="บ่อและรุ่นการเลี้ยง" sub={farm?.name ?? ''} back="/" />
<main class="page">
  {#if farm}
    <button class="btn primary" onclick={() => { show = !show; editing = null }}>{show ? 'ยกเลิก' : 'เพิ่มบ่อใหม่'}</button>
    {#if show}
      <div class="card mt">
        <h3>{editing ? 'แก้ไขบ่อ' : 'บ่อใหม่'}</h3>
        <label for="pn">ชื่อบ่อ</label><input id="pn" bind:value={name} placeholder="เช่น บ่อ 1, บ่อหลังบ้าน" />
        <div class="grid3">
          <div><label for="ar">พื้นที่ (ไร่)</label><input id="ar" type="number" inputmode="decimal" step="0.1" bind:value={areaRai} placeholder="เช่น 1.5" /></div>
          <div><label for="dp">ความลึกน้ำ (ม.)</label><input id="dp" type="number" inputmode="decimal" step="0.1" bind:value={depth} /></div>
          <div><label for="pt">ชนิดบ่อ</label><select id="pt" bind:value={ptype}><option value="earthen">บ่อดิน</option><option value="concrete">บ่อปูน</option><option value="cage">กระชัง</option><option value="liner">บ่อผ้าใบ/พลาสติก</option></select></div>
        </div>
        <button class="btn success mt" onclick={addPond} disabled={busy || !name}>{editing ? 'บันทึกการแก้ไข' : 'เพิ่มบ่อ'}</button>
      </div>
    {/if}
    {#each farm.ponds as p}
      {@const c = activeCrop(p.id)}
      <div class="card mt">
        <div class="row" style="justify-content:space-between;align-items:flex-start">
          <div><h3>{p.name}</h3><div class="small muted">{p.pond_type === 'earthen' ? 'บ่อดิน' : p.pond_type === 'concrete' ? 'บ่อปูน' : p.pond_type === 'cage' ? 'กระชัง' : 'บ่อผ้าใบ'}{p.area_rai ? ` · ${p.area_rai} ไร่` : ''}{p.depth_m ? ` · ลึก ${p.depth_m} ม.` : ''}</div></div>
          <button class="btn link" onclick={() => startEdit(p)}>แก้ไข</button>
        </div>
        {#if c}
          <div class="mt small">กำลังเลี้ยง: {c.species_code === 'nile_tilapia' ? 'ปลานิล' : c.species_code === 'red_tilapia' ? 'ปลาทับทิม' : 'ปลาดุก'} ปล่อย {thDate(c.stocked_at)} จำนวน {n(c.stocked_count)} ตัว × {c.stock_weight_g} ก.</div>
          <a class="btn ghost mt" href="#/pond/{c.id}">เปิดบ่อนี้</a>
        {:else}
          <div class="mt small muted">บ่อว่าง</div>
          <div class="grid2 mt"><a class="btn primary" href="#/new-crop/{p.id}">ปล่อยปลารุ่นใหม่</a><button class="btn ghost" onclick={() => hidePond(p)}>ซ่อนบ่อ</button></div>
        {/if}
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
