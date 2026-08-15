<script lang="ts">
  import { submit } from '../lib/api'
  import { go, toast } from '../lib/ui.svelte'
  import { todayISO, EXPENSE_CATEGORIES } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'

  let { kind, cropId }: { kind: string; cropId: string } = $props()
  let date = $state(todayISO())
  let category = $state('medicine')
  let amount = $state('')
  let note = $state('')
  let kg = $state('')
  let count = $state('')
  let price = $state('')
  let buyer = $state('')
  let product = $state('')
  let dose = $state('')
  let withdrawal = $state('0')
  let symptom = $state('')
  let busy = $state(false)
  const titles: Record<string, string> = { expense: 'เพิ่มค่าใช้จ่าย', harvest: 'บันทึกการจับขาย', treatment: 'บันทึกยา / การรักษา' }

  async function save() {
    busy = true
    try {
      let r
      if (kind === 'expense') r = await submit('expense', cropId, { expense_date: date, category, amount: parseFloat(amount), note: note || null }, 'ค่าใช้จ่าย')
      else if (kind === 'harvest') r = await submit('harvest', cropId, { harvest_date: date, kg: parseFloat(kg), count: count ? parseInt(count) : null, price_per_kg: price ? parseFloat(price) : null, buyer: buyer || null, note: note || null }, 'การจับขาย')
      else r = await submit('treatment', cropId, { start_date: date, product, dose: dose || null, withdrawal_days: parseInt(withdrawal || '0'), symptom: symptom || null, note: note || null }, 'การรักษา')
      if (!r.queued) toast('บันทึกเรียบร้อย', 'success')
      go(`/pond/${cropId}/${kind === 'treatment' ? 'health' : 'money'}`)
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      busy = false
    }
  }
</script>

<TopBar title={titles[kind]} back="/pond/{cropId}/money" />
<main class="page">
  <label>วันที่</label>
  <input type="date" bind:value={date} max={todayISO()} />
  {#if kind === 'expense'}
    <label for="cat">ประเภท</label>
    <select id="cat" bind:value={category}>{#each EXPENSE_CATEGORIES as c}<option value={c[0]}>{c[1]}</option>{/each}</select>
    <label for="amt">จำนวนเงิน (บาท)</label>
    <input id="amt" type="number" inputmode="decimal" bind:value={amount} required />
  {:else if kind === 'harvest'}
    <div class="grid2">
      <div><label for="kg">น้ำหนักที่จับ (กก.)</label><input id="kg" type="number" inputmode="decimal" bind:value={kg} /></div>
      <div><label for="ct">จำนวนตัว <span class="hint">(ถ้ารู้)</span></label><input id="ct" type="number" inputmode="numeric" bind:value={count} /></div>
      <div><label for="pr">ราคาขาย/กก. (บาท)</label><input id="pr" type="number" inputmode="decimal" bind:value={price} /></div>
      <div><label for="by">ผู้ซื้อ</label><input id="by" bind:value={buyer} placeholder="เช่น แพปลา ตลาดสด" /></div>
    </div>
    {#if kg && price}<div class="card tint-green mt"><b>รายได้ {(parseFloat(kg) * parseFloat(price)).toLocaleString('th-TH')} บาท</b><div class="small">ราคานี้จะช่วยอัปเดตราคาตลาดในพื้นที่ให้เพื่อนเกษตรกรโดยไม่ระบุชื่อฟาร์ม</div></div>{/if}
  {:else}
    <label for="pd">ชื่อยา / สารเคมี</label>
    <input id="pd" bind:value={product} placeholder="เช่น เกลือ, ปูนขาว, ออกซีเตตร้าไซคลิน" />
    <div class="grid2">
      <div><label for="ds">ปริมาณ/วิธีใช้</label><input id="ds" bind:value={dose} placeholder="เช่น 3 กก./บ่อ" /></div>
      <div><label for="wd">ระยะหยุดยาก่อนจับ (วัน)</label><input id="wd" type="number" inputmode="numeric" bind:value={withdrawal} /></div>
    </div>
    <label for="sy">อาการที่พบ</label>
    <input id="sy" bind:value={symptom} placeholder="เช่น ตัวแดง เหงือกซีด ว่ายเอียง" />
    <div class="alert info small mt">ถ้าเป็นโรค แนะนำแจ้งไว้ในหน้าโรคในพื้นที่ด้วย เพื่อเตือนฟาร์มใกล้เคียง (ไม่ระบุชื่อฟาร์ม)</div>
  {/if}
  <label for="nt">หมายเหตุ</label>
  <input id="nt" bind:value={note} />
  <button class="btn success mt2" onclick={save} disabled={busy}>บันทึก</button>
</main>
