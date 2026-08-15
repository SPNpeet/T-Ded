<script lang="ts">
  // แถบเงินแบบชาวบ้านอ่านออก: ได้เท่าไหร่ จ่ายเท่าไหร่ เหลือเท่าไหร่ (ไม่ใช่กราฟเส้น)
  import { baht } from './format'
  let { rows }: { rows: { label: string; value: number; color: string; note?: string }[] } = $props()
  const max = $derived(Math.max(1, ...rows.map((r) => Math.abs(r.value))))
</script>

<div class="stack" style="--gap:10px">
  {#each rows as r}
    <div>
      <div class="row" style="justify-content:space-between;align-items:baseline">
        <span class="bold">{r.label}</span>
        <span class="bold num" style="color:{r.color};font-size:1.15rem;white-space:nowrap">{baht(r.value)}</span>
      </div>
      <div class="progress" style="height:18px;margin-top:4px"><div style="width:{Math.max(2, (Math.abs(r.value) / max) * 100)}%;background:{r.color}"></div></div>
      {#if r.note}<div class="small muted" style="margin-top:2px">{r.note}</div>{/if}
    </div>
  {/each}
</div>
