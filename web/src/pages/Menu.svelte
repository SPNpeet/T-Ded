<script lang="ts">
  import { session, currentFarm, logout, isStaff, ui } from '../lib/ui.svelte'
  import Icon from '../lib/Icon.svelte'
  import { FONT_SIZES, getFontSize, getHighContrast, setFontSize, setHighContrast } from '../lib/prefs'
  let fs = $state(getFontSize())
  let hc = $state(getHighContrast())
  const items = [
    ['#/calc', 'calc', 'คำนวณอาหาร (แบบเร็ว)'],
    ['#/simulate', 'chart', 'จำลองรุ่นเลี้ยงก่อนลงทุน'],
    ['#/feed', 'feed', 'อาหารและโปรตีน / ผสมอาหารเอง'],
    ['#/prices', 'money', 'ราคาปลาในพื้นที่'],
    ['#/diseases', 'map', 'โรคปลาในพื้นที่'],
    ['#/stock', 'stock', 'สต๊อกอาหาร'],
    ['#/ponds', 'pond', 'จัดการบ่อ / ปล่อยรุ่นใหม่'],
    ['#/settings', 'settings', 'ตั้งค่าฟาร์ม, เชื่อม LINE, PIN'],
  ]
</script>

<header class="topbar"><div class="brand"><img src="/mark.png" alt="" />เพิ่มเติม</div></header>
<main class="page">
  <div class="card">
    <div class="row"><div style="flex:1"><b>{session.user?.name}</b><div class="small muted">{session.user?.phone} · {session.user?.role === 'owner' ? 'เจ้าของฟาร์ม' : session.user?.role === 'worker' ? 'คนงาน' : session.user?.role === 'officer' ? 'เจ้าหน้าที่ส่งเสริม' : 'ผู้ดูแลระบบ'}</div><div class="small muted">{currentFarm()?.name ?? ''} · {session.user?.org_name}</div></div></div>
  </div>
  <div class="card mt">
    <h3>ขนาดตัวอักษร</h3>
    <div class="fs-row mt">
      {#each FONT_SIZES as f}<button class:active={fs === f.key} style="font-size:{f.px}px" onclick={() => { setFontSize(f.key); fs = f.key }}>{f.label}</button>{/each}
    </div>
    <h3 class="mt2">โหมดกลางแจ้ง (คอนทราสต์สูง)</h3>
    <p class="small muted">พื้นขาว ตัวหนังสือดำ เห็นชัดกลางแดด</p>
    <div class="fs-row mt"><button class:active={!hc} onclick={() => { setHighContrast(false); hc = false }}>ปกติ</button><button class:active={hc} onclick={() => { setHighContrast(true); hc = true }}>เปิดโหมดกลางแจ้ง</button></div>
  </div>
  <div class="card mt" style="padding:6px 18px">
    <div class="list">
      {#each items as [href, icon, label]}
        <a class="list-item" {href} style="text-decoration:none;color:inherit"><Icon name={icon} /><div class="main title">{label}</div><Icon name="back" size={18} /></a>
      {/each}
      {#if isStaff()}<a class="list-item" href="#/admin" style="text-decoration:none;color:inherit"><Icon name="users" /><div class="main title">หลังบ้านเจ้าหน้าที่</div></a>{/if}
    </div>
  </div>
  {#if ui.queue}<div class="alert warn mt">มี {ui.queue} รายการรอส่งเมื่อมีสัญญาณ ดูได้ในหน้าตั้งค่า</div>{/if}
  <button class="btn ghost mt2" onclick={logout}><Icon name="logout" />ออกจากระบบ</button>
  <p class="center tiny muted mt2">ทีเด็ดปลาน้ำจืด · ด้วยอาหารคุณภาพ และคำปรึกษาจากมืออาชีพ</p>
</main>
