<script lang="ts">
  // กราฟเส้นแบบเบา (SVG) รองรับหลายชุดข้อมูล ใช้กับการโต/น้ำ/ต้นทุน
  type Series = { name: string; color: string; points: { x: number; y: number }[]; dashed?: boolean }
  let {
    series,
    xLabel = (x: number) => String(x),
    yLabel = (y: number) => String(Math.round(y)),
    height = 220,
    yMin,
    bands = [],
  }: {
    series: Series[]
    xLabel?: (x: number) => string
    yLabel?: (y: number) => string
    height?: number
    yMin?: number
    bands?: { from: number; to: number; color: string }[]
  } = $props()

  // วาดตามความกว้างจริงของกล่อง (1 หน่วย = 1 พิกเซล) กราฟจึงไม่ถูกบีบบนมือถือ
  let boxW = $state(0)
  const W = $derived(Math.max(280, boxW || 640))
  const P = { l: 56, r: 40, t: 12, b: 32 }
  const all = $derived(series.flatMap((s) => s.points))
  const xs = $derived(all.map((p) => p.x))
  const ys = $derived(all.map((p) => p.y).concat(bands.flatMap((b) => [b.from, b.to])))
  const x0 = $derived(xs.length ? Math.min(...xs) : 0)
  const x1 = $derived(xs.length ? Math.max(...xs) : 1)
  const y0 = $derived(yMin !== undefined ? yMin : ys.length ? Math.min(0, Math.min(...ys)) : 0)
  const y1raw = $derived(ys.length ? Math.max(...ys) : 1)
  const y1 = $derived(y1raw === y0 ? y0 + 1 : y1raw * 1.08)
  const sx = (x: number) => P.l + ((x - x0) / Math.max(1e-9, x1 - x0)) * (W - P.l - P.r)
  const sy = (y: number) => height - P.b - ((y - y0) / Math.max(1e-9, y1 - y0)) * (height - P.t - P.b)
  const path = (pts: { x: number; y: number }[]) => pts.map((p, i) => `${i ? 'L' : 'M'}${sx(p.x).toFixed(1)},${sy(p.y).toFixed(1)}`).join(' ')
  const nice = (v: number) => { if (v === 0) return 0; const p = Math.pow(10, Math.floor(Math.log10(Math.abs(v)))); const m = v / p; const r = m <= 1 ? 1 : m <= 2 ? 2 : m <= 5 ? 5 : 10; return r * p }
  const yTicks = $derived.by(() => { const step = nice((y1 - y0) / 4); const out: number[] = []; for (let t = Math.ceil(y0 / step) * step; t <= y1 + 1e-9; t += step) out.push(t); return out })
  const xTicks = $derived.by(() => { const span = x1 - x0; if (span <= 0) return [x0]; const step = span <= 14 ? 2 : span <= 60 ? 7 : span <= 200 ? 14 : 30; const out: number[] = []; for (let t = Math.ceil(x0 / step) * step; t <= x1; t += step) out.push(t); if (!out.length || out[out.length - 1] < x1 - step / 2) out.push(x1); return out })
</script>

<div bind:clientWidth={boxW} style="width:100%">
<svg class="chart" viewBox="0 0 {W} {height}" width={W} {height} role="img">
  {#each bands as b}
    <rect x={P.l} y={sy(b.to)} width={W - P.l - P.r} height={Math.max(0, sy(b.from) - sy(b.to))} fill={b.color} opacity="0.35" />
  {/each}
  {#each yTicks as t}
    <line x1={P.l} x2={W - P.r} y1={sy(t)} y2={sy(t)} stroke="#e6ebf3" />
    <text x={P.l - 6} y={sy(t) + 4} text-anchor="end">{yLabel(t)}</text>
  {/each}
  {#each xTicks as t}
    <text x={sx(t)} y={height - 8} text-anchor={t >= x1 - 1e-9 ? "end" : t <= x0 + 1e-9 ? "start" : "middle"}>{xLabel(t)}</text>
  {/each}
  {#each series as s}
    {#if s.points.length > 1}
      <path d={path(s.points)} fill="none" stroke={s.color} stroke-width="3" stroke-dasharray={s.dashed ? '7 6' : undefined} stroke-linejoin="round" stroke-linecap="round" />
    {/if}
    {#each s.points as p}
      <circle cx={sx(p.x)} cy={sy(p.y)} r={s.points.length > 40 ? 0 : 4} fill={s.color} />
    {/each}
  {/each}
</svg>
</div>
<div class="row wrap small" style="gap:14px;margin-top:4px">
  {#each series as s}
    <span class="row" style="gap:6px"><span style="width:18px;height:4px;background:{s.color};display:inline-block;border-radius:2px"></span>{s.name}</span>
  {/each}
</div>
