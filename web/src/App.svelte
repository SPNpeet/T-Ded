<script lang="ts">
  import { onMount } from 'svelte'
  import { ui, session, loadSession, isStaff } from './lib/ui.svelte'
  import Icon from './lib/Icon.svelte'
  import Login from './pages/Login.svelte'
  import Today from './pages/Today.svelte'
  import CropDetail from './pages/CropDetail.svelte'
  import LogForm from './pages/LogForm.svelte'
  import WeighForm from './pages/WeighForm.svelte'
  import WaterForm from './pages/WaterForm.svelte'
  import MoneyForm from './pages/MoneyForm.svelte'
  import Stock from './pages/Stock.svelte'
  import Finance from './pages/Finance.svelte'
  import Calculator from './pages/Calculator.svelte'
  import Simulator from './pages/Simulator.svelte'
  import Prices from './pages/Prices.svelte'
  import Diseases from './pages/Diseases.svelte'
  import Settings from './pages/Settings.svelte'
  import Ponds from './pages/Ponds.svelte'
  import NewCrop from './pages/NewCrop.svelte'
  import Admin from './pages/Admin.svelte'
  import Report from './pages/Report.svelte'
  import Menu from './pages/Menu.svelte'
  import FeedMix from './pages/FeedMix.svelte'

  onMount(() => {
    loadSession()
  })

  const parts = $derived(ui.route.split('?')[0].split('/').filter(Boolean))
  const seg = (i: number) => parts[i] ?? ''
  const isPublic = $derived(['calc', 'simulate', 'login', 'register', 'feed'].includes(seg(0)))
  const showNav = $derived(!!session.user && !['login', 'register'].includes(seg(0)))
  const navActive = (k: string) => (k === '/' ? parts.length === 0 || seg(0) === 'pond' : seg(0) === k.slice(1))
</script>

{#if !ui.online}
  <div class="offline-bar">ออฟไลน์อยู่ ใช้งานได้ ข้อมูลที่บันทึกจะส่งให้เมื่อมีสัญญาณ{ui.queue ? ` (ค้างส่ง ${ui.queue})` : ''}</div>
{/if}

{#if session.loading}
  <div class="page center" style="padding-top:30vh">
    <div class="skeleton" style="max-width:320px;margin:0 auto"></div>
    <p class="muted mt">กำลังโหลด...</p>
  </div>
{:else if !session.user && !isPublic}
  <Login />
{:else if seg(0) === 'login' || seg(0) === 'register'}
  <Login mode={seg(0)} />
{:else if seg(0) === 'calc'}
  <Calculator />
{:else if seg(0) === 'simulate'}
  <Simulator />
{:else if seg(0) === 'pond'}
  <CropDetail cropId={seg(1)} tab={seg(2) || 'feed'} />
{:else if seg(0) === 'log'}
  <LogForm cropId={seg(1)} />
{:else if seg(0) === 'weigh'}
  <WeighForm cropId={seg(1)} />
{:else if seg(0) === 'water'}
  <WaterForm pondId={seg(1)} />
{:else if seg(0) === 'expense' || seg(0) === 'harvest' || seg(0) === 'treatment'}
  <MoneyForm kind={seg(0)} cropId={seg(1)} />
{:else if seg(0) === 'stock'}
  <Stock />
{:else if seg(0) === 'finance'}
  <Finance />
{:else if seg(0) === 'prices'}
  <Prices />
{:else if seg(0) === 'diseases'}
  <Diseases />
{:else if seg(0) === 'settings'}
  <Settings />
{:else if seg(0) === 'ponds'}
  <Ponds />
{:else if seg(0) === 'new-crop'}
  <NewCrop pondId={seg(1)} />
{:else if seg(0) === 'admin'}
  <Admin sub={seg(1) || 'farms'} id={seg(2)} />
{:else if seg(0) === 'report'}
  <Report cropId={seg(1)} />
{:else if seg(0) === 'feed'}
  <FeedMix />
{:else if seg(0) === 'menu'}
  <Menu />
{:else}
  <Today />
{/if}

{#if showNav}
  <nav class="bottomnav" aria-label="เมนูหลัก">
    <a href="#/" class:active={navActive('/')}><Icon name="home" />วันนี้</a>
    <a href="#/ponds" class:active={navActive('/ponds')}><Icon name="pond" />บ่อ</a>
    <a href="#/finance" class:active={navActive('/finance')}><Icon name="money" />เงิน</a>
    {#if isStaff()}
      <a href="#/admin" class:active={navActive('/admin')}><Icon name="users" />ฟาร์มทั้งหมด</a>
    {:else}
      <a href="#/stock" class:active={navActive('/stock')}><Icon name="stock" />อาหาร</a>
    {/if}
    <a href="#/menu" class:active={navActive('/menu')}><Icon name="settings" />เพิ่มเติม</a>
  </nav>
{/if}

<div aria-live="polite">
  {#each ui.toasts as t (t.id)}
    <div class="toast {t.kind}">{t.text}</div>
  {/each}
</div>
