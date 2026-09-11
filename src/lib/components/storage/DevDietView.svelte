<script>
  import { system } from '../../stores/system.svelte.js';
  import { fetchDevDiet, purgeDevTarget, showInFolder } from '../../api.js';
  import { settings } from '../../stores/settings.svelte.js';
  import { formatBytes, formatDate } from '../../utils.js';
  import Badge from '../common/Badge.svelte';
  import ConfirmModal from '../common/ConfirmModal.svelte';
  import { FolderCode, Trash2, RefreshCw, AlertCircle, CheckCircle2, FolderOpen } from 'lucide-svelte';

  const items = $derived(system.devDietItems ?? []);
  const isLoading = $derived(system.isScanningDevDiet);
  const hasScanned = $derived(system.devDietItems !== null);
  let purgeMsg = $state('');
  let purgeError = $state(false);
  let onlyDormant = $state(false);

  let confirmState = $state({
    isOpen: false,
    title: '',
    message: '',
    confirmText: '',
    isDanger: false,
    action: () => {}
  });

  async function scanDiet() {
    if (system.isScanningDevDiet) return;
    purgeMsg = '';
    system.setIsScanningDevDiet(true);
    try {
      if (system.isScanningLargeFiles) {
        await new Promise((r) => setTimeout(r, 100));
      }
      const roots = system.selectedDriveRoot ? [system.selectedDriveRoot] : null;
      const results = await fetchDevDiet(roots, settings.devDietThresholdDays);
      system.setDevDietItems(results);
    } catch (e) {
      console.error('Failed to scan dev diet:', e);
    } finally {
      system.setIsScanningDevDiet(false);
    }
  }

  function promptPurge(item) {
    confirmState = {
      isOpen: true,
      title: `Purge "${item.name}"?`,
      message: `Move build directory "${item.name}" to the Windows Recycle Bin?\n\nPath: ${item.path}\nReclaimable: ${formatBytes(item.size_bytes)}\n\nYou can restore it from the Recycle Bin or re-install/rebuild anytime with your package manager (npm, cargo, pip).`,
      confirmText: 'Move to Recycle Bin',
      isDanger: true,
      action: async () => {
        try {
          const reclaimed = await purgeDevTarget(item.path);
          const remaining = items.filter((i) => i.path !== item.path);
          system.setDevDietItems(remaining);
          system.refreshHealth();
          system.refreshHardware();
          purgeError = false;
          purgeMsg = `Successfully moved "${item.name}" to Recycle Bin and reclaimed ${formatBytes(reclaimed)}!`;
          setTimeout(() => (purgeMsg = ''), 4000);
        } catch (e) {
          purgeError = true;
          purgeMsg = `Failed to purge directory: ${e}`;
          setTimeout(() => (purgeMsg = ''), 5000);
        }
      }
    };
  }

  async function handleRevealInExplorer(item) {
    try {
      await showInFolder(item.path);
    } catch (e) {
      purgeError = true;
      purgeMsg = `Could not open folder in Explorer: ${e}`;
      setTimeout(() => (purgeMsg = ''), 5000);
    }
  }

  const displayedItems = $derived(
    onlyDormant ? items.filter((i) => i.is_dormant) : items
  );

  const totalReclaimableBytes = $derived(
    displayedItems.reduce((acc, i) => acc + i.size_bytes, 0)
  );

  function getKindBadge(kind) {
    switch (kind) {
      case 'node_modules': return { label: 'Node.js', variant: 'success' };
      case 'rust_target': return { label: 'Rust', variant: 'warning' };
      case 'python_venv': return { label: 'Python', variant: 'accent' };
      case 'next_cache': return { label: 'Next.js', variant: 'purple' };
      default: return { label: 'Build', variant: 'default' };
    }
  }
</script>

<div class="bg-slate-900/70 border border-slate-800 rounded-xl p-5 shadow-lg mt-6">
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-4">
    <div>
      <div class="flex items-center gap-2">
        <FolderCode class="w-4 h-4 text-emerald-400" />
        <h3 class="text-base font-semibold text-slate-100">Developer Diet Scanner</h3>
        <Badge variant="accent">High ROI</Badge>
      </div>
      <p class="text-xs text-slate-400 mt-0.5">
        Automatically detect dormant <code class="text-cyan-400">node_modules</code>, <code class="text-amber-400">target</code>, and <code class="text-emerald-400">.venv</code> folders untouched for >{settings.devDietThresholdDays} days.
      </p>
    </div>

    <div class="flex items-center gap-2">
      <button
        class="flex items-center gap-1.5 text-xs px-2.5 py-1.5 rounded-lg border transition-colors font-medium {onlyDormant ? 'bg-cyan-950/80 text-cyan-300 border-cyan-700' : 'bg-slate-800 text-slate-400 border-slate-700 hover:text-slate-200'}"
        onclick={() => (onlyDormant = !onlyDormant)}
      >
        <span>Dormant Only (&gt;{settings.devDietThresholdDays}d)</span>
      </button>

      <button
        class="p-1.5 bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 rounded-lg transition-colors"
        onclick={scanDiet}
        disabled={isLoading}
        title="Refresh Dev Diet Scan"
      >
        <RefreshCw class="w-4 h-4 {isLoading ? 'animate-spin text-cyan-400' : ''}" />
      </button>
    </div>
  </div>

  {#if purgeMsg}
    <div class="mb-4 px-3 py-2 border text-xs rounded-lg flex items-center gap-2 {purgeError ? 'bg-rose-950/70 border-rose-800 text-rose-300' : 'bg-emerald-950/70 border-emerald-800 text-emerald-300'}">
      {#if purgeError}
        <AlertCircle class="w-4 h-4 text-rose-400 shrink-0" />
      {:else}
        <CheckCircle2 class="w-4 h-4 text-emerald-400 shrink-0" />
      {/if}
      <span>{purgeMsg}</span>
    </div>
  {/if}

  {#if hasScanned}
    <!-- Stats Header -->
    <div class="mb-3 px-4 py-2.5 bg-emerald-950/20 border border-emerald-800/40 rounded-lg flex items-center justify-between text-xs">
      <span class="text-slate-300">
        Identified <strong class="text-slate-100">{displayedItems.length}</strong> developer project build folders
      </span>
      <span class="text-slate-300">
        Reclaimable: <strong class="text-emerald-400 font-mono font-bold text-sm">{formatBytes(totalReclaimableBytes)}</strong>
      </span>
    </div>
  {/if}

  {#if isLoading}
    <div class="p-12 text-center text-slate-400 flex flex-col items-center justify-center gap-2">
      <RefreshCw class="w-6 h-6 animate-spin text-emerald-400" />
      <span class="text-xs">Scanning developer project directories and calculating folder sizes...</span>
    </div>
  {:else if !hasScanned}
    <div class="p-10 text-center flex flex-col items-center justify-center gap-3 border border-slate-800/60 rounded-lg bg-slate-950/20">
      <div class="p-3 bg-emerald-950/50 text-emerald-400 rounded-full border border-emerald-800/40">
        <FolderCode class="w-6 h-6" />
      </div>
      <div>
        <p class="text-sm font-semibold text-slate-200">Scan Developer Dependencies & Build Caches</p>
        <p class="text-xs text-slate-400 max-w-sm mt-1">Scan for dormant node_modules, target, and .venv folders untouched for &gt;{settings.devDietThresholdDays} days.</p>
      </div>
      <button
        class="mt-1 px-4 py-2 bg-emerald-600 hover:bg-emerald-500 active:scale-95 text-white text-xs font-semibold rounded-lg transition-all shadow-md shadow-emerald-950/40 flex items-center gap-1.5"
        onclick={scanDiet}
      >
        <RefreshCw class="w-3.5 h-3.5" />
        <span>Scan Now</span>
      </button>
    </div>
  {:else if displayedItems.length === 0}
    <div class="p-8 text-center text-slate-500 text-xs">
      No developer build/dependency folders detected. Your developer workspace is lean!
    </div>
  {:else}
    <div class="overflow-x-auto max-h-[380px] overflow-y-auto border border-slate-800/80 rounded-lg">
      <table class="w-full text-left text-xs text-slate-300 border-collapse">
        <thead class="sticky top-0 bg-slate-950 text-slate-400 uppercase tracking-wider font-semibold border-b border-slate-800 z-10">
          <tr>
            <th class="py-2.5 px-3">Type</th>
            <th class="py-2.5 px-3">Directory Path</th>
            <th class="py-2.5 px-3">Size</th>
            <th class="py-2.5 px-3">Dormancy</th>
            <th class="py-2.5 px-3 text-right">Action</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60">
          {#each displayedItems as item}
            {@const badge = getKindBadge(item.kind)}
            <tr class="hover:bg-slate-800/40 transition-colors">
              <td class="py-2.5 px-3 whitespace-nowrap">
                <Badge variant={badge.variant}>{badge.label}</Badge>
              </td>
              <td class="py-2.5 px-3 font-mono text-[11px] text-slate-300 max-w-[340px] truncate" title={item.path}>
                {item.path}
              </td>
              <td class="py-2.5 px-3 font-mono font-semibold text-emerald-400 whitespace-nowrap">
                {formatBytes(item.size_bytes)}
              </td>
              <td class="py-2.5 px-3 whitespace-nowrap">
                {#if item.is_dormant}
                  <span class="text-amber-400 font-medium font-mono">{item.days_dormant}d dormant</span>
                {:else}
                  <span class="text-slate-400 font-mono">{item.days_dormant}d active</span>
                {/if}
              </td>
              <td class="py-2.5 px-3 text-right whitespace-nowrap space-x-1">
                <!-- Reveal in Explorer Button -->
                <button
                  class="p-1.5 text-slate-400 hover:text-cyan-300 hover:bg-slate-800 rounded transition-colors inline-flex items-center justify-center"
                  onclick={() => handleRevealInExplorer(item)}
                  title="Reveal in Windows File Explorer"
                  aria-label="Reveal in File Explorer"
                >
                  <FolderOpen class="w-3.5 h-3.5" />
                </button>

                <!-- Purge Button -->
                <button
                  class="px-2.5 py-1 bg-rose-950/60 hover:bg-rose-900/80 text-rose-300 border border-rose-800/60 rounded text-[11px] font-medium transition-colors inline-flex items-center gap-1 align-middle"
                  onclick={() => promptPurge(item)}
                  title="Safely move to Windows Recycle Bin"
                >
                  <Trash2 class="w-3 h-3" />
                  <span>Purge</span>
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<!-- In-app Confirmation Modal -->
<ConfirmModal
  isOpen={confirmState.isOpen}
  title={confirmState.title}
  message={confirmState.message}
  confirmText={confirmState.confirmText}
  isDanger={confirmState.isDanger}
  onConfirm={confirmState.action}
  onClose={() => (confirmState.isOpen = false)}
/>
