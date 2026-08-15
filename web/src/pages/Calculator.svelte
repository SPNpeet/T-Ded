<script lang="ts">
  import { onMount } from 'svelte'
  import { api } from '../lib/api'
  import { session, toast } from '../lib/ui.svelte'
  import { n, n1, n2, todayISO, thDate, FEEDING_RESPONSE } from '../lib/format'
  import TopBar from '../lib/TopBar.svelte'
  import Icon from '../lib/Icon.svelte'
  import { recommendLocal, speciesList } from '../lib/engine'

  let species: any[] = $state([])
  let code = $state('nile_tilapia')
  let weight = $state('')
  let count = $state('')
  let envOn = $state(false)
  let src: 'auto' | 'manual' = $state('auto')
  let date = $state(todayISO())
  let lat = $state('19.83')
  let lng = $state('99.78')
  let wx: any = $state(null)
  let wxStatus = $state('')
  let mTemp = $state('')
  let mSky = $state('1')
  let mRain = $state('')
  let stress = $state(0)
  let doVal = $state('')
  let rec: any = $state(null)

  onMount(async () => {
    species = await speciesList()
  })
  const sp = $derived(species.find((s) => s.code === code))
  const env = $derived.by(() => {
    if (!envOn) return null
    const e: any = { stress: ['normal', 'slow_eating', 'gasping'][stress] }
    if (src === 'auto' && wx) {
      e.tmax_c = wx.tmax_c
      e.tmin_c = wx.tmin_c
      e.rain_mm = wx.rain_mm
      e.cloud_pct = wx.cloud_pct
    } else if (src === 'manual' && mTemp) {
      e.tmax_c = parseFloat(mTemp)
      e.tmin_c = parseFloat(mTemp)
      e.rain_mm = parseFloat(mRain || '0')
      e.cloud_pct = mSky === '2' ? 85 : mSky === '1' ? 40 : 10
    }
    if (doVal) e.do_morning = parseFloat(doVal)
    return e
  })
  $effect(() => {
    const w = parseFloat(weight)
    const c = parseFloat(count)
    if (!sp || !(w > 0) || !(c > 0)) {
      rec = null
      return
    }
    recommendLocal({ species: sp, avg_weight_g: w, count: c, env }).then((r) => (rec = r)).catch((e) => console.error(e))
  })
  async function fetchWx() {
    wxStatus = 'กำลังดึงพยากรณ์...'
    try {
      wx = await api.get(`/weather?lat=${lat}&lng=${lng}&date=${date}`)
      wxStatus = `ได้ข้อมูล ${thDate(date)} แล้ว`
    } catch (e: any) {
      wxStatus = 'ดึงไม่สำเร็จ ลองใหม่หรือกรอกเอง'
      wx = null
    }
  }
  function gps() {
    navigator.geolocation?.getCurrentPosition(
      (p) => {
        lat = p.coords.latitude.toFixed(2)
        lng = p.coords.longitude.toFixed(2)
        fetchWx()
      },
      () => toast('จับพิกัดไม่ได้', 'error'),
      { enableHighAccuracy: true, timeout: 10000 },
    )
  }
</script>

<TopBar title="คำนวณอาหารปลา" sub="ใช้ได้โดยไม่ต้องสมัคร" back={session.user ? '/' : '/login'} />
<main class="page">
  <div class="card">
    <h2>1. ข้อมูลปลา</h2>
    <label for="sp">ชนิดปลา</label>
    <select id="sp" bind:value={code}>{#each species as s}<option value={s.code}>{s.name_th}{s.approximate ? ' (ตารางโดยประมาณ)' : ''}</option>{/each}</select>
    <div class="grid2">
      <div><label for="w">น้ำหนักเฉลี่ย <span class="hint">กรัม/ตัว</span></label><input id="w" type="number" inputmode="decimal" bind:value={weight} placeholder="เช่น 300" /></div>
      <div><label for="c">จำนวนปลา <span class="hint">ตัว</span></label><input id="c" type="number" inputmode="numeric" bind:value={count} placeholder="เช่น 1000" /></div>
    </div>
  </div>
  <div class="card mt">
    <div class="row" style="justify-content:space-between"><h2>2. ปรับตามสภาพแวดล้อม</h2><button class="btn sm {envOn ? 'primary' : 'ghost'}" onclick={() => (envOn = !envOn)}>{envOn ? 'เปิดอยู่' : 'ปิดอยู่'}</button></div>
    {#if envOn}
      <div class="tabs mt"><button class:active={src === 'auto'} onclick={() => (src = 'auto')}>ดึงพยากรณ์อัตโนมัติ</button><button class:active={src === 'manual'} onclick={() => (src = 'manual')}>กรอกเอง</button></div>
      {#if src === 'auto'}
        <label>วันที่ <span class="hint">(ล่วงหน้าได้ 16 วัน ย้อนหลังได้)</span></label><input type="date" bind:value={date} />
        <div class="grid2"><div><label for="la">ละติจูด</label><input id="la" bind:value={lat} /></div><div><label for="lo">ลองจิจูด</label><input id="lo" bind:value={lng} /></div></div>
        <div class="grid2 mt"><button class="btn ghost" onclick={gps}>ใช้ตำแหน่งฉัน</button><button class="btn primary" onclick={fetchWx}>ดึงพยากรณ์</button></div>
        {#if wxStatus}<p class="small muted mt">{wxStatus}</p>{/if}
        {#if wx}<div class="kpi mt"><div class="k"><div class="lbl">สูงสุด</div><div class="val">{n(wx.tmax_c)}°</div></div><div class="k"><div class="lbl">ต่ำสุด</div><div class="val">{n(wx.tmin_c)}°</div></div><div class="k"><div class="lbl">ฝน</div><div class="val">{n(wx.rain_mm)} มม.</div></div><div class="k"><div class="lbl">เมฆ</div><div class="val">{n(wx.cloud_pct)}%</div></div></div>{/if}
      {:else}
        <div class="grid3">
          <div><label for="mt">อุณหภูมิ (องศา)</label><input id="mt" type="number" inputmode="decimal" bind:value={mTemp} placeholder="เช่น 30" /></div>
          <div><label for="ms">สภาพฟ้า</label><select id="ms" bind:value={mSky}><option value="0">แดดจัด</option><option value="1">เมฆบางส่วน</option><option value="2">ครึ้ม</option></select></div>
          <div><label for="mr">ฝน (มม./วัน)</label><input id="mr" type="number" inputmode="decimal" bind:value={mRain} placeholder="0" /></div>
        </div>
      {/if}
      <label>สังเกตจากบ่อ</label>
      <div class="segment">{#each FEEDING_RESPONSE as f}<button class:active={stress === f.v} class={f.v === 1 ? 'warn' : f.v === 2 ? 'danger' : ''} onclick={() => (stress = f.v)}>{f.label}</button>{/each}</div>
      <label for="do">ออกซิเจนเช้า (มก./ล.) <span class="hint">ถ้ามีเครื่องวัด</span></label><input id="do" type="number" inputmode="decimal" step="0.1" bind:value={doVal} />
    {:else}
      <p class="small muted mt">เปิดเพื่อปรับปริมาณตามอุณหภูมิ ฝน เมฆ ออกซิเจน และการกินของปลา</p>
    {/if}
  </div>
  <div class="card mt">
    <h2>3. ผลการคำนวณ</h2>
    {#if rec}
      <div class="big-number mt">{n2(rec.final_kg)}<small>กก./วัน</small></div>
      <div class="small">แบ่ง {rec.meals_per_day} มื้อ มื้อละ {n2(rec.per_meal_kg)} กก. · เม็ด {rec.pellet_mm} มม. · {rec.pct}% ต่อวัน · ชีวมวล {n(rec.biomass_kg)} กก.</div>
      <div class="mt"><span class="pill {rec.band === 'cut' ? 'danger' : rec.band === 'down' ? 'warn' : 'good'}">{rec.headline_th}</span>{#if rec.factor !== 1}<span class="small muted"> ฐาน {n2(rec.base_kg)} กก. × {rec.factor}</span>{/if}</div>
      {#each rec.reasons as r}<div class="reason"><b>× {r.factor.toFixed(2)}</b><span>{r.text_th}</span></div>{/each}
      {#each rec.warnings as w}<div class="alert warn small mt">{w}</div>{/each}
      <p class="tiny muted mt">สูตร: น้ำหนัก × จำนวน ÷ 1,000 × % ÷ 100 × ตัวปรับ · ตัวปรับหลายข้อคูณกัน ต่ำสุด ×0.40</p>
    {:else}
      <p class="muted">กรอกน้ำหนักและจำนวนปลาเพื่อดูผล</p>
    {/if}
  </div>
  {#if !session.user}
    <div class="card tint-cyan mt">
      <div class="row"><Icon name="star" /><b>อยากให้แอปจำบ่อของคุณ คำนวณให้ทุกเช้า และบอกกำไร?</b></div>
      <a class="btn primary mt" href="#/register">สมัครฟาร์มฟรี</a>
    </div>
  {/if}
  {#if sp}
    <details class="mt2"><summary>ดูตารางมาตรฐาน{sp.name_th}</summary>
      <div class="table-wrap"><table><thead><tr><th>น้ำหนัก (ก.)</th><th class="num">% ต่อวัน</th><th class="num">เม็ด (มม.)</th><th class="num">กก./วัน ต่อ 1,000 ตัว</th></tr></thead><tbody>
        {#each sp.feed_table as r}<tr><td>{r.weight_g}</td><td class="num">{r.pct}</td><td class="num">{r.pellet_mm}</td><td class="num">{n1((r.weight_g * r.pct) / 100)}</td></tr>{/each}
      </tbody></table></div>
      <h3 class="mt">มาตรฐานการโต</h3>
      <div class="table-wrap"><table><thead><tr><th>วันที่</th><th class="num">น้ำหนัก (ก.)</th><th class="num">ADG (ก./วัน)</th></tr></thead><tbody>
        {#each sp.growth as r}<tr><td>{r.day_from}-{r.day_to}</td><td class="num">{r.weight_g}</td><td class="num">{r.adg}</td></tr>{/each}
      </tbody></table></div>
    </details>
  {/if}
</main>
