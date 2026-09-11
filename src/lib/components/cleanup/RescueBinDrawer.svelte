<script>
  import { onMount } from 'svelte';
  import { system } from '../../stores/system.svelte.js';
  import { settings } from '../../stores/settings.svelte.js';
  import {
    fetchRescueStages,
    restoreRescueStage,
    purgeRescueStage,
    autoPurgeRescueStages,
    getStageManifest
  } from '../../api.js';
  import { formatBytes, formatDate, timeAgo } from '../../utils.js';
  import Badge from '../common/Badge.svelte';
  import ConfirmModal from '../common/ConfirmModal.svelte';
  import {
    ShieldCheck,
    RotateCcw,
    Trash2,
    X,
    RefreshCw,
    CheckCircle2,
    ChevronDown,
    ChevronUp,
    FileText,
    AlertTriangle,
    AlertCircle
  } from 'lucide-svelte';

  let stages = $state([]);
  let isLoading = $state(false);
  let actionMsg = $state('');
  let actionIsError = $state(false);
  let actionErrorsList = $state([]);
  let stageManifests = $state({});

  let confirmState = $state({
    isOpen: false,
    title: '',
    message: '',
    confirmText: '',
    isDanger: false,
    action: () => {}
  });

  const isOpen = $derived(system.activeRescueBinDrawer);

  async function loadStages() {
    isLoading = true;
    try {
      stages = await fetchRescueStages();
    } catch (e) {
      console.error('Failed to load rescue stages:', e);
    } finally {
      isLoading = false;
    }
  }

  async function toggleManifest(stageId) {
    const current = stageManifests[stageId] || { isOpen: false, isLoading: false, items: null, error: null };
    if (!current.isOpen && !current.items && !current.isLoading) {
      stageManifests = {
        ...stageManifests,
        [stageId]: { ...current, isOpen: true, isLoading: true, error: null }
      };
      try {
        const manifest = await getStageManifest(stageId);
        stageManifests = {
          ...stageManifests,
          [stageId]: { isOpen: true, isLoading: false, items: manifest?.items ?? [], error: null }
        };
      } catch (e) {
        stageManifests = {
          ...stageManifests,
          [stageId]: { isOpen: true, isLoading: false, items: null, error: `${e}` }
        };
      }
    } else {
      stageManifests = {
        ...stageManifests,
        [stageId]: { ...current, isOpen: !current.isOpen }
      };
    }
  }

  function handleRestore(stage) {
    confirmState = {
      isOpen: true,
      title: 'Restore Cleanup Run?',
      message: `Restore all ${stage.file_count} files (${formatBytes(stage.total_bytes)}) in stage "${stage.stage_id}" back to their original locations?`,
      confirmText: 'Restore Files',
      isDanger: false,
      action: async () => {
        try {
          const res = await restoreRescueStage(stage.stage_id);
          if (res.failed_count > 0) {
            actionIsError = true;
            actionMsg = `Partially restored: ${res.restored_count} files restored, but ${res.failed_count} file(s) failed.`;
            actionErrorsList = res.errors || [];
          } else {
            actionIsError = false;
            actionMsg = `Successfully restored all ${res.restored_count} files (${formatBytes(res.restored_bytes)})!`;
            actionErrorsList = [];
          }
          await loadStages();
          system.refreshHealth();
          system.refreshHardware();
          setTimeout(() => {
            if (!actionIsError) actionMsg = '';
          }, 5000);
        } catch (e) {
          actionIsError = true;
          actionMsg = `Restore failed: ${e}`;
          actionErrorsList = [];
        }
      }
    };
  }

  function handlePurge(stage) {
    confirmState = {
      isOpen: true,
      title: 'Purge Staged Files?',
      message: `Permanently delete stage "${stage.stage_id}" from disk?\n\nFiles: ${stage.file_count}\nSize: ${formatBytes(stage.total_bytes)}\n\nWARNING: This permanent deletion cannot be undone.`,
      confirmText: 'Permanently Purge',
      isDanger: true,
      action: async () => {
        try {
          await purgeRescueStage(stage.stage_id);
          stages = stages.filter((s) => s.stage_id !== stage.stage_id);
          actionIsError = false;
          actionMsg = `Permanently purged stage "${stage.stage_id}".`;
          actionErrorsList = [];
          system.refreshHealth();
          system.refreshHardware();
          setTimeout(() => (actionMsg = ''), 3000);
        } catch (e) {
          actionIsError = true;
          actionMsg = `Purge failed: ${e}`;
          actionErrorsList = [];
        }
      }
    };
  }

  async function handleAutoPurge() {
    try {
      const count = await autoPurgeRescueStages(settings.retentionDays);
      actionIsError = false;
      actionMsg = `Auto-purged ${count} stages older than ${settings.retentionDays} days.`;
      actionErrorsList = [];
      await loadStages();
      system.refreshHealth();
      system.refreshHardware();
      setTimeout(() => (actionMsg = ''), 3000);
    } catch (e) {
      actionIsError = true;
      actionMsg = `Auto purge error: ${e}`;
      actionErrorsList = [];
    }
  }

  $effect(() => {
    if (isOpen) {
      loadStages();
    }
  });

  const totalStagedBytes = $derived(
    stages.reduce((acc, s) => acc + s.total_bytes, 0)
  );
</script>

{#if isOpen}
  <!-- Backdrop -->
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
  <div
    class="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex justify-end transition-opacity"
    onclick={() => system.closeRescueBin()}
    role="presentation"
  >
    <!-- Drawer Panel -->
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
    <div
      class="w-full max-w-lg bg-slate-900 border-l border-slate-800 h-full flex flex-col shadow-2xl animate-in slide-in-from-right duration-300"
      onclick={(e) => e.stopPropagation()}
      role="dialog"
      aria-modal="true"
      tabindex="-1"
    >
      <!-- Header -->
      <div class="px-5 py-4 border-b border-slate-800 flex items-center justify-between bg-slate-950/40">
        <div class="flex items-center gap-2">
          <ShieldCheck class="w-5 h-5 text-cyan-400" />
          <h3 class="text-base font-bold text-slate-100">Rescue Bin Staging</h3>
        </div>
        <button
          class="text-slate-400 hover:text-slate-100 p-1.5 rounded-lg transition-colors"
          onclick={() => system.closeRescueBin()}
          aria-label="Close Rescue Bin"
        >
          <X class="w-5 h-5" />
        </button>
      </div>

      <!-- Banner -->
      <div class="p-4 bg-cyan-950/20 border-b border-cyan-800/30 text-xs text-cyan-300">
        Cleaned files are safely isolated here for {settings.retentionDays} days. You can inspect staged manifests or roll back anytime.
      </div>

      {#if actionMsg}
        <div class="m-4 p-3 border text-xs rounded-lg flex flex-col gap-1.5 {actionIsError ? 'bg-amber-950/70 border-amber-800 text-amber-300' : 'bg-emerald-950/80 border-emerald-800 text-emerald-300'}">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              {#if actionIsError}
                <AlertTriangle class="w-4 h-4 shrink-0 text-amber-400" />
              {:else}
                <CheckCircle2 class="w-4 h-4 shrink-0 text-emerald-400" />
              {/if}
              <span class="font-medium">{actionMsg}</span>
            </div>
            <button class="p-0.5 text-slate-400 hover:text-slate-200" onclick={() => (actionMsg = '')}>
              <X class="w-3.5 h-3.5" />
            </button>
          </div>
          {#if actionErrorsList.length > 0}
            <div class="mt-1 pt-1.5 border-t border-amber-900/60 font-mono text-[11px] text-amber-200/90 space-y-0.5 max-h-24 overflow-y-auto">
              {#each actionErrorsList as err}
                <div>• {err}</div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <!-- Quick Metrics -->
      <div class="px-5 py-3 border-b border-slate-800/80 flex items-center justify-between text-xs">
        <span class="text-slate-400">
          Staged: <strong class="text-slate-200">{stages.length}</strong> cleanups
        </span>
        <span class="text-slate-400">
          Staged Size: <strong class="text-cyan-400 font-mono">{formatBytes(totalStagedBytes)}</strong>
        </span>
      </div>

      <!-- Stages List -->
      <div class="flex-1 overflow-y-auto p-4 space-y-3">
        {#if isLoading}
          <div class="p-8 text-center text-slate-400 flex flex-col items-center gap-2">
            <RefreshCw class="w-5 h-5 animate-spin text-cyan-400" />
            <span class="text-xs">Reading rescue stages...</span>
          </div>
        {:else if stages.length === 0}
          <div class="p-8 text-center text-slate-500 text-xs">
            Rescue Bin is currently empty. No staged cleanups pending.
          </div>
        {:else}
          {#each stages as stage}
            {@const manifestState = stageManifests[stage.stage_id] || { isOpen: false, isLoading: false, items: null }}
            {@const formattedDate = formatDate(stage.created_at_secs)}
            <div class="bg-slate-950/60 border border-slate-800 rounded-xl p-3.5 flex flex-col gap-2.5 transition-all">
              <!-- Stage Header with Human Date -->
              <div class="flex items-center justify-between">
                <div>
                  <h4 class="text-xs font-bold text-slate-100">
                    Cleanup Run • {formattedDate} ({timeAgo(stage.created_at_secs)})
                  </h4>
                  <span class="font-mono text-[10px] text-slate-500 truncate block max-w-[240px]" title={stage.stage_id}>
                    {stage.stage_id}
                  </span>
                </div>
                <Badge variant={stage.days_old > 10 ? 'warning' : 'accent'}>
                  {stage.days_old === 0 ? 'Today' : `${stage.days_old}d ago`}
                </Badge>
              </div>

              <!-- Metrics & Accordion Toggle -->
              <div class="flex items-center justify-between text-xs text-slate-400 font-mono">
                <button
                  class="flex items-center gap-1.5 text-cyan-400 hover:text-cyan-300 transition-colors font-sans text-xs"
                  onclick={() => toggleManifest(stage.stage_id)}
                  title="Inspect staged files without restoring"
                >
                  <FileText class="w-3.5 h-3.5" />
                  <span>{stage.file_count} files</span>
                  {#if manifestState.isOpen}
                    <ChevronUp class="w-3.5 h-3.5" />
                  {:else}
                    <ChevronDown class="w-3.5 h-3.5" />
                  {/if}
                </button>
                <span class="text-emerald-400 font-bold">{formatBytes(stage.total_bytes)}</span>
              </div>

              <!-- Expandable Manifest Inspector Accordion -->
              {#if manifestState.isOpen}
                <div class="mt-1 p-2.5 bg-slate-900/90 border border-slate-800 rounded-lg text-xs space-y-2 animate-in fade-in duration-200">
                  <div class="flex items-center justify-between text-[11px] text-slate-400 font-medium">
                    <span>Staged Files Manifest</span>
                    {#if manifestState.items}
                      <span class="font-mono">{manifestState.items.length} items</span>
                    {/if}
                  </div>

                  {#if manifestState.isLoading}
                    <div class="py-3 flex items-center justify-center gap-2 text-slate-400">
                      <RefreshCw class="w-3.5 h-3.5 animate-spin text-cyan-400" />
                      <span class="text-[11px]">Loading manifest details...</span>
                    </div>
                  {:else if manifestState.error}
                    <div class="text-[11px] text-rose-400 flex items-center gap-1.5">
                      <AlertCircle class="w-3.5 h-3.5" />
                      <span>{manifestState.error}</span>
                    </div>
                  {:else if manifestState.items?.length === 0}
                    <div class="text-[11px] text-slate-500 py-1">No files listed in manifest.</div>
                  {:else if manifestState.items}
                    <div class="max-h-36 overflow-y-auto space-y-1 font-mono text-[10.5px] pr-1">
                      {#each manifestState.items as item}
                        <div class="flex items-center justify-between py-0.5 text-slate-300 border-b border-slate-800/40 last:border-0 hover:text-slate-100">
                          <span class="truncate max-w-[260px]" title={item.original_path}>
                            {item.original_path}
                          </span>
                          <span class="text-emerald-400/90 whitespace-nowrap shrink-0 ml-2">
                            {formatBytes(item.size_bytes)}
                          </span>
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/if}

              <!-- Actions -->
              <div class="pt-2 border-t border-slate-800/80 flex items-center justify-end gap-2">
                <button
                  class="px-2.5 py-1 text-xs font-medium text-cyan-300 hover:bg-cyan-950/80 border border-cyan-800/60 rounded flex items-center gap-1.5 transition-colors"
                  onclick={() => handleRestore(stage)}
                >
                  <RotateCcw class="w-3.5 h-3.5" />
                  <span>Restore</span>
                </button>
                <button
                  class="px-2.5 py-1 text-xs font-medium text-rose-300 hover:bg-rose-950/80 border border-rose-800/60 rounded flex items-center gap-1.5 transition-colors"
                  onclick={() => handlePurge(stage)}
                >
                  <Trash2 class="w-3.5 h-3.5" />
                  <span>Purge</span>
                </button>
              </div>
            </div>
          {/each}
        {/if}
      </div>

      <!-- Footer action -->
      {#if stages.length > 0}
        <div class="p-4 border-t border-slate-800 bg-slate-950/50 flex justify-between items-center">
          <button
            class="text-xs text-slate-400 hover:text-slate-200 transition-colors"
            onclick={handleAutoPurge}
          >
            Purge Expired (&gt;{settings.retentionDays}d)
          </button>
          <button
            class="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium rounded-lg transition-colors"
            onclick={() => system.closeRescueBin()}
          >
            Close
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}

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
