// แพตช์ครั้งเดียว: การ์ดโภชนาการในแท็บอาหาร + ช่องโปรตีนในสต๊อก
import { readFileSync, writeFileSync } from 'node:fs'

let c = readFileSync('src/pages/CropDetail.svelte', 'utf8')
const anchor = '      {#if s.weather}'
if (!c.includes(anchor)) throw new Error('anchor CropDetail')
const card = `      {#if s.nutrition}
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
            <div class="mt"><b>อาหารในสต๊อก:</b> {s.feed_on_hand.brand ?? ''} โปรตีน {n(s.feed_on_hand.protein_pct)}%{s.feed_on_hand.pellet_mm ? \` เม็ด \${s.feed_on_hand.pellet_mm} มม.\` : ''} — <b>{nu.status_th}</b></div>
          {/if}
          {#each nu.messages_th as m}<div class="small mt">{m}</div>{/each}
          {#if nu.price_per_kg_protein}<div class="small muted mt">ราคาต่อโปรตีน 1 กก. {n2(nu.price_per_kg_protein)} บาท · ปลาได้โปรตีนวันละ {n2(nu.protein_intake_kg_day)} กก.</div>{/if}
        </div>
      {/if}
`
c = c.replace(anchor, card + anchor)
writeFileSync('src/pages/CropDetail.svelte', c)

let st = readFileSync('src/pages/Stock.svelte', 'utf8')
const rep = (a, b) => {
  if (!st.includes(a)) throw new Error('anchor Stock: ' + a.slice(0, 40))
  st = st.replace(a, b)
}
rep(
  "let f = $state({ move_date: todayISO(), bags: '', bag_kg: '', brand: '', pellet_mm: '3', price_total: '', kind: 'in', note: '' })",
  "let f = $state({ move_date: todayISO(), bags: '', bag_kg: '', brand: '', pellet_mm: '3', protein_pct: '', form: 'floating', price_total: '', kind: 'in', note: '' })",
)
rep(
  'const body: any = { move_date: f.move_date, kind: f.kind, brand: f.brand || null, pellet_mm: f.pellet_mm ? parseFloat(f.pellet_mm) : null, note: f.note || null }',
  'const body: any = { move_date: f.move_date, kind: f.kind, brand: f.brand || null, pellet_mm: f.pellet_mm ? parseFloat(f.pellet_mm) : null, protein_pct: f.protein_pct ? parseFloat(f.protein_pct) : null, form: f.form || null, note: f.note || null }',
)
rep(
  '<div><label for="pm">ขนาดเม็ด (มม.)</label><input id="pm" type="number" inputmode="decimal" bind:value={f.pellet_mm} /></div>',
  '<div><label for="pm">ขนาดเม็ด (มม.)</label><input id="pm" type="number" inputmode="decimal" bind:value={f.pellet_mm} /></div>\n          <div><label for="pp">โปรตีน (%) <span class="hint">ดูข้างถุง</span></label><input id="pp" type="number" inputmode="decimal" bind:value={f.protein_pct} placeholder="เช่น 30" /></div>\n          <div><label for="fm">ชนิดเม็ด</label><select id="fm" bind:value={f.form}><option value="floating">ลอยน้ำ</option><option value="sinking">จมน้ำ</option><option value="crumble">เม็ดเล็ก/ป่น</option></select></div>',
)
rep(
  '{#if data.avg_price_per_kg}<span>ราคาเฉลี่ย {n1(data.avg_price_per_kg)} บาท/กก.</span>{/if}',
  '{#if data.avg_price_per_kg}<span>ราคาเฉลี่ย {n1(data.avg_price_per_kg)} บาท/กก.</span>{/if}\n        {#if data.current_feed?.protein_pct}<span class="pill info">ล่าสุด: {data.current_feed.brand ?? \'อาหาร\'} โปรตีน {n(data.current_feed.protein_pct)}%</span>{/if}',
)
rep(
  "{m.brand ? ' · ' + m.brand : ''}{m.note ? ' · ' + m.note : ''}</div>",
  "{m.brand ? ' · ' + m.brand : ''}{m.protein_pct ? ` · โปรตีน ${m.protein_pct}%` : ''}{m.note ? ' · ' + m.note : ''}</div>",
)
writeFileSync('src/pages/Stock.svelte', st)
console.log('patched')
