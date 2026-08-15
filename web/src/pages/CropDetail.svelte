<script lang="ts">
  import { onMount } from 'svelte'
  import { api, cachedGet } from '../lib/api'
  import { go, toast } from '../lib/ui.svelte'
  import { thDate, thDateShort, thDateTime, n, n1, n2, baht, bahtShort, pct, bandPill, expenseLabel, addDays, healthColor, portionText, bagLastsDays } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'
  import ScoreRing from '../lib/ScoreRing.svelte'
  import LineChart from '../lib/LineChart.svelte'
  import Icon from '../lib/Icon.svelte'
  import MoneyBars from '../lib/MoneyBars.svelte'
  import Timeline from '../lib/Timeline.svelte'
  import Collapse from '../lib/Collapse.svelte'
  import { speciesByCode } from '../lib/engine'

  let { cropId, tab = 'feed' }: { cropId: string; tab?: string } = $props()
  let s: any = $state(null)
  let err = $state('')
  let logs: any[] = $state([])
  let weighings: any[] = $state([])
  let water: any[] = $state([])
  let expenses: any[] = $state([])
  let harvests: any[] = $state([])
  let treatments: any[] = $state([])
  let species: any = $state(null)
  let sellPrice = $state('')
  let feedPrice = $state('')
  let otherCost = $state('')
  let projBusy = $state(false)

  async function load() {
    try {
      const r = await cachedGet(`/crops/${cropId}/today`)
      s = r.data
      err = ''
      species = await speciesByCode(s.species.code)
      if (!sellPrice && s.market_price_per_kg) sellPrice = String(s.market_price_per_kg)
      if (!feedPrice && s.stock?.avg_price_per_kg) feedPrice = String(s.stock.avg_price_per_kg)
    } catch (e: any) {
      err = e.message
    }
  }
  async function loadTab(t: string) {
    try {
      if (t === 'history' && !logs.length) logs = (await cachedGet(`/crops/${cropId}/logs?limit=120`)).data
      if (t === 'growth' && !weighings.length) weighings = (await cachedGet(`/crops/${cropId}/weighings`)).data
      if (t === 'water' && s && !water.length) water = (await cachedGet(`/ponds/${s.crop.pond_id}/water?limit=60`)).data
      if (t === 'money') {
        if (!expenses.length) expenses = (await cachedGet(`/crops/${cropId}/expenses`)).data
        if (!harvests.length) harvests = (await cachedGet(`/crops/${cropId}/harvests`)).data
        if (!treatments.length) treatments = (await cachedGet(`/crops/${cropId}/treatments`)).data
      }
    } catch (e: any) {
      toast(e.message, 'error')
    }
  }
  onMount(load)
  $effect(() => {
    if (s) loadTab(tab)
  })
  const setTab = (t: string) => go(`/pond/${cropId}/${t}`)

  async function reproject() {
    projBusy = true
    try {
      const q = new URLSearchParams({ weather: '1', forecast: '1' })
      if (sellPrice) q.set('sell_price', sellPrice)
      if (feedPrice) q.set('feed_price', feedPrice)
      if (otherCost) q.set('other_cost_per_day', otherCost)
      s = await api.get(`/crops/${cropId}/today?${q}`)
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      projBusy = false
    }
  }
  async function closeCrop() {
    if (!confirm('ปิดรุ่นนี้? ควรบันทึกการจับให้ครบก่อน ปิดแล้วบ่อจะว่างพร้อมปล่อยรุ่นใหม่')) return
    try {
      await api.post(`/crops/${cropId}/close`)
      toast('ปิดรุ่นเรียบร้อย', 'success')
      go('/ponds')
    } catch (e: any) {
      toast(e.message, 'error')
    }
  }

  const growthSeries = $derived.by(() => {
    if (!s || !species) return []
    const stocked = s.crop.stocked_at
    const actual = weighings.map((w) => ({ x: Math.max(0, Math.round((new Date(w.weigh_date).getTime() - new Date(stocked).getTime()) / 86400000)), y: w.avg_weight_g }))
    const std: { x: number; y: number }[] = []
    const maxDay = Math.max(s.day + 30, ...actual.map((a) => a.x), 60)
    const pts: [number, number][] = species.growth.map((g: any) => [g.day_from, g.weight_g])
    const last = species.growth[species.growth.length - 1]
    pts.push([last.day_to, last.weight_g + last.adg * (last.day_to - last.day_from)])
    const wAt = (d: number) => {
      if (d <= pts[0][0]) return pts[0][1]
      for (let i = 0; i < pts.length - 1; i++) if (d >= pts[i][0] && d <= pts[i + 1][0]) return pts[i][1] + ((d - pts[i][0]) / (pts[i + 1][0] - pts[i][0])) * (pts[i + 1][1] - pts[i][1])
      return pts[pts.length - 1][1] + (d - pts[pts.length - 1][0]) * last.adg
    }
    const dAt = (w: number) => {
      if (w <= pts[0][1]) return pts[0][0]
      for (let i = 0; i < pts.length - 1; i++) if (w >= pts[i][1] && w <= pts[i + 1][1]) return pts[i][0] + ((w - pts[i][1]) / (pts[i + 1][1] - pts[i][1])) * (pts[i + 1][0] - pts[i][0])
      return pts[pts.length - 1][0] + (w - pts[pts.length - 1][1]) / last.adg
    }
    const d0 = dAt(s.crop.stock_weight_g)
    for (let d = 0; d <= maxDay; d += 7) std.push({ x: d, y: wAt(d0 + d) })
    return [
      { name: 'มาตรฐาน', color: '#9aa7b8', points: std, dashed: true },
      { name: 'ปลาของคุณ', color: '#0e8ea7', points: actual },
    ]
  })
  const waterSeries = $derived.by(() => {
    const pts = [...water].reverse()
    const base = pts.length ? new Date(pts[0].measured_at).getTime() : 0
    const mk = (k: string, name: string, color: string) => ({ name, color, points: pts.filter((w) => w[k] != null).map((w) => ({ x: Math.round((new Date(w.measured_at).getTime() - base) / 86400000), y: w[k] })) })
    return { do: [mk('do_mg_l', 'ออกซิเจน', '#0e8ea7')], temp: [mk('temp_c', 'อุณหภูมิน้ำ', '#d0850e')], ph: [mk('ph', 'pH', '#e23d8f')] }
  })
  const feedSeries = $derived.by(() => {
    const pts = [...logs].reverse()
    const base = pts.length ? new Date(pts[0].log_date).getTime() : 0
    const x = (d: string) => Math.round((new Date(d).getTime() - base) / 86400000)
    return [
      { name: 'แนะนำ', color: '#9aa7b8', points: pts.filter((l) => l.recommended_kg != null).map((l) => ({ x: x(l.log_date), y: l.recommended_kg })), dashed: true },
      { name: 'ให้จริง', color: '#1f9d5a', points: pts.filter((l) => l.fed_kg != null).map((l) => ({ x: x(l.log_date), y: l.fed_kg })) },
    ]
  })
</script>

<TopBar title={s ? `${s.crop.pond_name} · ${s.species.name_th}` : 'บ่อ'} sub={s ? `รุ่นวันที่ ${s.day} · ปล่อย ${thDate(s.crop.stocked_at)}` : ''} back="/" />

<main class="page">
  {#if err && !s}
    <div class="alert danger">{err}</div>
  {:else if !s}
    <div class="stack"><div class="skeleton" style="min-height:160px"></div><div class="skeleton"></div></div>
  {:else}
    {@const rec = s.recommendation}
    {@const bp = bandPill(rec.band)}
    <div class="tabs">
      <button class:active={tab === 'feed'} onclick={() => setTab('feed')}>อาหาร</button>
      <button class:active={tab === 'health'} onclick={() => setTab('health')}>สุขภาพ</button>
      <button class:active={tab === 'growth'} onclick={() => setTab('growth')}>การโต</button>
      <button class:active={tab === 'water'} onclick={() => setTab('water')}>น้ำ</button>
      <button class:active={tab === 'money'} onclick={() => setTab('money')}>เงิน</button>
      <button class:active={tab === 'history'} onclick={() => setTab('history')}>ประวัติ</button>
    </div>

    {#if tab === 'feed'}
      <section class="hero mt">
        <div class="muted small">อาหารที่แนะนำวันนี้ {thDate(s.date)}</div>
        <div class="big-number">{n2(rec.final_kg)}<small>กก./วัน</small></div>
        <div class="mt small">แบ่ง {rec.meals_per_day} มื้อ มื้อละ <b>{n2(rec.per_meal_kg)} กก.</b> ({portionText(rec.per_meal_kg)}) · เม็ด {rec.pellet_mm} มม. · {rec.pct}% ของน้ำหนักตัว</div>
        <div class="small" style="margin-top:6px">อาหาร 1 กระสอบ ({s.crop.bag_kg ?? 20} กก.) ใช้ได้ประมาณ <b>{n1(bagLastsDays(rec.final_kg, s.crop.bag_kg ?? 20) ?? 0)} วัน</b> ที่อัตรานี้{s.stock?.balance_kg ? ` · ในสต๊อกเหลือ ${n1(s.stock.balance_kg)} กก. พอถึงอีก ${n(Math.floor(s.stock.balance_kg / Math.max(0.01, rec.final_kg)))} วัน` : ''}</div>
        <div class="row wrap mt small" style="gap:8px">
          <span class="pill {bp.cls}">{rec.headline_th}</span>
          {#if rec.factor !== 1}<span class="pill neutral">ฐาน {n2(rec.base_kg)} กก. × {rec.factor}</span>{/if}
        </div>
      </section>
      {#if s.today_log?.fed_kg != null}
        <div class="alert good mt"><Icon name="check" />วันนี้บันทึกให้แล้ว {n2(s.today_log.fed_kg)} กก.{s.today_log.mortality ? ` · ตาย ${s.today_log.mortality} ตัว` : ''}</div>
      {/if}
      <div class="grid2 mt">
        <a class="btn success" href="#/log/{cropId}">{s.today_log?.fed_kg != null ? 'แก้ไขบันทึกวันนี้' : 'บันทึกให้อาหารวันนี้'}</a>
        <a class="btn ghost" href="#/weigh/{cropId}">สุ่มชั่งน้ำหนัก</a>
      </div>

      <div class="card mt2">
        <div class="card-title"><h3>ทำไมถึงแนะนำเท่านี้</h3></div>
        <div class="reason"><span>น้ำหนักเฉลี่ย {n(s.avg_weight_g)} ก. × {n(s.alive_count)} ตัว = ชีวมวล {n1(rec.biomass_kg)} กก.</span></div>
        <div class="reason"><span>ตาราง{s.species.name_th}: ปลา {n(s.avg_weight_g)} ก. ให้ {rec.pct}% ต่อวัน = {n2(rec.base_kg)} กก.</span></div>
        {#each rec.reasons as r}
          <div class="reason"><b>× {r.factor.toFixed(2)}</b><span>{r.text_th}</span></div>
        {/each}
        {#if rec.temp_optimal}<div class="reason"><b>× 1.00</b><span>อุณหภูมิเหมาะสมทั้งวัน ปลากินดี</span></div>{/if}
        {#if !rec.reasons.length && !rec.temp_optimal}<div class="reason"><span class="muted">ไม่มีข้อมูลอากาศหรือน้ำที่ต้องปรับ ให้ตามมาตรฐาน{!s.weather ? ' (ตั้งพิกัดฟาร์มในหน้าตั้งค่าเพื่อดึงอากาศอัตโนมัติ)' : ''}</span></div>{/if}
        {#each rec.warnings as w}<div class="alert warn small mt">{w}</div>{/each}
        {#if s.avg_weight_source === 'estimated'}
          <p class="small muted mt">น้ำหนักวันนี้ประมาณจากการชั่งล่าสุด {thDate(s.last_weighed.date)} ({n(s.last_weighed.avg_weight_g)} ก.) เมื่อ {s.last_weighed.days_ago} วันก่อน ชั่งใหม่ทุก 1-2 สัปดาห์เพื่อความแม่นยำ</p>
        {/if}
      </div>

      {#if s.nutrition}
        {@const nu = s.nutrition}
        <div class="card mt {nu.status === 'ok' ? 'tint-green' : nu.status === 'unknown' ? '' : 'tint-amber'}">
          <div class="card-title"><h3>อาหารที่ควรใช้ตอนนี้ ({nu.stage.name_th})</h3><a class="btn link" href="#/feed">ดูตาราง</a></div>
          <div class="row wrap" style="gap:6px">
            <span class="pill info">โปรตีน {n(nu.stage.protein_min)}-{n(nu.stage.protein_max)}%</span>
            <span class="pill neutral">เม็ด {nu.stage.pellet_mm} มม. {nu.stage.form_th}</span>
            <span class="pill neutral">{nu.stage.meals_per_day} มื้อ: {nu.stage.feeding_times.join(' / ')}</span>
          </div>
          <div class="mt small">{nu.stage.note_th}</div>
          {#if s.feed_on_hand?.protein_pct}
            <div class="mt"><b>อาหารในสต๊อก:</b> {s.feed_on_hand.brand ?? ''} โปรตีน {n(s.feed_on_hand.protein_pct)}%{s.feed_on_hand.pellet_mm ? ` เม็ด ${s.feed_on_hand.pellet_mm} มม.` : ''} — <b>{nu.status_th}</b></div>
          {/if}
          {#each nu.messages_th as m}<div class="small mt">{m}</div>{/each}
          {#if nu.price_per_kg_protein}<div class="small muted mt">ราคาต่อโปรตีน 1 กก. {n2(nu.price_per_kg_protein)} บาท · ปลาได้โปรตีนวันละ {n2(nu.protein_intake_kg_day)} กก.</div>{/if}
        </div>
      {/if}
      {#if s.weather}
        <div class="card mt">
          <div class="card-title"><h3>สภาพอากาศวันนี้</h3><span class="tiny muted">{s.weather.source?.includes('archive') ? 'ข้อมูลจริง' : 'พยากรณ์'} Open-Meteo</span></div>
          <div class="kpi">
            <div class="k"><div class="lbl">สูงสุด</div><div class="val">{n(s.weather.tmax_c)}°</div></div>
            <div class="k"><div class="lbl">ต่ำสุด</div><div class="val">{n(s.weather.tmin_c)}°</div></div>
            <div class="k"><div class="lbl">ฝน</div><div class="val">{n(s.weather.rain_mm)} มม.</div></div>
            <div class="k"><div class="lbl">เมฆ</div><div class="val">{n(s.weather.cloud_pct)}%</div></div>
          </div>
        </div>
      {/if}
      {#each s.alerts as a}
        <div class="alert {a.level === 'warn' ? 'warn' : 'info'} mt"><Icon name="alert" />{a.text}</div>
      {/each}
    {/if}

    {#if tab === 'health'}
      {@const h = s.health}
      <div class="card mt">
        <div class="row" style="gap:16px">
          <ScoreRing score={h.score} label="เต็ม 100" size={110} />
          <div style="flex:1">
            <h2 style="color:{healthColor(h.score)}">{h.grade_th}</h2>
            <div class="small muted">แนวโน้ม: {h.trend === 'up' ? 'ดีขึ้น' : h.trend === 'down' ? 'แย่ลง' : h.trend === 'flat' ? 'คงที่' : 'ยังไม่มีข้อมูลเทียบ'}</div>
            <div class="small muted">ความครบของข้อมูล {h.data_completeness}%</div>
          </div>
        </div>
        <div class="divider"></div>
        {#each h.components as c}
          <div class="mt">
            <div class="row" style="justify-content:space-between"><b>{c.label_th}</b><span class="small muted">{c.note_th}</span></div>
            <div class="progress {c.score >= 0.8 ? 'green' : c.score >= 0.5 ? '' : 'amber'}"><div style="width:{Math.round(c.score * 100)}%"></div></div>
          </div>
        {/each}
        {#if !h.components.length}<p class="muted">ยังไม่มีข้อมูลพอให้คะแนน บันทึกอาหาร ตาย และค่าน้ำแล้วกลับมาดู</p>{/if}
      </div>
      <div class="card mt">
        <h3>การแจ้งเตือนสำคัญวันนี้</h3>
        {#if h.alerts_th.length}
          {#each h.alerts_th as a}<div class="alert warn mt small">{a}</div>{/each}
        {:else}
          <p class="muted mt">ไม่มีเรื่องเร่งด่วน</p>
        {/if}
      </div>
      <div class="kpi mt">
        <div class="k"><div class="lbl">อัตรารอด</div><div class="val">{pct(s.performance.survival_pct, 1)}</div></div>
        <div class="k"><div class="lbl">ตายสะสม</div><div class="val">{n(s.totals.dead)} ตัว</div></div>
        <div class="k"><div class="lbl">มีชีวิต</div><div class="val">{n(s.alive_count)} ตัว</div></div>
      </div>
      <div class="grid2 mt2">
        <a class="btn ghost" href="#/water/{s.crop.pond_id}">จดค่าน้ำ</a>
        <a class="btn ghost" href="#/treatment/{cropId}">บันทึกยา/การรักษา</a>
      </div>
      <a class="btn ghost mt" href="#/diseases">แจ้ง/ดูโรคในพื้นที่</a>
    {/if}

    {#if tab === 'growth'}
      {@const g = s.growth}
      <div class="card mt">
        <div class="row" style="justify-content:space-between;align-items:flex-start">
          <div>
            <div class="small muted">น้ำหนักเฉลี่ยล่าสุด ({thDate(s.last_weighed.date)})</div>
            <div class="big-number">{n(s.last_weighed.avg_weight_g)}<small>กรัม/ตัว</small></div>
          </div>
          <span class="pill {g.status === 'on_track' || g.status === 'ahead' ? 'good' : g.status === 'behind' ? 'warn' : 'danger'}">{g.status_th}</span>
        </div>
        <div class="mt small">มาตรฐานวันที่ {g.day} ควรหนัก <b>{n(g.expected_g)} ก.</b> · ต่างจากเกณฑ์ {g.deviation_pct > 0 ? '+' : ''}{g.deviation_pct}%</div>
        <div class="kpi mt">
          <div class="k"><div class="lbl">โตต่อวัน (ADG) จริง</div><div class="val">{g.actual_adg_recent ?? g.actual_adg_overall} ก.</div></div>
          <div class="k"><div class="lbl">ADG มาตรฐาน</div><div class="val">{g.standard_adg} ก.</div></div>
          <div class="k"><div class="lbl">เป้าหมาย {n(s.crop.target_weight_g ?? s.species.market_weight_g)} ก.</div><div class="val">{g.days_to_target != null ? `อีก ${g.days_to_target} วัน` : '-'}</div></div>
        </div>
        <div class="progress mt" style="height:14px"><div style="width:{Math.min(100, Math.round((s.avg_weight_g / (s.crop.target_weight_g ?? s.species.market_weight_g)) * 100))}%"></div></div>
        <div class="small muted mt" style="margin-top:4px">ความคืบหน้าสู่ขนาดจับ {Math.min(100, Math.round((s.avg_weight_g / (s.crop.target_weight_g ?? s.species.market_weight_g)) * 100))}%</div>
      </div>
      <div class="card mt"><Collapse title="ดูกราฟการโตเทียบมาตรฐาน">
        {#if growthSeries.length}
          <LineChart series={growthSeries} xLabel={(x) => `วัน ${Math.round(x)}`} yLabel={(y) => `${Math.round(y)} ก.`} />
        {/if}
      </Collapse></div>
      <div class="card mt">
        <h3>คำแนะนำ</h3>
        {#each g.advice_th as a}<div class="reason"><span>{a}</span></div>{/each}
      </div>
      <a class="btn primary mt" href="#/weigh/{cropId}">บันทึกการชั่งครั้งใหม่</a>
      {#if weighings.length}
        <div class="card mt">
          <h3>ประวัติการชั่ง</h3>
          <div class="table-wrap"><table><thead><tr><th>วันที่</th><th class="num">เฉลี่ย (ก.)</th><th class="num">ตัวอย่าง</th><th>วิธี</th></tr></thead><tbody>
            {#each [...weighings].reverse() as w}<tr><td>{thDate(w.weigh_date)}</td><td class="num">{n(w.avg_weight_g)}</td><td class="num">{w.sample_count ?? '-'}</td><td>{w.method === 'stocking' ? 'ตอนปล่อย' : w.method === 'sample' ? 'สุ่มชั่ง' : w.method}</td></tr>{/each}
          </tbody></table></div>
        </div>
      {/if}
    {/if}

    {#if tab === 'water'}
      {#if s.water}
        <div class="card mt">
          <div class="card-title"><h3>ค่าน้ำล่าสุด</h3><span class="tiny muted">{thDateTime(s.water.measured_at)}</span></div>
          <div class="kpi">
            {#if s.water.do_mg_l != null}<div class="k"><div class="lbl">ออกซิเจน</div><div class="val">{s.water.do_mg_l} มก./ล.</div></div>{/if}
            {#if s.water.ph != null}<div class="k"><div class="lbl">pH</div><div class="val">{s.water.ph}</div></div>{/if}
            {#if s.water.temp_c != null}<div class="k"><div class="lbl">อุณหภูมิน้ำ</div><div class="val">{s.water.temp_c}°</div></div>{/if}
            {#if s.water.nh3 != null}<div class="k"><div class="lbl">แอมโมเนีย</div><div class="val">{s.water.nh3}</div></div>{/if}
            {#if s.water.no2 != null}<div class="k"><div class="lbl">ไนไตรท์</div><div class="val">{s.water.no2}</div></div>{/if}
            {#if s.water.secchi_cm != null}<div class="k"><div class="lbl">ความใส</div><div class="val">{s.water.secchi_cm} ซม.</div></div>{/if}
          </div>
        </div>
      {:else}
        <div class="card mt center"><p class="muted">ยังไม่เคยจดค่าน้ำของบ่อนี้ จดออกซิเจนตอนเช้ามืดสำคัญที่สุด</p></div>
      {/if}
      <a class="btn primary mt" href="#/water/{s.crop.pond_id}">จดค่าน้ำตอนนี้</a>
      {#if water.length > 1}
        <div class="card mt"><h3>ออกซิเจนละลายน้ำ</h3><LineChart series={waterSeries.do} height={180} xLabel={(x) => `+${Math.round(x)} วัน`} yLabel={(y) => y.toFixed(1)} bands={[{ from: 0, to: species?.water.do_min ?? 3, color: '#fbe4e1' }, { from: species?.water.do_min ?? 3, to: species?.water.do_ideal ?? 5, color: '#fdf1da' }]} /></div>
        <div class="card mt"><h3>อุณหภูมิน้ำ</h3><LineChart series={waterSeries.temp} height={160} xLabel={(x) => `+${Math.round(x)} วัน`} yMin={15} /></div>
        <div class="card mt"><h3>pH</h3><LineChart series={waterSeries.ph} height={160} xLabel={(x) => `+${Math.round(x)} วัน`} yMin={5} yLabel={(y) => y.toFixed(1)} /></div>
      {/if}
      {#if water.length}
        <div class="card mt">
          <h3>ประวัติค่าน้ำ</h3>
          <div class="table-wrap"><table><thead><tr><th>เวลา</th><th class="num">DO</th><th class="num">pH</th><th class="num">อุณหภูมิ</th><th class="num">NH3</th></tr></thead><tbody>
            {#each water as w}<tr><td>{thDateTime(w.measured_at)}</td><td class="num">{w.do_mg_l ?? '-'}</td><td class="num">{w.ph ?? '-'}</td><td class="num">{w.temp_c ?? '-'}</td><td class="num">{w.nh3 ?? '-'}</td></tr>{/each}
          </tbody></table></div>
        </div>
      {/if}
    {/if}

    {#if tab === 'money'}
      {@const p = s.performance}
      <div class="card mt">
        <h3>ผลงานรุ่นนี้ถึงวันนี้</h3>
        <div class="kpi mt">
          <div class="k"><div class="lbl">อัตราแลกเนื้อ (FCR)</div><div class="val">{p.fcr ?? 'ยังไม่มีข้อมูล'}</div>{#if p.fcr_grade_th}<div class="tiny muted">{p.fcr_grade_th}</div>{:else}<div class="tiny muted">ต้องบันทึกอาหารและชั่งก่อน</div>{/if}</div>
          <div class="k"><div class="lbl">ปลาในบ่อ (กก.)</div><div class="val">{n(p.biomass_kg)}</div><div class="tiny muted">{n(s.alive_count)} ตัว × {n(s.avg_weight_g)} ก.</div></div>
          <div class="k"><div class="lbl">ต้นทุนสะสม (บาท)</div><div class="val">{bahtShort(s.totals.cost_total)}</div></div>
          <div class="k"><div class="lbl">ต้นทุน/กก. (บาท)</div><div class="val">{p.cost_per_kg ? n1(p.cost_per_kg) : 'ยังไม่มีข้อมูล'}</div></div>
          <div class="k"><div class="lbl">มูลค่าปลาในบ่อ (บาท)</div><div class="val">{p.stock_value != null ? bahtShort(p.stock_value) : 'ใส่ราคาขายก่อน'}</div></div>
          {#if p.profit_if_harvest_today != null}
            <div class="k"><div class="lbl">กำไรถ้าจับวันนี้ (บาท)</div><div class="val" style="color:{p.profit_if_harvest_today >= 0 ? 'var(--green)' : 'var(--red)'}">{bahtShort(p.profit_if_harvest_today)}</div></div>
          {/if}
        </div>
        <p class="small muted mt">อาหารสะสม {n1(s.totals.fed_kg)} กก. · ค่าอาหาร {baht(s.totals.feed_cost)} · ค่าใช้จ่ายอื่น {baht(s.totals.expenses)} · ขายแล้ว {baht(s.totals.revenue)}</p>
      </div>

      <div class="card mt">
        <h3>พยากรณ์ถึงวันจับ</h3>
        <div class="grid3 mt">
          <div><label for="sp">ราคาขาย/กก.</label><input id="sp" type="number" inputmode="decimal" bind:value={sellPrice} placeholder="บาท" /></div>
          <div><label for="fp">ราคาอาหาร/กก.</label><input id="fp" type="number" inputmode="decimal" bind:value={feedPrice} placeholder="บาท" /></div>
          <div><label for="oc">ค่าอื่น/วัน</label><input id="oc" type="number" inputmode="decimal" bind:value={otherCost} placeholder="บาท" /></div>
        </div>
        <button class="btn ghost mt" onclick={reproject} disabled={projBusy}>{projBusy ? 'กำลังคำนวณ...' : 'คำนวณใหม่'}</button>
        {#if s.projection}
          {@const pj = s.projection}
          {@const hasPrice = !!(sellPrice || s.market_price_per_kg)}
          <div class="divider"></div>
          {#if hasPrice}
            <div class="card tint-cyan" style="box-shadow:none">
              <div class="big-number" style="font-size:1.6rem;color:{pj.profit >= 0 ? '#146b3c' : '#8f1f15'}">{pj.profit >= 0 ? 'คาดว่าจะเหลือกำไร' : 'คาดว่าจะขาดทุน'} {baht(Math.abs(pj.profit))}</div>
            </div>
          {:else}
            <div class="alert info">ยังไม่ได้ใส่ราคาขาย จึงยังคิดกำไรไม่ได้ — ใส่ราคาขาย/กก. ด้านบนแล้วกด "คำนวณใหม่" หรือดูราคาตลาดที่หน้าราคาปลา</div>
          {/if}
          <div class="card flat mt" style="line-height:1.8">
            เลี้ยงต่ออีก <b>{pj.days_remaining} วัน</b> จับได้ประมาณ <b>{thDateShort(addDays(s.date, pj.days_remaining))}</b><br />
            ได้ปลา <b>{n(pj.final_biomass_kg)} กก.</b> ({n(pj.final_count)} ตัว ตัวละ {n(pj.final_avg_weight_g)} ก.)<br />
            ต้องซื้ออาหารอีก <b>{n1(pj.feed_bags_remaining)} กระสอบ</b>{pj.feed_cost_remaining > 0 ? ` (${baht(pj.feed_cost_remaining)})` : ''}
          </div>
          {#if hasPrice}
            <div class="mt2">
              <MoneyBars rows={[
                { label: 'ขายปลาได้ (คาด)', value: pj.revenue, color: 'var(--green)', note: `ราคา ${n(sellPrice ? parseFloat(sellPrice) : (s.market_price_per_kg ?? 0))} บาท/กก.` },
                { label: 'จ่ายไปแล้ว', value: s.totals.cost_total, color: 'var(--navy-2)', note: `อาหาร ${baht(s.totals.feed_cost)} + อื่น ๆ ${baht(s.totals.expenses)}` },
                { label: 'ต้องจ่ายอีก', value: pj.cost_remaining, color: 'var(--amber)', note: 'อาหารและค่าใช้จ่ายรายวันจนถึงวันจับ' },
                { label: pj.profit >= 0 ? 'เหลือกำไร' : 'ขาดทุน', value: Math.abs(pj.profit), color: pj.profit >= 0 ? 'var(--green)' : 'var(--red)', note: `คุ้มทุนเมื่อขายได้อย่างน้อย ${n2(pj.breakeven_price_per_kg)} บาท/กก.` },
              ]} />
            </div>
          {/if}
          <div class="mt2"><Timeline total={s.day + pj.days_remaining} today={s.day} marks={[{ day: 0, label: 'ปล่อย', sub: thDateShort(s.crop.stocked_at) }, { day: s.day, label: 'วันนี้', sub: `${n(s.avg_weight_g)} ก.` }, { day: s.day + pj.days_remaining, label: 'จับขาย', sub: thDateShort(addDays(s.date, pj.days_remaining)) }]} /></div>
          {#if pj.curve?.length > 1}
            <div class="mt2"><Collapse title="ดูกราฟ (สำหรับผู้ที่ต้องการรายละเอียด)">
              <LineChart series={[{ name: 'น้ำหนักเฉลี่ย (ก.)', color: '#0e8ea7', points: pj.curve.map((c: any) => ({ x: c.day, y: c.avg_weight_g })) }]} height={170} xLabel={(x) => `วัน ${Math.round(x)}`} />
              <LineChart series={[{ name: 'อาหารสะสม (กก.)', color: '#1f9d5a', points: pj.curve.map((c: any) => ({ x: c.day, y: c.feed_kg_cum + s.totals.fed_kg })) }]} height={150} xLabel={(x) => `วัน ${Math.round(x)}`} />
            </Collapse></div>
          {/if}
        {/if}
      </div>

      <div class="grid3 mt">
        <a class="btn ghost" href="#/expense/{cropId}">เพิ่มค่าใช้จ่าย</a>
        <a class="btn ghost" href="#/harvest/{cropId}">บันทึกการจับ</a>
        <a class="btn ghost" href="#/report/{cropId}">การ์ดสรุปแชร์</a>
      </div>
      {#if expenses.length || harvests.length}
        <div class="card mt">
          <h3>รายการเงิน</h3>
          <div class="list">
            {#each harvests as h}<div class="list-item"><div class="main"><div class="title">ขายปลา {n(h.kg)} กก.{h.count ? ` (${n(h.count)} ตัว)` : ''}</div><div class="sub">{thDate(h.harvest_date)}{h.buyer ? ' · ' + h.buyer : ''}</div></div><b style="color:var(--green)">+{baht((h.kg || 0) * (h.price_per_kg || 0))}</b></div>{/each}
            {#each expenses as e}<div class="list-item"><div class="main"><div class="title">{expenseLabel(e.category)}</div><div class="sub">{thDate(e.expense_date)}{e.note ? ' · ' + e.note : ''}</div></div><b style="color:var(--red)">-{baht(e.amount)}</b></div>{/each}
          </div>
        </div>
      {/if}
      {#if treatments.length}
        <div class="card mt"><h3>ยา/การรักษา</h3>
          {#each treatments as t}<div class="list-item"><div class="main"><div class="title">{t.product}{t.dose ? ' · ' + t.dose : ''}</div><div class="sub">{thDate(t.start_date)}{t.end_date ? ' - ' + thDate(t.end_date) : ''}{t.withdrawal_days ? ` · หยุดยา ${t.withdrawal_days} วัน` : ''}{t.symptom ? ' · ' + t.symptom : ''}</div></div></div>{/each}
        </div>
      {/if}
      <div class="row mt2" style="justify-content:space-between">
        <a class="btn link" href="{'/api'}/crops/{cropId}/export.csv" target="_blank" rel="noopener">ดาวน์โหลด CSV</a>
        <button class="btn link" style="color:var(--red)" onclick={closeCrop}>ปิดรุ่นนี้</button>
      </div>
    {/if}

    {#if tab === 'history'}
      {#if logs.length > 1}
        <div class="card mt"><h3>อาหารแนะนำ vs ให้จริง (กก./วัน)</h3><LineChart series={feedSeries} height={180} xLabel={(x) => `+${Math.round(x)} วัน`} yLabel={(y) => y.toFixed(1)} /></div>
      {/if}
      <div class="card mt">
        <h3>บันทึกประจำวัน</h3>
        {#if !logs.length}<p class="muted mt">ยังไม่มีบันทึก</p>{/if}
        <div class="list">
          {#each logs as l}
            <div class="list-item">
              <div class="main">
                <div class="title">{thDate(l.log_date)} · ให้ {l.fed_kg != null ? n2(l.fed_kg) + ' กก.' : '-'}{l.recommended_kg != null ? ` (แนะนำ ${n2(l.recommended_kg)})` : ''}</div>
                <div class="sub">{l.mortality ? `ตาย ${l.mortality} ตัว · ` : ''}{['กินดี', 'กินช้า', 'ลอยหัว'][l.feeding_response ?? 0]}{l.factor && l.factor !== 1 ? ` · ปรับ ×${l.factor}` : ''}{l.note ? ' · ' + l.note : ''}</div>
              </div>
              <a class="btn link" href="#/log/{cropId}?date={l.log_date}">แก้</a>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {/if}
</main>
