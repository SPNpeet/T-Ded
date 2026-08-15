<script lang="ts">
  import { onMount } from 'svelte'
  import { api } from '../lib/api'
  import { toast, go } from '../lib/ui.svelte'
  import { todayISO } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'
  import { speciesList } from '../lib/engine'

  let { pondId }: { pondId: string } = $props()
  let species: any[] = $state([])
  let code = $state('nile_tilapia')
  let stockedAt = $state(todayISO())
  let count = $state('')
  let weight = $state('30')
  let fryPrice = $state('')
  let target = $state('')
  let note = $state('')
  let busy = $state(false)
  onMount(async () => {
    species = await speciesList()
  })
  const sp = $derived(species.find((s) => s.code === code))
  async function save() {
    const miss: string[] = []
    if (!count || !(parseInt(count) > 0)) miss.push('จำนวนที่ปล่อย')
    if (!weight || !(parseFloat(weight) > 0)) miss.push('น้ำหนักเฉลี่ยตอนปล่อย')
    if (miss.length) return toast('กรอก' + miss.join(' และ ') + 'ก่อนครับ', 'error', 3500)
    busy = true
    try {
      const r = await api.post(`/ponds/${pondId}/crops`, { species_code: code, stocked_at: stockedAt, stocked_count: parseInt(count), stock_weight_g: parseFloat(weight), fry_price_each: fryPrice ? parseFloat(fryPrice) : 0, target_weight_g: target ? parseFloat(target) : null, note: note || null })
      toast('เริ่มรุ่นใหม่แล้ว ระบบจะคำนวณอาหารให้ทุกวัน', 'success')
      go(`/pond/${r.id}`)
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      busy = false
    }
  }
</script>

<TopBar title="ปล่อยปลารุ่นใหม่" back="/ponds" />
<main class="page">
  <label for="sp">ชนิดปลา</label>
  <select id="sp" bind:value={code}>{#each species as s}<option value={s.code}>{s.name_th}{s.approximate ? ' (ตารางโดยประมาณ)' : ''}</option>{/each}</select>
  <label>วันที่ปล่อย</label><input type="date" bind:value={stockedAt} max={todayISO()} />
  <div class="grid2">
    <div><label for="c">จำนวนที่ปล่อย (ตัว)</label><input id="c" type="number" inputmode="numeric" bind:value={count} placeholder="เช่น 5000" /></div>
    <div><label for="w">น้ำหนักเฉลี่ยตอนปล่อย (ก.)</label><input id="w" type="number" inputmode="decimal" bind:value={weight} /></div>
    <div><label for="fp">ราคาลูกปลา (บาท/ตัว)</label><input id="fp" type="number" inputmode="decimal" bind:value={fryPrice} placeholder="เช่น 2" /></div>
    <div><label for="tg">เป้าหมายจับ (ก./ตัว)</label><input id="tg" type="number" inputmode="numeric" bind:value={target} placeholder={sp ? String(sp.market_weight_g) : ''} /></div>
  </div>
  <label for="nt">หมายเหตุ</label><input id="nt" bind:value={note} placeholder="เช่น ลูกปลาจากฟาร์ม..." />
  {#if count && fryPrice}<div class="card tint-cyan mt small">ค่าลูกปลา {(parseInt(count) * parseFloat(fryPrice)).toLocaleString('th-TH')} บาท จะถูกบันทึกเป็นต้นทุนแรกของรุ่นอัตโนมัติ</div>{/if}
  <button class="btn success mt2" onclick={save} disabled={busy}>เริ่มรุ่นการเลี้ยง</button>
  <a class="btn link mt" href="#/simulate">อยากจำลองก่อนตัดสินใจ? เปิดตัวจำลองรุ่น</a>
</main>
