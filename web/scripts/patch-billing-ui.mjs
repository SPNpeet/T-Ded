// เพิ่มหน้าแพ็กเกจให้ผู้ใช้ และแท็บลูกค้า/รายได้ให้แอดมิน
import { readFileSync, writeFileSync } from 'node:fs'
const edit = (file, pairs) => {
  let c = readFileSync(file, 'utf8')
  for (const [a, b] of pairs) {
    if (!c.includes(a)) throw new Error(file + ' missing: ' + a.slice(0, 70))
    c = c.replace(a, b)
  }
  writeFileSync(file, c)
}

edit('src/App.svelte', [
  ["import ServerSetup from './pages/ServerSetup.svelte'", "import ServerSetup from './pages/ServerSetup.svelte'\n  import Plan from './pages/Plan.svelte'"],
  ["{:else if seg(0) === 'server'}", "{:else if seg(0) === 'plan'}\n  <Plan />\n{:else if seg(0) === 'server'}"],
])

edit('src/pages/Menu.svelte', [
  ["['#/feed', 'feed', 'อาหารและโปรตีน / ผสมอาหารเอง'],", "['#/feed', 'feed', 'อาหารและโปรตีน / ผสมอาหารเอง'],\n    ['#/plan', 'star', 'แพ็กเกจการใช้งาน'],"],
])

edit('src/pages/Admin.svelte', [
  ["    ['line', 'LINE'],", "    ['line', 'LINE'],\n    ['customers', 'ลูกค้า/รายได้'],"],
  [
    "      if (sub === 'line') lineStatus = await api.get('/admin/line')",
    "      if (sub === 'line') lineStatus = await api.get('/admin/line')\n      if (sub === 'customers') {\n        customers = await api.get('/admin/subscriptions')\n        revenue = await api.get('/admin/revenue')\n      }",
  ],
  [
    '  let lineStatus: any = $state(null)',
    `  let lineStatus: any = $state(null)
  let customers: any[] = $state([])
  let revenue: any = $state(null)
  let planEdit: any = $state(null)
  let payEdit: any = $state(null)
  async function savePlan() {
    busy = true
    try {
      await api.put('/admin/subscriptions/' + planEdit.org_id, { plan: planEdit.plan, months: Number(planEdit.months) || 1, note: planEdit.note })
      toast('อัปเดตแพ็กเกจแล้ว', 'success')
      planEdit = null
      load()
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      busy = false
    }
  }
  async function savePayment() {
    busy = true
    try {
      await api.post('/admin/subscriptions/' + payEdit.org_id + '/payments', { amount: Number(payEdit.amount), months: Number(payEdit.months) || 1, method: payEdit.method, reference: payEdit.reference })
      toast('บันทึกการชำระเงินและต่ออายุแล้ว', 'success')
      payEdit = null
      load()
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      busy = false
    }
  }`,
  ],
  [
    "    {#if sub === 'line'}",
    `    {#if sub === 'customers'}
      {#if revenue}
        <div class="kpi mt">
          <div class="k"><div class="lbl">รายได้รวม (บาท)</div><div class="val">{n(revenue.total)}</div></div>
          <div class="k"><div class="lbl">เดือนนี้ (บาท)</div><div class="val">{n(revenue.this_month)}</div></div>
          <div class="k"><div class="lbl">รายได้ต่อเดือน (MRR)</div><div class="val">{n(revenue.mrr)}</div></div>
          <div class="k"><div class="lbl">ลูกค้าที่ใช้งานอยู่</div><div class="val">{n(revenue.active_orgs)}</div></div>
          <div class="k"><div class="lbl">ลูกค้าที่จ่ายเงิน</div><div class="val">{n(revenue.paying_orgs)}</div></div>
        </div>
      {/if}
      {#if planEdit}
        <div class="card mt">
          <h3>เปลี่ยนแพ็กเกจ: {planEdit.org_name}</h3>
          <label>แพ็กเกจ</label>
          <select bind:value={planEdit.plan}><option value="trial">ทดลองใช้</option><option value="free">ฟรี</option><option value="basic">มาตรฐาน 199</option><option value="pro">มืออาชีพ 590</option><option value="unlimited">ไม่จำกัด (หน่วยงาน)</option></select>
          <label>ให้สิทธิ์กี่เดือน</label><input type="number" bind:value={planEdit.months} />
          <label>หมายเหตุ</label><input bind:value={planEdit.note} />
          <div class="grid2 mt"><button class="btn primary" onclick={savePlan} disabled={busy}>บันทึก</button><button class="btn ghost" onclick={() => (planEdit = null)}>ยกเลิก</button></div>
        </div>
      {/if}
      {#if payEdit}
        <div class="card mt">
          <h3>บันทึกการชำระเงิน: {payEdit.org_name}</h3>
          <div class="grid2">
            <div><label>จำนวนเงิน (บาท)</label><input type="number" bind:value={payEdit.amount} /></div>
            <div><label>ต่ออายุกี่เดือน</label><input type="number" bind:value={payEdit.months} /></div>
            <div><label>ช่องทาง</label><select bind:value={payEdit.method}><option value="promptpay">พร้อมเพย์</option><option value="transfer">โอนธนาคาร</option><option value="cash">เงินสด</option></select></div>
            <div><label>เลขอ้างอิง</label><input bind:value={payEdit.reference} /></div>
          </div>
          <div class="grid2 mt"><button class="btn success" onclick={savePayment} disabled={busy}>บันทึกและต่ออายุ</button><button class="btn ghost" onclick={() => (payEdit = null)}>ยกเลิก</button></div>
        </div>
      {/if}
      <div class="card mt"><div class="table-wrap"><table><thead><tr><th>ลูกค้า</th><th>แพ็กเกจ</th><th>หมดอายุ</th><th class="num">ฟาร์ม</th><th class="num">จ่ายแล้ว</th><th></th></tr></thead><tbody>
        {#each customers as c}
          <tr>
            <td><b>{c.org_name}</b><div class="small muted">{c.contact_phone ?? '-'}</div></td>
            <td>{c.plan ?? 'trial'}{c.price_per_month ? ' (' + n(c.price_per_month) + ')' : ''}</td>
            <td>{c.expires_at ? thDate(c.expires_at) : '-'}</td>
            <td class="num">{c.farms}</td>
            <td class="num">{n(c.paid_total)}</td>
            <td style="white-space:nowrap">
              <button class="btn link" onclick={() => (planEdit = { org_id: c.org_id, org_name: c.org_name, plan: c.plan ?? 'basic', months: 12, note: '' })}>แพ็กเกจ</button>
              <button class="btn link" onclick={() => (payEdit = { org_id: c.org_id, org_name: c.org_name, amount: c.price_per_month || 199, months: 1, method: 'promptpay', reference: '' })}>รับเงิน</button>
            </td>
          </tr>
        {/each}
      </tbody></table></div></div>
    {/if}

    {#if sub === 'line'}`,
  ],
])
console.log('billing ui wired')
