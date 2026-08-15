<script lang="ts">
  import { onMount } from 'svelte'
  import { api } from '../lib/api'
  import { toast, go, isStaff, session } from '../lib/ui.svelte'
  import { thDate, thDateTime, n, n2, baht, pct } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'
  import ScoreRing from '../lib/ScoreRing.svelte'

  let { sub = 'farms', id = '' }: { sub?: string; id?: string } = $props()
  let farms: any[] = $state([])
  let detail: any[] = $state([])
  let rules: any = $state(null)
  let species: any[] = $state([])
  let users: any[] = $state([])
  let audit: any[] = $state([])
  let ann = $state({ title: '', body: '' })
  let newUser = $state({ name: '', phone: '', pin: '', role: 'officer' })
  let busy = $state(false)
  let editSpecies: any = $state(null)
  let editRules: any[] = $state([])
  let filter = $state('')
  let products: any[] = $state([])
  let editProd: any = $state(null)
  async function saveProd() {
    busy = true
    try {
      if (editProd.id) await api.patch(`/feed-products/${editProd.id}`, editProd)
      else await api.post('/feed-products', editProd)
      toast('บันทึกสินค้าแล้ว', 'success')
      editProd = null
      load()
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      busy = false
    }
  }
  async function delProd(id: string) {
    if (!confirm('ซ่อนสินค้านี้จากรายการ?')) return
    await api.del(`/feed-products/${id}`)
    load()
  }

  async function load() {
    try {
      if (sub === 'farms') farms = await api.get('/admin/farms')
      if (sub === 'farm' && id) detail = await api.get(`/admin/farms/${id}`)
      if (sub === 'rules') {
        rules = await api.get('/admin/rules')
        editRules = JSON.parse(JSON.stringify(rules.rules))
      }
      if (sub === 'species') species = await api.get('/admin/species')
      if (sub === 'users') users = await api.get('/admin/users')
      if (sub === 'audit') audit = await api.get('/admin/audit?limit=200')
      if (sub === 'products') products = await api.get('/feed-products')
    } catch (e: any) {
      toast(e.message, 'error')
    }
  }
  onMount(load)
  $effect(() => {
    sub
    id
    load()
  })
  const tabs = [
    ['farms', 'ฟาร์ม'],
    ['rules', 'กติกาปรับ'],
    ['species', 'ตารางปลา'],
    ['products', 'ยี่ห้ออาหาร'],
    ['users', 'ผู้ใช้'],
    ['announce', 'ประกาศ'],
    ['audit', 'ประวัติแก้ไข'],
  ]
  async function saveRules() {
    busy = true
    try {
      await api.put('/admin/rules', { rules: editRules })
      toast('บันทึกกติกาแล้ว มีผลกับทุกฟาร์มในหน่วยงานทันที', 'success')
      load()
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      busy = false
    }
  }
  async function resetRules() {
    if (!confirm('กลับไปใช้กติกามาตรฐาน?')) return
    await api.put('/admin/rules', { rules: [] })
    load()
  }
  async function saveSpecies() {
    busy = true
    try {
      await api.put(`/admin/species/${editSpecies.code}`, { profile: editSpecies })
      toast('บันทึกตารางแล้ว', 'success')
      editSpecies = null
      load()
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      busy = false
    }
  }
  async function resetSpecies(code: string) {
    if (!confirm('กลับไปใช้ตารางมาตรฐาน?')) return
    await api.put(`/admin/species/${code}`, { reset: true })
    load()
  }
  async function sendAnn() {
    busy = true
    try {
      await api.post('/announcements', ann)
      toast('ส่งประกาศแล้ว (แสดงในหน้าวันนี้ของทุกฟาร์ม และ LINE ที่เชื่อมไว้)', 'success')
      ann = { title: '', body: '' }
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      busy = false
    }
  }
  async function createUser() {
    busy = true
    try {
      await api.post('/users', newUser)
      toast('สร้างผู้ใช้แล้ว', 'success')
      newUser = { name: '', phone: '', pin: '', role: 'officer' }
      load()
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      busy = false
    }
  }
  async function morning() {
    const r = await api.post('/admin/line/morning')
    toast(`ส่งสรุปเช้าไป ${r.sent} ข้อความ`, 'success')
  }
  const groupTh: Record<string, string> = { heat: 'ร้อน (tmax)', cold: 'หนาว (tmin)', rain: 'ฝน', cloud: 'เมฆ', oxygen: 'ออกซิเจน', ammonia: 'แอมโมเนีย', observation: 'สังเกตปลา' }
  const opTh: Record<string, string> = { gt: '>', gte: '>=', lt: '<', between: 'ระหว่าง', eq: '=' }
  const filtered = $derived(farms.filter((f) => !filter || f.name?.includes(filter) || f.owners?.includes(filter) || f.province?.includes(filter)))
</script>

<TopBar title="หลังบ้านหน่วยส่งเสริม" sub={session.user?.org_name ?? ''} back={sub === 'farm' ? '/admin' : '/'} />
<main class="page wide">
  {#if !isStaff()}
    <div class="alert danger">เฉพาะเจ้าหน้าที่</div>
  {:else}
    {#if sub !== 'farm'}
      <div class="tabs">{#each tabs as [k, l]}<button class:active={sub === k} onclick={() => go(`/admin/${k}`)}>{l}</button>{/each}</div>
    {/if}

    {#if sub === 'farms'}
      <div class="row mt"><input bind:value={filter} placeholder="ค้นหาชื่อฟาร์ม เจ้าของ จังหวัด" /><button class="btn ghost sm" onclick={morning}>ส่งสรุปเช้าตอนนี้</button></div>
      <div class="kpi mt">
        <div class="k"><div class="lbl">ฟาร์มทั้งหมด</div><div class="val">{farms.length}</div></div>
        <div class="k"><div class="lbl">ต้องติดตาม</div><div class="val" style="color:var(--amber)">{farms.filter((f) => f.needs_attention).length}</div></div>
        <div class="k"><div class="lbl">บ่อที่เลี้ยงอยู่</div><div class="val">{farms.reduce((a, f) => a + (f.active_crops || 0), 0)}</div></div>
      </div>
      {#each filtered as f}
        <a class="card mt" href="#/admin/farm/{f.id}" style="display:block;text-decoration:none;color:inherit;margin-top:10px">
          <div class="row" style="justify-content:space-between">
            <div><b>{f.name}</b><div class="small muted">{f.owners ?? '-'}{f.owner_phone ? ' · ' + f.owner_phone : ''}{f.province ? ' · ' + f.province : ''}</div></div>
            {#if f.needs_attention}<span class="pill warn">ต้องติดตาม</span>{:else}<span class="pill good">ปกติ</span>{/if}
          </div>
          <div class="row wrap small mt" style="gap:12px">
            <span>บ่อเลี้ยง {f.active_crops}</span>
            <span>บันทึกล่าสุด {f.last_log_date ? thDate(f.last_log_date) : 'ยังไม่เคย'}{f.days_silent != null && f.days_silent > 0 ? ` (${f.days_silent} วันก่อน)` : ''}</span>
            {#if f.health_avg != null}<span>สุขภาพเฉลี่ย {f.health_avg}</span>{/if}
          </div>
        </a>
      {/each}
    {/if}

    {#if sub === 'farm'}
      {#if !detail.length}<div class="card mt center muted">ฟาร์มนี้ยังไม่มีบ่อที่เลี้ยงอยู่</div>{/if}
      {#each detail as s}
        <div class="card mt">
          <div class="row" style="gap:14px">
            <ScoreRing score={s.health.score} size={80} label="สุขภาพ" />
            <div style="flex:1">
              <h3>{s.crop.farm_name} · {s.crop.pond_name}</h3>
              <div class="small muted">{s.species.name_th} วันที่ {s.day} · {n(s.alive_count)} ตัว · {n(s.avg_weight_g)} ก. · โต {s.growth.status_th}</div>
              <div class="small">อาหารวันนี้ {n2(s.recommendation.final_kg)} กก. · FCR {s.performance.fcr ?? '-'} · รอด {pct(s.performance.survival_pct, 1)} · ต้นทุน {baht(s.totals.cost_total)}</div>
              {#if s.projection}<div class="small">คาดจับอีก {s.projection.days_remaining} วัน กำไรคาด {baht(s.projection.profit)}</div>{/if}
            </div>
          </div>
          {#each s.alerts as a}<div class="alert {a.level === 'warn' ? 'warn' : 'info'} small mt">{a.text}</div>{/each}
          <a class="btn ghost sm mt" href="#/pond/{s.crop.id}">เปิดบ่อในมุมมองเกษตรกร</a>
        </div>
      {/each}
    {/if}

    {#if sub === 'rules' && rules}
      <p class="small muted mt">กติกาปรับอาหารตามอากาศ/น้ำ/การกิน ใช้กับทุกฟาร์มในหน่วยงาน แต่ละกลุ่มใช้กติกาแรกที่ตรง (เรียงบนลงล่าง) แล้วคูณทุกกลุ่มเข้าด้วยกัน ต่ำสุด ×0.40{rules.custom ? ` · แก้ไขล่าสุด ${thDateTime(rules.custom.updated_at)}` : ' · ใช้ค่ามาตรฐานอยู่'}</p>
      <div class="table-wrap card mt"><table><thead><tr><th>กลุ่ม</th><th>เงื่อนไข</th><th class="num">ค่า</th><th class="num">ตัวคูณ</th><th>คำอธิบาย</th></tr></thead><tbody>
        {#each editRules as r}
          <tr><td>{groupTh[r.group] ?? r.group}</td><td>{r.metric} {opTh[r.op] ?? r.op}</td><td class="num"><input type="number" step="0.1" bind:value={r.a} style="min-height:40px;width:90px;padding:4px 8px" /></td><td class="num"><input type="number" step="0.01" bind:value={r.factor} style="min-height:40px;width:90px;padding:4px 8px" /></td><td><input bind:value={r.label_th} style="min-height:40px;padding:4px 8px" /></td></tr>
        {/each}
      </tbody></table></div>
      <div class="grid2 mt"><button class="btn primary" onclick={saveRules} disabled={busy}>บันทึกกติกา</button><button class="btn ghost" onclick={resetRules}>กลับค่ามาตรฐาน</button></div>
    {/if}

    {#if sub === 'species'}
      {#each species as sp}
        <div class="card mt">
          <div class="row" style="justify-content:space-between"><h3>{sp.profile.name_th} <span class="small muted">({sp.profile.code})</span></h3>{#if sp.custom}<span class="pill info">ปรับแล้ว</span>{:else}<span class="pill neutral">มาตรฐาน</span>{/if}</div>
          {#if editSpecies?.code === sp.profile.code}
            <h4 class="mt">ตารางอัตราให้อาหาร</h4>
            <div class="table-wrap"><table><thead><tr><th>น้ำหนัก (ก.)</th><th>% ต่อวัน</th><th>เม็ด (มม.)</th><th></th></tr></thead><tbody>
              {#each editSpecies.feed_table as r, i}<tr><td><input type="number" bind:value={r.weight_g} style="min-height:40px;padding:4px 8px" /></td><td><input type="number" step="0.1" bind:value={r.pct} style="min-height:40px;padding:4px 8px" /></td><td><input type="number" step="0.5" bind:value={r.pellet_mm} style="min-height:40px;padding:4px 8px" /></td><td><button class="btn link" style="color:var(--red)" onclick={() => editSpecies.feed_table.splice(i, 1)}>ลบ</button></td></tr>{/each}
            </tbody></table></div>
            <button class="btn ghost sm mt" onclick={() => editSpecies.feed_table.push({ weight_g: (editSpecies.feed_table.at(-1)?.weight_g ?? 0) + 100, pct: 1.5, pellet_mm: 3 })}>เพิ่มแถว</button>
            <h4 class="mt">ตารางการโตมาตรฐาน</h4>
            <div class="table-wrap"><table><thead><tr><th>วันจาก</th><th>ถึง</th><th>น้ำหนัก (ก.)</th><th>ADG</th></tr></thead><tbody>
              {#each editSpecies.growth as g}<tr><td><input type="number" bind:value={g.day_from} style="min-height:40px;padding:4px 8px" /></td><td><input type="number" bind:value={g.day_to} style="min-height:40px;padding:4px 8px" /></td><td><input type="number" bind:value={g.weight_g} style="min-height:40px;padding:4px 8px" /></td><td><input type="number" step="0.1" bind:value={g.adg} style="min-height:40px;padding:4px 8px" /></td></tr>{/each}
            </tbody></table></div>
            <div class="grid3 mt">
              <div><label>น้ำหนักตลาด (ก.)</label><input type="number" bind:value={editSpecies.market_weight_g} /></div>
              <div><label>มื้อ/วัน</label><input type="number" bind:value={editSpecies.meals_per_day} /></div>
              <div><label>DO ต่ำสุด</label><input type="number" step="0.1" bind:value={editSpecies.water.do_min} /></div>
            </div>
            <div class="grid2 mt"><button class="btn primary" onclick={saveSpecies} disabled={busy}>บันทึก</button><button class="btn ghost" onclick={() => (editSpecies = null)}>ยกเลิก</button></div>
          {:else}
            <div class="small muted">ตารางอาหาร {sp.profile.feed_table.length} แถว · ตารางโต {sp.profile.growth.length} ช่วง · น้ำหนักตลาด {sp.profile.market_weight_g} ก.</div>
            <div class="grid2 mt"><button class="btn ghost sm" onclick={() => (editSpecies = JSON.parse(JSON.stringify(sp.profile)))}>แก้ไขตาราง</button>{#if sp.custom}<button class="btn ghost sm" onclick={() => resetSpecies(sp.profile.code)}>กลับค่ามาตรฐาน</button>{/if}</div>
          {/if}
        </div>
      {/each}
    {/if}

    {#if sub === 'products'}
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

    {#if sub === 'users'}
      <div class="card mt">
        <h3>เพิ่มเจ้าหน้าที่ / ผู้ใช้</h3>
        <div class="grid2">
          <div><label>ชื่อ</label><input bind:value={newUser.name} /></div>
          <div><label>เบอร์โทร</label><input type="tel" bind:value={newUser.phone} /></div>
          <div><label>PIN</label><input inputmode="numeric" bind:value={newUser.pin} /></div>
          <div><label>บทบาท</label><select bind:value={newUser.role}><option value="officer">เจ้าหน้าที่ส่งเสริม</option><option value="owner">เจ้าของฟาร์ม (ยังไม่ผูกฟาร์ม)</option>{#if session.user?.role === 'admin'}<option value="admin">ผู้ดูแลระบบ</option>{/if}</select></div>
        </div>
        <button class="btn primary mt" onclick={createUser} disabled={busy || !newUser.name || !newUser.phone || !newUser.pin}>สร้างผู้ใช้</button>
        <p class="tiny muted mt">รหัสหน่วยงานสำหรับให้เกษตรกรกรอกตอนสมัคร: <b>{session.user?.org_id}</b></p>
      </div>
      <div class="card mt"><div class="table-wrap"><table><thead><tr><th>ชื่อ</th><th>เบอร์</th><th>บทบาท</th><th>ฟาร์ม</th><th>LINE</th></tr></thead><tbody>
        {#each users as u}<tr><td>{u.name}</td><td>{u.phone}</td><td>{u.role}</td><td>{u.farms ?? '-'}</td><td>{u.line_linked ? 'เชื่อมแล้ว' : '-'}</td></tr>{/each}
      </tbody></table></div></div>
    {/if}

    {#if sub === 'announce'}
      <div class="card mt">
        <h3>ส่งประกาศถึงทุกฟาร์ม</h3>
        <label>หัวข้อ</label><input bind:value={ann.title} placeholder="เช่น เตือนอากาศหนาวสัปดาห์หน้า" />
        <label>ข้อความ</label><textarea bind:value={ann.body} placeholder="ลดอาหารช่วงเช้า เปิดตีน้ำ..."></textarea>
        <button class="btn primary mt" onclick={sendAnn} disabled={busy || !ann.title}>ส่งประกาศ</button>
      </div>
    {/if}

    {#if sub === 'audit'}
      <div class="card mt"><div class="table-wrap"><table><thead><tr><th>เวลา</th><th>ผู้ใช้</th><th>การกระทำ</th><th>รายการ</th></tr></thead><tbody>
        {#each audit as a}<tr><td>{thDateTime(a.at)}</td><td>{a.user_name ?? '-'}</td><td>{a.action}</td><td>{a.entity} {a.entity_id?.slice(0, 8)}</td></tr>{/each}
      </tbody></table></div></div>
    {/if}
  {/if}
</main>
