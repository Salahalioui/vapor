<script>
  import { system } from '../../stores/system.svelte.js';
  import {
    ShieldCheck,
    Sliders,
    Flame,
    ToggleLeft,
    ToggleRight,
    Trash2,
    RefreshCw,
    Info,
    AlertTriangle,
    CheckCircle2,
    Sparkles,
    Shield
  } from 'lucide-svelte';

  let restoreDesc = $state('Vapor Pre-Optimization Checkpoint');
  let selectedCategory = $state('all');
  let applyingTweakId = $state(null);
  let tweakError = $state(null);
  let uninstallingPkg = $state(null);
  let uninstallError = $state(null);

  const tweaks = $derived(system.pcTweaks || []);
  const sponsored = $derived(system.sponsoredApps || []);

  const categories = [
    { id: 'all', label: 'All Tweaks' },
    { id: 'privacy', label: 'Privacy & Search' },
    { id: 'taskbar', label: 'Taskbar & Shell' },
    { id: 'gaming', label: 'Gaming & DVR' },
    { id: 'network', label: 'Network & P2P' },
  ];

  const filteredTweaks = $derived(
    selectedCategory === 'all'
      ? tweaks
      : tweaks.filter((t) => t.category === selectedCategory)
  );

  async function handleToggle(tweak) {
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

  async function handleCreateRestorePoint() {
    try {
      await system.createRestorePoint(restoreDesc);
    } catch (e) {
      console.error(e);
    }
  }

  async function handleUninstall(pkg) {
    uninstallingPkg = pkg;
    uninstallError = null;
    try {
      await system.uninstallSponsored(pkg);
    } catch (e) {
      uninstallError = typeof e === 'string' ? e : e?.message || 'Failed to uninstall app';
    } finally {
      uninstallingPkg = null;
    }
  }
</script>

<div class="space-y-6 pb-12">
  <!-- Header Banner -->
  <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 bg-slate-900/90 border border-slate-800/90 rounded-2xl p-6 shadow-xl">
    <div>
      <div class="flex items-center gap-2.5">
        <div class="p-2 rounded-xl bg-cyan-500/10 border border-cyan-500/30 text-cyan-400">
          <Sliders class="w-5 h-5" />
        </div>
        <div>
          <h1 class="text-xl font-bold text-slate-100 tracking-tight">Safe Windows Debloat & Performance Center</h1>
          <p class="text-xs text-slate-400 mt-0.5">
            Non-destructive registry performance toggles, sponsored Appx uninstaller, and System Restore points.
          </p>
        </div>
      </div>
    </div>

    <div class="flex items-center gap-2">
      <button
        onclick={() => {
          system.refreshPcTweaks();
          system.refreshSponsoredApps();
        }}
        class="flex items-center gap-2 px-3 py-1.5 text-xs font-medium text-slate-300 bg-slate-800 hover:bg-slate-700 active:scale-95 border border-slate-700 rounded-lg transition-all"
      >
        <RefreshCw class="w-3.5 h-3.5 {system.isRefreshingTweaks ? 'animate-spin text-cyan-400' : ''}" />
        <span>Refresh All</span>
      </button>
    </div>
  </div>

  <!-- Optional System Restore Point Creator Card -->
  <div class="bg-slate-900/80 border border-slate-800 rounded-2xl p-6 shadow-xl space-y-4">
    <div class="flex items-center gap-2.5">
      <div class="p-1.5 rounded-lg bg-emerald-500/10 border border-emerald-500/30 text-emerald-400">
        <Shield class="w-4 h-4" />
      </div>
      <div>
        <h2 class="text-sm font-bold text-slate-100">Optional Windows System Restore Point</h2>
        <p class="text-xs text-slate-400">Creates a Windows restore snapshot before modifying system settings.</p>
      </div>
    </div>

    <div class="flex flex-col sm:flex-row items-center gap-3">
      <input
        type="text"
        bind:value={restoreDesc}
        placeholder="Restore point label (e.g. Vapor Pre-Optimization Checkpoint)"
        class="w-full flex-1 bg-slate-950 border border-slate-800 focus:border-cyan-500/60 rounded-xl px-4 py-2.5 text-xs text-slate-200 placeholder-slate-500 outline-none font-mono"
      />

      <button
        onclick={handleCreateRestorePoint}
        disabled={system.isCreatingRestorePoint}
        class="w-full sm:w-auto shrink-0 flex items-center justify-center gap-2 px-5 py-2.5 bg-slate-800 hover:bg-slate-700 active:scale-95 text-xs font-semibold text-slate-200 border border-slate-700 rounded-xl transition-all cursor-pointer disabled:opacity-50"
      >
        <ShieldCheck class="w-4 h-4 text-emerald-400 {system.isCreatingRestorePoint ? 'animate-spin' : ''}" />
        <span>{system.isCreatingRestorePoint ? 'Creating Snapshot...' : 'Create Restore Point'}</span>
      </button>
    </div>

    {#if system.restorePointMessage}
      <div class="p-3 rounded-xl text-xs flex items-start gap-2.5 {system.restorePointMessage.type === 'success' ? 'bg-emerald-950/50 border border-emerald-700/60 text-emerald-300' : 'bg-slate-950/90 border border-amber-800/60 text-amber-300'}">
        {#if system.restorePointMessage.type === 'success'}
          <CheckCircle2 class="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
        {:else}
          <Info class="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
        {/if}
        <span class="leading-relaxed">{system.restorePointMessage.text}</span>
      </div>
    {/if}
  </div>

  <!-- Safe Windows Registry Tweaks -->
  <div class="bg-slate-900/80 border border-slate-800 rounded-2xl p-6 shadow-xl space-y-4">
    <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
      <div>
        <h2 class="text-base font-bold text-slate-100">Safe Reversible Windows Tweaks</h2>
        <p class="text-xs text-slate-400">All tweaks directly query and modify standard user registry values. Zero external services.</p>
      </div>

      <!-- Category Filter Tabs -->
      <div class="flex items-center gap-1 bg-slate-950/80 p-1 rounded-xl border border-slate-800 overflow-x-auto">
        {#each categories as cat}
          <button
            onclick={() => (selectedCategory = cat.id)}
            class="px-3 py-1 text-xs font-medium rounded-lg transition-all whitespace-nowrap {selectedCategory === cat.id ? 'bg-cyan-950 text-cyan-300 border border-cyan-800/60' : 'text-slate-400 hover:text-slate-200'}"
          >
            {cat.label}
          </button>
        {/each}
      </div>
    </div>

    {#if tweakError}
      <div class="p-3 rounded-xl text-xs bg-rose-950/60 border border-rose-800 text-rose-300 flex items-center gap-2">
        <AlertTriangle class="w-4 h-4 text-rose-400 shrink-0" />
        <span>{tweakError}</span>
      </div>
    {/if}

    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each filteredTweaks as tweak}
        <div class="bg-slate-950/60 border border-slate-800 hover:border-slate-700 rounded-2xl p-5 flex items-center justify-between gap-4 transition-all">
          <div class="space-y-1.5">
            <div class="flex items-center gap-2">
              <span class="text-xs font-bold text-slate-200">{tweak.title}</span>
              <span class="text-[10px] font-mono px-1.5 py-0.2 rounded uppercase {tweak.is_applied ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/30' : 'bg-slate-800 text-slate-400 border border-slate-700'}">
                {tweak.is_applied ? 'Enabled' : 'Default'}
              </span>
            </div>
            <p class="text-xs text-slate-400 leading-snug">{tweak.description}</p>
            <div class="flex items-center gap-2 text-[11px] text-cyan-400 font-medium pt-1">
              <Sparkles class="w-3 h-3 shrink-0" />
              <span>{tweak.impact}</span>
            </div>
          </div>

          <button
            onclick={() => handleToggle(tweak)}
            disabled={applyingTweakId === tweak.id}
            class="shrink-0 p-2 rounded-xl transition-all cursor-pointer {tweak.is_applied ? 'text-emerald-400 hover:text-emerald-300' : 'text-slate-600 hover:text-slate-400'}"
            title={tweak.is_applied ? 'Click to revert tweak' : 'Click to apply tweak'}
          >
            {#if tweak.is_applied}
              <ToggleRight class="w-9 h-9" />
            {:else}
              <ToggleLeft class="w-9 h-9" />
            {/if}
          </button>
        </div>
      {/each}
    </div>
  </div>

  <!-- Sponsored Promotional Bloatware Remover -->
  <div class="bg-slate-900/80 border border-slate-800 rounded-2xl p-6 shadow-xl space-y-4">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2.5">
        <div class="p-1.5 rounded-lg bg-amber-500/10 border border-amber-500/30 text-amber-400">
          <Flame class="w-4 h-4" />
        </div>
        <div>
          <h2 class="text-base font-bold text-slate-100">Sponsored Promotional App Remover</h2>
          <p class="text-xs text-slate-400">Scans for pre-installed commercial partner apps bundled with Windows 10/11.</p>
        </div>
      </div>

      <span class="px-2.5 py-1 rounded-full text-xs font-mono font-bold bg-slate-800 text-slate-300 border border-slate-700">
        {sponsored.length} Found
      </span>
    </div>

    {#if uninstallError}
      <div class="p-3 rounded-xl text-xs bg-rose-950/60 border border-rose-800 text-rose-300 flex items-center gap-2">
        <AlertTriangle class="w-4 h-4 text-rose-400 shrink-0" />
        <span>{uninstallError}</span>
      </div>
    {/if}

    {#if sponsored.length === 0}
      <div class="p-8 text-center bg-slate-950/40 border border-slate-800/80 rounded-2xl">
        <CheckCircle2 class="w-8 h-8 text-emerald-400 mx-auto mb-2" />
        <span class="text-xs font-bold text-slate-200 block">Clean System</span>
        <span class="text-[11px] text-slate-400 mt-1 block">No sponsored promotional apps detected on this installation.</span>
      </div>
    {:else}
      <div class="overflow-x-auto border border-slate-800 rounded-2xl bg-slate-950/40">
        <table class="w-full text-left text-xs">
          <thead>
            <tr class="border-b border-slate-800 bg-slate-900/60 text-slate-400 font-semibold uppercase text-[10px] tracking-wider">
              <th class="py-3 px-4">Application</th>
              <th class="py-3 px-4">Publisher</th>
              <th class="py-3 px-4">Description</th>
              <th class="py-3 px-4 text-right">Action</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-800/60">
            {#each sponsored as app}
              <tr class="hover:bg-slate-900/30 transition-colors">
                <td class="py-3 px-4 font-bold text-slate-200">{app.display_name}</td>
                <td class="py-3 px-4 text-slate-400 font-mono text-[11px] truncate max-w-[200px]">{app.publisher}</td>
                <td class="py-3 px-4 text-slate-300 text-[11px]">{app.description}</td>
                <td class="py-3 px-4 text-right">
                  <button
                    onclick={() => handleUninstall(app.package_full_name)}
                    disabled={uninstallingPkg === app.package_full_name}
                    class="inline-flex items-center gap-1.5 px-3 py-1 bg-rose-500/10 hover:bg-rose-500/20 active:scale-95 text-rose-400 border border-rose-500/30 rounded-lg text-xs font-semibold transition-all cursor-pointer"
                  >
                    <Trash2 class="w-3.5 h-3.5 {uninstallingPkg === app.package_full_name ? 'animate-spin' : ''}" />
                    <span>{uninstallingPkg === app.package_full_name ? 'Removing...' : 'Uninstall'}</span>
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
</div>
