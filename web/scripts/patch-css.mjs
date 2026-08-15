// ปรับ CSS ให้อ่านง่ายขึ้น (รันครั้งเดียว)
import { readFileSync, writeFileSync } from 'node:fs'
let c = readFileSync('src/app.css', 'utf8')
const rep = (a, b) => {
  if (!c.includes(a)) console.warn('not found:', a.slice(0, 50))
  c = c.replace(a, b)
}
rep('--muted: #5b6878;', '--muted: #3f4b5c;')
rep('.small { font-size: 0.9rem; }', '.small { font-size: 0.95rem; }')
rep('.tiny { font-size: 0.8rem; }', '.tiny { font-size: 0.9rem; }')
rep('.kpi { display: grid; grid-template-columns: repeat(auto-fit, minmax(120px, 1fr)); gap: 10px; }', '.kpi { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 10px; }')
rep('.kpi .k .lbl { font-size: 0.85rem; color: var(--muted); }', '.kpi .k .lbl { font-size: 0.9rem; color: var(--muted); font-weight: 600; }')
rep(
  ".kpi .k .val { font-family: 'Prompt', sans-serif; font-size: 1.35rem; font-weight: 700; color: var(--navy); font-variant-numeric: tabular-nums; }",
  ".kpi .k .val { font-family: 'Prompt', sans-serif; font-size: 1.3rem; font-weight: 700; color: var(--navy); font-variant-numeric: tabular-nums; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }",
)
rep('.hero .muted { color: rgba(255,255,255,0.78); }', '.hero .muted, .hero .small { color: rgba(255,255,255,0.95); }\n.hero .big-number small { color: #ffffff; }')
rep(
  '.big-number small { font-size: 1rem; font-weight: 600; color: var(--muted); margin-left: 6px; }',
  '.big-number small { font-size: 1rem; font-weight: 600; color: var(--muted); margin-left: 6px; white-space: nowrap; }',
)
rep('.pill { display: inline-block; padding: 4px 12px; border-radius: 999px; font-size: 0.9rem; font-weight: 700; white-space: nowrap; }', '.pill { display: inline-block; padding: 5px 12px; border-radius: 999px; font-size: 0.95rem; font-weight: 700; white-space: nowrap; }')
rep('.pill.good { background: var(--green-tint); color: var(--green); }', '.pill.good { background: var(--green-tint); color: #146b3c; }')
rep('.pill.warn { background: var(--amber-tint); color: var(--amber); }', '.pill.warn { background: var(--amber-tint); color: #7a4c00; }')
rep('.pill.danger { background: var(--red-tint); color: var(--red); }', '.pill.danger { background: var(--red-tint); color: #8f1f15; }')
rep('.pill.info { background: var(--cyan-tint); color: var(--cyan-deep); }', '.pill.info { background: var(--cyan-tint); color: #06596b; }')
rep('.pill.neutral { background: #eef1f6; color: var(--muted); }', '.pill.neutral { background: #e6ebf3; color: #2b3546; }')
rep('.pill.pink { background: var(--pink-tint); color: var(--pink); }', '.pill.pink { background: var(--pink-tint); color: #9c1f62; }')
rep('.list-item .sub { color: var(--muted); font-size: 0.9rem; }', '.list-item .sub { color: var(--muted); font-size: 0.95rem; }')
rep('text-decoration: none; color: var(--muted); font-size: 0.78rem;', 'text-decoration: none; color: #2b3546; font-size: 0.85rem;')
rep('.topbar .sub { font-size: 0.8rem; opacity: 0.8; }', '.topbar .sub { font-size: 0.9rem; opacity: 0.95; }')
rep(".chart text { font-family: 'Sarabun', sans-serif; font-size: 11px; fill: var(--muted); }", ".chart text { font-family: 'Sarabun', sans-serif; font-size: 13px; fill: #2b3546; font-weight: 600; }")
rep('th { background: var(--paper); font-weight: 700; color: var(--navy); font-size: 0.85rem; }', 'th { background: var(--paper); font-weight: 700; color: var(--navy); font-size: 0.9rem; }')
rep('font-weight: 700; font-size: 0.85rem; cursor: pointer; padding: 6px;', 'font-weight: 700; font-size: 0.95rem; cursor: pointer; padding: 6px;')
c += `
/* โหมดคอนทราสต์สูง (กลางแจ้ง): พื้นขาว ตัวหนังสือดำ ไม่มีสีจาง */
body.hc { --paper: #ffffff; --card: #ffffff; --muted: #111827; --line: #94a3b8; --shadow: none; }
body.hc .card { border: 2px solid #111827; }
body.hc .hero { background: #ffffff; color: #111827; border: 3px solid #111827; }
body.hc .hero::after { display: none; }
body.hc .hero .big-number, body.hc .hero .big-number small, body.hc .hero .muted, body.hc .hero .small, body.hc .hero h1, body.hc .hero h2 { color: #111827; }
body.hc .hero .kpi .k { background: #f1f5f9 !important; }
body.hc .hero .kpi .lbl, body.hc .hero .kpi .val { color: #111827 !important; }
body.hc .topbar { background: #000000; }
body.hc .btn.primary { background: #0b5f73; }
body.hc .btn.success { background: #0f6b3a; }
body.hc .pill { border: 1.5px solid currentColor; }
body.hc .bottomnav a.active { background: #111827; color: #ffffff; }
body.hc input, body.hc select, body.hc textarea { border-color: #111827; }
.fs-row { display: flex; gap: 8px; }
.fs-row button { flex: 1; min-height: 56px; border: 2px solid var(--line); background: #fff; border-radius: 14px; font-weight: 800; color: var(--navy); cursor: pointer; }
.fs-row button.active { border-color: var(--cyan-deep); background: var(--cyan-tint); }
`
writeFileSync('src/app.css', c)
console.log('css updated')
