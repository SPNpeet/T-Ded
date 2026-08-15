<script lang="ts">
  const BASE = import.meta.env.BASE_URL
  import { onMount } from 'svelte'
  import { cachedGet } from '../lib/api'
  import { session, currentFarm, selectFarm, go, isStaff } from '../lib/ui.svelte'
  import { thDate, n1, n2, n, baht, bandPill } from '../lib/format'
  import ScoreRing from '../lib/ScoreRing.svelte'
  import Icon from '../lib/Icon.svelte'

  let data: any = $state(null)
  let err = $state('')
  let fromCache = $state(false)
  let loading = $state(true)

  async function load() {
    const farm = currentFarm()
    if (!farm) {
      loading = false
      return
    }
    loading = true
    try {
      const r = await cachedGet(`/farms/${farm.id}/today`)
      data = r.data
      fromCache = r.fromCache
      err = ''
    } catch (e: any) {
      err = e.message
    } finally {
      loading = false
    }
  }
  onMount(load)
  $effect(() => {
    session.farmId
    load()
  })

  const greet = () => {
    const h = new Date().getHours()
    return h < 11 ? 'สวัสดีตอนเช้า' : h < 16 ? 'สวัสดีตอนบ่าย' : 'สวัสดีตอนเย็น'
  }
  const wxText = (w: any) => {
    if (!w) return ''
    const parts = []
    if (w.tmin_c != null && w.tmax_c != null) parts.push(`${Math.round(w.tmin_c)}-${Math.round(w.tmax_c)} องศา`)
    if (w.rain_mm != null && w.rain_mm > 0) parts.push(`ฝน ${Math.round(w.rain_mm)} มม.`)
    if (w.cloud_pct != null && w.cloud_pct >= 75) parts.push('ฟ้าครึ้ม')
    return parts.join(' · ')
  }
  const doneToday = (p: any) => p.today_log && p.today_log.fed_kg != null
</script>

<header class="topbar">
  <div class="brand"><img src="{BASE}mark.png" alt="" />ทีเด็ดปลาน้ำจืด</div>
  <div class="spacer"></div>
  {#if (session.user?.farms?.length ?? 0) > 1}
    <select style="min-height:44px;width:auto;padding:6px 10px;border-radius:10px;font-size:0.9rem" value={session.farmId} onchange={(e) => selectFarm((e.target as HTMLSelectElement).value)}>
      {#each session.user!.farms as f}<option value={f.id}>{f.name}</option>{/each}
    </select>
  {/if}
</header>

<main class="page">
  {#if !currentFarm()}
    <div class="card center">
      <h2>ยังไม่มีฟาร์ม</h2>
      <p class="muted mt">{isStaff() ? 'บัญชีเจ้าหน้าที่ ดูฟาร์มทั้งหมดได้ที่เมนูฟาร์มทั้งหมด' : 'สร้างฟาร์มในหน้าตั้งค่าเพื่อเริ่มใช้งาน'}</p>
      <a class="btn primary mt2" href={isStaff() ? '#/admin' : '#/settings'}>{isStaff() ? 'ไปหน้าฟาร์มทั้งหมด' : 'ไปหน้าตั้งค่า'}</a>
    </div>
  {:else if loading && !data}
    <div class="stack"><div class="skeleton" style="min-height:140px"></div><div class="skeleton"></div><div class="skeleton"></div></div>
  {:else if err && !data}
    <div class="alert danger">{err}</div>
    <button class="btn mt" onclick={load}>ลองใหม่</button>
  {:else if data}
    <section class="hero">
      <div class="muted small">{greet()} คุณ{session.user?.name} · {thDate(data.date)}</div>
      <h1 style="margin-top:2px">{data.farm.name}</h1>
      <div class="row mt" style="align-items:flex-end;gap:18px">
        <div>
          <div class="big-number">{n1(data.totals.feed_today_kg)}<small>กก. อาหารวันนี้รวมทุกบ่อ</small></div>
        </div>
      </div>
      <div class="row wrap mt small" style="gap:14px">
        {#if data.weather}<span class="row" style="gap:6px"><Icon name={data.weather.rain_mm > 5 ? 'rain' : 'sun'} size={20} />{wxText(data.weather)}</span>{/if}
        {#if data.totals.stock_value > 0}<span>มูลค่าปลาในบ่อ {baht(data.totals.stock_value)}</span>{/if}
        {#if data.streak_days > 0}<span class="pill pink">บันทึกต่อเนื่อง {data.streak_days} วัน</span>{/if}
        {#if fromCache}<span class="pill warn">ข้อมูลล่าสุดที่เก็บไว้</span>{/if}
      </div>
    </section>

    {#if data.stock.low}
      <div class="alert warn mt"><Icon name="alert" />อาหารในสต๊อกเหลือ {n1(data.stock.balance_bags)} กระสอบ (ประมาณ {n(data.stock.days_left)} วัน) วางแผนสั่งซื้อ</div>
    {/if}
    {#each data.announcements ?? [] as a}
      <div class="alert info mt"><Icon name="bell" /><div><b>{a.title}</b><div class="small">{a.body}</div></div></div>
    {/each}

    <div class="fab-row mt2">
      <a href="#/stock"><Icon name="stock" />รับอาหารเข้า</a>
      <a href="#/prices"><Icon name="chart" />ราคาปลา</a>
      <a href="#/feed"><Icon name="feed" />อาหาร/โปรตีน</a>
      <a href="#/diseases"><Icon name="map" />โรคในพื้นที่</a>
    </div>

    <h2 class="mt2" style="margin-bottom:8px">บ่อของคุณ</h2>
    {#if !data.ponds.length && !data.empty_ponds.length}
      <div class="card center">
        <p>ยังไม่มีบ่อ เพิ่มบ่อแรกเพื่อเริ่มคำนวณอาหาร</p>
        <a class="btn primary mt" href="#/ponds">เพิ่มบ่อ</a>
      </div>
    {/if}
    {#each data.ponds as p (p.crop_id)}
      {@const bp = bandPill(p.recommendation.band)}
      <a class="card" href="#/pond/{p.crop_id}" style="display:block;text-decoration:none;color:inherit;margin-top:12px">
        <div class="row" style="align-items:flex-start;gap:14px">
          <ScoreRing score={p.health.score} label="สุขภาพ" size={84} />
          <div style="flex:1;min-width:0">
            <div class="row" style="justify-content:space-between">
              <h3>{p.pond_name}</h3>
              <span class="pill {doneToday(p) ? 'good' : 'neutral'}">{doneToday(p) ? 'ให้อาหารแล้ว' : 'ยังไม่บันทึกวันนี้'}</span>
            </div>
            <div class="small muted">{p.species.name_th} · วันที่ {p.day} · {n(p.alive_count)} ตัว · เฉลี่ย {n(p.avg_weight_g)} ก.{p.avg_weight_source === 'estimated' ? ' (ประมาณ)' : ''}</div>
            <div class="mt" style="margin-top:8px">
              <span class="big-number" style="font-size:1.9rem">{n2(p.recommendation.final_kg)}<small>กก./วัน · มื้อละ {n2(p.recommendation.per_meal_kg)}</small></span>
            </div>
            <div class="row wrap small" style="gap:8px;margin-top:6px">
              <span class="pill {bp.cls}">{p.recommendation.headline_th}</span>
              {#if p.growth?.status_th && p.day > 7}<span class="pill {p.growth.status === 'on_track' || p.growth.status === 'ahead' ? 'good' : 'warn'}">{p.growth.status_th}</span>{/if}
              {#if p.performance.fcr}<span class="pill neutral">FCR {p.performance.fcr}</span>{/if}
            </div>
          </div>
        </div>
        {#if p.alerts?.length}
          <div class="mt" style="margin-top:10px">
            {#each p.alerts.slice(0, 2) as a}
              <div class="alert {a.level === 'warn' ? 'warn' : 'info'} small" style="padding:8px 12px">{a.text}</div>
            {/each}
          </div>
        {/if}
        <div class="grid2 mt">
          <span class="btn success" style="min-height:52px" onclick={(e) => { e.preventDefault(); go(`/log/${p.crop_id}`) }} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && go(`/log/${p.crop_id}`)}>บันทึกให้อาหาร</span>
          <span class="btn ghost" style="min-height:52px">ดูรายละเอียด</span>
        </div>
      </a>
    {/each}
    {#each data.empty_ponds as p}
      <div class="card flat mt" style="margin-top:12px">
        <div class="row">
          <div style="flex:1"><b>{p.name}</b><div class="small muted">บ่อว่าง ยังไม่ได้ปล่อยปลา</div></div>
          <a class="btn primary sm" href="#/new-crop/{p.id}">ปล่อยปลารุ่นใหม่</a>
        </div>
      </div>
    {/each}
    <p class="center small muted mt2">ตัวเลขทั้งหมดคำนวณจากตารางมาตรฐาน ปรับตามอากาศ น้ำ และการกินที่คุณบันทึก กดที่บ่อเพื่อดูเหตุผล</p>
  {/if}
</main>
