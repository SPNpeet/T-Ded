// แพตช์ครั้งเดียว: เลือกยี่ห้ออาหารจากแคตตาล็อกตอนรับเข้าสต๊อก + แท็บยี่ห้อในหน้าอาหาร + แอดมินแก้แคตตาล็อก
import { readFileSync, writeFileSync } from 'node:fs'
const edit = (file, pairs) => {
  let c = readFileSync(file, 'utf8')
  for (const [a, b] of pairs) {
    if (!c.includes(a)) throw new Error(file + ' missing: ' + a.slice(0, 60))
    c = c.replace(a, b)
  }
  writeFileSync(file, c)
}

// ---- Stock: product picker ----
edit('src/pages/Stock.svelte', [
  ["  let busy = $state(false)\n  const farm = () => currentFarm()", "  let busy = $state(false)\n  let products: any[] = $state([])\n  let productId = $state('')\n  const farm = () => currentFarm()\n  function pickProduct(id: string) {\n    productId = id\n    const p = products.find((x) => x.id === id)\n    if (!p) return\n    f.brand = p.name_th\n    if (p.protein_pct != null) f.protein_pct = String(p.protein_pct)\n    if (p.pellet_mm != null) f.pellet_mm = String(p.pellet_mm)\n    if (p.form) f.form = p.form\n    if (p.bag_kg) f.bag_kg = String(p.bag_kg)\n    if (p.price_ref && f.bags) f.price_total = String(Math.round(p.price_ref * parseFloat(f.bags)))\n  }"],
  ["      data = (await cachedGet(`/farms/${farm()!.id}/stock`)).data\n      if (!f.bag_kg) f.bag_kg = String(data.bag_kg)", "      data = (await cachedGet(`/farms/${farm()!.id}/stock`)).data\n      if (!f.bag_kg) f.bag_kg = String(data.bag_kg)\n      try { products = (await cachedGet('/feed-products')).data } catch {}"],
  ["const body: any = { move_date: f.move_date, kind: f.kind, brand: f.brand || null,", "const body: any = { move_date: f.move_date, kind: f.kind, product_id: productId || null, brand: f.brand || null,"],
  ['        <label>วันที่</label><input type="date" bind:value={f.move_date} />', '        <label for="prod">เลือกจากยี่ห้อที่ขายในไทย <span class="hint">(ระบบใส่โปรตีน/เม็ด/ราคาให้)</span></label>\n        <select id="prod" value={productId} onchange={(e) => pickProduct((e.target as HTMLSelectElement).value)}>\n          <option value="">-- พิมพ์เองด้านล่าง หรือเลือกยี่ห้อ --</option>\n          {#each products as p}<option value={p.id}>{p.name_th} · โปรตีน {p.protein_pct ?? "?"}% · เม็ด {p.pellet_mm ?? "?"} มม.{p.verified ? "" : " (ตรวจฉลาก)"}</option>{/each}\n        </select>\n        <label>วันที่</label><input type="date" bind:value={f.move_date} />'],
])

// ---- FeedMix: brands tab ----
edit('src/pages/FeedMix.svelte', [
  ["  let tab = $state<'stages' | 'mix' | 'pearson' | 'tips'>('stages')", "  let tab = $state<'stages' | 'brands' | 'mix' | 'pearson' | 'tips'>('stages')\n  let products: any[] = $state([])\n  let brandFilter = $state('')"],
  ["    tips = e.nutrition_tips() as [string, string][]\n  })", "    tips = e.nutrition_tips() as [string, string][]\n    try { products = (await cachedGet('/feed-products')).data } catch {}\n  })\n  const targetOf = (c: string) => (c === 'catfish' ? 'catfish' : 'tilapia')\n  const brands = $derived([...new Set(products.map((p) => p.brand))])\n  const shown = $derived(products.filter((p) => (p.target === targetOf(code) || p.target === 'all' || (targetOf(code) === 'tilapia' && p.target === 'herbivore')) && (!brandFilter || p.brand === brandFilter)))"],
  ["  import { engine, speciesList } from '../lib/engine'", "  import { engine, speciesList } from '../lib/engine'\n  import { cachedGet } from '../lib/api'"],
  ["    <button class:active={tab === 'mix'} onclick={() => (tab = 'mix')}>ผสมเอง</button>", "    <button class:active={tab === 'brands'} onclick={() => (tab = 'brands')}>ยี่ห้อในไทย</button>\n    <button class:active={tab === 'mix'} onclick={() => (tab = 'mix')}>ผสมเอง</button>"],
  ["  {#if tab === 'mix'}", `  {#if tab === 'brands'}
    <div class="grid2">
      <div><label for="sp2">ชนิดปลา</label><select id="sp2" bind:value={code}>{#each species as s}<option value={s.code}>{s.name_th}</option>{/each}</select></div>
      <div><label for="br">ยี่ห้อ</label><select id="br" bind:value={brandFilter}><option value="">ทุกยี่ห้อ</option>{#each brands as b}<option value={b}>{b}</option>{/each}</select></div>
    </div>
    <p class="small muted mt">รวบรวมจากเว็บผู้ผลิต/ร้านค้า ณ ส.ค. 2569 สินค้าที่มีเครื่องหมาย "ตรวจฉลาก" คือค่าโดยประมาณ ให้ดูโปรตีนจากถุงจริง เจ้าหน้าที่แก้/เพิ่มได้ในหลังบ้าน</p>
    {#each shown as p}
      <div class="card mt">
        <div class="row" style="justify-content:space-between;align-items:flex-start">
          <div><b>{p.name_th}</b><div class="small muted">{p.brand}{p.stage_th ? ' · ' + p.stage_th : ''}</div></div>
          {#if !p.verified}<span class="pill warn">ตรวจฉลาก</span>{:else}<span class="pill good">ยืนยันแล้ว</span>{/if}
        </div>
        <div class="row wrap mt small" style="gap:6px">
          {#if p.protein_pct != null}<span class="pill info">โปรตีน {p.protein_pct}%</span>{/if}
          {#if p.pellet_mm != null}<span class="pill neutral">เม็ด {p.pellet_mm} มม.</span>{/if}
          {#if p.form}<span class="pill neutral">{p.form === 'floating' ? 'ลอยน้ำ' : p.form === 'sinking' ? 'จมน้ำ' : p.form === 'crumble' ? 'เม็ดเล็ก' : 'ผง'}</span>{/if}
          {#if p.weight_to_g}<span class="pill neutral">ปลา {n(p.weight_from_g ?? 0)}{p.weight_to_g < 10000 ? '-' + n(p.weight_to_g) : ' ขึ้นไป'} ก.</span>{/if}
          {#if p.bag_kg}<span class="pill neutral">ถุง {p.bag_kg} กก.</span>{/if}
          {#if p.price_ref}<span class="pill pink">ราคาอ้างอิง {n(p.price_ref)} บาท{p.protein_pct ? \` (\${n2(p.price_ref / p.bag_kg / (p.protein_pct / 100))} บาท/กก.โปรตีน)\` : ''}</span>{/if}
        </div>
        {#if p.note}<div class="small muted mt">{p.note}</div>{/if}
      </div>
    {/each}
    {#if !shown.length}<div class="card mt center muted">ยังไม่มีข้อมูลยี่ห้อสำหรับตัวกรองนี้</div>{/if}
  {/if}

  {#if tab === 'mix'}`],
])

// ---- Admin: catalog tab ----
edit('src/pages/Admin.svelte', [
  ["  let filter = $state('')", "  let filter = $state('')\n  let products: any[] = $state([])\n  let editProd: any = $state(null)\n  async function saveProd() {\n    busy = true\n    try {\n      if (editProd.id) await api.patch(`/feed-products/${editProd.id}`, editProd)\n      else await api.post('/feed-products', editProd)\n      toast('บันทึกสินค้าแล้ว', 'success')\n      editProd = null\n      load()\n    } catch (e: any) {\n      toast(e.message, 'error')\n    } finally {\n      busy = false\n    }\n  }\n  async function delProd(id: string) {\n    if (!confirm('ซ่อนสินค้านี้จากรายการ?')) return\n    await api.del(`/feed-products/${id}`)\n    load()\n  }"],
  ["      if (sub === 'audit') audit = await api.get('/admin/audit?limit=200')", "      if (sub === 'audit') audit = await api.get('/admin/audit?limit=200')\n      if (sub === 'products') products = await api.get('/feed-products')"],
  ["    ['species', 'ตารางปลา'],", "    ['species', 'ตารางปลา'],\n    ['products', 'ยี่ห้ออาหาร'],"],
  ["    {#if sub === 'users'}", `    {#if sub === 'products'}
      <button class="btn primary mt" onclick={() => (editProd = { brand: '', product_code: '', name_th: '', target: 'tilapia', stage_th: '', weight_from_g: 0, weight_to_g: 100000, protein_pct: 30, fat_pct: 4, pellet_mm: 3, form: 'floating', bag_kg: 20, price_ref: null, source_url: '', verified: 1, note: '' })}>เพิ่มสินค้าใหม่</button>
      {#if editProd}
        <div class="card mt">
          <h3>{editProd.id ? 'แก้ไขสินค้า' : 'สินค้าใหม่'}</h3>
          <div class="grid2">
            <div><label>ยี่ห้อ</label><input bind:value={editProd.brand} /></div>
            <div><label>เบอร์/รหัส</label><input bind:value={editProd.product_code} /></div>
          </div>
          <label>ชื่อที่แสดง</label><input bind:value={editProd.name_th} />
          <div class="grid3">
            <div><label>กลุ่มปลา</label><select bind:value={editProd.target}><option value="tilapia">ปลานิล/ทับทิม</option><option value="catfish">ปลาดุก</option><option value="herbivore">ปลากินพืช</option><option value="carnivore">ปลากินเนื้อ</option><option value="all">ทุกชนิด</option></select></div>
            <div><label>ช่วง (ข้อความ)</label><input bind:value={editProd.stage_th} /></div>
            <div><label>ชนิดเม็ด</label><select bind:value={editProd.form}><option value="floating">ลอยน้ำ</option><option value="sinking">จมน้ำ</option><option value="crumble">เม็ดเล็ก</option><option value="powder">ผง</option></select></div>
            <div><label>ปลาหนักจาก (ก.)</label><input type="number" bind:value={editProd.weight_from_g} /></div>
            <div><label>ถึง (ก.)</label><input type="number" bind:value={editProd.weight_to_g} /></div>
            <div><label>โปรตีน %</label><input type="number" step="0.5" bind:value={editProd.protein_pct} /></div>
            <div><label>ไขมัน %</label><input type="number" step="0.5" bind:value={editProd.fat_pct} /></div>
            <div><label>เม็ด (มม.)</label><input type="number" step="0.1" bind:value={editProd.pellet_mm} /></div>
            <div><label>ถุง (กก.)</label><input type="number" bind:value={editProd.bag_kg} /></div>
            <div><label>ราคาอ้างอิง/ถุง</label><input type="number" bind:value={editProd.price_ref} /></div>
            <div><label>ยืนยันจากฉลากแล้ว</label><select bind:value={editProd.verified}><option value={1}>ใช่</option><option value={0}>ยัง (ค่าประมาณ)</option></select></div>
          </div>
          <label>ที่มา (URL)</label><input bind:value={editProd.source_url} />
          <label>หมายเหตุ</label><input bind:value={editProd.note} />
          <div class="grid2 mt"><button class="btn primary" onclick={saveProd} disabled={busy}>บันทึก</button><button class="btn ghost" onclick={() => (editProd = null)}>ยกเลิก</button></div>
        </div>
      {/if}
      <div class="card mt"><div class="table-wrap"><table><thead><tr><th>ยี่ห้อ / สินค้า</th><th>กลุ่ม</th><th class="num">โปรตีน</th><th class="num">เม็ด</th><th class="num">ถุง</th><th class="num">ราคา</th><th>สถานะ</th><th></th></tr></thead><tbody>
        {#each products as p}<tr><td><b>{p.name_th}</b><div class="small muted">{p.brand}</div></td><td>{p.target}</td><td class="num">{p.protein_pct ?? '-'}%</td><td class="num">{p.pellet_mm ?? '-'}</td><td class="num">{p.bag_kg ?? '-'}</td><td class="num">{p.price_ref ?? '-'}</td><td>{p.verified ? 'ยืนยัน' : 'ประมาณ'}</td><td style="white-space:nowrap"><button class="btn link" onclick={() => (editProd = { ...p })}>แก้</button><button class="btn link" style="color:var(--red)" onclick={() => delProd(p.id)}>ซ่อน</button></td></tr>{/each}
      </tbody></table></div></div>
    {/if}

    {#if sub === 'users'}`],
])
console.log('ui patched')
