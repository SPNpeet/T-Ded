<script lang="ts">
  // กล่องพับ/กางที่กดได้เต็มแถบ และเนื้อหาถูก mount ใหม่ทุกครั้งที่กาง (กราฟจึงวาดขนาดถูกเสมอ)
  import Icon from './Icon.svelte'
  let { title, open = false, children }: { title: string; open?: boolean; children: import('svelte').Snippet } = $props()
  let show = $state(open)
</script>

<div class="collapse">
  <button type="button" class="collapse-head" aria-expanded={show} onclick={() => (show = !show)}>
    <span>{title}</span>
    <span class="chev" class:open={show}><Icon name="back" size={20} /></span>
  </button>
  {#if show}
    <div class="collapse-body">{@render children()}</div>
  {/if}
</div>

<style>
  .collapse {
    border: 2px solid var(--line);
    border-radius: 14px;
    background: #fff;
    overflow: hidden;
  }
  .collapse-head {
    width: 100%;
    min-height: 56px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 12px 16px;
    background: #fff;
    border: none;
    cursor: pointer;
    font-weight: 700;
    font-size: 1.02rem;
    color: var(--navy);
    text-align: left;
  }
  .chev {
    display: inline-flex;
    transform: rotate(-90deg);
    transition: transform 0.15s ease;
    color: var(--cyan-deep);
  }
  .chev.open {
    transform: rotate(90deg);
  }
  .collapse-body {
    padding: 4px 16px 16px;
    border-top: 1px solid var(--line);
  }
</style>
