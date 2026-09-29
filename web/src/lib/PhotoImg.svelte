<script lang="ts">
  import { photoUrl } from './photos'
  import Icon from './Icon.svelte'
  let { id, thumb = false, alt = '', fit = 'cover' }: { id: string | null | undefined; thumb?: boolean; alt?: string; fit?: 'cover' | 'contain' } = $props()
  let src: string | null = $state(null)
  let failed = $state(false)
  $effect(() => {
    const want = id
    src = null
    failed = false
    if (!want) return
    photoUrl(want, thumb)
      .then((u) => {
        if (want !== id) return
        if (u) src = u
        else failed = true
      })
      .catch(() => (failed = true))
  })
</script>

{#if src}
  <img {src} {alt} style="object-fit:{fit}" loading="lazy" onerror={() => { src = null; failed = true }} />
{:else}
  <div class="ph" class:failed aria-label={alt}>
    {#if !id || failed}<Icon name="image" size={28} />{/if}
  </div>
{/if}

<style>
  img, .ph { width: 100%; height: 100%; display: block; }
  .ph { display: flex; align-items: center; justify-content: center; background: linear-gradient(90deg, #eef2f7, #e3e9f1, #eef2f7); color: var(--muted); }
  .ph.failed { background: #eef2f7; }
</style>
