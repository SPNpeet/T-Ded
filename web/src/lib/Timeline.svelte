<script lang="ts">
  // เส้นเวลา ปล่อย -> วันนี้ -> จับ : หมุดที่อยู่ใกล้กันจะถูกรวมกันเพื่อไม่ให้ตัวหนังสือทับกัน
  type Mark = { day: number; label: string; sub?: string }
  let { total, today = 0, marks }: { total: number; today?: number; marks: Mark[] } = $props()

  const pct = (d: number) => Math.max(0, Math.min(100, (d / Math.max(1, total)) * 100))
  // รวมหมุดที่ห่างกันน้อยกว่า 12% ของเส้น (เช่น วันปล่อย = วันนี้)
  const merged = $derived.by(() => {
    const sorted = [...marks].sort((a, b) => a.day - b.day)
    const out: Mark[] = []
    for (const m of sorted) {
      const last = out[out.length - 1]
      if (last && Math.abs(pct(m.day) - pct(last.day)) < 12) {
        last.label = last.label === m.label ? m.label : `${last.label} = ${m.label}`
        last.sub = [last.sub, m.sub].filter(Boolean).join(' · ')
      } else {
        out.push({ ...m })
      }
    }
    return out
  })
  const align = (i: number, n: number) => (i === 0 ? 'left' : i === n - 1 ? 'right' : 'center')
  const shift = (i: number, n: number) => (i === 0 ? '0' : i === n - 1 ? '-100%' : '-50%')
</script>

<div class="tl">
  <div class="tl-track"><div class="tl-fill" style="width:{pct(today)}%"></div></div>
  {#each merged as m, i}
    <div class="tl-label" style="left:{pct(m.day)}%;transform:translateX({shift(i, merged.length)});text-align:{align(i, merged.length)}">
      <div class="tl-name">{m.label}</div>
      {#if m.sub}<div class="tl-sub">{m.sub}</div>{/if}
    </div>
    <div class="tl-dot" style="left:{pct(m.day)}%;border-color:{m.day <= today ? 'var(--cyan-deep)' : 'var(--navy)'}"></div>
  {/each}
</div>

<style>
  .tl {
    position: relative;
    padding: 46px 6px 10px;
  }
  .tl-track {
    height: 14px;
    background: #e6ebf3;
    border-radius: 999px;
    overflow: hidden;
  }
  .tl-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--cyan), var(--cyan-deep));
    border-radius: 999px;
  }
  .tl-label {
    position: absolute;
    top: 0;
    max-width: 46%;
    line-height: 1.25;
  }
  .tl-name {
    font-weight: 700;
    font-size: 0.95rem;
    color: var(--navy);
  }
  .tl-sub {
    font-size: 0.85rem;
    color: var(--muted);
  }
  .tl-dot {
    position: absolute;
    top: 43px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #fff;
    border: 4px solid var(--navy);
    transform: translateX(-50%);
  }
</style>
