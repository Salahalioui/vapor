<script>
  import { onMount } from 'svelte';
  import {
    fetchInstalledSoftware,
    showInFolder,
    launchUninstaller,
    openExternalUrl
  } from '../../api.js';
  import { formatBytes, formatDate } from '../../utils.js';
  import Badge from '../common/Badge.svelte';
  import ConfirmModal from '../common/ConfirmModal.svelte';
  import {
    Package,
    Search,
    Skull,
    RefreshCw,
    AlertTriangle,
    Filter,
    FolderOpen,
    Trash2,
    Globe,
    CheckCircle2
  } from 'lucide-svelte';

  let apps = $state([]);
  let isLoading = $state(false);
  let searchQuery = $state('');
  let zombieFilter = $state(false);
  let bloatwareFilter = $state(false);
  let feedbackMsg = $state('');

  // Confirmation modal state
  let confirmState = $state({
    isOpen: false,
    title: '',
    message: '',
    confirmText: '',
    isDanger: false,
    action: () => {}
  });

  async function loadApps() {
    isLoading = true;
    try {
      apps = await fetchInstalledSoftware();
    } catch (e) {
      console.error('Failed to load apps:', e);
    } finally {
      isLoading = false;
    }
  }

  function promptUninstall(app) {
    confirmState = {
      isOpen: true,
      title: `Uninstall ${app.name}?`,
      message: `Launch uninstaller for "${app.name}"?\n\nIf the application does not have a registered direct uninstaller, Windows Installed Apps settings will open.`,
      confirmText: 'Uninstall',
      isDanger: true,
      action: async () => {
        try {
          await launchUninstaller(app.uninstall_string);
          feedbackMsg = `Triggered uninstallation process for "${app.name}".`;
          setTimeout(() => (feedbackMsg = ''), 4000);
        } catch (e) {
          feedbackMsg = `Failed to launch uninstaller: ${e}`;
          setTimeout(() => (feedbackMsg = ''), 5000);
        }
      }
    };
  }

  async function handleOpenFolder(app) {
    if (!app.install_location) {
      feedbackMsg = `No install location directory registered for "${app.name}".`;
      setTimeout(() => (feedbackMsg = ''), 3000);
      return;
    }
    try {
      await showInFolder(app.install_location);
    } catch (e) {
      feedbackMsg = `Could not open folder: ${e}`;
      setTimeout(() => (feedbackMsg = ''), 4000);
    }
  }

  async function handleSearchWeb(app) {
    const term = `${app.name} ${app.publisher || ''} Windows`;
    const url = `https://www.google.com/search?q=${encodeURIComponent(term)}`;
    try {
      await openExternalUrl(url);
    } catch (e) {
      feedbackMsg = `Could not launch browser: ${e}`;
      setTimeout(() => (feedbackMsg = ''), 4000);
    }
  }

  onMount(() => {
    loadApps();
  });

  const filteredApps = $derived(
    apps.filter((app) => {
      if (zombieFilter && !app.is_zombie) return false;
      if (bloatwareFilter && !app.is_bloatware) return false;
      if (!searchQuery.trim()) return true;
      const q = searchQuery.toLowerCase();
      return (
        app.name.toLowerCase().includes(q) ||
        (app.publisher && app.publisher.toLowerCase().includes(q))
      );
    })
  );

  const zombieCount = $derived(apps.filter((a) => a.is_zombie).length);
  const bloatwareCount = $derived(apps.filter((a) => a.is_bloatware).length);
</script>

<div class="bg-slate-900/70 border border-slate-800 rounded-xl p-5 shadow-lg">
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-4">
    <div>
      <h2 class="text-lg font-bold text-slate-100 flex items-center gap-2">
        <Package class="w-5 h-5 text-indigo-400" />
        Installed Apps & Zombie Hunter
      </h2>
      <p class="text-xs text-slate-400 mt-0.5">
        Windows Registry audit combined with Windows Prefetch execution traces to identify heavy unused software.
      </p>
    </div>

    <!-- Controls -->
    <div class="flex items-center gap-2 flex-wrap">
      <div class="relative">
        <Search class="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
        <input
          type="text"
          placeholder="Filter installed apps..."
          bind:value={searchQuery}
          class="bg-slate-800 text-xs text-slate-200 pl-8 pr-3 py-1.5 rounded-lg border border-slate-700 focus:outline-none focus:border-cyan-500 w-48 sm:w-56 font-medium placeholder:text-slate-500"
        />
      </div>

      <!-- Zombie Toggle -->
      <button
        class="flex items-center gap-1.5 text-xs px-2.5 py-1.5 rounded-lg border transition-colors font-medium {zombieFilter ? 'bg-amber-950/80 text-amber-300 border-amber-700' : 'bg-slate-800 text-slate-400 border-slate-700 hover:text-slate-200'}"
        onclick={() => (zombieFilter = !zombieFilter)}
      >
        <Skull class="w-3 h-3 text-amber-400" />
        <span>Zombie Apps ({zombieCount})</span>
      </button>

      <!-- Bloatware Toggle -->
      <button
        class="flex items-center gap-1.5 text-xs px-2.5 py-1.5 rounded-lg border transition-colors font-medium {bloatwareFilter ? 'bg-rose-950/80 text-rose-300 border-rose-700' : 'bg-slate-800 text-slate-400 border-slate-700 hover:text-slate-200'}"
        onclick={() => (bloatwareFilter = !bloatwareFilter)}
      >
        <AlertTriangle class="w-3 h-3 text-rose-400" />
        <span>Bloatware ({bloatwareCount})</span>
      </button>

      <button
        class="p-1.5 bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 rounded-lg transition-colors"
        onclick={loadApps}
        disabled={isLoading}
        title="Refresh App List"
      >
        <RefreshCw class="w-4 h-4 {isLoading ? 'animate-spin text-indigo-400' : ''}" />
      </button>
    </div>
  </div>

  {#if feedbackMsg}
    <div class="mb-3 px-3 py-2 bg-indigo-950/80 border border-indigo-800 text-indigo-200 text-xs rounded-lg flex items-center gap-2 animate-in fade-in">
      <CheckCircle2 class="w-4 h-4 text-indigo-400 shrink-0" />
      <span>{feedbackMsg}</span>
    </div>
  {/if}

  <!-- Summary Info -->
  <div class="mb-3 px-3 py-2 bg-slate-950/40 border border-slate-800/80 rounded-lg flex items-center justify-between text-xs">
    <span class="text-slate-400">
      Total Apps Installed: <strong class="text-slate-200">{apps.length}</strong>
    </span>
    <span class="text-slate-400">
      Matching Filter: <strong class="text-cyan-400 font-mono">{filteredApps.length}</strong>
    </span>
  </div>

  {#if isLoading}
    <div class="py-12 text-center text-slate-400 flex flex-col items-center gap-2">
      <RefreshCw class="w-6 h-6 animate-spin text-indigo-400" />
      <span class="text-xs">Auditing 64-bit and 32-bit Windows uninstall registries...</span>
    </div>
  {:else if filteredApps.length === 0}
    <div class="py-8 text-center text-slate-500 text-xs">
      No installed software matching your current search/filter criteria.
    </div>
  {:else}
    <div class="overflow-x-auto max-h-[380px] overflow-y-auto border border-slate-800 rounded-lg bg-slate-950/30">
      <table class="w-full text-left text-xs text-slate-300 border-collapse">
        <thead class="sticky top-0 bg-slate-950 text-slate-400 uppercase tracking-wider font-semibold border-b border-slate-800 z-10 text-[11px]">
          <tr>
            <th class="py-2.5 px-3">Application Name</th>
            <th class="py-2.5 px-3">Publisher</th>
            <th class="py-2.5 px-3">Size</th>
            <th class="py-2.5 px-3">Estimated Last Used</th>
            <th class="py-2.5 px-3">Status Tag</th>
            <th class="py-2.5 px-3 text-right">Actions</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60 font-sans">
          {#each filteredApps as app}
            <tr class="hover:bg-slate-800/40 transition-colors">
              <td class="py-2.5 px-3 font-semibold text-slate-100 max-w-[220px] truncate" title={app.name}>
                <div class="flex items-center gap-1.5">
                  <span class="truncate">{app.name}</span>
                  {#if app.version}
                    <span class="text-[10px] text-slate-500 font-mono">{app.version}</span>
                  {/if}
                </div>
              </td>
              <td class="py-2.5 px-3 text-slate-400 max-w-[160px] truncate" title={app.publisher || 'Unknown'}>
                {app.publisher || 'Unknown'}
              </td>
              <td class="py-2.5 px-3 font-mono font-semibold text-emerald-400 whitespace-nowrap">
                {app.size_bytes > 0 ? formatBytes(app.size_bytes) : '—'}
              </td>
              <td class="py-2.5 px-3 text-slate-400 whitespace-nowrap font-mono text-[11px]">
                {#if app.days_since_last_used !== null}
                  {#if app.days_since_last_used > 60}
                    <span class="text-amber-400 font-bold">{app.days_since_last_used}d ago</span>
                  {:else}
                    <span class="text-slate-300">{app.days_since_last_used}d ago</span>
                  {/if}
                {:else}
                  <span class="text-slate-500">Unused / Stale</span>
                {/if}
              </td>
              <td class="py-2.5 px-3 whitespace-nowrap space-x-1">
                {#if app.is_zombie}
                  <Badge variant="warning">Zombie App</Badge>
                {/if}
                {#if app.is_bloatware}
                  <Badge variant="danger">Bloatware Candidate</Badge>
                {/if}
                {#if !app.is_zombie && !app.is_bloatware}
                  <Badge variant="default">Normal</Badge>
                {/if}
              </td>
              <td class="py-2.5 px-3 text-right whitespace-nowrap space-x-1">
                <!-- Open Folder Button -->
                <button
                  class="p-1 text-slate-400 hover:text-cyan-300 hover:bg-slate-800 rounded transition-colors disabled:opacity-30 disabled:cursor-not-allowed inline-flex items-center justify-center"
                  onclick={() => handleOpenFolder(app)}
                  disabled={!app.install_location}
                  title={app.install_location ? `Open folder: ${app.install_location}` : 'No install location path registered'}
                  aria-label="Open Folder"
                >
                  <FolderOpen class="w-3.5 h-3.5" />
                </button>

                <!-- Search Web Button -->
                <button
                  class="p-1 text-slate-400 hover:text-indigo-300 hover:bg-slate-800 rounded transition-colors inline-flex items-center justify-center"
                  onclick={() => handleSearchWeb(app)}
                  title="Search Google for information about this application"
                  aria-label="Search Web"
                >
                  <Globe class="w-3.5 h-3.5" />
                </button>

                <!-- Uninstall Button -->
                <button
                  class="px-2 py-0.5 text-[11px] font-medium bg-rose-950/60 hover:bg-rose-900/80 text-rose-300 border border-rose-800/60 rounded transition-colors inline-flex items-center gap-1"
                  onclick={() => promptUninstall(app)}
                  title="Launch uninstaller"
                >
                  <Trash2 class="w-3 h-3" />
                  <span>Uninstall</span>
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
