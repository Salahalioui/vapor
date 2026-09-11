<script>
  import { onMount } from 'svelte';
  import { fetchStartupSoftware, toggleStartupSoftware } from '../../api.js';
  import { system } from '../../stores/system.svelte.js';
  import Badge from '../common/Badge.svelte';
  import ConfirmModal from '../common/ConfirmModal.svelte';
  import { Rocket, PowerOff, Power, RefreshCw, AlertCircle, CheckCircle2, Shield, ShieldAlert } from 'lucide-svelte';

  let items = $state([]);
  let isLoading = $state(false);
  let statusMsg = $state('');
  let errorMsg = $state('');

  let confirmState = $state({
    isOpen: false,
    title: '',
    message: '',
    confirmText: '',
    isDanger: false,
    action: () => {}
  });

  async function loadStartupItems() {
    isLoading = true;
    errorMsg = '';
    try {
      items = await fetchStartupSoftware();
    } catch (e) {
      console.error('Failed to fetch startup software:', e);
      errorMsg = `Failed to read startup items: ${e}`;
    } finally {
      isLoading = false;
    }
  }

  function promptToggle(item) {
    if (item.requires_admin) {
      errorMsg = `Administrator permissions required to modify system-wide (${item.hive}) startup item "${item.name}". Please relaunch Vapor as Administrator.`;
      setTimeout(() => (errorMsg = ''), 6000);
      return;
    }

    const nextState = !item.enabled;
    const actionWord = nextState ? 'Enable' : 'Disable';

    confirmState = {
      isOpen: true,
      title: `${actionWord} "${item.name}" autostart?`,
      message: nextState
        ? `Re-enable "${item.name}" to start automatically when Windows boots?\n\nThis will restore the registry entry in ${item.hive}.`
        : `Disable "${item.name}" from starting with Windows?\n\nThis will safely archive the command to your local Vapor profile so you can re-enable it anytime.`,
      confirmText: actionWord,
      isDanger: !nextState,
      action: async () => {
        try {
          await toggleStartupSoftware(item.name, item.hive, nextState);
          statusMsg = `${nextState ? 'Enabled' : 'Disabled'} autostart for "${item.name}".`;
          await loadStartupItems();
          system.refreshHealth();
          setTimeout(() => (statusMsg = ''), 4000);
        } catch (e) {
          errorMsg = `Action failed: ${e}`;
          setTimeout(() => (errorMsg = ''), 6000);
        }
      }
    };
  }

  onMount(() => {
    loadStartupItems();
  });

  function getImpactBadgeVariant(impact) {
    switch (impact) {
      case 'High': return 'danger';
      case 'Medium': return 'warning';
      case 'Low': return 'success';
      default: return 'default';
    }
  }
</script>

<div class="bg-slate-900/70 border border-slate-800 rounded-xl p-5 shadow-lg mt-6">
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-4">
    <div>
      <h3 class="text-base font-semibold text-slate-100 flex items-center gap-2">
        <Rocket class="w-4 h-4 text-cyan-400" />
        Startup Application Impact Radar
      </h3>
      <p class="text-xs text-slate-400 mt-0.5">
        Reversible autostart management. Disabled items are safely preserved in local configuration and can be restored with a single toggle.
      </p>
    </div>

    <button
      class="p-1.5 bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 rounded-lg transition-colors"
      onclick={loadStartupItems}
      disabled={isLoading}
      title="Refresh Startup Entries"
    >
      <RefreshCw class="w-4 h-4 {isLoading ? 'animate-spin text-cyan-400' : ''}" />
    </button>
  </div>

  {#if statusMsg}
    <div class="mb-4 px-3 py-2 bg-emerald-950/80 border border-emerald-800 text-emerald-300 text-xs rounded-lg flex items-center gap-2 animate-in fade-in">
      <CheckCircle2 class="w-4 h-4 text-emerald-400 shrink-0" />
      <span>{statusMsg}</span>
    </div>
  {/if}

  {#if errorMsg}
    <div class="mb-4 px-3 py-2 bg-rose-950/80 border border-rose-800 text-rose-300 text-xs rounded-lg flex items-center gap-2 animate-in fade-in">
      <AlertCircle class="w-4 h-4 text-rose-400 shrink-0" />
      <span>{errorMsg}</span>
    </div>
  {/if}

  {#if isLoading}
    <div class="py-12 text-center text-slate-400 flex flex-col items-center gap-2">
      <RefreshCw class="w-6 h-6 animate-spin text-cyan-400" />
      <span class="text-xs">Reading autostart registry branches and disabled archive...</span>
    </div>
  {:else if items.length === 0}
    <div class="py-8 text-center text-slate-500 text-xs">
      No autostart programs detected in user or system run registries. Clean boot profile!
    </div>
  {:else}
    <div class="overflow-x-auto max-h-[340px] overflow-y-auto border border-slate-800 rounded-lg bg-slate-950/30">
      <table class="w-full text-left text-xs text-slate-300 border-collapse">
        <thead class="sticky top-0 bg-slate-950 text-slate-400 uppercase tracking-wider font-semibold border-b border-slate-800 z-10 text-[11px]">
          <tr>
            <th class="py-2.5 px-3">Startup Name</th>
            <th class="py-2.5 px-3">Hive</th>
            <th class="py-2.5 px-3">Estimated Boot Impact</th>
            <th class="py-2.5 px-3">Launch Command</th>
            <th class="py-2.5 px-3 text-right">State / Toggle</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60 font-sans">
          {#each items as item}
            <tr class="hover:bg-slate-800/40 transition-colors {!item.enabled ? 'opacity-60 bg-slate-950/30' : ''}">
              <td class="py-2.5 px-3 font-semibold text-slate-100 whitespace-nowrap">
                <div class="flex items-center gap-2">
                  <span class={item.enabled ? 'text-slate-100' : 'text-slate-400 line-through'}>{item.name}</span>
                  {#if item.requires_admin}
                    <span class="inline-flex items-center gap-1 text-[10px] px-1.5 py-0.5 rounded bg-amber-950/60 text-amber-300 border border-amber-800/60" title="Administrator elevation required to modify this HKLM entry">
                      <ShieldAlert class="w-3 h-3 text-amber-400" />
                      <span>Admin Required</span>
                    </span>
                  {/if}
                </div>
              </td>
              <td class="py-2.5 px-3 font-mono text-slate-400 text-[11px] whitespace-nowrap">
                {item.hive}
              </td>
              <td class="py-2.5 px-3 whitespace-nowrap">
                <Badge variant={item.enabled ? getImpactBadgeVariant(item.impact) : 'default'}>
                  {item.impact} Impact
                </Badge>
              </td>
              <td class="py-2.5 px-3 font-mono text-slate-400 text-[11px] max-w-[280px] truncate" title={item.command}>
                {item.command}
              </td>
              <td class="py-2.5 px-3 text-right whitespace-nowrap">
                <!-- Reversible Enable / Disable Toggle Switch -->
                <button
                  type="button"
                  class="inline-flex items-center gap-2 px-2.5 py-1 rounded-lg border text-xs font-medium transition-all {item.enabled ? 'bg-emerald-950/70 border-emerald-700/80 text-emerald-300 hover:bg-emerald-900/80' : 'bg-slate-800/90 border-slate-700 text-slate-400 hover:text-slate-200 hover:bg-slate-700'}"
                  onclick={() => promptToggle(item)}
                  title={item.enabled ? 'Click to disable and archive this startup item' : 'Click to restore and re-enable this startup item'}
                >
                  <span class="relative flex h-2 w-2">
                    {#if item.enabled}
                      <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                      <span class="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
                    {:else}
                      <span class="relative inline-flex rounded-full h-2 w-2 bg-slate-500"></span>
                    {/if}
                  </span>
                  <span>{item.enabled ? 'Enabled' : 'Disabled'}</span>
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

