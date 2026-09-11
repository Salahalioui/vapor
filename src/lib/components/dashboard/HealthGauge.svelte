<script>
  import { system } from '../../stores/system.svelte.js';
  import { formatBytes } from '../../utils.js';
  import Badge from '../common/Badge.svelte';
  import ProgressBar from '../common/ProgressBar.svelte';

  const isInitialLoading = $derived(!system.health && system.isRefreshingHealth);
  const score = $derived(system.health?.overall_score ?? (isInitialLoading ? '--' : 100));
  const grade = $derived(system.health?.grade ?? (isInitialLoading ? 'Evaluating...' : 'Ready'));
  const h = $derived(system.health);
  const pillars = $derived(system.health?.pillars ?? {
    disk_score: 100,
    memory_score: 100,
    cleanup_score: 100,
    startup_score: 100,
  });

  const diskSublabel = $derived(
    h ? `${formatBytes(h.primary_disk_free_bytes)} free (${h.primary_disk_free_percent.toFixed(0)}%)` : ''
  );
  const diskColor = $derived(
    !h ? 'emerald' : h.primary_disk_free_percent < 10 ? 'rose' : h.primary_disk_free_percent < 20 ? 'amber' : 'emerald'
  );

  const memorySublabel = $derived(
    h ? `${formatBytes(h.used_memory_bytes)} used (${h.memory_percent.toFixed(0)}%)` : ''
  );
  const memoryColor = $derived(
    !h ? 'emerald' : h.memory_percent > 85 ? 'rose' : h.memory_percent > 70 ? 'amber' : 'emerald'
  );

  const cleanupSublabel = $derived(
    h ? `${formatBytes(h.safe_cleanup_bytes)} cleanable` : ''
  );

  const startupSublabel = $derived(
    h ? `${h.startup_count + h.zombie_count} apps (${h.startup_count} boot, ${h.zombie_count} dormant)` : ''
  );

  // Calculate circumference and stroke dash offset for SVG circle
  const radius = 64;
  const circumference = 2 * Math.PI * radius;
  const numericScore = $derived(typeof score === 'number' ? score : 0);
  const strokeDashoffset = $derived(circumference - (numericScore / 100) * circumference);

  const scoreColor = $derived(
    typeof score !== 'number'
      ? 'text-slate-500 stroke-slate-700'
      : score >= 85
      ? 'text-emerald-400 stroke-emerald-500'
      : score >= 70
      ? 'text-cyan-400 stroke-cyan-500'
      : score >= 50
      ? 'text-amber-400 stroke-amber-500'
      : 'text-rose-400 stroke-rose-500'
  );

  const gradeBadgeVariant = $derived(
    typeof score !== 'number'
      ? 'default'
      : score >= 85
      ? 'success'
      : score >= 70
      ? 'accent'
      : score >= 50
      ? 'warning'
      : 'danger'
  );
</script>

<div class="flex flex-col md:flex-row items-center gap-8 bg-slate-900/80 border border-slate-800/90 rounded-2xl p-6 shadow-xl relative overflow-hidden">
  <!-- Glow effect background -->
  <div class="absolute -top-24 -left-24 w-64 h-64 bg-cyan-500/10 rounded-full blur-3xl pointer-events-none"></div>

  <!-- Radial Gauge -->
  <div class="relative flex items-center justify-center shrink-0">
    <svg class="w-44 h-44 -rotate-90 transform" viewBox="0 0 160 160">
      <!-- Background track -->
      <circle
        cx="80"
        cy="80"
        r={radius}
        class="stroke-slate-800"
        stroke-width="12"
        fill="transparent"
      />
      <!-- Progress circle -->
      <circle
        cx="80"
        cy="80"
        r={radius}
        class="{scoreColor} transition-all duration-1000 ease-out"
        stroke-width="12"
        stroke-linecap="round"
        stroke-dasharray={circumference}
        stroke-dashoffset={strokeDashoffset}
        fill="transparent"
      />
    </svg>

    <div class="absolute flex flex-col items-center justify-center text-center">
      <span class="text-4xl font-extrabold tracking-tight font-mono text-slate-100">
        {score}
      </span>
      <span class="text-[11px] uppercase tracking-widest text-slate-400 font-semibold mt-0.5">
        Health
      </span>
    </div>
  </div>

  <!-- Details & Pillars -->
  <div class="flex-1 w-full flex flex-col justify-center">
    <div class="flex items-center justify-between mb-4">
      <div>
        <div class="flex items-center gap-2.5">
          <h2 class="text-xl font-bold text-slate-100 tracking-tight">System Status</h2>
          <Badge variant={gradeBadgeVariant}>{grade}</Badge>
        </div>
        <p class="text-xs text-slate-400 mt-1">
          Composite evaluation of storage headroom, active memory load, junk volume, and startup drag.
        </p>
      </div>

      <button
        class="flex items-center gap-2 px-3 py-1.5 text-xs font-medium text-slate-300 bg-slate-800 hover:bg-slate-700 active:scale-95 border border-slate-700 rounded-lg transition-all"
        onclick={() => system.refreshHealth()}
        disabled={system.isRefreshingHealth}
      >
        <svg
          class="w-3.5 h-3.5 {system.isRefreshingHealth ? 'animate-spin text-cyan-400' : ''}"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
          />
        </svg>
        <span>{system.isRefreshingHealth ? 'Scanning...' : 'Re-Evaluate'}</span>
      </button>
    </div>

    <!-- 4 Pillars Grid -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 mt-1">
      <ProgressBar
        label="Disk Headroom (35%)"
        sublabel={diskSublabel}
        value={pillars.disk_score}
        color={diskColor}
        size="sm"
      />
      <ProgressBar
        label="Memory Pressure (25%)"
        sublabel={memorySublabel}
        value={pillars.memory_score}
        color={memoryColor}
        size="sm"
      />
      <ProgressBar
        label="Safe Cache Cleanliness (20%)"
        sublabel={cleanupSublabel}
        value={pillars.cleanup_score}
        color={pillars.cleanup_score >= 80 ? 'emerald' : 'amber'}
        size="sm"
      />
      <ProgressBar
        label="Startup & Zombie Load (20%)"
        sublabel={startupSublabel}
        value={pillars.startup_score}
        color={pillars.startup_score >= 80 ? 'emerald' : 'amber'}
        size="sm"
      />
    </div>
  </div>
</div>
