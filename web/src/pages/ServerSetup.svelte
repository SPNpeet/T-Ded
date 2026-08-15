<script lang="ts">
  // ตั้งที่อยู่เซิร์ฟเวอร์ฟาร์ม สำหรับกรณีเปิดแอปจากหน้าเว็บฟรี (GitHub Pages) ที่ไม่มีเซิร์ฟเวอร์ในตัว
  import { getApiBase, setApiBase, testApiBase, IS_STATIC_HOST } from '../lib/api'
  import { toast, go, loadSession } from '../lib/ui.svelte'
  import TopBar from '../lib/TopBar.svelte'
  import Icon from '../lib/Icon.svelte'

  let url = $state(getApiBase())
  let busy = $state(false)
  let status: 'idle' | 'ok' | 'fail' = $state(getApiBase() ? 'ok' : 'idle')

  async function save() {
    if (!url.trim()) return toast('ใส่ที่อยู่เซิร์ฟเวอร์ก่อนครับ', 'error')
    busy = true
    status = 'idle'
    try {
      const ok = await testApiBase(url)
      status = ok ? 'ok' : 'fail'
      if (!ok) return toast('ต่อไม่ได้ ตรวจว่าเซิร์ฟเวอร์เปิดอยู่และคัดลอกที่อยู่มาครบ', 'error', 4000)
      setApiBase(url)
      toast('เชื่อมต่อเซิร์ฟเวอร์ได้แล้ว', 'success')
      await loadSession()
      go('/')
    } finally {
      busy = false
    }
  }
  function clear() {
    setApiBase(null)
    url = ''
    status = 'idle'
    toast('ล้างที่อยู่เซิร์ฟเวอร์แล้ว', 'info')
  }
</script>

<TopBar title="ตั้งค่าเซิร์ฟเวอร์" sub="เชื่อมแอปกับระบบบันทึกของฟาร์ม" back="/login" />
<main class="page">
  {#if IS_STATIC_HOST}
    <div class="alert info">
      <Icon name="info" />
      <div>หน้านี้เปิดจากเว็บฟรี ({location.hostname}) ซึ่งเก็บได้แค่ไฟล์แอป <b>ไม่มีที่เก็บข้อมูล</b> เครื่องคำนวณและตารางอาหารใช้ได้เลย แต่ถ้าจะ <b>เข้าสู่ระบบและบันทึกข้อมูลบ่อ</b> ต้องใส่ที่อยู่เซิร์ฟเวอร์ของฟาร์มก่อน</div>
    </div>
  {/if}

  <div class="card mt">
    <h3>ที่อยู่เซิร์ฟเวอร์ฟาร์ม</h3>
    <label for="u">วางลิงก์ที่ได้จากเครื่องเซิร์ฟเวอร์</label>
    <input id="u" bind:value={url} placeholder="เช่น https://xxxx.trycloudflare.com" inputmode="url" autocapitalize="off" autocorrect="off" spellcheck="false" />
    <div class="row mt" style="gap:10px">
      <button class="btn primary" onclick={save} disabled={busy}>{busy ? 'กำลังตรวจสอบ...' : 'เชื่อมต่อ'}</button>
      {#if getApiBase()}<button class="btn ghost" onclick={clear}>ล้าง</button>{/if}
    </div>
    {#if status === 'ok'}<div class="alert good mt">ต่อกับเซิร์ฟเวอร์ได้ ({getApiBase() || url})</div>{/if}
    {#if status === 'fail'}<div class="alert danger mt">ต่อไม่ได้ ตรวจ 3 อย่าง: เซิร์ฟเวอร์เปิดอยู่หรือไม่ · ลิงก์ถูกต้องครบทั้งบรรทัดหรือไม่ · ลิงก์เป็น https หรือไม่</div>{/if}
  </div>

  <div class="card mt">
    <h3>ยังไม่มีเซิร์ฟเวอร์?</h3>
    <div class="reason"><span><b>ทางฟรี:</b> เปิดเครื่องที่บ้าน ดับเบิลคลิก <b>start-mobile.cmd</b> จะได้ลิงก์ https มาวางที่ช่องด้านบน (ลิงก์เปลี่ยนทุกครั้งที่เปิดใหม่)</span></div>
    <div class="reason"><span><b>ทางถาวรฟรี:</b> ติดตั้งบน Oracle Cloud Always Free แล้วได้ลิงก์คงที่ ไม่ต้องเปิดคอมทิ้งไว้ (ดูขั้นตอนใน DEPLOY.md)</span></div>
    <div class="reason"><span>ระหว่างนี้ใช้เครื่องคำนวณได้เลยโดยไม่ต้องมีเซิร์ฟเวอร์</span></div>
    <div class="grid2 mt">
      <a class="btn ghost" href="#/calc">คำนวณอาหารปลา</a>
      <a class="btn ghost" href="#/feed">อาหารและโปรตีน</a>
    </div>
  </div>

  <div class="card mt">
    <h3>ความปลอดภัย</h3>
    <div class="reason"><span>ที่อยู่นี้เก็บไว้ในเครื่องของคุณเท่านั้น ไม่ได้ส่งให้ใคร</span></div>
    <div class="reason"><span>ใส่เฉพาะลิงก์เซิร์ฟเวอร์ของฟาร์มคุณเอง อย่าใส่ลิงก์ที่คนอื่นส่งมาให้ เพราะข้อมูลฟาร์มจะถูกส่งไปที่นั่น</span></div>
    <div class="reason"><span>ทุกการเชื่อมต่อควรเป็น https เท่านั้น</span></div>
  </div>
</main>
