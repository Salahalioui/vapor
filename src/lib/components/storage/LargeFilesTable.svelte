<script>
  import { system } from '../../stores/system.svelte.js';
  import { settings } from '../../stores/settings.svelte.js';
  import { fetchLargeFiles, recycleFile, showInFolder } from '../../api.js';
  import { formatBytes, formatDate } from '../../utils.js';
  import Badge from '../common/Badge.svelte';
  import ConfirmModal from '../common/ConfirmModal.svelte';
  import { File, Trash2, RefreshCw, Filter, AlertTriangle, FolderOpen, CheckCircle2 } from 'lucide-svelte';

  const files = $derived(system.largeFiles ?? []);
  const isLoading = $derived(system.isScanningLargeFiles);
  const hasScanned = $derived(system.largeFiles !== null);
  let thresholdMb = $state(settings.largeFileSizeThresholdMb);
  let onlyDownloads = $state(false);
  let deleteSuccessMsg = $state('');
  let errorMsg = $state('');

  $effect(() => {
    thresholdMb = settings.largeFileSizeThresholdMb;
  });

  let confirmState = $state({
    isOpen: false,
    title: '',
    message: '',
    confirmText: '',
    isDanger: false,
    action: () => {}
  });

  async function loadFiles() {
    if (system.isScanningLargeFiles) return;
    deleteSuccessMsg = '';
    errorMsg = '';
    system.setIsScanningLargeFiles(true);
    try {
      if (system.isScanningDevDiet) {
        await new Promise((r) => setTimeout(r, 100));
      }
      const roots = system.selectedDriveRoot ? [system.selectedDriveRoot] : null;
      const results = await fetchLargeFiles(roots, thresholdMb, 60);
      system.setLargeFiles(results);
    } catch (e) {
      console.error('Failed to scan large files:', e);
      errorMsg = `Failed to scan large files: ${e}`;
    } finally {
      system.setIsScanningLargeFiles(false);
    }
  }

  function handleThresholdChange() {
    settings.setLargeFileSizeThreshold(thresholdMb);
    loadFiles();
  }

  function promptDelete(file) {
    confirmState = {
      isOpen: true,
      title: `Recycle "${file.name}"?`,
      message: `Move this file to the Windows Recycle Bin?\n\nFile: ${file.name}\nSize: ${formatBytes(file.size_bytes)}\nPath: ${file.path}\n\nYou can safely recover this file from the Windows Recycle Bin if needed.`,
      confirmText: 'Move to Trash',
      isDanger: true,
      action: async () => {
        try {
          await recycleFile(file.path);
          const remaining = files.filter((f) => f.path !== file.path);
          system.setLargeFiles(remaining);
          system.refreshHealth();
          system.refreshHardware();
          deleteSuccessMsg = `Moved "${file.name}" to Windows Recycle Bin.`;
          setTimeout(() => (deleteSuccessMsg = ''), 4000);
        } catch (e) {
          errorMsg = `Failed to move file to Recycle Bin: ${e}`;
          setTimeout(() => (errorMsg = ''), 5000);
        }
      }
    };
  }

  async function handleRevealInExplorer(file) {
    try {
      await showInFolder(file.path);
    } catch (e) {
      errorMsg = `Could not open file in Explorer: ${e}`;
      setTimeout(() => (errorMsg = ''), 5000);
    }
  }

  const filteredFiles = $derived(
    onlyDownloads ? files.filter((f) => f.is_in_downloads) : files
  );

  const totalFilteredBytes = $derived(
    filteredFiles.reduce((acc, f) => acc + f.size_bytes, 0)
  );
</script>

<div class="bg-slate-900/70 border border-slate-800 rounded-xl p-5 shadow-lg">
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-4">
    <div>
      <h3 class="text-base font-semibold text-slate-100 flex items-center gap-2">
        <File class="w-4 h-4 text-cyan-400" />
        Large Files & Heavy Downloads
      </h3>
      <p class="text-xs text-slate-400 mt-0.5">
        Target oversized ISOs, video archives, and stale downloads. Items can be moved safely to the Recycle Bin.
      </p>
    </div>

    <!-- Controls -->
    <div class="flex items-center gap-2 flex-wrap">
      <!-- Threshold selector -->
      <select
        bind:value={thresholdMb}
        onchange={handleThresholdChange}
        class="bg-slate-800 text-xs text-slate-200 border border-slate-700 rounded-lg px-2.5 py-1.5 focus:outline-none focus:border-cyan-500 font-medium"
      >
        <option value={50}>Min 50 MB</option>
        <option value={100}>Min 100 MB</option>
        <option value={250}>Min 250 MB</option>
        <option value={500}>Min 500 MB</option>
        <option value={1024}>Min 1 GB</option>
      </select>

      <!-- Downloads only toggle -->
      <button
        class="flex items-center gap-1.5 text-xs px-2.5 py-1.5 rounded-lg border transition-colors font-medium {onlyDownloads ? 'bg-cyan-950/80 text-cyan-300 border-cyan-700' : 'bg-slate-800 text-slate-400 border-slate-700 hover:text-slate-200'}"
        onclick={() => (onlyDownloads = !onlyDownloads)}
      >
        <Filter class="w-3 h-3" />
        <span>Old Downloads Only</span>
      </button>

      <!-- Refresh -->
      <button
        class="p-1.5 bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 rounded-lg transition-colors"
        onclick={loadFiles}
        disabled={isLoading}
        title="Refresh Scan"
      >
        <RefreshCw class="w-4 h-4 {isLoading ? 'animate-spin text-cyan-400' : ''}" />
      </button>
    </div>
  </div>

  {#if deleteSuccessMsg}
    <div class="mb-4 px-3 py-2 bg-emerald-950/70 border border-emerald-800 text-emerald-300 text-xs rounded-lg flex items-center gap-2">
      <CheckCircle2 class="w-4 h-4 text-emerald-400 shrink-0" />
      <span>{deleteSuccessMsg}</span>
    </div>
  {/if}

  {#if errorMsg}
    <div class="mb-4 px-3 py-2 bg-rose-950/70 border border-rose-800 text-rose-300 text-xs rounded-lg flex items-center gap-2">
      <AlertTriangle class="w-4 h-4 text-rose-400 shrink-0" />
      <span>{errorMsg}</span>
    </div>
  {/if}

  <!-- Summary banner -->
  {#if hasScanned}
    <div class="mb-3 px-3 py-2 bg-slate-950/40 border border-slate-800/80 rounded-lg flex items-center justify-between text-xs">
      <span class="text-slate-400">
        Found <strong class="text-slate-200">{filteredFiles.length}</strong> large files
      </span>
      <span class="text-slate-400">
        Total: <strong class="text-cyan-400 font-mono">{formatBytes(totalFilteredBytes)}</strong>
      </span>
    </div>
  {/if}

  <!-- Table -->
  {#if isLoading}
    <div class="p-12 text-center text-slate-400 flex flex-col items-center justify-center gap-2">
      <RefreshCw class="w-6 h-6 animate-spin text-cyan-400" />
      <span class="text-xs">Scanning storage drives for oversized files...</span>
    </div>
  {:else if !hasScanned}
    <div class="p-10 text-center flex flex-col items-center justify-center gap-3 border border-slate-800/60 rounded-lg bg-slate-950/20">
      <div class="p-3 bg-cyan-950/50 text-cyan-400 rounded-full border border-cyan-800/40">
        <File class="w-6 h-6" />
      </div>
      <div>
        <p class="text-sm font-semibold text-slate-200">Scan Storage for Large Files</p>
        <p class="text-xs text-slate-400 max-w-sm mt-1">Discover files exceeding {thresholdMb} MB across Downloads, Documents, Videos, and Desktop without slowing down your system.</p>
      </div>
      <button
        class="mt-1 px-4 py-2 bg-cyan-600 hover:bg-cyan-500 active:scale-95 text-white text-xs font-semibold rounded-lg transition-all shadow-md shadow-cyan-950/40 flex items-center gap-1.5"
        onclick={loadFiles}
      >
        <RefreshCw class="w-3.5 h-3.5" />
        <span>Scan Now</span>
      </button>
    </div>
  {:else if filteredFiles.length === 0}
    <div class="p-8 text-center text-slate-500 text-xs">
      No files exceeding {thresholdMb} MB found in monitored user locations.
    </div>
  {:else}
    <div class="overflow-x-auto max-h-[380px] overflow-y-auto border border-slate-800/80 rounded-lg">
      <table class="w-full text-left text-xs text-slate-300 border-collapse">
        <thead class="sticky top-0 bg-slate-950 text-slate-400 uppercase tracking-wider font-semibold border-b border-slate-800 z-10">
          <tr>
            <th class="py-2.5 px-3">File Name</th>
            <th class="py-2.5 px-3">Size</th>
            <th class="py-2.5 px-3">Modified</th>
            <th class="py-2.5 px-3">Location</th>
            <th class="py-2.5 px-3 text-right">Actions</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60">
          {#each filteredFiles as file}
            <tr class="hover:bg-slate-800/40 transition-colors">
              <td class="py-2 px-3 font-medium text-slate-100 max-w-[220px] truncate" title={file.name}>
                <div class="flex items-center gap-1.5">
                  <span class="font-mono text-cyan-400 uppercase text-[10px] px-1 py-0.5 bg-slate-800 rounded">
                    {file.extension || 'BIN'}
                  </span>
                  <span class="truncate">{file.name}</span>
                </div>
              </td>
              <td class="py-2 px-3 font-mono font-semibold text-emerald-400 whitespace-nowrap">
                {formatBytes(file.size_bytes)}
              </td>
              <td class="py-2 px-3 text-slate-400 whitespace-nowrap">
                {formatDate(file.modified_epoch_secs)}
                {#if file.age_days > 90}
                  <Badge variant="warning" class="ml-1 text-[10px]">{file.age_days}d old</Badge>
                {/if}
              </td>
              <td class="py-2 px-3 text-slate-500 font-mono text-[11px] max-w-[260px] truncate" title={file.path}>
                {file.path}
              </td>
              <td class="py-2 px-3 text-right whitespace-nowrap space-x-1">
                <!-- Reveal in Explorer Button -->
                <button
                  class="p-1.5 text-slate-400 hover:text-cyan-300 hover:bg-slate-800 rounded transition-colors inline-flex items-center justify-center"
                  onclick={() => handleRevealInExplorer(file)}
                  title="Reveal in Windows File Explorer"
                  aria-label="Reveal in File Explorer"
                >
                  <FolderOpen class="w-3.5 h-3.5" />
                </button>

                <!-- Move to Trash Button -->
                <button
                  class="p-1.5 text-rose-400 hover:text-rose-300 hover:bg-rose-950/50 rounded transition-colors inline-flex items-center justify-center"
                  onclick={() => promptDelete(file)}
                  title="Move to Windows Recycle Bin"
                  aria-label="Move to Recycle Bin"
                >
                  <Trash2 class="w-3.5 h-3.5" />
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
