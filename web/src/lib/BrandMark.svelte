<script lang="ts">
  import { brand } from './brand'
  let { size = 'sm' }: { size?: 'sm' | 'lg' } = $props()
  const BASE = import.meta.env.BASE_URL
</script>

{#if brand.id === 'teedet'}
  {#if size === 'lg'}
    <img src="{BASE}logo-full.png" alt={brand.appName} style="width:260px;height:auto;display:block" />
  {:else}
    <img src="{BASE}mark.png" alt="" />
  {/if}
{:else if size === 'lg'}
  <div class="wordmark lg"><span class="nb">NB</span><div><div class="t1">{brand.appName}</div><div class="t2">{brand.companyTh}</div></div></div>
{:else}
  <span class="wordmark sm"><span class="nb">NB</span></span>
{/if}

<style>
  .wordmark { display: flex; align-items: center; gap: 12px; }
  .nb { font-family: 'Prompt', sans-serif; font-weight: 800; letter-spacing: -0.02em; background: #F28C28; color: #fff; border-radius: 10px; display: inline-flex; align-items: center; justify-content: center; }
  .sm .nb { width: 34px; height: 34px; font-size: 0.95rem; }
  .lg .nb { width: 64px; height: 64px; font-size: 1.7rem; border-radius: 16px; }
  .t1 { font-family: 'Prompt', sans-serif; font-weight: 800; font-size: 1.35rem; color: var(--navy); line-height: 1.2; }
  .t2 { font-size: 0.85rem; color: var(--muted); }
</style>
