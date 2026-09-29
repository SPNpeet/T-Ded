<script lang="ts">
  import { onDestroy } from 'svelte'
  import Icon from './Icon.svelte'
  import { Locator, stampPhoto, stampLines, addPhoto, FRESH_MS, type Fix, type AddResult } from './photos'
  import { PHOTO_KINDS } from './pond'
  import { toast } from './ui.svelte'

  let { pond, farmName = '', firstPhoto = false, big = true, onsaved }: { pond: any; farmName?: string; firstPhoto?: boolean; big?: boolean; onsaved?: (r: AddResult) => void } = $props()

  let input: HTMLInputElement
  let locator: Locator | null = null
  let stopTimer: any = null
  let phase: 'idle' | 'working' | 'review' = $state('idle')
  let status = $state('')
  let file: File | null = null
  let takenAt: Date = new Date()
  let fresh = $state(true)
  let fix: Fix | null = $state(null)
  let gpsError = $state('')
  let shot: { image: Blob; thumb: Blob; width: number; height: number } | null = null
  let preview = $state('')
  let kind = $state('overview')
  let caption = $state('')
  let saving = $state(false)
  let retrying = $state(false)

  function stopLocator() {
    locator?.stop()
    locator = null
    clearTimeout(stopTimer)
  }
  onDestroy(() => {
    stopLocator()
    if (preview) URL.revokeObjectURL(preview)
  })

  function open() {
    // เริ่มจับ GPS ทันทีที่กด ระหว่างเล็งกล้องพิกัดจะแม่นขึ้น
    stopLocator()
    locator = new Locator().start()
    stopTimer = setTimeout(stopLocator, 3 * 60 * 1000)
    kind = firstPhoto ? 'overview' : kind
    input.value = ''
    input.click()
  }

  async function render() {
    if (!file) return
    const s = { pondName: pond.name, farmName, takenAt, fix, fresh }
    shot = await stampPhoto(file, s)
    if (preview) URL.revokeObjectURL(preview)
    preview = URL.createObjectURL(shot.image)
  }

  async function picked() {
    const f = input.files?.[0]
    if (!f) return
    if (!f.type.startsWith('image/')) return toast('เลือกได้เฉพาะรูปภาพ', 'error')
    file = f
    phase = 'working'
    const now = Date.now()
    const lm = f.lastModified && f.lastModified <= now ? f.lastModified : now
    fresh = now - lm <= FRESH_MS
    takenAt = new Date(lm)
    fix = null
    gpsError = ''
    try {
      if (fresh && locator) {
        status = 'กำลังจับพิกัด GPS ของบ่อ...'
        const got = await locator.wait(locator.best ? 2500 : 9000)
        // พิกัดต้องได้มาในช่วงเวลาเดียวกับที่ถ่าย
        fix = got && Math.abs(got.at - lm) <= FRESH_MS ? got : null
        gpsError = fix ? '' : locator.error || 'ยังจับสัญญาณ GPS ไม่ได้'
      }
      stopLocator()
      status = 'กำลังใส่วันเวลาและพิกัดลงบนรูป...'
      await render()
      phase = 'review'
    } catch (e: any) {
      phase = 'idle'
      toast('เปิดรูปไม่สำเร็จ: ' + (e?.message ?? e), 'error', 4000)
    }
  }

  async function retryGps() {
    retrying = true
    const l = new Locator().start()
    const got = await l.wait(12000)
    l.stop()
    retrying = false
    if (got) {
      fix = got
      gpsError = ''
      await render()
      toast(`ได้พิกัดแล้ว คลาดเคลื่อน ±${got.accuracy_m} ม.`, 'success')
    } else {
      gpsError = l.error || 'ยังจับสัญญาณ GPS ไม่ได้'
      toast(gpsError + ' ลองเปิด GPS/ตำแหน่งในเครื่อง แล้วยืนในที่โล่ง', 'error', 4500)
    }
  }

  async function save() {
    if (!shot) return
    saving = true
    try {
      const r = await addPhoto(pond.id, shot, { taken_at: takenAt.toISOString(), fix, kind, caption: caption.trim() || null })
      if (r.queued) toast('ยังไม่มีสัญญาณ เก็บรูปไว้ในเครื่องแล้ว จะส่งให้อัตโนมัติเมื่อมีเน็ต', 'info', 4500)
      else if (r.pond_located) toast('บันทึกรูปแล้ว และปักหมุดตำแหน่งบ่อจากรูปนี้ให้แล้ว', 'success', 4000)
      else toast('บันทึกรูปแล้ว', 'success')
      close()
      onsaved?.(r)
    } catch (e: any) {
      toast(e.message, 'error', 4000)
    } finally {
      saving = false
    }
  }

  function close() {
    phase = 'idle'
    caption = ''
    shot = null
    file = null
    if (preview) URL.revokeObjectURL(preview)
    preview = ''
  }
  function again() {
    close()
    open()
  }
  const lines = $derived(stampLines({ pondName: pond.name, farmName, takenAt, fix, fresh }))
</script>

<input bind:this={input} type="file" accept="image/*" capture="environment" onchange={picked} hidden />

{#if big}
  <button class="btn primary shoot" onclick={open} disabled={phase !== 'idle'}>
    <Icon name="camera" size={28} />
    <span><b>ถ่ายรูปบ่อ</b><small>วันเวลาและพิกัด GPS จะติดบนรูปให้เอง</small></span>
  </button>
{:else}
  <button class="btn ghost" onclick={open} disabled={phase !== 'idle'}><Icon name="camera" size={20} /> ถ่ายรูป</button>
{/if}

{#if phase !== 'idle'}
  <div class="sheet" role="dialog" aria-modal="true" aria-label="ตรวจรูปก่อนบันทึก">
    <div class="panel">
      {#if phase === 'working'}
        <div class="working"><div class="spin"></div><p>{status}</p></div>
      {:else}
        <div class="pic"><img src={preview} alt="รูปที่ถ่าย" /></div>
        <div class="facts">
          <div class="fact"><Icon name="calendar" size={18} /><span>{lines[0]}</span></div>
          <div class="fact" class:bad={!fix}><Icon name="pin" size={18} /><span>{fix ? `${fix.lat.toFixed(6)}, ${fix.lng.toFixed(6)} · คลาดเคลื่อน ±${fix.accuracy_m} ม.` : fresh ? `ไม่มีพิกัด: ${gpsError}` : 'รูปนี้ไม่ได้ถ่ายตอนนี้ จึงไม่ใส่พิกัด'}</span></div>
        </div>
        {#if !fix && fresh}
          <button class="btn ghost mt" onclick={retryGps} disabled={retrying}>{retrying ? 'กำลังจับพิกัด...' : 'ลองจับพิกัดอีกครั้ง'}</button>
        {:else if fix && fix.accuracy_m > 50}
          <div class="alert warn mt small">พิกัดยังคลาดเคลื่อนมาก (±{fix.accuracy_m} ม.) ถ้าจะใช้ปักหมุดบ่อ ยืนในที่โล่งแล้วกด "จับพิกัดใหม่"<button class="btn link" onclick={retryGps} disabled={retrying}>{retrying ? '...' : 'จับพิกัดใหม่'}</button></div>
        {/if}
        {#if !fresh}<div class="alert warn mt small">รูปนี้ถ่ายไว้ก่อนแล้ว ({lines[0]}) บนรูปจะแสดงเวลาที่ถ่ายจริง ถ้าต้องการพิกัดบ่อให้กด "ถ่ายใหม่" ที่ข้างบ่อ</div>{/if}
        <div class="small bold mt">รูปนี้คือ</div>
        <div class="chips mt">{#each PHOTO_KINDS as k}<button class="chip" class:on={kind === k.v} onclick={() => (kind = k.v)}>{k.th}</button>{/each}</div>
        <label for="cap" class="mt">บันทึกสั้น ๆ (ไม่ใส่ก็ได้)</label>
        <input id="cap" bind:value={caption} maxlength="200" placeholder="เช่น น้ำเริ่มเขียว, ตลิ่งด้านเหนือพัง" />
        <div class="grid2 mt">
          <button class="btn ghost" onclick={again} disabled={saving}>ถ่ายใหม่</button>
          <button class="btn success" onclick={save} disabled={saving}>{saving ? 'กำลังบันทึก...' : 'บันทึกรูปนี้'}</button>
        </div>
        <button class="btn link mt" onclick={close} disabled={saving}>ยกเลิก</button>
      {/if}
    </div>
  </div>
{/if}

<style>
  .shoot { display: flex; align-items: center; justify-content: center; gap: 12px; min-height: 72px; width: 100%; }
  .shoot span { display: flex; flex-direction: column; align-items: flex-start; line-height: 1.25; text-align: left; }
  .shoot b { font-size: 1.15rem; }
  .shoot small { font-weight: 500; opacity: 0.9; font-size: 0.85rem; }
  .sheet { position: fixed; inset: 0; z-index: 60; background: rgba(10, 18, 36, 0.62); display: flex; align-items: flex-end; justify-content: center; }
  .panel { background: var(--bg, #fff); width: 100%; max-width: 560px; max-height: 94vh; overflow-y: auto; border-radius: 20px 20px 0 0; padding: 14px 16px calc(16px + env(safe-area-inset-bottom)); box-sizing: border-box; }
  .pic { border-radius: 12px; overflow: hidden; background: #000; }
  .pic img { width: 100%; max-height: 52vh; object-fit: contain; display: block; }
  .facts { margin-top: 10px; display: grid; gap: 6px; }
  .fact { display: flex; gap: 8px; align-items: flex-start; font-weight: 600; font-size: 0.95rem; color: var(--navy); }
  .fact.bad { color: #7a4c00; }
  .fact :global(svg) { flex-shrink: 0; margin-top: 2px; }
  .working { padding: 40px 10px; text-align: center; font-weight: 600; }
  .spin { width: 40px; height: 40px; margin: 0 auto 14px; border-radius: 50%; border: 4px solid var(--line); border-top-color: var(--cyan-deep); animation: spin 0.9s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
  .alert :global(.btn.link) { padding: 0 0 0 6px; min-height: 0; display: inline; }
</style>
