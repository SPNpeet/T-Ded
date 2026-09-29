<script lang="ts">
  // วางโปรแกรมการเลี้ยงตามเป้าหมาย: เลือกสัตว์น้ำ -> ปล่อยปลาและเป้าหมาย -> เลือกอาหาร -> ได้โปรแกรมรายวัน
  import { onMount } from 'svelte'
  import { session, toast, go } from '../lib/ui.svelte'
  import { n, n1, n2, baht, todayISO, addDays, thDateShort, portionText } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'
  import LineChart from '../lib/LineChart.svelte'
  import Collapse from '../lib/Collapse.svelte'
  import { speciesList, planLocal, proteinFitLocal } from '../lib/engine'
  import { loadProducts, toEngineProduct, suits, sackColor, type Product } from '../lib/catalog'
  import { brand } from '../lib/brand'

  const DRAFT_KEY = 'teedet.plan.draft'
  let step = $state(1)
  let species: any[] = $state([])
  let products: Product[] = $state([])
  let code = $state('')
  let count = $state('5000')
  let stockW = $state('')
  let targetW = $state('')
  let months = $state<number | null>(null)
  let feedMode = $state<'auto' | 'custom'>('auto')
  let picked = $state<string[]>([])
  let prices = $state<Record<string, string>>({})
  let result: any = $state(null)
  let sel = $state(0)
  let busy = $state(false)
  let fits: Record<string, any> = $state({})

  onMount(async () => {
    species = await speciesList()
    products = await loadProducts()
    try {
      const d = JSON.parse(localStorage.getItem(DRAFT_KEY) || 'null')
      if (d) ({ code, count, stockW, targetW, months, feedMode, picked, prices } = { ...d, prices: d.prices ?? {} })
    } catch {
      /* ร่างเสีย เริ่มใหม่ */
    }
  })
  $effect(() => {
    const d = { code, count, stockW, targetW, months, feedMode, picked, prices }
    try {
      localStorage.setItem(DRAFT_KEY, JSON.stringify(d))
    } catch {
      /* เต็ม */
    }
  })

  const sp = $derived(species.find((s) => s.code === code))
  const forSpecies = $derived(products.filter((p) => code && suits(p, code)))
  const days = $derived(months ? Math.round(months * 30) : null)

  // ความเหมาะของโปรตีนแต่ละเบอร์กับขนาดปลาตอนปล่อย
  $effect(() => {
    const w = parseFloat(stockW)
    if (!code || !(w > 0)) return
    const out: Record<string, any> = {}
    Promise.all(forSpecies.map(async (p) => (out[p.product_code] = await proteinFitLocal(code, Math.max(w, p.weight_from_g || w), p.protein_pct)))).then(() => (fits = out))
  })

  function chooseSpecies(c: string) {
    code = c
    const s = species.find((x) => x.code === c)
    if (s && !targetW) targetW = String(s.market_weight_g)
    if (!stockW) stockW = c === 'catfish' ? '5' : '30'
    picked = []
    step = 2
  }

  function toggle(c: string) {
    picked = picked.includes(c) ? picked.filter((x) => x !== c) : [...picked, c]
  }

  async function calculate() {
    const miss: string[] = []
    if (!(parseInt(count) > 0)) miss.push('จำนวนปลาที่ปล่อย')
    if (!(parseFloat(stockW) > 0)) miss.push('น้ำหนักตอนปล่อย')
    if (!(parseFloat(targetW) > 0)) miss.push('น้ำหนักเป้าหมาย')
    if (miss.length) return toast('กรอก' + miss.join(', ') + 'ก่อนครับ', 'error')
    if (parseFloat(targetW) <= parseFloat(stockW)) return toast('น้ำหนักเป้าหมายต้องมากกว่าน้ำหนักตอนปล่อย', 'error')
    if (feedMode === 'custom' && !picked.length) return toast('เลือกอาหารอย่างน้อย 1 เบอร์ หรือเลือก "ให้ระบบเลือกให้"', 'error')
    busy = true
    try {
      const priceMap: Record<string, number> = {}
      for (const [k, v] of Object.entries(prices)) if (parseFloat(v) > 0) priceMap[k] = parseFloat(v)
      result = await planLocal({
        species: sp,
        stock_weight_g: parseFloat(stockW),
        count: parseInt(count),
        target_weight_g: parseFloat(targetW),
        target_days: days,
        products: products.map((p) => toEngineProduct(p, priceMap)),
        custom_codes: feedMode === 'custom' ? picked : [],
      })
      sel = result.recommended
      step = 4
      window.scrollTo(0, 0)
    } catch (e: any) {
      toast('คำนวณไม่สำเร็จ: ' + (e?.message ?? e), 'error')
    } finally {
      busy = false
    }
  }

  const COLORS = ['#1F4FA3', '#F28C28', '#1F9D5A', '#E23D8F']
  const series = $derived.by(() => {
    if (!result) return []
    return result.plans.map((p: any, i: number) => ({
      name: p.strategy_th + (i === result.recommended ? ' (แนะนำ)' : ''),
      color: COLORS[i % COLORS.length],
      dashed: i !== sel,
      points: p.days.filter((d: any, j: number) => j % 5 === 0 || j === p.days.length - 1).map((d: any) => ({ x: d.day, y: d.weight_g })),
    }))
  })
  const plan = $derived(result ? result.plans[sel] : null)
  const product = (c: string) => products.find((p) => p.product_code === c)
  const stageFeed = (s: any) => {
    const ds = plan.days.filter((d: any) => d.day >= s.from_day && d.day <= s.to_day)
    return { from: ds[0]?.feed_kg ?? 0, to: ds[ds.length - 1]?.feed_kg ?? 0, mealFrom: ds[0]?.per_meal_kg ?? 0, mealTo: ds[ds.length - 1]?.per_meal_kg ?? 0 }
  }
  const shopping = $derived.by(() => {
    if (!plan) return []
    const m = new Map<string, { code: string; bags: number; kg: number }>()
    for (const s of plan.stages) {
      const cur = m.get(s.product_code) ?? { code: s.product_code, bags: 0, kg: 0 }
      cur.bags += s.bags
      cur.kg += s.feed_kg
      m.set(s.product_code, cur)
    }
    return [...m.values()]
  })
  const fastest = $derived(result ? result.plans.reduce((a: any, b: any) => ((b.day_reached ?? 9999) < (a.day_reached ?? 9999) ? b : a)) : null)
  const slowest = $derived(result ? result.plans.reduce((a: any, b: any) => ((b.day_reached ?? 9999) > (a.day_reached ?? 9999) ? b : a)) : null)
  const weekly = $derived(plan ? plan.days.filter((d: any, i: number) => i % 7 === 0 || i === plan.days.length - 1) : [])

  function saveAsPond() {
    try {
      localStorage.setItem('teedet.plan.pending', JSON.stringify({ code, count: parseInt(count), stockW: parseFloat(stockW), targetW: parseFloat(targetW), days, plan: { strategy: plan.strategy, stages: plan.stages.map((s: any) => ({ product_code: s.product_code, from_day: s.from_day, to_day: s.to_day, meals_per_day: s.meals_per_day })), days: plan.days.map((d: any) => [d.day, d.weight_g, d.feed_kg, d.meals, d.product_code]) } }))
    } catch {
      /* ignore */
    }
    if (!session.user) {
      toast('เข้าสู่ระบบหรือเริ่มแบบเก็บในเครื่องก่อน แล้วระบบจะใช้โปรแกรมนี้กับบ่อให้', 'info', 4000)
      go('/login')
      return
    }
    go('/ponds?from=plan')
  }
</script>

<TopBar title="วางโปรแกรมการเลี้ยง" sub={brand.id === 'nb' ? 'อาหาร NB โตเท่าไร รู้ล่วงหน้า' : 'ตั้งเป้าแล้วรู้ว่าต้องให้อาหารอย่างไร'} back={session.user ? '/' : '/login'} />

<main class="page">
  <div class="steps" aria-label="ขั้นตอน">
    {#each ['สัตว์น้ำ', 'ปล่อยปลา/เป้าหมาย', 'อาหาร', 'โปรแกรม'] as label, i}
      <button class="step" class:on={step === i + 1} class:done={step > i + 1} onclick={() => (i + 1 < step || (i + 1 === 4 && result)) && (step = i + 1)}>
        <span class="num">{i + 1}</span><span class="lbl">{label}</span>
      </button>
    {/each}
  </div>

  {#if step === 1}
    <h2 class="mt2">เลี้ยงสัตว์น้ำอะไร</h2>
    <div class="stack mt">
      {#each species as s}
        <button class="card choice" class:active={code === s.code} onclick={() => chooseSpecies(s.code)}>
          <div class="row" style="justify-content:space-between">
            <b style="font-size:1.2rem">{s.name_th}</b>
            <span class="pill neutral">{products.filter((p) => suits(p, s.code)).length} เบอร์อาหาร</span>
          </div>
          <div class="small muted mt">ขนาดตลาดทั่วไป {n(s.market_weight_g)} ก./ตัว{s.approximate ? ' · ตารางการโตเป็นค่าประมาณ' : ''}</div>
        </button>
      {/each}
    </div>
  {/if}

  {#if step === 2 && sp}
    <h2 class="mt2">{sp.name_th}: ปล่อยปลาและเป้าหมาย</h2>
    <div class="card mt">
      <div class="grid2">
        <div><label for="cnt">จำนวนที่ปล่อย (ตัว)</label><input id="cnt" type="number" inputmode="numeric" bind:value={count} /></div>
        <div><label for="sw">น้ำหนักตอนปล่อย (กรัม/ตัว)</label><input id="sw" type="number" inputmode="decimal" bind:value={stockW} /></div>
      </div>
      <div class="chips mt">{#each [1, 5, 10, 30, 50] as g}<button class="chip" class:on={parseFloat(stockW) === g} onclick={() => (stockW = String(g))}>{g} ก.</button>{/each}</div>
    </div>
    <div class="card mt">
      <label for="tw">อยากได้ขนาดเท่าไร (กรัม/ตัว)</label>
      <input id="tw" type="number" inputmode="numeric" bind:value={targetW} />
      <div class="chips mt">{#each [300, 500, 800, 1000] as g}<button class="chip" class:on={parseFloat(targetW) === g} onclick={() => (targetW = String(g))}>{n(g)} ก.</button>{/each}</div>
      <label class="mt2">ภายในเวลาเท่าไร</label>
      <div class="chips">
        <button class="chip" class:on={months === null} onclick={() => (months = null)}>ไม่กำหนด (ดูว่ากี่วันถึง)</button>
        {#each [2, 3, 4, 6, 8] as m}<button class="chip" class:on={months === m} onclick={() => (months = m)}>{m} เดือน</button>{/each}
      </div>
    </div>
    <button class="btn primary mt2" onclick={() => (step = 3)}>ต่อไป: เลือกอาหาร</button>
  {/if}

  {#if step === 3 && sp}
    <h2 class="mt2">ใช้อาหารเบอร์ไหน</h2>
    <div class="segment mt">
      <button class:active={feedMode === 'auto'} onclick={() => (feedMode = 'auto')}>ให้ระบบเลือกให้ (แนะนำ)</button>
      <button class:active={feedMode === 'custom'} onclick={() => (feedMode = 'custom')}>ฉันเลือกเอง</button>
    </div>
    <p class="small muted mt">{feedMode === 'auto' ? 'ระบบจะเปลี่ยนเบอร์ตามขนาดปลาให้เอง และเทียบให้ดูว่าใช้เบอร์โปรตีนสูงกับเบอร์ประหยัดโตต่างกันเท่าไร' : 'แตะเลือกเบอร์ที่ใช้อยู่หรืออยากใช้ ระบบจะใช้ตามขนาดปลา และเทียบกับโปรแกรมที่แนะนำ'}</p>
    {#if !forSpecies.length}<div class="alert warn mt">ยังไม่มีอาหารในแคตตาล็อกที่ระบุว่าใช้กับ{sp.name_th}ได้</div>{/if}
    <div class="stack mt">
      {#each forSpecies as p}
        {@const f = fits[p.product_code]}
        <button class="card product" class:active={feedMode === 'custom' && picked.includes(p.product_code)} onclick={() => feedMode === 'custom' && toggle(p.product_code)} style="text-align:left">
          <div class="row" style="gap:12px;align-items:flex-start">
            <span class="sack" style="background:{sackColor(p.color_th) ?? '#DDE3EC'}" title={p.color_th ?? 'ยังไม่ทราบสีกระสอบ'}></span>
            <div style="flex:1;min-width:0">
              <div class="row" style="justify-content:space-between;gap:8px"><b>{p.sack_label_th ?? p.product_code}</b><span class="pill info">โปรตีน {p.protein_pct}%</span></div>
              {#if (p.sack_label_th ?? p.product_code) !== p.name_th}<div class="small muted">{p.name_th}</div>{/if}
              <div class="row wrap small" style="gap:6px;margin-top:6px">
                {#if p.pellet_mm}<span class="pill neutral">เม็ด {p.pellet_mm} มม.</span>{/if}
                {#if p.color_th}<span class="pill neutral">กระสอบสี{p.color_th}</span>{/if}
                <span class="pill neutral">โปรตีน {n1((p.bag_kg * p.protein_pct) / 100)} กก./กระสอบ</span>
                {#if f}<span class="pill {f.level === 'ok' ? 'good' : f.level === 'high' ? 'info' : 'warn'}">{f.level === 'ok' ? 'เหมาะกับขนาดนี้' : f.level === 'high' ? 'โปรตีนสูงกว่าที่ต้องการ' : 'โปรตีนต่ำกว่าที่ต้องการ'}</span>{/if}
              </div>
            </div>
            {#if feedMode === 'custom'}<span class="tick">{picked.includes(p.product_code) ? 'เลือกแล้ว' : ''}</span>{/if}
          </div>
        </button>
      {/each}
    </div>
    <div class="mt2">
      <Collapse title="ใส่ราคากระสอบ (ไม่บังคับ) เพื่อดูต้นทุนและโปรแกรมประหยัดที่สุด">
        <div class="grid2">
          {#each forSpecies as p}
            <div><label for={'pr' + p.id}>{p.sack_label_th ?? p.product_code} (บาท/กระสอบ)</label><input id={'pr' + p.id} type="number" inputmode="decimal" bind:value={prices[p.product_code]} placeholder={p.price_ref ? String(p.price_ref) : 'เช่น 550'} /></div>
          {/each}
        </div>
      </Collapse>
    </div>
    <button class="btn primary mt2" onclick={calculate} disabled={busy}>{busy ? 'กำลังคำนวณ...' : 'คำนวณโปรแกรมการเลี้ยง'}</button>
  {/if}

  {#if step === 4 && result && plan}
    <div class="card mt {result.plans[result.recommended].reached_target ? 'tint-green' : 'tint-amber'}" style="box-shadow:none">
      <b style="font-size:1.1rem">{result.message_th}</b>
    </div>

    {#if result.plans.length > 1 && fastest?.day_reached && slowest?.day_reached && fastest !== slowest}
      <div class="card mt">
        <h3>โปรตีนต่างกัน โตต่างกันเท่าไร</h3>
        <p class="mt">
          ใช้โปรแกรม <b>{fastest.strategy_th}</b> ถึง {n(parseFloat(targetW))} ก. ในวันที่ <b>{fastest.day_reached}</b>
          ส่วน <b>{slowest.strategy_th}</b> ถึงในวันที่ <b>{slowest.day_reached}</b> —
          เร็วกว่ากัน <b>{slowest.day_reached - fastest.day_reached} วัน</b>
        </p>
        <p class="small muted mt">เหมือนคนกินข้าวกับกินเวย์น้ำหนักเท่ากัน อิ่มเท่ากัน แต่เวย์สร้างกล้ามเนื้อได้มากกว่า อาหารปลาที่โปรตีนพอดีกับวัยจึงได้เนื้อมากกว่าจากอาหารปริมาณเท่ากัน แต่ถ้าโปรตีนเกินที่ปลาต้องการไปมาก จะโตเพิ่มขึ้นไม่มาก</p>
      </div>
    {/if}

    <div class="card mt">
      <h3>น้ำหนักปลาตามวัน</h3>
      <LineChart {series} height={220} xLabel={(x) => `วัน ${Math.round(x)}`} yLabel={(y) => `${Math.round(y)} ก.`} bands={[{ from: parseFloat(targetW) * 0.995, to: parseFloat(targetW) * 1.005, color: '#E23D8F' }]} />
      <p class="tiny muted mt">เส้นชมพู = น้ำหนักเป้าหมาย {n(parseFloat(targetW))} ก.{days ? ` · กำหนดภายใน ${days} วัน` : ''}</p>
    </div>

    <div class="tabs mt2">
      {#each result.plans as p, i}
        <button class:active={sel === i} onclick={() => (sel = i)}>{p.strategy_th}{i === result.recommended ? ' *' : ''}</button>
      {/each}
    </div>

    <div class="kpi mt">
      <div class="k"><div class="lbl">ถึงเป้าวันที่</div><div class="val">{plan.day_reached ?? 'ไม่ถึง'}</div><div class="tiny muted">{plan.day_reached ? thDateShort(addDays(todayISO(), plan.day_reached)) + ' ถ้าปล่อยวันนี้' : 'ขาด ' + n(plan.shortfall_g) + ' ก.'}</div></div>
      <div class="k"><div class="lbl">น้ำหนักตอนจบ (ก./ตัว)</div><div class="val">{n(plan.final_weight_g)}</div><div class="tiny muted">วันที่ {plan.final_day}</div></div>
      <div class="k"><div class="lbl">ใช้อาหารทั้งหมด</div><div class="val">{n1(plan.bags_total)} กระสอบ</div><div class="tiny muted">{n(plan.feed_kg_total)} กก.</div></div>
      <div class="k"><div class="lbl">อัตราแลกเนื้อ (FCR)</div><div class="val">{plan.fcr ?? '-'}</div><div class="tiny muted">อาหาร {plan.fcr ?? '-'} กก. ได้เนื้อ 1 กก.</div></div>
      <div class="k"><div class="lbl">ได้ปลา</div><div class="val">{n(plan.biomass_kg)} กก.</div><div class="tiny muted">{n(plan.final_count)} ตัว</div></div>
      {#if plan.cost_total != null}<div class="k"><div class="lbl">ค่าอาหาร</div><div class="val">{baht(plan.cost_total)}</div></div>{/if}
    </div>

    {#each plan.warnings_th as w}<div class="alert warn mt small">{w}</div>{/each}

    <h3 class="mt2">โปรแกรมการให้อาหาร</h3>
    <div class="stack mt">
      {#each plan.stages as s}
        {@const f = stageFeed(s)}
        {@const pd = product(s.product_code)}
        <div class="card flat">
          <div class="row" style="gap:12px;align-items:flex-start">
            <span class="sack" style="background:{sackColor(pd?.color_th) ?? '#DDE3EC'}"></span>
            <div style="flex:1;min-width:0">
              <div class="row" style="justify-content:space-between;gap:8px"><b>วันที่ {s.from_day}-{s.to_day}</b><span class="small muted">{n(s.from_weight_g)} → {n(s.to_weight_g)} ก.</span></div>
              <div class="mt" style="margin-top:4px"><b>{pd?.sack_label_th ?? s.product_code}</b> · โปรตีน {s.protein_pct}%{s.pellet_mm ? ` · เม็ด ${s.pellet_mm} มม.` : ''}{pd?.color_th ? ` · กระสอบสี${pd.color_th}` : ''}</div>
              <div class="small mt" style="margin-top:4px">ให้วันละ <b>{n1(f.from)}–{n1(f.to)} กก.</b> แบ่ง <b>{s.meals_per_day} มื้อ</b> ({s.feeding_times.join(' / ')}) มื้อละ {n1(f.mealFrom)}–{n1(f.mealTo)} กก. ({portionText(f.mealTo)})</div>
              <div class="small muted" style="margin-top:4px">ใช้ {n1(s.bags)} กระสอบ ({n(s.feed_kg)} กก.){s.cost != null ? ` · ${baht(s.cost)}` : ''} · ปลาต้องการโปรตีน {s.requirement_pct}% {s.growth_factor < 0.95 ? `· โตช้ากว่ามาตรฐาน ${n((1 - s.growth_factor) * 100)}%` : s.growth_factor > 1.005 ? `· โตเร็วกว่ามาตรฐาน ${n((s.growth_factor - 1) * 100)}%` : ''}</div>
            </div>
          </div>
        </div>
      {/each}
    </div>

    <div class="card mt">
      <h3>รายการอาหารที่ต้องซื้อทั้งรุ่น</h3>
      {#each shopping as it}
        {@const pd = product(it.code)}
        <div class="list-item"><span class="sack sm" style="background:{sackColor(pd?.color_th) ?? '#DDE3EC'}"></span><div class="main"><div class="title">{pd?.sack_label_th ?? it.code}</div><div class="sub">{pd?.name_th ?? ''}{pd?.bag_kg && pd.bag_kg !== 20 ? ` · ถุงละ ${pd.bag_kg} กก.` : ''}</div></div><b class="num">{Math.ceil(it.bags)} {pd?.bag_kg && pd.bag_kg < 10 ? 'ถุง' : 'กระสอบ'}</b></div>
      {/each}
      <p class="tiny muted mt">ปัดขึ้นเป็นกระสอบเต็ม คิดจากอัตรารอด 85% และน้ำหนักกระสอบตามฉลาก</p>
    </div>

    <div class="mt">
      <Collapse title="ตารางรายสัปดาห์ (สำหรับผู้ที่ต้องการรายละเอียด)">
        <div class="table-wrap"><table><thead><tr><th>วันที่</th><th class="num">น้ำหนัก (ก.)</th><th class="num">อาหาร/วัน (กก.)</th><th class="num">มื้อ</th><th>เบอร์</th></tr></thead><tbody>
          {#each weekly as d}<tr><td>{d.day}</td><td class="num">{n(d.weight_g)}</td><td class="num">{n2(d.feed_kg)}</td><td class="num">{d.meals}</td><td>{product(d.product_code)?.sack_label_th ?? d.product_code}</td></tr>{/each}
        </tbody></table></div>
      </Collapse>
    </div>

    <div class="grid2 mt2">
      <button class="btn success" onclick={saveAsPond}>ใช้โปรแกรมนี้กับบ่อของฉัน</button>
      <button class="btn ghost" onclick={() => (step = 2)}>ปรับเป้าหมาย</button>
    </div>
    <p class="tiny muted mt2">ตัวเลขเป็นการคาดการณ์จากตารางการโตมาตรฐานและความต้องการโปรตีนตามวัยของปลา อุณหภูมิ คุณภาพน้ำ และการจัดการจริงมีผลต่อผลลัพธ์ เมื่อบันทึกการชั่งน้ำหนักจริง ระบบจะปรับการคาดการณ์ตามบ่อของคุณ · คิดจากอาหารเม็ดอย่างเดียว บ่อดินที่ทำน้ำเขียวได้ ปลากินพืชได้อาหารธรรมชาติเสริม เบอร์โปรตีนต่ำจะโตได้ดีกว่าตัวเลขนี้</p>
  {/if}
</main>

<style>
  .steps { display: grid; grid-template-columns: repeat(4, 1fr); gap: 6px; }
  .step { display: flex; flex-direction: column; align-items: center; gap: 4px; background: none; border: none; padding: 6px 2px; color: var(--muted); font-weight: 700; font-size: 0.8rem; cursor: pointer; }
  .step .num { width: 34px; height: 34px; border-radius: 50%; display: flex; align-items: center; justify-content: center; background: #e6ebf3; color: var(--muted); font-size: 1rem; }
  .step.on .num { background: var(--navy); color: #fff; }
  .step.done .num { background: var(--green); color: #fff; }
  .step.on { color: var(--navy); }
  .step .lbl { text-align: center; line-height: 1.2; }
  .choice { width: 100%; text-align: left; border: 2px solid transparent; cursor: pointer; }
  .choice.active, .product.active { border-color: var(--cyan-deep); background: var(--cyan-tint); }
  .product { width: 100%; border: 2px solid var(--line); box-shadow: none; cursor: pointer; }
  .sack { width: 30px; height: 40px; border-radius: 6px; flex-shrink: 0; border: 2px solid rgba(0, 0, 0, 0.12); }
  .sack.sm { width: 22px; height: 30px; }
  .tick { font-weight: 700; color: var(--cyan-deep); font-size: 0.85rem; white-space: nowrap; }
</style>
