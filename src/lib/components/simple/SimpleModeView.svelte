<script>
  import { system } from '../../stores/system.svelte.js';
  import { settings } from '../../stores/settings.svelte.js';
  import { formatBytes } from '../../utils.js';
  import {
    Sparkles,
    ShieldCheck,
    Cpu,
    HardDrive,
    Gamepad2,
    Code,
    Clock,
    AlertTriangle,
    CheckCircle2,
    ToggleLeft,
    ToggleRight,
    Trash2,
    HelpCircle,
    Info,
    RefreshCw,
    Sliders,
    Flame
  } from 'lucide-svelte';

  let smartCleanRunning = $state(false);
  let smartCleanError = $state(null);
  let applyingTweakId = $state(null);
  let tweakError = $state(null);
  let uninstallingPkg = $state(null);
  let sponsoredError = $state(null);

  const safeCleanupBytes = $derived(system.health?.safe_cleanup_bytes ?? 0);
  const specs = $derived(system.pcSpecs);
  const capabilities = $derived(system.pcSpecs?.capabilities);
  const tweaks = $derived(system.pcTweaks || []);
  const sponsored = $derived(system.sponsoredApps || []);

  async function handleSmartClean() {
    smartCleanRunning = true;
    smartCleanError = null;
    try {
      await system.runSmartClean();
    } catch (e) {
      smartCleanError = typeof e === 'string' ? e : e?.message || 'Smart clean failed';
    } finally {
      smartCleanRunning = false;
    }
  }

  async function handleToggleTweak(tweak) {
    applyingTweakId = tweak.id;
    tweakError = null;
    try {
      await system.togglePcTweak(tweak.id, !tweak.is_applied);
    } catch (e) {
      tweakError = typeof e === 'string' ? e : e?.message || `Failed to update tweak ${tweak.title}`;
    } finally {
      applyingTweakId = null;
    }
  }

  async function handleUninstallSponsored(pkg) {
    uninstallingPkg = pkg;
    sponsoredError = null;
    try {
      await system.uninstallSponsored(pkg);
    } catch (e) {
      sponsoredError = typeof e === 'string' ? e : e?.message || 'Failed to uninstall app';
    } finally {
      uninstallingPkg = null;
    }
  }
</script>

<div class="space-y-6 pb-12">
  <!-- Welcome & Trust System Banner -->
  <div class="bg-gradient-to-r from-slate-900 via-slate-900/90 to-cyan-950/40 border border-slate-800 rounded-3xl p-6 md:p-8 relative overflow-hidden shadow-xl">
    <div class="relative z-10 flex flex-col md:flex-row md:items-center justify-between gap-6">
      <div class="max-w-2xl">
        <div class="inline-flex items-center gap-2 px-3 py-1 bg-cyan-500/10 border border-cyan-500/30 rounded-full text-xs font-semibold text-cyan-400 mb-3">
          <Sparkles class="w-3.5 h-3.5" />
          <span>Vapor Simple Mode &bull; Safe & Reversible</span>
        </div>
        <h1 class="text-2xl md:text-3xl font-bold text-slate-100 tracking-tight">
          Welcome to Honest PC Optimization
        </h1>
        <p class="text-sm text-slate-300 mt-2 leading-relaxed">
          Vapor operates with zero scareware, no fake "critical problem" counts, and no commercial upsells. Every cleanup is non-destructive and safely staged to the Rescue Bin.
        </p>

        <!-- Traffic Light Trust System Legend -->
        <div class="flex flex-wrap items-center gap-4 mt-4 pt-4 border-t border-slate-800/80 text-xs">
          <div class="flex items-center gap-1.5 text-slate-300">
            <span class="w-2.5 h-2.5 rounded-full bg-emerald-400 shadow-sm shadow-emerald-500/50"></span>
            <span class="font-semibold text-emerald-400">Green (Safe):</span>
            <span class="text-slate-400">Temporary caches & log files</span>
          </div>
          <div class="flex items-center gap-1.5 text-slate-300">
            <span class="w-2.5 h-2.5 rounded-full bg-amber-400 shadow-sm shadow-amber-500/50"></span>
            <span class="font-semibold text-amber-400">Yellow (Review):</span>
            <span class="text-slate-400">Optional tweaks & background apps</span>
          </div>
          <div class="flex items-center gap-1.5 text-slate-300">
            <span class="w-2.5 h-2.5 rounded-full bg-rose-400 shadow-sm shadow-rose-500/50"></span>
            <span class="font-semibold text-rose-400">Red (Protected):</span>
            <span class="text-slate-400">Windows core & system drivers (never touched)</span>
          </div>
        </div>
      </div>

      <!-- Quick Switch to Power User Mode button -->
      <div class="shrink-0 flex flex-col items-start md:items-end gap-2">
        <button
          onclick={() => settings.setUiMode('expert')}
          class="flex items-center gap-2 px-4 py-2 text-xs font-semibold text-slate-300 bg-slate-800/80 hover:bg-slate-700 active:scale-95 border border-slate-700 rounded-xl transition-all shadow-sm"
        >
          <Sliders class="w-4 h-4 text-cyan-400" />
          <span>Switch to Power User Mode</span>
        </button>
        <span class="text-[11px] text-slate-500">Access deep tables, Dev Diet & process radar</span>
      </div>
    </div>
  </div>

  <!-- 1-Click Smart Clean Card -->
  <div class="bg-slate-900/80 border border-slate-800 rounded-3xl p-6 md:p-8 shadow-xl relative overflow-hidden">
    <div class="flex flex-col md:flex-row md:items-center justify-between gap-6">
      <div class="space-y-2">
        <div class="flex items-center gap-2.5">
          <div class="p-2 rounded-xl bg-emerald-500/10 border border-emerald-500/30 text-emerald-400">
            <ShieldCheck class="w-5 h-5" />
          </div>
          <div>
            <h2 class="text-lg font-bold text-slate-100 tracking-tight">1-Click Smart Clean</h2>
            <p class="text-xs text-slate-400">Safely archives junk and cache files directly into the reversible Rescue Bin.</p>
          </div>
        </div>

        {#if system.smartCleanSuccess}
          <div class="mt-3 p-3 bg-emerald-950/50 border border-emerald-700/60 rounded-xl text-xs text-emerald-300 flex items-center gap-2">
            <CheckCircle2 class="w-4 h-4 shrink-0 text-emerald-400" />
            <span>
              Successfully staged {system.smartCleanSuccess.fileCount} safe files ({formatBytes(system.smartCleanSuccess.bytesSaved)} freed). You can view or restore them at any time from the Rescue Bin.
            </span>
          </div>
        {/if}

        {#if smartCleanError}
          <div class="mt-3 p-3 bg-rose-950/50 border border-rose-700/60 rounded-xl text-xs text-rose-300 flex items-center gap-2">
            <AlertTriangle class="w-4 h-4 shrink-0 text-rose-400" />
            <span>{smartCleanError}</span>
          </div>
        {/if}

        {#if safeCleanupBytes === 0 && !system.smartCleanSuccess}
          <div class="mt-3 p-3 bg-slate-950/50 border border-slate-800 rounded-xl text-xs text-slate-300 flex items-center gap-2">
            <CheckCircle2 class="w-4 h-4 shrink-0 text-emerald-400" />
            <span>Your PC is currently in great shape. No safe temporary junk files need cleaning right now.</span>
          </div>
        {/if}
      </div>

      <div class="flex flex-col sm:flex-row items-center gap-4">
        <div class="text-center sm:text-right">
          <span class="text-xs uppercase tracking-wider text-slate-400 font-semibold block">Cleanable Safe Junk</span>
          <span class="text-2xl font-black font-mono {safeCleanupBytes > 0 ? 'text-emerald-400' : 'text-slate-400'}">
            {formatBytes(safeCleanupBytes)}
          </span>
        </div>

        <button
          onclick={handleSmartClean}
          disabled={smartCleanRunning || safeCleanupBytes === 0}
          class="w-full sm:w-auto flex items-center justify-center gap-2.5 px-6 py-3.5 {safeCleanupBytes > 0 ? 'bg-emerald-600 hover:bg-emerald-500 text-slate-950 shadow-emerald-900/40' : 'bg-slate-800 text-slate-400'} active:scale-95 disabled:opacity-50 disabled:pointer-events-none font-bold text-sm rounded-2xl shadow-lg transition-all cursor-pointer"
        >
          <Sparkles class="w-4 h-4 {smartCleanRunning ? 'animate-spin' : ''}" />
          <span>{smartCleanRunning ? 'Staging to Rescue Bin...' : safeCleanupBytes > 0 ? '1-Click Smart Clean' : 'System is Clean'}</span>
        </button>
      </div>
    </div>
  </div>

  <!-- Hardware Specs & Capability Rating Grid -->
  {#if capabilities && specs}
    <div class="bg-slate-900/80 border border-slate-800 rounded-3xl p-6 md:p-8 shadow-xl space-y-6">
      <div class="flex items-center justify-between">
        <div>
          <div class="flex items-center gap-2.5">
            <h2 class="text-lg font-bold text-slate-100 tracking-tight">Honest PC Specs & Capability Rating</h2>
            <span class="px-2.5 py-0.5 rounded-full text-xs font-bold font-mono bg-cyan-500/10 text-cyan-400 border border-cyan-500/30">
              Grade {capabilities.overall_grade}
            </span>
          </div>
          <p class="text-xs text-slate-400 mt-1">Calculated directly from real hardware sensors and DXGI diagnostics.</p>
        </div>

        <button
          onclick={() => system.refreshPcSpecs()}
          class="p-2 text-slate-400 hover:text-slate-200 hover:bg-slate-800 rounded-xl transition-all"
          title="Refresh Specs"
        >
          <RefreshCw class="w-4 h-4 {system.isRefreshingSpecs ? 'animate-spin text-cyan-400' : ''}" />
        </button>
      </div>

      <!-- Hardware Summary Bar -->
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-3 bg-slate-950/60 p-4 rounded-2xl border border-slate-800/80">
        <div>
          <span class="text-[11px] text-slate-500 font-semibold uppercase tracking-wider block">Processor</span>
          <span class="text-xs font-bold text-slate-200 line-clamp-1" title={specs.cpu.model}>{specs.cpu.model}</span>
          <span class="text-[10px] text-slate-400">{specs.cpu.physical_cores} Cores &bull; {specs.cpu.logical_threads} Threads</span>
        </div>
        <div>
          <span class="text-[11px] text-slate-500 font-semibold uppercase tracking-wider block">RAM Capacity</span>
          <span class="text-xs font-bold text-slate-200">{specs.memory.total_gb.toFixed(1)} GB Total</span>
          <span class="text-[10px] text-slate-400">{specs.memory.available_gb.toFixed(1)} GB Free ({specs.memory.used_percent.toFixed(0)}% Used)</span>
        </div>
        <div>
          <span class="text-[11px] text-slate-500 font-semibold uppercase tracking-wider block">Primary Storage</span>
          <span class="text-xs font-bold {specs.primary_storage.is_ssd ? 'text-emerald-400' : 'text-amber-400'}">
            {specs.primary_storage.is_ssd ? 'Fast SSD' : 'Mechanical HDD'}
          </span>
          <span class="text-[10px] text-slate-400">{specs.primary_storage.free_gb.toFixed(0)} GB Free on {specs.primary_storage.drive_letter}</span>
        </div>
        <div>
          <span class="text-[11px] text-slate-500 font-semibold uppercase tracking-wider block">Graphics (GPU)</span>
          <span class="text-xs font-bold text-slate-200 line-clamp-1" title={specs.primary_gpu.name}>{specs.primary_gpu.name}</span>
          <span class="text-[10px] text-slate-400">
            {specs.primary_gpu.is_dedicated ? `${(specs.primary_gpu.dedicated_vram_mb / 1024).toFixed(1)} GB Dedicated VRAM` : 'Integrated GPU'}
          </span>
        </div>
      </div>

      <!-- 3 Real Capability Pillars -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <!-- Everyday & Office -->
        <div class="bg-slate-950/40 border border-slate-800/80 rounded-2xl p-4 space-y-2.5">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <Cpu class="w-4 h-4 text-cyan-400" />
              <span class="text-xs font-bold text-slate-200">Everyday & Office</span>
            </div>
            <span class="px-2 py-0.5 rounded text-[11px] font-mono font-bold bg-slate-800 text-cyan-300">
              {capabilities.office_everyday.score.toFixed(1)}/10 &bull; {capabilities.office_everyday.letter_grade}
            </span>
          </div>
          <div class="w-full bg-slate-800 h-1.5 rounded-full overflow-hidden">
            <div class="bg-cyan-400 h-full rounded-full transition-all duration-700" style="width: {capabilities.office_everyday.score * 10}%"></div>
          </div>
          <p class="text-[11px] text-slate-400 leading-relaxed">{capabilities.office_everyday.summary}</p>
        </div>

        <!-- Software Development -->
        <div class="bg-slate-950/40 border border-slate-800/80 rounded-2xl p-4 space-y-2.5">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <Code class="w-4 h-4 text-emerald-400" />
              <span class="text-xs font-bold text-slate-200">Software Development</span>
            </div>
            <span class="px-2 py-0.5 rounded text-[11px] font-mono font-bold bg-slate-800 text-emerald-300">
              {capabilities.software_development.score.toFixed(1)}/10 &bull; {capabilities.software_development.letter_grade}
            </span>
          </div>
          <div class="w-full bg-slate-800 h-1.5 rounded-full overflow-hidden">
            <div class="bg-emerald-400 h-full rounded-full transition-all duration-700" style="width: {capabilities.software_development.score * 10}%"></div>
          </div>
          <p class="text-[11px] text-slate-400 leading-relaxed">{capabilities.software_development.summary}</p>
        </div>

        <!-- 3D Gaming -->
        <div class="bg-slate-950/40 border border-slate-800/80 rounded-2xl p-4 space-y-2.5">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <Gamepad2 class="w-4 h-4 text-purple-400" />
              <span class="text-xs font-bold text-slate-200">3D Gaming</span>
            </div>
            <span class="px-2 py-0.5 rounded text-[11px] font-mono font-bold bg-slate-800 text-purple-300">
              {capabilities.gaming_3d.score.toFixed(1)}/10 &bull; {capabilities.gaming_3d.letter_grade}
            </span>
          </div>
          <div class="w-full bg-slate-800 h-1.5 rounded-full overflow-hidden">
            <div class="bg-purple-400 h-full rounded-full transition-all duration-700" style="width: {capabilities.gaming_3d.score * 10}%"></div>
          </div>
          <p class="text-[11px] text-slate-400 leading-relaxed">{capabilities.gaming_3d.summary}</p>
        </div>
      </div>

      <!-- Bottleneck Detective Card -->
      <div class="bg-cyan-950/20 border border-cyan-800/40 rounded-2xl p-4 flex items-start gap-3">
        <Info class="w-4 h-4 text-cyan-400 shrink-0 mt-0.5" />
        <div>
          <div class="flex items-center gap-2">
            <span class="text-xs font-bold text-cyan-300 uppercase tracking-wider">Bottleneck Detective:</span>
            <span class="text-xs font-bold text-slate-200">{capabilities.bottleneck_headline}</span>
          </div>
          <p class="text-xs text-slate-300 mt-1 leading-relaxed">{capabilities.bottleneck_explanation}</p>
        </div>
      </div>

      <!-- Windows Fast Startup Trap Warning -->
      {#if capabilities.fast_startup_warning}
        <div class="bg-amber-950/30 border border-amber-800/50 rounded-2xl p-4 flex items-start gap-3">
          <Clock class="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
          <div class="space-y-1">
            <div class="flex items-center gap-2">
              <span class="text-xs font-bold text-amber-300 uppercase tracking-wider">Windows Uptime Alert:</span>
              <span class="text-xs font-bold text-slate-200">{capabilities.uptime_days.toFixed(1)} days continuous runtime</span>
            </div>
            <p class="text-xs text-slate-300 leading-relaxed">
              {capabilities.fast_startup_message}
            </p>
          </div>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Safe Windows Performance Tweaks -->
  <div class="bg-slate-900/80 border border-slate-800 rounded-3xl p-6 md:p-8 shadow-xl space-y-4">
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-lg font-bold text-slate-100 tracking-tight">Safe Windows Performance & Privacy Tweaks</h2>
        <p class="text-xs text-slate-400 mt-0.5">Toggle non-destructive registry tweaks with instant on/off status.</p>
      </div>
      <button
        onclick={() => system.refreshPcTweaks()}
        class="p-2 text-slate-400 hover:text-slate-200 hover:bg-slate-800 rounded-xl transition-all"
        title="Refresh Tweaks"
      >
        <RefreshCw class="w-4 h-4 {system.isRefreshingTweaks ? 'animate-spin text-cyan-400' : ''}" />
      </button>
    </div>

    {#if tweakError}
      <div class="p-3 bg-rose-950/50 border border-rose-700/60 rounded-xl text-xs text-rose-300 flex items-center gap-2">
        <AlertTriangle class="w-4 h-4 shrink-0 text-rose-400" />
        <span>{tweakError}</span>
      </div>
    {/if}

    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      {#each tweaks as tweak}
        <div class="bg-slate-950/50 border border-slate-800/80 hover:border-slate-700/80 rounded-2xl p-4 flex items-center justify-between gap-4 transition-all">
          <div class="space-y-1">
            <div class="flex items-center gap-2">
              <span class="text-xs font-bold text-slate-200">{tweak.title}</span>
              {#if tweak.is_applied}
                <span class="text-[10px] font-mono px-1.5 py-0.2 bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 rounded">Applied</span>
              {/if}
            </div>
            <p class="text-[11px] text-slate-400 leading-snug">{tweak.description}</p>
            <span class="text-[10px] text-cyan-400/90 font-medium block">&bull; {tweak.impact}</span>
          </div>

          <button
            onclick={() => handleToggleTweak(tweak)}
            disabled={applyingTweakId === tweak.id}
            class="shrink-0 p-1.5 rounded-xl transition-all {tweak.is_applied ? 'text-emerald-400 hover:text-emerald-300' : 'text-slate-500 hover:text-slate-400'}"
            title={tweak.is_applied ? 'Click to revert' : 'Click to apply'}
          >
            {#if tweak.is_applied}
              <ToggleRight class="w-8 h-8" />
            {:else}
              <ToggleLeft class="w-8 h-8" />
            {/if}
          </button>
        </div>
      {/each}
    </div>
  </div>

  <!-- Sponsored Promotional Bloatware Remover -->
  {#if sponsored.length > 0}
    <div class="bg-slate-900/80 border border-slate-800 rounded-3xl p-6 md:p-8 shadow-xl space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <div class="flex items-center gap-2">
            <Flame class="w-4 h-4 text-amber-400" />
            <h2 class="text-lg font-bold text-slate-100 tracking-tight">Sponsored Promotional Apps ({sponsored.length})</h2>
          </div>
          <p class="text-xs text-slate-400 mt-0.5">Pre-installed partner applications that consume background space. Safe to remove.</p>
        </div>
        <button
          onclick={() => system.refreshSponsoredApps()}
          class="p-2 text-slate-400 hover:text-slate-200 hover:bg-slate-800 rounded-xl transition-all"
          title="Refresh Bloatware List"
        >
          <RefreshCw class="w-4 h-4 {system.isRefreshingSponsored ? 'animate-spin text-cyan-400' : ''}" />
        </button>
      </div>

      {#if sponsoredError}
        <div class="p-3 bg-rose-950/50 border border-rose-700/60 rounded-xl text-xs text-rose-300 flex items-center gap-2">
          <AlertTriangle class="w-4 h-4 shrink-0 text-rose-400" />
          <span>{sponsoredError}</span>
        </div>
      {/if}

      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
        {#each sponsored as app}
          <div class="bg-slate-950/50 border border-slate-800/80 rounded-2xl p-4 flex items-center justify-between gap-3">
            <div class="space-y-0.5">
              <span class="text-xs font-bold text-slate-200 block">{app.display_name}</span>
              <span class="text-[11px] text-slate-400 block">{app.description}</span>
              <span class="text-[10px] text-slate-500 font-mono block truncate max-w-[200px]">{app.publisher}</span>
            </div>

            <button
              onclick={() => handleUninstallSponsored(app.package_full_name)}
              disabled={uninstallingPkg === app.package_full_name}
              class="shrink-0 flex items-center gap-1.5 px-3 py-1.5 bg-rose-500/10 hover:bg-rose-500/20 active:scale-95 text-rose-400 border border-rose-500/30 rounded-xl text-xs font-semibold transition-all"
              title="Uninstall App"
            >
              <Trash2 class="w-3.5 h-3.5 {uninstallingPkg === app.package_full_name ? 'animate-spin' : ''}" />
              <span>{uninstallingPkg === app.package_full_name ? 'Removing...' : 'Uninstall'}</span>
            </button>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>
