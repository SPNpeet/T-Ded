<script lang="ts">
  import { onMount } from 'svelte'
  import { session } from '../lib/ui.svelte'
  import { n, n1, n2, baht, thDate, addDays, todayISO } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'
  import LineChart from '../lib/LineChart.svelte'
  import MoneyBars from '../lib/MoneyBars.svelte'
  import Timeline from '../lib/Timeline.svelte'
  import Collapse from '../lib/Collapse.svelte'
  import { simulateLocal, speciesList } from '../lib/engine'

  let species: any[] = $state([])
  let code = $state('nile_tilapia')
  let f = $state({ count: '5000', stock_weight_g: '30', target_weight_g: '800', expected_survival_pct: '85', fry_price_each: '2', feed_price_per_kg: '28', other_cost_per_day: '100', fixed_cost: '5000', sell_price_per_kg: '60', bag_kg: '20' })
  let mode: 'weight' | 'days' = $state('weight')
  let targetDays = $state('120')
  let out: any = $state(null)
  onMount(async () => {
    species = await speciesList()
  })
  const sp = $derived(species.find((s) => s.code === code))
  $effect(() => {
    if (!sp) return
    const num = (k: keyof typeof f) => parseFloat(f[k] || '0')
    simulateLocal({
      species: sp,
      count: num('count'),
      stock_weight_g: num('stock_weight_g'),
      target_weight_g: mode === 'weight' ? num('target_weight_g') : null,
      target_days: mode === 'days' ? parseInt(targetDays || '0') : null,
      expected_survival_pct: num('expected_survival_pct'),
      fry_price_each: num('fry_price_each'),
      feed_price_per_kg: num('feed_price_per_kg'),
      other_cost_per_day: num('other_cost_per_day'),
      fixed_cost: num('fixed_cost'),
      sell_price_per_kg: num('sell_price_per_kg'),
      bag_kg: num('bag_kg'),
    })
      .then((r) => (out = r))
      .catch((e) => console.error(e))
  })
</script>

<TopBar title="จำลองรุ่นเลี้ยงก่อนลงทุน" sub="ปล่อยเท่าไหร่ ใช้อาหารกี่กระสอบ ได้กำไรเท่าไหร่" back={session.user ? '/' : '/login'} />
<main class="page">
  <div class="card">
    <label for="sp">ชนิดปลา</label>
    <select id="sp" bind:value={code}>{#each species as s}<option value={s.code}>{s.name_th}</option>{/each}</select>
    <div class="grid2">
      <div><label for="c">จำนวนปล่อย (ตัว)</label><input id="c" type="number" inputmode="numeric" bind:value={f.count} /></div>
      <div><label for="sw">น้ำหนักตอนปล่อย (ก.)</label><input id="sw" type="number" inputmode="decimal" bind:value={f.stock_weight_g} /></div>
    </div>
    <label>เลี้ยงจนถึง</label>
    <div class="segment"><button class:active={mode === 'weight'} onclick={() => (mode = 'weight')}>ขนาดเป้าหมาย</button><button class:active={mode === 'days'} onclick={() => (mode = 'days')}>จำนวนวัน</button></div>
    {#if mode === 'weight'}<label for="tw">น้ำหนักเป้าหมาย (ก./ตัว)</label><input id="tw" type="number" inputmode="numeric" bind:value={f.target_weight_g} />{:else}<label for="td">จำนวนวันที่เลี้ยง</label><input id="td" type="number" inputmode="numeric" bind:value={targetDays} />{/if}
    <div class="grid2">
      <div><label for="sr">อัตรารอดที่คาด (%)</label><input id="sr" type="number" inputmode="numeric" bind:value={f.expected_survival_pct} /></div>
      <div><label for="fp">ราคาลูกปลา (บาท/ตัว)</label><input id="fp" type="number" inputmode="decimal" bind:value={f.fry_price_each} /></div>
      <div><label for="fd">ราคาอาหาร (บาท/กก.)</label><input id="fd" type="number" inputmode="decimal" bind:value={f.feed_price_per_kg} /></div>
      <div><label for="oc">ค่าอื่นต่อวัน (บาท)</label><input id="oc" type="number" inputmode="decimal" bind:value={f.other_cost_per_day} /></div>
      <div><label for="fx">ค่าเตรียมบ่อ/คงที่ (บาท)</label><input id="fx" type="number" inputmode="decimal" bind:value={f.fixed_cost} /></div>
      <div><label for="sl">ราคาขาย (บาท/กก.)</label><input id="sl" type="number" inputmode="decimal" bind:value={f.sell_price_per_kg} /></div>
    </div>
  </div>
  {#if out}
    {@const feedCost = out.feed_cost_remaining}
    {@const fryCost = parseFloat(f.count || '0') * parseFloat(f.fry_price_each || '0')}
    {@const otherCost = out.cost_total - feedCost - fryCost - parseFloat(f.fixed_cost || '0')}
    <section class="hero mt">
      <div class="small">ถ้าปล่อย{sp?.name_th} {n(parseFloat(f.count || '0'))} ตัววันนี้</div>
      <div class="big-number" style="margin-top:6px">{out.profit >= 0 ? 'เหลือกำไร' : 'ขาดทุน'} {baht(Math.abs(out.profit))}</div>
      <div class="mt" style="font-size:1.05rem;line-height:1.7">
        เลี้ยง <b>{out.harvest_day} วัน</b> จับได้ประมาณ <b>{thDate(addDays(todayISO(), out.harvest_day))}</b><br />
        ได้ปลา <b>{n(out.final_biomass_kg)} กก.</b> ({n(out.final_count)} ตัว ตัวละประมาณ {n(out.final_avg_weight_g)} ก.)<br />
        ขายได้ <b>{baht(out.revenue)}</b> จ่ายรวม <b>{baht(out.cost_total)}</b>
      </div>
    </section>
    <div class="card mt">
      <h3>เงินเข้า-เงินออก</h3>
      <div class="mt">
        <MoneyBars rows={[
          { label: 'ขายปลาได้', value: out.revenue, color: 'var(--green)', note: `${n(out.final_biomass_kg)} กก. × ${n(parseFloat(f.sell_price_per_kg || '0'))} บาท` },
          { label: 'ค่าอาหาร', value: feedCost, color: 'var(--amber)', note: `${n1(out.feed_bags_remaining)} กระสอบ (${n(out.feed_kg_total)} กก.)` },
          { label: 'ค่าลูกปลา', value: fryCost, color: 'var(--cyan-deep)' },
          { label: 'ค่าอื่น ๆ (ไฟ แรง เตรียมบ่อ)', value: otherCost + parseFloat(f.fixed_cost || '0'), color: 'var(--navy-2)' },
          { label: out.profit >= 0 ? 'เหลือกำไร' : 'ขาดทุน', value: Math.abs(out.profit), color: out.profit >= 0 ? 'var(--green)' : 'var(--red)', note: `คุ้มทุนเมื่อขายได้อย่างน้อย ${n2(out.breakeven_price_per_kg)} บาท/กก.` },
        ]} />
      </div>
    </div>
    <div class="card mt">
      <h3>เส้นเวลา</h3>
      <Timeline total={out.harvest_day} today={0} marks={[{ day: 0, label: 'ปล่อยวันนี้', sub: `${n(parseFloat(f.stock_weight_g || '0'))} ก.` }, { day: Math.round(out.harvest_day / 2), label: 'ครึ่งทาง', sub: `${n(out.curve[Math.floor(out.curve.length / 2)]?.avg_weight_g ?? 0)} ก.` },{ day: out.harvest_day, label: 'จับขาย', sub: thDate(addDays(todayISO(), out.harvest_day), false) }]} />
      <div class="small muted">อาหารต่อวันตอนต้น {n1(out.curve[1]?.feed_kg_day ?? 0)} กก. ตอนปลาย {n1(out.curve[out.curve.length - 1]?.feed_kg_day ?? 0)} กก. · อัตรารอดที่ใช้คิด {out.survival_pct}% · FCR คาด {out.projected_fcr ?? '-'}</div>
    </div>
    {#if !out.reached_target && mode === 'weight'}<div class="alert warn mt">ภายใน 400 วันยังไม่ถึงน้ำหนักเป้าหมาย ลองลดเป้าหมาย</div>{/if}
    <div class="mt"><Collapse title="ดูกราฟ (สำหรับผู้ที่ต้องการรายละเอียด)">
      <h3 class="mt">น้ำหนักเฉลี่ยตามเวลา</h3>
      <LineChart series={[{ name: 'น้ำหนัก (ก.)', color: '#0e8ea7', points: out.curve.map((c: any) => ({ x: c.day, y: c.avg_weight_g })) }]} height={180} xLabel={(x) => `วัน ${Math.round(x)}`} />
      <h3 class="mt">อาหารต่อวัน</h3>
      <LineChart series={[{ name: 'อาหาร กก./วัน', color: '#1f9d5a', points: out.curve.slice(1).map((c: any) => ({ x: c.day, y: c.feed_kg_day })) }]} height={160} xLabel={(x) => `วัน ${Math.round(x)}`} />
    </Collapse></div>
    <p class="small muted mt">การจำลองใช้ตารางการโตและอัตราให้อาหารมาตรฐาน อากาศจริง คุณภาพน้ำ และการจัดการมีผลต่อผลลัพธ์จริง ใช้เพื่อวางแผนเบื้องต้น</p>
  {/if}
</main>
