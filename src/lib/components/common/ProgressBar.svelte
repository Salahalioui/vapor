<script>
  let {
    value = 0,
    max = 100,
    color = 'cyan', // 'cyan' | 'emerald' | 'amber' | 'rose'
    size = 'md',   // 'sm' | 'md' | 'lg'
    label = '',
    sublabel = '',
    class: className = ''
  } = $props();

  const percentage = $derived(Math.min(100, Math.max(0, (value / (max || 1)) * 100)));

  const colorClasses = {
    cyan: 'bg-cyan-500 shadow-[0_0_12px_rgba(6,182,212,0.5)]',
    emerald: 'bg-emerald-500 shadow-[0_0_12px_rgba(16,185,129,0.5)]',
    amber: 'bg-amber-500 shadow-[0_0_12px_rgba(245,158,11,0.5)]',
    rose: 'bg-rose-500 shadow-[0_0_12px_rgba(239,68,68,0.5)]',
  };

  const sizeClasses = {
    sm: 'h-1.5',
    md: 'h-2.5',
    lg: 'h-4',
  };
</script>

<div class="w-full {className}">
  {#if label || sublabel}
    <div class="flex justify-between items-center text-xs mb-1.5 font-medium">
      <span class="text-slate-300">{label}</span>
      <span class="text-slate-400 font-mono">{sublabel || `${percentage.toFixed(0)}%`}</span>
    </div>
  {/if}
  <div class="w-full bg-slate-800/80 rounded-full overflow-hidden border border-slate-700/50 {sizeClasses[size] || sizeClasses.md}">
    <div
      class="h-full rounded-full transition-all duration-500 ease-out {colorClasses[color] || colorClasses.cyan}"
      style="width: {percentage}%;"
    ></div>
  </div>
</div>
