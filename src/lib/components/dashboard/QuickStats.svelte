<script>
  import { system } from '../../stores/system.svelte.js';
  import { formatBytes } from '../../utils.js';
  import ProgressBar from '../common/ProgressBar.svelte';
  import { Cpu, HardDrive, MemoryStick, Trash2, ArrowUpRight } from 'lucide-svelte';

  const ramUsed = $derived(system.hardware?.used_memory_bytes ?? 0);
  const ramTotal = $derived(system.hardware?.total_memory_bytes ?? 1);
  const ramPercent = $derived(system.hardware?.memory_percent ?? 0);

  const cpuPercent = $derived(system.hardware?.global_cpu_percent ?? 0);
  const cpuCores = $derived(system.hardware?.cpu_core_count ?? 8);

  const primaryDisk = $derived(system.hardware?.disks?.[0]);
  const diskFree = $derived(primaryDisk?.available_bytes ?? 0);
  const diskTotal = $derived(primaryDisk?.total_bytes ?? 1);
  const diskFreePercent = $derived(primaryDisk?.free_percent ?? 0);

  const safeJunk = $derived(system.health?.safe_cleanup_bytes ?? 0);
</script>

<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mt-6">
  <!-- RAM -->
  <div
    class="bg-slate-900/70 border border-slate-800 rounded-xl p-4 flex flex-col justify-between hover:border-slate-700 transition-colors cursor-pointer group"
    onclick={() => system.setActiveTab('processes')}
    role="button"
    tabindex="0"
    onkeydown={(e) => e.key === 'Enter' && system.setActiveTab('processes')}
  >
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2 text-xs font-semibold text-slate-400 uppercase tracking-wider">
        <MemoryStick class="w-3.5 h-3.5 text-cyan-400" />
        <span>Memory</span>
      </div>
      <ArrowUpRight class="w-3.5 h-3.5 text-slate-500 group-hover:text-cyan-400 transition-colors" />
    </div>

    <div class="mb-2">
      <div class="flex items-baseline gap-1.5">
        <span class="text-2xl font-bold font-mono text-slate-100">{ramPercent.toFixed(0)}%</span>
        <span class="text-xs text-slate-400">used</span>
      </div>
      <p class="text-xs text-slate-500 mt-0.5 font-mono">
        {formatBytes(ramUsed)} / {formatBytes(ramTotal)}
      </p>
    </div>

    <ProgressBar
      value={ramPercent}
      color={ramPercent > 85 ? 'rose' : ramPercent > 70 ? 'amber' : 'cyan'}
      size="sm"
    />
  </div>

  <!-- CPU -->
  <div
    class="bg-slate-900/70 border border-slate-800 rounded-xl p-4 flex flex-col justify-between hover:border-slate-700 transition-colors cursor-pointer group"
    onclick={() => system.setActiveTab('processes')}
    role="button"
    tabindex="0"
    onkeydown={(e) => e.key === 'Enter' && system.setActiveTab('processes')}
  >
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2 text-xs font-semibold text-slate-400 uppercase tracking-wider">
        <Cpu class="w-3.5 h-3.5 text-indigo-400" />
        <span>CPU Load</span>
      </div>
      <ArrowUpRight class="w-3.5 h-3.5 text-slate-500 group-hover:text-indigo-400 transition-colors" />
    </div>

    <div class="mb-2">
      <div class="flex items-baseline gap-1.5">
        <span class="text-2xl font-bold font-mono text-slate-100">{cpuPercent.toFixed(1)}%</span>
        <span class="text-xs text-slate-400">sampled</span>
      </div>
      <p class="text-xs text-slate-500 mt-0.5">
        {cpuCores} Logical Processors active
      </p>
    </div>

    <ProgressBar
      value={cpuPercent}
      color={cpuPercent > 80 ? 'rose' : cpuPercent > 50 ? 'amber' : 'cyan'}
      size="sm"
    />
  </div>

  <!-- Disk -->
  <div
    class="bg-slate-900/70 border border-slate-800 rounded-xl p-4 flex flex-col justify-between hover:border-slate-700 transition-colors cursor-pointer group"
    onclick={() => system.setActiveTab('storage')}
    role="button"
    tabindex="0"
    onkeydown={(e) => e.key === 'Enter' && system.setActiveTab('storage')}
  >
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2 text-xs font-semibold text-slate-400 uppercase tracking-wider">
        <HardDrive class="w-3.5 h-3.5 text-emerald-400" />
        <span>Primary Storage</span>
      </div>
      <ArrowUpRight class="w-3.5 h-3.5 text-slate-500 group-hover:text-emerald-400 transition-colors" />
    </div>

    <div class="mb-2">
      <div class="flex items-baseline gap-1.5">
        <span class="text-2xl font-bold font-mono text-slate-100">{formatBytes(diskFree)}</span>
        <span class="text-xs text-slate-400">free</span>
      </div>
      <p class="text-xs text-slate-500 mt-0.5 font-mono">
        {diskFreePercent.toFixed(1)}% free of {formatBytes(diskTotal)}
      </p>
    </div>

    <ProgressBar
      value={100 - diskFreePercent}
      color={diskFreePercent < 10 ? 'rose' : diskFreePercent < 20 ? 'amber' : 'emerald'}
      size="sm"
    />
  </div>

  <!-- Safe Cleanup -->
  <div
    class="bg-slate-900/70 border border-slate-800 rounded-xl p-4 flex flex-col justify-between hover:border-slate-700 transition-colors cursor-pointer group"
    onclick={() => system.setActiveTab('cleanup')}
    role="button"
    tabindex="0"
    onkeydown={(e) => e.key === 'Enter' && system.setActiveTab('cleanup')}
  >
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2 text-xs font-semibold text-slate-400 uppercase tracking-wider">
        <Trash2 class="w-3.5 h-3.5 text-amber-400" />
        <span>Safe Junk Cache</span>
      </div>
      <ArrowUpRight class="w-3.5 h-3.5 text-slate-500 group-hover:text-amber-400 transition-colors" />
    </div>

    <div class="mb-2">
      <div class="flex items-baseline gap-1.5">
        <span class="text-2xl font-bold font-mono text-emerald-400">{formatBytes(safeJunk)}</span>
        <span class="text-xs text-slate-400">ready</span>
      </div>
      <p class="text-xs text-slate-500 mt-0.5">
        Safe temporary & cache files
      </p>
    </div>

    <div class="pt-2 border-t border-slate-800/80 flex items-center justify-between">
      <span class="text-xs font-semibold text-cyan-400 group-hover:text-cyan-300">
        Review & Clean →
      </span>
    </div>
  </div>
</div>
