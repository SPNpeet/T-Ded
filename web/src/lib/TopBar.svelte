<script lang="ts">
  import Icon from './Icon.svelte'
  let { title, sub = '', back = '', right }: { title: string; sub?: string; back?: string; right?: import('svelte').Snippet } = $props()
  function goBack() {
    if (back) location.hash = back
    else if (history.length > 1) history.back()
    else location.hash = '/'
  }
</script>

<header class="topbar">
  {#if back !== null}
    <button class="icon" onclick={goBack} aria-label="ย้อนกลับ"><Icon name="back" /></button>
  {/if}
  <div style="flex:1;min-width:0">
    <div class="brand" style="font-size:1.05rem;white-space:nowrap;overflow:hidden;text-overflow:ellipsis">{title}</div>
    {#if sub}<div class="sub">{sub}</div>{/if}
  </div>
  {#if right}{@render right()}{/if}
</header>
