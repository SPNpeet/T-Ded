<script lang="ts">
  import { healthColor } from './format'
  let { score, label = 'คะแนน', size = 96 }: { score: number; label?: string; size?: number } = $props()
  const r = 42
  const c = 2 * Math.PI * r
  const dash = $derived((Math.max(0, Math.min(100, score)) / 100) * c)
</script>

<div class="score-ring" style="width:{size}px;height:{size}px">
  <svg viewBox="0 0 100 100" width={size} height={size}>
    <circle cx="50" cy="50" r={r} stroke="#e6ebf3" stroke-width="10" fill="none" />
    <circle cx="50" cy="50" r={r} stroke={healthColor(score)} stroke-width="10" fill="none" stroke-linecap="round" stroke-dasharray="{dash} {c - dash}" />
  </svg>
  <div class="val" style="font-size:{size * 0.28}px">{score}<small>{label}</small></div>
</div>
