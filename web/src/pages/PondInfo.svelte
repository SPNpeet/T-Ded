<script lang="ts">
  import { cachedGet, api } from '../lib/api'
  import { ui, session, currentFarm, toast, go } from '../lib/ui.svelte'
  import { thDate, thDateTime, n } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'
  import Icon from '../lib/Icon.svelte'
  import PhotoImg from '../lib/PhotoImg.svelte'
  import PondCamera from '../lib/PondCamera.svelte'
  import { listPhotos, removePhoto, setCover, useLocationOf, photoBlob, Locator, type PondPhoto } from '../lib/photos'
  import { pondTypeTh, sizeText, volumeM3, waterSourceTh, aerationTh, photoKindTh, mapUrl, speciesTh, areaM2 } from '../lib/pond'

  let { pondId }: { pondId: string } = $props()
  let farm: any = $state(null)
  let pond: any = $state(null)
  let crop: any = $state(null)
  let photos: PondPhoto[] = $state([])
  let loading = $state(true)
  let viewing: number | null = $state(null)
  let locating = $state(false)

  async function load() {
    // เจ้าหน้าที่เปิดจากหลังบ้านจะส่งรหัสฟาร์มมาด้วย (ไม่ได้เป็นสมาชิกฟาร์ม)
    const fromLink = new URLSearchParams(ui.route.split('?')[1] ?? '').get('farm')
    const ids = [fromLink, currentFarm()?.id, ...(session.user?.farms ?? []).map((f: any) => f.id)].filter(Boolean) as string[]
    try {
      for (const id of [...new Set(ids)]) {
        const f = (await cachedGet(`/farms/${id}`)).data
        const p = f.ponds?.find((x: any) => x.id === pondId)
        if (p) {
          farm = f
          pond = p
          break
        }
      }
      if (!pond) {
        toast('ไม่พบบ่อนี้', 'error')
        return go('/ponds')
      }
      const crops = (await cachedGet(`/farms/${farm.id}/crops`)).data as any[]
      crop = crops.find((c) => c.pond_id === pondId) ?? null
      photos = await listPhotos(pondId)
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      loading = false
    }
  }
  $effect(() => {
    pondId
    session.user
    load()
  })
  $effect(() => {
    // รูปที่ค้างส่งขึ้นระบบแล้ว ให้ป้าย "รอส่ง" หายทันที
    window.addEventListener('teedet:photos', load)
    return () => window.removeEventListener('teedet:photos', load)
  })

  const cover = $derived(photos.find((p) => p.id === pond?.cover_photo_id) ?? photos[0] ?? null)
  const vol = $derived(pond ? volumeM3(pond) : null)
  const area = $derived(pond ? areaM2(pond) : null)
  const density = $derived(crop && area ? crop.stocked_count / area : null)
  const current = $derived(viewing != null ? photos[viewing] : null)

  async function locateHere() {
    locating = true
    const l = new Locator().start()
    const fix = await l.wait(12000)
    l.stop()
    locating = false
    if (!fix) return toast((l.error || 'จับพิกัดไม่ได้') + ' เปิด GPS แล้วยืนข้างบ่อในที่โล่ง', 'error', 4500)
    if (fix.accuracy_m > 100 && !confirm(`พิกัดยังคลาดเคลื่อน ±${fix.accuracy_m} ม. ใช้ค่านี้เลยไหม`)) return
    await api.patch(`/ponds/${pondId}`, { lat: fix.lat, lng: fix.lng, location_accuracy_m: fix.accuracy_m })
    toast(`ปักหมุดตำแหน่งบ่อแล้ว (±${fix.accuracy_m} ม.)`, 'success')
    load()
  }

  async function del(p: PondPhoto) {
    if (!confirm('ลบรูปนี้? ลบแล้วเอาคืนไม่ได้')) return
    try {
      await removePhoto(p)
      viewing = null
      toast('ลบรูปแล้ว')
      load()
    } catch (e: any) {
      toast(e.message, 'error')
    }
  }
  async function makeCover(p: PondPhoto) {
    try {
      await setCover(p)
      toast('ตั้งเป็นรูปหน้าบ่อแล้ว', 'success')
      load()
    } catch (e: any) {
      toast(e.message, 'error')
    }
  }
  async function useLoc(p: PondPhoto) {
    try {
      await useLocationOf(p)
      toast('ใช้พิกัดจากรูปนี้เป็นตำแหน่งบ่อแล้ว', 'success')
      load()
    } catch (e: any) {
      toast(e.message, 'error')
    }
  }
  async function share(p: PondPhoto) {
    const blob = await photoBlob(p.id)
    if (!blob) return toast('ยังโหลดรูปไม่ได้', 'error')
    const name = `${pond.name}-${p.taken_at.slice(0, 19).replace(/[:T]/g, '-')}.jpg`
    const file = new File([blob], name, { type: 'image/jpeg' })
    if ((navigator as any).canShare?.({ files: [file] })) {
      try {
        await navigator.share({ files: [file], title: pond.name })
      } catch {
        /* ผู้ใช้กดยกเลิก */
      }
      return
    }
    const a = document.createElement('a')
    a.href = URL.createObjectURL(blob)
    a.download = name
    a.click()
    setTimeout(() => URL.revokeObjectURL(a.href), 5000)
  }
  function step(d: number) {
    if (viewing == null) return
    viewing = (viewing + d + photos.length) % photos.length
  }
  function onKey(e: KeyboardEvent) {
    if (viewing == null) return
    if (e.key === 'Escape') viewing = null
    if (e.key === 'ArrowRight') step(1)
    if (e.key === 'ArrowLeft') step(-1)
  }
  let touchX = 0
</script>

<svelte:window onkeydown={onKey} />

<TopBar title={pond?.name ?? 'ข้อมูลบ่อ'} sub={farm?.name ?? ''} back={ui.route.includes('farm=') && farm ? `/admin/farm/${farm.id}` : '/ponds'}>
  {#snippet right()}
    {#if pond}<a class="btn link" href="#/pond-edit/{pond.id}{ui.route.includes('farm=') ? `?farm=${farm.id}` : ''}" aria-label="แก้ไขข้อมูลบ่อ"><Icon name="edit" size={20} /> แก้ไข</a>{/if}
  {/snippet}
</TopBar>

<main class="page">
  {#if loading && !pond}
    <div class="skeleton" style="height:220px"></div>
  {:else if pond}
    <button class="cover" onclick={() => cover && (viewing = photos.indexOf(cover))} disabled={!cover} aria-label="ดูรูปบ่อ">
      {#if cover}
        <PhotoImg id={cover.id} alt="รูปบ่อ {pond.name}" />
        <span class="stampchip">{thDateTime(cover.taken_at, true)}</span>
      {:else}
        <div class="empty"><Icon name="camera" size={40} /><b>ยังไม่มีรูปบ่อนี้</b><span>ถ่ายรูปแรกที่ข้างบ่อ ระบบจะปักหมุดตำแหน่งบ่อให้เอง</span></div>
      {/if}
    </button>

    <div class="mt"><PondCamera {pond} farmName={farm?.name ?? ''} firstPhoto={!photos.length} onsaved={load} /></div>

    <div class="card mt">
      <div class="card-title"><h3>ข้อมูลบ่อ</h3><a class="btn link" href="#/pond-edit/{pond.id}{ui.route.includes('farm=') ? `?farm=${farm.id}` : ''}">แก้ไข</a></div>
      <div class="facts">
        <div><span>ชนิด</span><b>{pondTypeTh(pond.pond_type)}</b></div>
        <div><span>ขนาด</span><b>{sizeText(pond) || 'ยังไม่ระบุ'}</b></div>
        <div><span>ความลึกน้ำ</span><b>{pond.depth_m ? `${pond.depth_m} ม.` : 'ยังไม่ระบุ'}</b></div>
        <div><span>ปริมาณน้ำ</span><b>{vol ? `ประมาณ ${n(vol)} ลบ.ม. (${n(vol * 1000)} ลิตร)` : 'ใส่ขนาดและความลึกเพื่อคำนวณ'}</b></div>
        <div><span>แหล่งน้ำ</span><b>{waterSourceTh(pond.water_source) || 'ยังไม่ระบุ'}</b></div>
        <div><span>เครื่องให้อากาศ</span><b>{aerationTh(pond.aeration) || 'ยังไม่ระบุ'}</b></div>
        {#if pond.note}<div><span>หมายเหตุ</span><b>{pond.note}</b></div>{/if}
      </div>
      {#if vol}<p class="tiny muted mt">ปริมาณน้ำใช้คำนวณขนาดยา/ปูน/เกลือต่อบ่อ ถ้าระดับน้ำเปลี่ยนให้แก้ความลึก</p>{/if}
    </div>

    <div class="card mt">
      <div class="card-title"><h3>ตำแหน่งบ่อ</h3></div>
      {#if pond.lat != null && pond.lng != null}
        <div class="row" style="gap:10px;align-items:flex-start">
          <span class="pinicon"><Icon name="pin" size={26} /></span>
          <div style="flex:1;min-width:0">
            <div class="bold">{Number(pond.lat).toFixed(6)}, {Number(pond.lng).toFixed(6)}</div>
            <div class="small muted">{pond.location_accuracy_m ? `คลาดเคลื่อนประมาณ ±${Math.round(pond.location_accuracy_m)} ม.` : 'ปักหมุดแล้ว'}</div>
          </div>
        </div>
        <div class="grid2 mt">
          <a class="btn primary" href={mapUrl(Number(pond.lat), Number(pond.lng))} target="_blank" rel="noopener">เปิดในแผนที่</a>
          <button class="btn ghost" onclick={locateHere} disabled={locating}>{locating ? 'กำลังจับพิกัด...' : 'ปักหมุดใหม่ตรงนี้'}</button>
        </div>
      {:else}
        <p class="small muted">ยังไม่ได้ปักหมุด ถ่ายรูปที่ข้างบ่อระบบจะปักให้เอง หรือยืนที่บ่อแล้วกดปุ่มนี้</p>
        <button class="btn primary mt" onclick={locateHere} disabled={locating}><Icon name="pin" size={20} /> {locating ? 'กำลังจับพิกัด...' : 'ใช้ตำแหน่งที่ยืนอยู่ตอนนี้'}</button>
      {/if}
    </div>

    <div class="card mt">
      <div class="card-title"><h3>การเลี้ยงในบ่อนี้</h3></div>
      {#if crop}
        <div class="small">กำลังเลี้ยง <b>{speciesTh(crop.species_code)}</b> ปล่อย {thDate(crop.stocked_at)} จำนวน {n(crop.stocked_count)} ตัว</div>
        {#if density}<div class="small muted">ความหนาแน่นตอนปล่อย {density >= 10 ? n(density) : density.toFixed(1)} ตัว/ตร.ม.</div>{/if}
        <a class="btn primary mt" href="#/pond/{crop.id}">เปิดบ่อนี้ (อาหาร น้ำ การโต)</a>
      {:else}
        <p class="small muted">บ่อว่าง</p>
        <a class="btn primary mt" href="#/new-crop/{pond.id}">ปล่อยปลารุ่นใหม่</a>
      {/if}
    </div>

    <div class="card mt">
      <div class="card-title"><h3>รูปบ่อนี้ ({photos.length})</h3>{#if photos.length}<PondCamera {pond} farmName={farm?.name ?? ''} big={false} onsaved={load} />{/if}</div>
      {#if photos.length}
        <div class="grid">
          {#each photos as p, i (p.id)}
            <button class="tile" onclick={() => (viewing = i)} aria-label="ดูรูป {thDateTime(p.taken_at, true)}">
              <PhotoImg id={p.id} thumb alt="" />
              <span class="cap">{thDateTime(p.taken_at)}</span>
              {#if p.pending}<span class="badge">รอส่ง</span>{:else if p.lat == null}<span class="badge warn">ไม่มีพิกัด</span>{/if}
            </button>
          {/each}
        </div>
      {:else}
        <p class="small muted">ถ่ายเก็บไว้เรื่อย ๆ เช่น สีน้ำ ตัวปลา ตลิ่ง จะเห็นความเปลี่ยนแปลงของบ่อย้อนหลัง และใช้ยืนยันกับเจ้าหน้าที่ได้ว่าถ่ายที่บ่อนี้จริง เวลาไหน</p>
      {/if}
    </div>
  {/if}
</main>

{#if current}
  <div class="viewer" role="dialog" tabindex="-1" aria-modal="true" aria-label="ดูรูป" ontouchstart={(e) => (touchX = e.touches[0].clientX)} ontouchend={(e) => { const dx = e.changedTouches[0].clientX - touchX; if (Math.abs(dx) > 60) step(dx < 0 ? 1 : -1) }}>
    <div class="vtop">
      <span>{(viewing ?? 0) + 1} / {photos.length}</span>
      <button class="x" onclick={() => (viewing = null)} aria-label="ปิด"><Icon name="close" size={26} /></button>
    </div>
    <div class="vimg">
      <PhotoImg id={current.id} fit="contain" alt="รูปบ่อ {pond.name}" />
      {#if photos.length > 1}
        <button class="nav l" onclick={() => step(-1)} aria-label="รูปก่อนหน้า"><Icon name="back" size={28} /></button>
        <button class="nav r" onclick={() => step(1)} aria-label="รูปถัดไป"><span style="display:inline-block;transform:scaleX(-1)"><Icon name="back" size={28} /></span></button>
      {/if}
    </div>
    <div class="vinfo">
      <div class="bold">{thDateTime(current.taken_at, true)}{current.kind ? ` · ${photoKindTh(current.kind)}` : ''}</div>
      {#if current.caption}<div>{current.caption}</div>{/if}
      {#if current.lat != null && current.lng != null}
        <a class="maplink" href={mapUrl(current.lat, current.lng)} target="_blank" rel="noopener"><Icon name="pin" size={16} /> {current.lat.toFixed(6)}, {current.lng.toFixed(6)}{current.accuracy_m ? ` (±${current.accuracy_m} ม.)` : ''}</a>
      {:else}
        <div class="muted2">ไม่มีพิกัด</div>
      {/if}
      {#if current.pending}<div class="muted2">ยังไม่ได้ส่งขึ้นระบบ จะส่งเองเมื่อมีสัญญาณ</div>{/if}
      <div class="vact">
        {#if pond.cover_photo_id !== current.id}<button onclick={() => makeCover(current)}>ตั้งเป็นรูปหน้าบ่อ</button>{/if}
        {#if current.lat != null && (pond.lat !== current.lat || pond.lng !== current.lng)}<button onclick={() => useLoc(current)}>ใช้พิกัดรูปนี้เป็นตำแหน่งบ่อ</button>{/if}
        <button onclick={() => share(current)}>ส่งต่อ/บันทึกลงเครื่อง</button>
        <button class="danger" onclick={() => del(current)}><Icon name="trash" size={18} /> ลบ</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .cover { display: block; width: 100%; aspect-ratio: 16 / 10; max-height: 380px; border: none; padding: 0; border-radius: 16px; overflow: hidden; background: #e8edf4; position: relative; cursor: pointer; }
  .cover:disabled { cursor: default; }
  .empty { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 6px; color: var(--muted); padding: 16px; text-align: center; }
  .empty b { color: var(--navy); font-size: 1.05rem; }
  .stampchip { position: absolute; left: 10px; top: 10px; background: rgba(0, 0, 0, 0.55); color: #fff; font-size: 0.8rem; font-weight: 700; padding: 4px 10px; border-radius: 999px; }
  .facts { display: grid; gap: 0; }
  .facts > div { display: flex; justify-content: space-between; gap: 12px; padding: 9px 0; border-bottom: 1px dashed var(--line); }
  .facts > div:last-child { border-bottom: none; }
  .facts span { color: var(--muted); white-space: nowrap; }
  .facts b { text-align: right; color: var(--navy); }
  .pinicon { color: var(--cyan-deep); }
  .grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px; }
  .tile { position: relative; aspect-ratio: 1; border: none; padding: 0; border-radius: 10px; overflow: hidden; cursor: pointer; background: #e8edf4; }
  .tile .cap { position: absolute; left: 0; right: 0; bottom: 0; background: linear-gradient(transparent, rgba(0, 0, 0, 0.7)); color: #fff; font-size: 0.7rem; font-weight: 700; padding: 12px 5px 4px; text-align: left; }
  .badge { position: absolute; top: 5px; right: 5px; background: var(--cyan-deep); color: #fff; font-size: 0.65rem; font-weight: 800; padding: 2px 6px; border-radius: 999px; }
  .badge.warn { background: #b26a00; }
  .viewer { position: fixed; inset: 0; z-index: 70; background: #05080f; color: #fff; display: flex; flex-direction: column; }
  .vtop { display: flex; justify-content: space-between; align-items: center; padding: calc(8px + env(safe-area-inset-top)) 12px 8px; font-weight: 700; }
  .x { background: none; border: none; color: #fff; min-width: 48px; min-height: 48px; cursor: pointer; }
  .vimg { flex: 1; min-height: 0; position: relative; }
  .nav { position: absolute; top: 50%; transform: translateY(-50%); background: rgba(0, 0, 0, 0.45); border: none; color: #fff; width: 48px; height: 64px; border-radius: 12px; cursor: pointer; }
  .nav.l { left: 6px; }
  .nav.r { right: 6px; }
  .vinfo { padding: 12px 16px calc(14px + env(safe-area-inset-bottom)); display: grid; gap: 4px; font-size: 0.95rem; }
  .maplink { color: #9fd8ff; display: inline-flex; align-items: center; gap: 4px; font-weight: 600; }
  .muted2 { color: #b9c2d3; }
  .vact { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 8px; }
  .vact button { min-height: 44px; padding: 8px 12px; border-radius: 10px; border: 1px solid rgba(255, 255, 255, 0.35); background: rgba(255, 255, 255, 0.08); color: #fff; font-weight: 700; font-family: inherit; cursor: pointer; display: inline-flex; align-items: center; gap: 4px; }
  .vact .danger { border-color: #ff8a80; color: #ffb4ab; }
</style>
