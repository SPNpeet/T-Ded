<script lang="ts">
  import { api, cachedGet } from '../lib/api'
  import { ui, session, currentFarm, toast, go } from '../lib/ui.svelte'
  import { n } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'
  import Icon from '../lib/Icon.svelte'
  import { Locator } from '../lib/photos'
  import { POND_TYPES, SHAPES, WATER_SOURCES, AERATION, areaM2, areaThai, withArea, toRaiNganWa, fromRaiNganWa } from '../lib/pond'

  let { pondId }: { pondId: string } = $props()
  const isNew = $derived(pondId === 'new')
  const q = $derived(new URLSearchParams(ui.route.split('?')[1] ?? ''))
  const next = $derived(q.get('next'))

  let farm: any = $state(null)
  let loaded = $state(false)
  let busy = $state(false)
  let hasCrop = $state(false)
  let f = $state({
    name: '',
    pond_type: 'earthen',
    shape: 'rect',
    width_m: '',
    length_m: '',
    diameter_m: '',
    rai: '',
    ngan: '',
    wa: '',
    depth_m: '1.5',
    water_source: '' as string,
    aeration: '' as string,
    note: '',
    lat: null as number | null,
    lng: null as number | null,
    location_accuracy_m: null as number | null,
  })
  let locating = $state(false)

  const str = (v: any) => (v == null ? '' : String(v))

  async function load() {
    const farmId = q.get('farm') || currentFarm()?.id
    if (!farmId) return
    try {
      farm = (await cachedGet(`/farms/${farmId}`)).data
      if (pondId === 'new') {
        f.name = `บ่อ ${(farm.ponds?.length ?? 0) + 1}`
      } else {
        const p = farm.ponds.find((x: any) => x.id === pondId)
        if (!p) {
          toast('ไม่พบบ่อนี้', 'error')
          return go('/ponds')
        }
        const shape = p.shape || (p.area_m2 || p.area_rai ? 'free' : 'rect')
        const m2 = areaM2(p)
        const rnw = m2 ? toRaiNganWa(m2) : null
        Object.assign(f, {
          name: p.name,
          pond_type: p.pond_type || 'earthen',
          shape,
          width_m: str(p.width_m),
          length_m: str(p.length_m),
          diameter_m: str(p.diameter_m),
          rai: rnw ? str(rnw.rai || '') : '',
          ngan: rnw ? str(rnw.ngan || '') : '',
          wa: rnw ? str(rnw.wa || '') : '',
          depth_m: str(p.depth_m),
          water_source: p.water_source ?? '',
          aeration: p.aeration ?? '',
          note: p.note ?? '',
          lat: p.lat ?? null,
          lng: p.lng ?? null,
          location_accuracy_m: p.location_accuracy_m ?? null,
        })
        const crops = (await cachedGet(`/farms/${farmId}/crops`)).data as any[]
        hasCrop = crops.some((c) => c.pond_id === pondId)
      }
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      loaded = true
    }
  }
  $effect(() => {
    pondId
    session.farmId
    load()
  })

  const num = (v: string) => {
    const x = parseFloat(v)
    return Number.isFinite(x) && x > 0 ? x : null
  }
  const body = $derived.by(() =>
    withArea({
      name: f.name.trim(),
      pond_type: f.pond_type,
      shape: f.shape,
      width_m: f.shape === 'rect' ? num(f.width_m) : null,
      length_m: f.shape === 'rect' ? num(f.length_m) : null,
      diameter_m: f.shape === 'round' ? num(f.diameter_m) : null,
      area_m2: f.shape === 'free' ? fromRaiNganWa(f.rai, f.ngan, f.wa) || null : null,
      depth_m: num(f.depth_m),
      water_source: f.water_source || null,
      aeration: f.aeration || null,
      note: f.note.trim() || null,
      lat: f.lat,
      lng: f.lng,
      location_accuracy_m: f.location_accuracy_m,
    }),
  )
  const vol = $derived(body.area_m2 && body.depth_m ? body.area_m2 * body.depth_m : null)

  async function locate() {
    locating = true
    const l = new Locator().start()
    const fix = await l.wait(12000)
    l.stop()
    locating = false
    if (!fix) return toast((l.error || 'จับพิกัดไม่ได้') + ' เปิด GPS แล้วลองใหม่', 'error', 4000)
    f.lat = fix.lat
    f.lng = fix.lng
    f.location_accuracy_m = fix.accuracy_m
    toast(`ได้พิกัดแล้ว คลาดเคลื่อน ±${fix.accuracy_m} ม.`, 'success')
  }

  async function save() {
    if (!body.name) return toast('ตั้งชื่อบ่อก่อนครับ เช่น บ่อ 1', 'error')
    if (f.shape === 'rect' && (f.width_m || f.length_m) && !(body.width_m && body.length_m)) return toast('ใส่ทั้งความกว้างและความยาว', 'error')
    busy = true
    try {
      if (isNew) {
        const r = await api.post(`/farms/${farm.id}/ponds`, body)
        toast('เพิ่มบ่อแล้ว', 'success')
        go(next === 'crop' ? `/new-crop/${r.id}` : `/pond-info/${r.id}`)
      } else {
        await api.patch(`/ponds/${pondId}`, body)
        toast('บันทึกข้อมูลบ่อแล้ว', 'success')
        go(`/pond-info/${pondId}${q.get('farm') ? `?farm=${q.get('farm')}` : ''}`)
      }
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      busy = false
    }
  }

  async function hidePond() {
    if (!confirm(`ซ่อนบ่อ "${f.name}"? ข้อมูลเก่ายังอยู่ แต่จะไม่แสดงในหน้าวันนี้`)) return
    await api.patch(`/ponds/${pondId}`, { active: 0 })
    toast('ซ่อนบ่อแล้ว')
    go('/ponds')
  }
  const toggle = (cur: string, v: string) => (cur === v ? '' : v)
</script>

<TopBar title={isNew ? 'เพิ่มบ่อใหม่' : 'แก้ไขข้อมูลบ่อ'} sub={farm?.name ?? ''} back={isNew ? '/ponds' : `/pond-info/${pondId}${q.get('farm') ? `?farm=${q.get('farm')}` : ''}`} />
<main class="page">
  {#if !loaded}
    <div class="skeleton"></div>
  {:else}
    <div class="card">
      <label for="pn">ชื่อบ่อ</label>
      <input id="pn" bind:value={f.name} placeholder="เช่น บ่อ 1, บ่อหลังบ้าน" />
      <div class="small bold mt">ชนิดบ่อ</div>
      <div class="chips mt">{#each POND_TYPES as t}<button class="chip" class:on={f.pond_type === t.v} onclick={() => (f.pond_type = t.v)}>{t.th}</button>{/each}</div>
    </div>

    <div class="card mt">
      <div class="card-title"><h3>ขนาดบ่อ</h3><Icon name="ruler" size={20} /></div>
      <div class="chips">{#each SHAPES as s}<button class="chip" class:on={f.shape === s.v} onclick={() => (f.shape = s.v)}>{s.th}</button>{/each}</div>
      {#if f.shape === 'rect'}
        <div class="grid2 mt">
          <div><label for="w">กว้าง (เมตร)</label><input id="w" type="number" inputmode="decimal" step="0.1" bind:value={f.width_m} placeholder="เช่น 40" /></div>
          <div><label for="l">ยาว (เมตร)</label><input id="l" type="number" inputmode="decimal" step="0.1" bind:value={f.length_m} placeholder="เช่น 60" /></div>
        </div>
        <p class="tiny muted mt">ไม่มีตลับเมตร? เดินนับก้าว 1 ก้าวปกติประมาณ 0.7 เมตร</p>
      {:else if f.shape === 'round'}
        <div class="mt"><label for="d">ความกว้างวัดผ่านกลางบ่อ (เมตร)</label><input id="d" type="number" inputmode="decimal" step="0.1" bind:value={f.diameter_m} placeholder="เช่น 5 (บ่อผ้าใบกลม)" /></div>
      {:else}
        <div class="grid3 mt">
          <div><label for="r">ไร่</label><input id="r" type="number" inputmode="numeric" bind:value={f.rai} placeholder="0" /></div>
          <div><label for="g">งาน</label><input id="g" type="number" inputmode="numeric" bind:value={f.ngan} placeholder="0" /></div>
          <div><label for="wa">ตร.วา</label><input id="wa" type="number" inputmode="numeric" bind:value={f.wa} placeholder="0" /></div>
        </div>
      {/if}
      <div class="result mt" class:off={!body.area_m2}>
        {#if body.area_m2}
          พื้นที่ผิวน้ำ <b>{areaThai(body.area_m2)}</b> <span class="muted">({n(body.area_m2)} ตร.ม.)</span>
        {:else}
          ใส่ขนาดแล้วระบบจะคิดพื้นที่ให้
        {/if}
      </div>
    </div>

    <div class="card mt">
      <div class="card-title"><h3>ความลึกของน้ำ</h3></div>
      <div class="chips">{#each [0.8, 1, 1.2, 1.5, 1.8, 2] as d}<button class="chip" class:on={parseFloat(f.depth_m) === d} onclick={() => (f.depth_m = String(d))}>{d} ม.</button>{/each}</div>
      <div class="mt"><label for="dp">หรือใส่เอง (เมตร)</label><input id="dp" type="number" inputmode="decimal" step="0.1" bind:value={f.depth_m} /></div>
      {#if vol}<div class="result mt">น้ำในบ่อประมาณ <b>{n(vol)} ลบ.ม.</b> <span class="muted">({n(vol * 1000)} ลิตร)</span></div>{/if}
    </div>

    <div class="card mt">
      <div class="card-title"><h3>น้ำและอากาศ</h3><span class="tiny muted">กดซ้ำเพื่อยกเลิก</span></div>
      <div class="small bold">แหล่งน้ำที่ใช้</div>
      <div class="chips mt">{#each WATER_SOURCES as s}<button class="chip" class:on={f.water_source === s.v} onclick={() => (f.water_source = toggle(f.water_source, s.v))}>{s.th}</button>{/each}</div>
      <div class="small bold mt2">เครื่องให้อากาศ</div>
      <div class="chips mt">{#each AERATION as s}<button class="chip" class:on={f.aeration === s.v} onclick={() => (f.aeration = toggle(f.aeration, s.v))}>{s.th}</button>{/each}</div>
    </div>

    <div class="card mt">
      <div class="card-title"><h3>ตำแหน่งบ่อ</h3><Icon name="pin" size={20} /></div>
      {#if f.lat != null && f.lng != null}
        <div class="result">ปักหมุดแล้ว <b>{f.lat.toFixed(6)}, {f.lng.toFixed(6)}</b>{#if f.location_accuracy_m} <span class="muted">(±{Math.round(f.location_accuracy_m)} ม.)</span>{/if}</div>
        <button class="btn ghost mt" onclick={locate} disabled={locating}>{locating ? 'กำลังจับพิกัด...' : 'ปักหมุดใหม่ตรงที่ยืนอยู่'}</button>
      {:else}
        <p class="small muted">ยืนข้างบ่อแล้วกด หรือข้ามไปก่อนก็ได้ ถ่ายรูปบ่อเมื่อไร ระบบจะปักหมุดให้เอง</p>
        <button class="btn ghost mt" onclick={locate} disabled={locating}><Icon name="pin" size={20} /> {locating ? 'กำลังจับพิกัด...' : 'ใช้ตำแหน่งที่ยืนอยู่ตอนนี้'}</button>
      {/if}
    </div>

    <div class="card mt">
      <label for="nt">หมายเหตุ</label>
      <input id="nt" bind:value={f.note} maxlength="300" placeholder="เช่น ตลิ่งด้านทิศเหนือต่ำ, ท่อระบายน้ำอยู่มุมตะวันออก" />
    </div>

    <button class="btn success mt2" onclick={save} disabled={busy}>{busy ? 'กำลังบันทึก...' : isNew ? 'เพิ่มบ่อ' : 'บันทึกข้อมูลบ่อ'}</button>
    {#if !isNew && !hasCrop}<button class="btn link mt" style="color:var(--red)" onclick={hidePond}>ซ่อนบ่อนี้</button>{/if}
  {/if}
</main>

<style>
  .result { background: var(--cyan-tint); border-radius: 12px; padding: 10px 12px; font-size: 1rem; color: var(--navy); }
  .result.off { background: #f1f4f8; color: var(--muted); }
</style>
