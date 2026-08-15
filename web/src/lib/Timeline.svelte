<script lang="ts">
  // เส้นเวลา ปล่อย -> วันนี้ -> จับ พร้อมหมุดใหญ่ อ่านง่ายกว่ากราฟ
  let { total, today = 0, marks }: { total: number; today?: number; marks: { day: number; label: string; sub?: string }[] } = $props()
  const pct = (d: number) => Math.max(0, Math.min(100, (d / Math.max(1, total)) * 100))
</script>

<div style="position:relative;padding:34px 8px 44px">
  <div style="height:14px;background:#e6ebf3;border-radius:999px;overflow:hidden">
    <div style="width:{pct(today)}%;height:100%;background:linear-gradient(90deg,var(--cyan),var(--cyan-deep));border-radius:999px"></div>
  </div>
  {#each marks as m, i}
    <div style="position:absolute;left:{pct(m.day)}%;top:0;transform:translateX({i === 0 ? '0' : i === marks.length - 1 ? '-100%' : '-50%'});text-align:{i === 0 ? 'left' : i === marks.length - 1 ? 'right' : 'center'};white-space:nowrap">
      <div class="bold" style="font-size:0.95rem">{m.label}</div>
    </div>
    <div style="position:absolute;left:{pct(m.day)}%;top:31px;width:20px;height:20px;border-radius:50%;background:#fff;border:4px solid {m.day <= today ? 'var(--cyan-deep)' : 'var(--navy)'};transform:translateX(-50%)"></div>
    {#if m.sub}
      <div class="small muted" style="position:absolute;left:{pct(m.day)}%;top:58px;transform:translateX({i === 0 ? '0' : i === marks.length - 1 ? '-100%' : '-50%'});white-space:nowrap">{m.sub}</div>
    {/if}
  {/each}
</div>
