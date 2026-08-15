// แก้ปัญหาที่พบบนมือถือ: ปุ่มขนาดไม่เท่ากัน, กราฟซ่อนใน <details> ไม่วาด/ไม่ปรับขนาด, ปุ่มในการ์ดบ่อไม่เท่ากัน
import { readFileSync, writeFileSync } from 'node:fs'
const edit = (file, pairs, optional = false) => {
  let c = readFileSync(file, 'utf8')
  for (const [a, b] of pairs) {
    if (!c.includes(a)) {
      if (optional) continue
      throw new Error(file + ' missing: ' + a.slice(0, 70))
    }
    c = c.split(a).join(b)
  }
  writeFileSync(file, c)
}

// 1) CSS: ปุ่มในกริดเท่ากันทุกใบ, ปุ่มเล็กก็เต็มช่อง, details ที่เหลือดูเป็นปุ่มกดได้
edit('src/app.css', [
  ['.btn.sm { min-height: 44px; padding: 8px 14px; font-size: 0.95rem; width: auto; }', '.btn.sm { min-height: 48px; padding: 8px 14px; font-size: 0.95rem; width: auto; }'],
  ['.grid2 { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }', '.grid2 { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; align-items: stretch; }\n.grid2 > .btn, .grid3 > .btn, .grid2 > a.btn, .grid3 > a.btn { width: 100%; height: 100%; }'],
  ['details summary { cursor: pointer; font-weight: 700; color: var(--cyan-deep); padding: 8px 0; }', 'details summary { cursor: pointer; font-weight: 700; color: var(--navy); padding: 14px 16px; min-height: 56px; display: flex; align-items: center; gap: 8px; background: #fff; border: 2px solid var(--line); border-radius: 14px; list-style: none; }\ndetails summary::-webkit-details-marker { display: none; }\ndetails summary::after { content: "\\25BC"; margin-left: auto; color: var(--cyan-deep); font-size: 0.8rem; }\ndetails[open] summary::after { content: "\\25B2"; }\ndetails[open] summary { border-bottom-left-radius: 0; border-bottom-right-radius: 0; }\ndetails[open] > :not(summary) { border: 2px solid var(--line); border-top: none; border-radius: 0 0 14px 14px; padding: 12px 16px; }'],
])

// 2) การ์ดบ่อในหน้าวันนี้: ปุ่มสองใบสูงเท่ากันและกดได้จริงทั้งคู่ (เดิมใบขวาเป็น span ในลิงก์)
edit('src/pages/Today.svelte', [
  [`        <div class="grid2 mt">
          <span class="btn success sm" style="width:100%" onclick={(e) => { e.preventDefault(); go(\`/log/\${p.crop_id}\`) }} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && go(\`/log/\${p.crop_id}\`)}>บันทึกให้อาหารวันนี้</span>
          <span class="btn ghost sm" style="width:100%">ดูรายละเอียดบ่อ</span>
        </div>`,
    `        <div class="grid2 mt">
          <span class="btn success" style="min-height:52px" onclick={(e) => { e.preventDefault(); go(\`/log/\${p.crop_id}\`) }} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && go(\`/log/\${p.crop_id}\`)}>บันทึกให้อาหาร</span>
          <span class="btn ghost" style="min-height:52px">ดูรายละเอียด</span>
        </div>`],
])

// 3) กราฟ: ใช้ Collapse (mount ใหม่ทุกครั้งที่กาง) แทน <details> เพื่อให้ SVG วาดขนาดถูก
edit('src/pages/Simulator.svelte', [
  ["  import Timeline from '../lib/Timeline.svelte'", "  import Timeline from '../lib/Timeline.svelte'\n  import Collapse from '../lib/Collapse.svelte'"],
  ['<details class="card mt"><summary>ดูกราฟ (สำหรับผู้ที่ต้องการรายละเอียด)</summary>', '<div class="mt"><Collapse title="ดูกราฟ (สำหรับผู้ที่ต้องการรายละเอียด)">'],
  ['    </details>\n    <p class="small muted mt">การจำลอง', '    </Collapse></div>\n    <p class="small muted mt">การจำลอง'],
])
edit('src/pages/CropDetail.svelte', [
  ["  import Timeline from '../lib/Timeline.svelte'", "  import Timeline from '../lib/Timeline.svelte'\n  import Collapse from '../lib/Collapse.svelte'"],
  ['<details class="mt2"><summary>ดูกราฟ (สำหรับผู้ที่ต้องการรายละเอียด)</summary>', '<div class="mt2"><Collapse title="ดูกราฟ (สำหรับผู้ที่ต้องการรายละเอียด)">'],
  [`            </details>
          {/if}`, `            </Collapse></div>
          {/if}`],
  ['<details class="card mt">\n        <summary>ดูกราฟการโตเทียบมาตรฐาน</summary>', '<div class="card mt"><Collapse title="ดูกราฟการโตเทียบมาตรฐาน">'],
  [`        {/if}
      </details>`, `        {/if}
      </Collapse></div>`],
], true)
console.log('ui fixes patched')
