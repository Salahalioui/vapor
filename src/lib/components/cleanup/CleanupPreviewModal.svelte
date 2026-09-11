<script>
  import { system } from '../../stores/system.svelte.js';
  import { stageCleanup, stageCleanupRules } from '../../api.js';
  import { formatBytes } from '../../utils.js';
  import Modal from '../common/Modal.svelte';
  import Badge from '../common/Badge.svelte';
  import { ShieldCheck, Trash2, ArrowLeft, CheckCircle2, RefreshCw, Lock, Info } from 'lucide-svelte';

  let isStaging = $state(false);
  let stageSuccess = $state(null);
  let errorMsg = $state('');
  let excludedMap = $state({});

  const preview = $derived(system.previewCleanup);
  const isOpen = $derived(preview !== null);

  const allFiles = $derived(
    preview?.results?.flatMap((r) => r.items) ?? []
  );

  const isSampleCapped = $derived(allFiles.length > 500);
  const displayedFiles = $derived(allFiles.slice(0, 500));

  const activeSelectedFiles = $derived(
    allFiles.filter((f) => !excludedMap[f.path])
  );
  const activeSelectedCount = $derived(activeSelectedFiles.length);

  const reclaimableBytes = $derived(
    activeSelectedFiles.reduce((acc, f) => acc + f.size_bytes, 0)
  );

  const allSelected = $derived(allFiles.length > 0 && activeSelectedCount === allFiles.length);
  const someSelected = $derived(activeSelectedCount > 0 && activeSelectedCount < allFiles.length);

  const progress = $derived(system.operationProgress);
  const progressPercent = $derived(
    progress?.percent ? Math.min(100, Math.max(0, Math.round(progress.percent))) : 0
  );

  function toggleSelectAll() {
    if (allSelected) {
      const next = {};
      for (const f of allFiles) {
        next[f.path] = true;
      }
      excludedMap = next;
    } else {
      excludedMap = {};
    }
  }

  function toggleItem(path) {
    const next = { ...excludedMap };
    if (next[path]) {
      delete next[path];
    } else {
      next[path] = true;
    }
    excludedMap = next;
  }

  async function handleExecuteCleanup() {
    if (activeSelectedCount === 0) return;
    isStaging = true;
    errorMsg = '';
    system.clearOperationProgress();
    try {
      let stageResult;
      const excludedList = Object.keys(excludedMap);
      if (preview?.rule_ids && preview.rule_ids.length > 0) {
        stageResult = await stageCleanupRules(
          preview.rule_ids,
          excludedList.length > 0 ? excludedList : null
        );
      } else {
        const paths = activeSelectedFiles.map((f) => f.path);
        stageResult = await stageCleanup(paths);
      }
      stageSuccess = stageResult;
      system.refreshHealth();
    } catch (e) {
      errorMsg = `Cleanup failed: ${e}`;
    } finally {
      isStaging = false;
      system.clearOperationProgress();
    }
  }

  function handleClose() {
    stageSuccess = null;
    errorMsg = '';
    excludedMap = {};
    system.clearOperationProgress();
    system.setPreviewCleanup(null);
  }

  $effect(() => {
    if (preview) {
      excludedMap = {};
      stageSuccess = null;
      errorMsg = '';
    }
  });
</script>

<Modal
  {isOpen}
  title={stageSuccess ? 'Cleanup Staged in Rescue Bin' : 'Interactive Cleanup Preview'}
  onClose={handleClose}
  maxWidth="max-w-3xl"
>
  {#if stageSuccess}
    <div class="text-center py-6 flex flex-col items-center">
      <div class="p-3 bg-emerald-950 text-emerald-400 rounded-full mb-3 border border-emerald-800">
        <CheckCircle2 class="w-10 h-10" />
      </div>
      <h3 class="text-lg font-bold text-slate-100 mb-1">Files Safely Staged!</h3>
      <p class="text-xs text-slate-400 max-w-md mb-3">
        {stageSuccess.file_count} files ({formatBytes(stageSuccess.total_bytes)}) were moved into your reversible Rescue Bin.
        You can restore them at any time from the Rescue Bin drawer.
      </p>

      {#if stageSuccess.skipped_locked > 0}
        <div class="inline-flex items-center gap-1.5 px-3 py-1.5 bg-amber-950/60 border border-amber-800/70 rounded-lg text-xs text-amber-300 mb-4">
          <Lock class="w-3.5 h-3.5 shrink-0" />
          <span>{stageSuccess.skipped_locked} in-use/locked file{stageSuccess.skipped_locked > 1 ? 's were' : ' was'} safely skipped</span>
        </div>
      {/if}

      <div class="bg-slate-950/60 border border-slate-800 rounded-lg p-3 font-mono text-xs text-slate-400 mb-6">
        Stage ID: <span class="text-cyan-400">{stageSuccess.stage_id}</span>
      </div>

      <div class="flex items-center gap-3">
        <button
          class="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-semibold rounded-lg transition-colors"
          onclick={handleClose}
        >
          Done
        </button>
        <button
          class="px-4 py-2 bg-cyan-600 hover:bg-cyan-500 text-white text-xs font-semibold rounded-lg transition-colors flex items-center gap-1.5"
          onclick={() => {
            handleClose();
            system.openRescueBin();
          }}
        >
          <ShieldCheck class="w-4 h-4" />
          <span>View in Rescue Bin</span>
        </button>
      </div>
    </div>
  {:else}
    <!-- Summary header -->
    <div class="bg-slate-950/50 border border-slate-800 rounded-xl p-4 mb-4 flex items-center justify-between">
      <div>
        <span class="text-xs text-slate-400">Selected Items to Clean</span>
        <div class="text-xl font-bold font-mono text-slate-100">
          {activeSelectedCount} <span class="text-xs font-normal text-slate-400">/ {allFiles.length} files</span>
        </div>
      </div>
      <div class="text-right">
        <span class="text-xs text-slate-400">Reclaimable Space</span>
        <div class="text-xl font-bold font-mono text-emerald-400">
          {formatBytes(reclaimableBytes)}
        </div>
      </div>
    </div>

    <!-- Live Staging Progress Drawer/Bar -->
    {#if isStaging}
      <div class="mb-4 p-4 bg-slate-950/90 border border-cyan-800/60 rounded-xl space-y-3">
        <div class="flex items-center justify-between text-xs">
          <span class="text-cyan-400 font-semibold flex items-center gap-2">
            <RefreshCw class="w-3.5 h-3.5 animate-spin" />
            <span>Staging files into Rescue Bin...</span>
          </span>
          <span class="font-mono text-cyan-300 font-bold">{progressPercent}%</span>
        </div>

        <!-- Progress Bar -->
        <div class="w-full bg-slate-800 rounded-full h-2.5 overflow-hidden">
          <div
            class="bg-gradient-to-r from-cyan-500 to-emerald-400 h-2.5 rounded-full transition-all duration-150"
            style="width: {progressPercent}%"
          ></div>
        </div>

        <div class="flex items-center justify-between text-[11px] text-slate-400">
          <span class="truncate max-w-[280px] font-mono text-slate-300" title={progress?.current_item}>
            {progress?.current_item ? `Current: ${progress.current_item}` : 'Preparing staging...'}
          </span>
          <span class="font-mono">
            {progress?.processed_count ?? 0} / {progress?.total_count ?? activeSelectedCount} files
          </span>
        </div>

        {#if progress?.skipped_locked > 0}
          <div class="flex items-center gap-1.5 text-xs text-amber-400 pt-1.5 border-t border-slate-800/80">
            <Lock class="w-3.5 h-3.5 shrink-0" />
            <span>Skipped {progress.skipped_locked} in-use/locked file{progress.skipped_locked > 1 ? 's' : ''}</span>
          </div>
        {/if}
      </div>
    {/if}

    {#if errorMsg}
      <div class="mb-4 p-3 bg-rose-950/80 border border-rose-800 text-rose-300 text-xs rounded-lg">
        {errorMsg}
      </div>
    {/if}

    <!-- Safety Notice -->
    <div class="mb-3 flex items-start gap-2.5 p-3 bg-cyan-950/30 border border-cyan-800/40 rounded-lg text-xs text-cyan-300">
      <ShieldCheck class="w-4 h-4 shrink-0 mt-0.5 text-cyan-400" />
      <div>
        <strong class="font-semibold">Reversible Rescue Bin Protection:</strong>
        Files are moved to an isolated local staging folder before permanent deletion. You can exclude individual files below or restore anytime with 1 click.
      </div>
    </div>

    <!-- >500 Sample Warning Banner -->
    {#if isSampleCapped}
      <div class="mb-3 px-3.5 py-2 bg-slate-900 border border-slate-700 text-xs text-slate-300 rounded-lg flex items-center gap-2">
        <Info class="w-4 h-4 text-cyan-400 shrink-0" />
        <span>Displaying first 500 sample files for review. All matching files will be safely staged.</span>
      </div>
    {/if}

    <!-- Itemized File List -->
    <div class="max-h-64 overflow-y-auto border border-slate-800 rounded-lg bg-slate-950/30 mb-6">
      <table class="w-full text-left text-xs text-slate-300">
        <thead class="sticky top-0 bg-slate-900 border-b border-slate-800 font-semibold text-slate-400 uppercase tracking-wider text-[11px] z-10">
          <tr>
            <th class="py-2 px-3 w-8 text-center">
              <input
                type="checkbox"
                checked={allSelected}
                onchange={toggleSelectAll}
                class="rounded border-slate-700 bg-slate-800 text-emerald-500 focus:ring-0 cursor-pointer"
                title="Select / Deselect all"
              />
            </th>
            <th class="py-2 px-3">File Name</th>
            <th class="py-2 px-3">Rule</th>
            <th class="py-2 px-3 text-right">Size</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60 font-mono text-[11px]">
          {#each displayedFiles as item}
            {@const isExcluded = !!excludedMap[item.path]}
            <tr class="hover:bg-slate-800/30 transition-colors {isExcluded ? 'opacity-40 line-through bg-slate-950/50' : ''}">
              <td class="py-1.5 px-3 w-8 text-center">
                <input
                  type="checkbox"
                  checked={!isExcluded}
                  onchange={() => toggleItem(item.path)}
                  class="rounded border-slate-700 bg-slate-800 text-emerald-500 focus:ring-0 cursor-pointer"
                />
              </td>
              <td class="py-1.5 px-3 max-w-[280px] truncate text-slate-200" title={item.path}>
                {item.name}
              </td>
              <td class="py-1.5 px-3 text-slate-400">
                {item.rule_id}
              </td>
              <td class="py-1.5 px-3 text-right text-emerald-400 whitespace-nowrap">
                {formatBytes(item.size_bytes)}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <!-- Action Buttons -->
    <div class="flex items-center justify-between pt-3 border-t border-slate-800">
      <button
        class="px-4 py-2 text-xs font-medium text-slate-400 hover:text-slate-200 transition-colors"
        onclick={handleClose}
        disabled={isStaging}
      >
        Cancel
      </button>

      <button
        class="px-5 py-2.5 bg-emerald-600 hover:bg-emerald-500 active:scale-95 text-white text-xs font-semibold rounded-lg shadow-lg shadow-emerald-950/40 transition-all flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
        onclick={handleExecuteCleanup}
        disabled={isStaging || activeSelectedCount === 0}
      >
        <Trash2 class="w-4 h-4" />
        <span>{isStaging ? 'Staging into Rescue Bin...' : `Clean & Stage (${formatBytes(reclaimableBytes)})`}</span>
      </button>
    </div>
  {/if}
</Modal>
