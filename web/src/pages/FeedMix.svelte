<script lang="ts">
  // ผสมอาหารเอง / เทียบโปรตีน / เคล็ดลับอาหาร — คำนวณในเครื่องด้วย engine ตัวเดียวกับ server
  import { onMount } from 'svelte'
  import { session } from '../lib/ui.svelte'
  import { n, n1, n2 } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'
  import { engine, speciesList } from '../lib/engine'
  import { cachedGet } from '../lib/api'

  let tab = $state<'stages' | 'brands' | 'mix' | 'pearson' | 'tips'>('stages')
  let products: any[] = $state([])
  let brandFilter = $state('')
  let species: any[] = $state([])
  let code = $state('nile_tilapia')
  let stages: any[] = $state([])
  let tips: [string, string][] = $state([])
  let ingredients: any[] = $state([])
  let batchKg = $state('100')
  let mix: any = $state(null)
  let pa = $state('58')
  let pb = $state('12')
  let target = $state('30')
  let pearson: [number, number] | null = $state(null)
  let checkW = $state('')
  let checkP = $state('')
  let checkMm = $state('')
  let checkPrice = $state('')
  let advice: any = $state(null)

  onMount(async () => {
    species = await speciesList()
    const e = await engine()
    ingredients = e.nutrition_ingredients() as any[]
    tips = e.nutrition_tips() as [string, string][]
    try { products = (await cachedGet('/feed-products')).data } catch { try { products = await fetch(import.meta.env.BASE_URL + 'feed-products.json').then((r) => r.json()) } catch {} }
  })
  const targetOf = (c: string) => (c === 'catfish' ? 'catfish' : 'tilapia')
  const brands = $derived([...new Set(products.map((p) => p.brand))])
  const shown = $derived(products.filter((p) => (p.target === targetOf(code) || p.target === 'all' || (targetOf(code) === 'tilapia' && p.target === 'herbivore')) && (!brandFilter || p.brand === brandFilter)))
  $effect(() => {
    engine().then((e) => (stages = e.nutrition_stages(code) as any[]))
  })
  $effect(() => {
    if (!ingredients.length) return
    const list = ingredients.map((i) => ({ ...i, protein_pct: +i.protein_pct || 0, price_per_kg: +i.price_per_kg || 0, share_pct: +i.share_pct || 0 }))
    engine().then((e) => (mix = e.nutrition_mix(list, batchKg ? parseFloat(batchKg) : undefined)))
  })
  $effect(() => {
    engine().then((e) => (pearson = e.nutrition_pearson(parseFloat(pa) || 0, parseFloat(pb) || 0, parseFloat(target) || 0) as any))
  })
  $effect(() => {
    const w = parseFloat(checkW)
    if (!(w > 0)) {
      advice = null
      return
    }
    engine().then((e) => (advice = e.nutrition_advise(code, w, 0, { brand: null, protein_pct: checkP ? parseFloat(checkP) : null, pellet_mm: checkMm ? parseFloat(checkMm) : null, price_per_kg: checkPrice ? parseFloat(checkPrice) : null, form: null })))
  })
  const addIng = () => ingredients.push({ name_th: 'วัตถุดิบใหม่', protein_pct: 0, price_per_kg: 0, share_pct: 0, max_share_pct: null })
</script>

<TopBar title="อาหารและโปรตีน" sub="ควรใช้อาหารแบบไหน ผสมเองได้ไหม" back={session.user ? '/menu' : '/login'} />
<main class="page">
  <div class="tabs">
    <button class:active={tab === 'stages'} onclick={() => (tab = 'stages')}>อาหารตามช่วง</button>
    <button class:active={tab === 'brands'} onclick={() => (tab = 'brands')}>ยี่ห้อในไทย</button>
    <button class:active={tab === 'mix'} onclick={() => (tab = 'mix')}>ผสมเอง</button>
    <button class:active={tab === 'pearson'} onclick={() => (tab = 'pearson')}>หาสัดส่วน</button>
    <button class:active={tab === 'tips'} onclick={() => (tab = 'tips')}>เคล็ดลับ</button>
  </div>

  {#if tab === 'stages'}
    <label for="sp">ชนิดปลา</label>
    <select id="sp" bind:value={code}>{#each species as s}<option value={s.code}>{s.name_th}</option>{/each}</select>
    <div class="card mt">
      <h3>เช็คอาหารที่ใช้อยู่ เหมาะกับปลาไหม</h3>
      <div class="grid2">
        <div><label for="cw">ปลาหนัก (ก./ตัว)</label><input id="cw" type="number" inputmode="decimal" bind:value={checkW} placeholder="เช่น 300" /></div>
        <div><label for="cp">โปรตีนอาหาร (%)</label><input id="cp" type="number" inputmode="decimal" bind:value={checkP} placeholder="ดูข้างถุง เช่น 30" /></div>
        <div><label for="cm">เม็ด (มม.)</label><input id="cm" type="number" inputmode="decimal" bind:value={checkMm} placeholder="เช่น 3" /></div>
        <div><label for="cpr">ราคา (บาท/กก.)</label><input id="cpr" type="number" inputmode="decimal" bind:value={checkPrice} placeholder="เช่น 28" /></div>
      </div>
      {#if advice}
        <div class="card mt {advice.status === 'ok' ? 'tint-green' : advice.status === 'unknown' ? 'tint-cyan' : 'tint-amber'}" style="box-shadow:none">
          <b>{advice.status_th}</b> — ช่วง{advice.stage.name_th}: โปรตีน {n(advice.stage.protein_min)}-{n(advice.stage.protein_max)}% เม็ด {advice.stage.pellet_mm} มม. {advice.stage.form_th} วันละ {advice.stage.meals_per_day} มื้อ ({advice.stage.feeding_times.join(', ')})
          {#each advice.messages_th as m}<div class="mt small">{m}</div>{/each}
          {#if advice.price_per_kg_protein}<div class="mt small">ราคาต่อโปรตีน 1 กก. = <b>{n2(advice.price_per_kg_protein)} บาท</b> (ใช้เทียบยี่ห้อ: ถูกกว่าต่อโปรตีน = คุ้มกว่า)</div>{/if}
        </div>
      {/if}
    </div>
    <div class="card mt">
      <h3>ตารางอาหารตามช่วงน้ำหนัก ({species.find((s) => s.code === code)?.name_th ?? ''})</h3>
      {#each stages as st}
        <div class="list-item" style="align-items:flex-start">
          <div class="main">
            <div class="title">{st.name_th} <span class="muted small">({n(st.weight_from_g)}{st.weight_to_g < 10000 ? `-${n(st.weight_to_g)}` : ' ขึ้นไป'} ก.)</span></div>
            <div class="row wrap small" style="gap:6px;margin-top:4px">
              <span class="pill info">โปรตีน {n(st.protein_min)}-{n(st.protein_max)}%</span>
              <span class="pill neutral">เม็ด {st.pellet_mm} มม. {st.form_th}</span>
              <span class="pill neutral">{st.meals_per_day} มื้อ: {st.feeding_times.join(' / ')}</span>
            </div>
            <div class="small muted" style="margin-top:4px">{st.note_th}</div>
          </div>
        </div>
      {/each}
    </div>
  {/if}

  {#if tab === 'brands'}
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
          {#if p.price_ref}<span class="pill pink">ราคาอ้างอิง {n(p.price_ref)} บาท{p.protein_pct ? ` (${n2(p.price_ref / p.bag_kg / (p.protein_pct / 100))} บาท/กก.โปรตีน)` : ''}</span>{/if}
        </div>
        {#if p.note}<div class="small muted mt">{p.note}</div>{/if}
      </div>
    {/each}
    {#if !shown.length}<div class="card mt center muted">ยังไม่มีข้อมูลยี่ห้อสำหรับตัวกรองนี้</div>{/if}
  {/if}

  {#if tab === 'mix'}
    <div class="card">
      <h3>สูตรผสมอาหารเอง</h3>
      <p class="small muted">แก้ตัวเลขได้ทุกช่อง (โปรตีน % ของวัตถุดิบดูจากผู้ขายหรือใช้ค่าประมาณนี้) ระบบรวมโปรตีนและต้นทุนให้ทันที</p>
      <div class="table-wrap mt"><table><thead><tr><th>วัตถุดิบ</th><th class="num">โปรตีน %</th><th class="num">บาท/กก.</th><th class="num">สัดส่วน %</th></tr></thead><tbody>
        {#each ingredients as ing, i}
          <tr>
            <td><input bind:value={ing.name_th} style="min-height:44px;padding:6px 8px;font-size:0.95rem" /></td>
            <td><input type="number" inputmode="decimal" bind:value={ing.protein_pct} style="min-height:44px;padding:6px 8px;width:80px" /></td>
            <td><input type="number" inputmode="decimal" bind:value={ing.price_per_kg} style="min-height:44px;padding:6px 8px;width:80px" /></td>
            <td><input type="number" inputmode="decimal" bind:value={ing.share_pct} style="min-height:44px;padding:6px 8px;width:80px" /></td>
          </tr>
        {/each}
      </tbody></table></div>
      <div class="row mt"><button class="btn ghost sm" onclick={addIng}>เพิ่มวัตถุดิบ</button><div class="spacer"></div><label for="bk" style="margin:0 8px 0 0">ผสมครั้งละ (กก.)</label><input id="bk" type="number" inputmode="decimal" bind:value={batchKg} style="width:110px" /></div>
    </div>
    {#if mix}
      <section class="hero mt">
        <div class="row" style="justify-content:space-between;align-items:flex-end">
          <div><div class="small">สูตรนี้ได้โปรตีน</div><div class="big-number">{n1(mix.protein_pct)}<small>%</small></div></div>
          <div class="right"><div class="small">ต้นทุน</div><div class="big-number" style="font-size:1.8rem">{n2(mix.cost_per_kg)}<small>บาท/กก.</small></div></div>
        </div>
        <div class="small mt">สัดส่วนรวม {mix.total_share_pct}% · ราคาต่อโปรตีน 1 กก. {mix.protein_pct > 0 ? n2(mix.cost_per_kg / (mix.protein_pct / 100)) : '-'} บาท (เทียบกับอาหารสำเร็จรูปในหน้า "อาหารตามช่วง")</div>
      </section>
      {#each mix.warnings_th as w}<div class="alert warn mt small">{w}</div>{/each}
      {#if mix.per_batch_kg?.length}
        <div class="card mt"><h3>ชั่งวัตถุดิบสำหรับ {batchKg} กก.</h3>
          {#each mix.per_batch_kg as [name, kg]}<div class="list-item"><div class="main title">{name}</div><b class="num">{n2(kg)} กก.</b></div>{/each}
        </div>
      {/if}
      <div class="alert info mt small">อาหารผสมเองมักเป็นเม็ดจม/นิ่ม ให้ครั้งละน้อย ดูให้กินหมด และควรมีปลาป่นหรือกากถั่วเหลืองเป็นแหล่งโปรตีนหลัก อย่าเก็บเกิน 3-5 วัน (ขึ้นราง่าย)</div>
    {/if}
  {/if}

  {#if tab === 'pearson'}
    <div class="card">
      <h3>หาสัดส่วนสองอย่างให้ได้โปรตีนที่ต้องการ (Pearson square)</h3>
      <div class="grid3">
        <div><label for="pa">โปรตีนสูง (%)</label><input id="pa" type="number" inputmode="decimal" bind:value={pa} /></div>
        <div><label for="pb">โปรตีนต่ำ (%)</label><input id="pb" type="number" inputmode="decimal" bind:value={pb} /></div>
        <div><label for="tg">เป้าหมาย (%)</label><input id="tg" type="number" inputmode="decimal" bind:value={target} /></div>
      </div>
      {#if pearson}
        <div class="card tint-green mt" style="box-shadow:none">
          <div class="big-number" style="font-size:1.6rem">{n1(pearson[0])}% : {n1(pearson[1])}%</div>
          <div class="small mt">ใช้วัตถุดิบโปรตีนสูง {n1(pearson[0])} ส่วน + โปรตีนต่ำ {n1(pearson[1])} ส่วน (ต่อ 100 กก. = {n1(pearson[0])} กก. + {n1(pearson[1])} กก.) จะได้โปรตีน {target}%</div>
        </div>
      {:else}
        <div class="alert warn mt small">เป้าหมายต้องอยู่ระหว่างโปรตีนของสองวัตถุดิบ</div>
      {/if}
      <p class="small muted mt">ตัวอย่าง: ปลาป่น 58% กับรำ 12% อยากได้ 30% = ปลาป่น 39 : รำ 61</p>
    </div>
  {/if}

  {#if tab === 'tips'}
    {#each tips as [title, body]}
      <div class="card mt"><h3>{title}</h3><p class="mt" style="line-height:1.7">{body}</p></div>
    {/each}
  {/if}
</main>
