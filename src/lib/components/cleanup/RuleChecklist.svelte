<script>
  import { onMount } from 'svelte';
  import { system } from '../../stores/system.svelte.js';
  import { fetchCleanupRules, scanRules } from '../../api.js';
  import { formatBytes } from '../../utils.js';
  import Badge from '../common/Badge.svelte';
  import { ShieldCheck, RefreshCw, Sparkles, AlertTriangle, Layers } from 'lucide-svelte';

  let rules = $state([]);
  let scanSummary = $state(null);
  let selectedRuleIds = $state(new Set());
  let isScanning = $state(false);

  async function loadAndScanRules() {
    isScanning = true;
    system.clearOperationProgress();
    try {
      if (rules.length === 0) {
        rules = await fetchCleanupRules();
        // Default select all safe rules
        selectedRuleIds = new Set(
          rules.filter((r) => r.risk === 'safe').map((r) => r.id)
        );
      }
      scanSummary = await scanRules(rules);
    } catch (e) {
      console.error('Error scanning rules:', e);
    } finally {
      isScanning = false;
      system.clearOperationProgress();
    }
  }

  onMount(() => {
    loadAndScanRules();
  });

  function toggleRule(id) {
    const next = new Set(selectedRuleIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    selectedRuleIds = next;
  }

  function selectAllSafe() {
    selectedRuleIds = new Set(
      rules.filter((r) => r.risk === 'safe').map((r) => r.id)
    );
  }

  function selectAll() {
    selectedRuleIds = new Set(rules.map((r) => r.id));
  }

  function clearAll() {
    selectedRuleIds = new Set();
  }

  function getRuleScan(id) {
    return scanSummary?.results?.find((r) => r.rule_id === id);
  }

  // Selected totals
  const selectedScanResults = $derived(
    scanSummary?.results?.filter((r) => selectedRuleIds.has(r.rule_id)) ?? []
  );

  const selectedBytes = $derived(
    selectedScanResults.reduce((acc, r) => acc + r.total_bytes, 0)
  );

  const selectedFileCount = $derived(
    selectedScanResults.reduce((acc, r) => acc + r.file_count, 0)
  );

  let validationMsg = $state('');

  function handleOpenPreview() {
    if (selectedScanResults.length === 0 || selectedBytes === 0) {
      validationMsg = 'Please select at least one rule with cleanable files to preview and stage.';
      setTimeout(() => (validationMsg = ''), 4000);
      return;
    }

    system.setPreviewCleanup({
      rule_ids: Array.from(selectedRuleIds),
      results: selectedScanResults,
      total_bytes: selectedBytes,
      total_files: selectedFileCount,
    });
  }

  const safeRules = $derived(rules.filter((r) => r.risk === 'safe'));
  const reviewRules = $derived(rules.filter((r) => r.risk === 'review'));
  const advancedRules = $derived(rules.filter((r) => r.risk === 'advanced'));
</script>

<div class="space-y-6">
  <!-- Top Bar -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 bg-slate-900/70 border border-slate-800 rounded-xl p-4 shadow-md">
    <div>
      <h2 class="text-lg font-bold text-slate-100 flex items-center gap-2">
        <Sparkles class="w-4 h-4 text-cyan-400" />
        Safe Cleanup Rules Engine
      </h2>
      <p class="text-xs text-slate-400 mt-0.5">
        Declarative Windows cleanup rules with safety risk tiers. All purges stage into the Rescue Bin.
      </p>
    </div>

    <div class="flex items-center gap-2 flex-wrap">
      <button
        class="px-3 py-1.5 text-xs font-semibold text-cyan-300 bg-cyan-950/60 hover:bg-cyan-900/60 border border-cyan-800/60 rounded-lg flex items-center gap-1.5 transition-colors"
        onclick={() => system.openRescueBin()}
      >
        <ShieldCheck class="w-3.5 h-3.5" />
        <span>Open Rescue Bin</span>
      </button>

      <button
        class="p-1.5 bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 rounded-lg transition-colors"
        onclick={loadAndScanRules}
        disabled={isScanning}
        title="Re-scan rules"
      >
        <RefreshCw class="w-4 h-4 {isScanning ? 'animate-spin text-cyan-400' : ''}" />
      </button>
    </div>
  </div>

  {#if validationMsg}
    <div class="px-4 py-2.5 bg-amber-950/70 border border-amber-800 text-amber-300 text-xs rounded-xl flex items-center gap-2 animate-in fade-in">
      <AlertTriangle class="w-4 h-4 text-amber-400 shrink-0" />
      <span>{validationMsg}</span>
    </div>
  {/if}

  {#if isScanning && system.operationProgress?.stage === 'scanning'}
    <div class="p-4 bg-slate-950/80 border border-cyan-800/60 rounded-xl space-y-2.5">
      <div class="flex items-center justify-between text-xs">
        <span class="text-cyan-400 font-semibold flex items-center gap-2">
          <RefreshCw class="w-3.5 h-3.5 animate-spin" />
          <span>Scanning: {system.operationProgress.current_item}</span>
        </span>
        <span class="font-mono text-cyan-300 font-bold">
          {Math.min(100, Math.max(0, Math.round(system.operationProgress.percent)))}%
        </span>
      </div>
      <div class="w-full bg-slate-800 rounded-full h-2 overflow-hidden">
        <div
          class="bg-gradient-to-r from-cyan-500 to-emerald-400 h-2 rounded-full transition-all duration-150"
          style="width: {Math.min(100, Math.max(0, Math.round(system.operationProgress.percent)))}%"
        ></div>
      </div>
      <div class="flex items-center justify-between text-[11px] text-slate-400 font-mono">
        <span>Rule {system.operationProgress.processed_count} of {system.operationProgress.total_count}</span>
        <span>{formatBytes(system.operationProgress.bytes_processed)} identified</span>
      </div>
    </div>
  {/if}

  <!-- Action Floating/Sticky Bar -->
  <div class="bg-gradient-to-r from-slate-900 via-slate-900 to-slate-950 border border-slate-800 rounded-xl p-4 shadow-xl flex flex-col sm:flex-row sm:items-center justify-between gap-4">
    <div class="flex items-center gap-4">
      <div>
        <div class="text-xs text-slate-400 font-medium">Selected For Cleanup</div>
        <div class="text-xl font-bold font-mono text-emerald-400">
          {formatBytes(selectedBytes)}
          <span class="text-xs font-normal text-slate-400 ml-1">({selectedFileCount} files)</span>
        </div>
      </div>

      <div class="h-8 w-px bg-slate-800 hidden sm:block"></div>

      <!-- Quick selectors -->
      <div class="flex items-center gap-1.5 text-xs">
        <button
          class="px-2.5 py-1 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded border border-slate-700 font-medium transition-colors"
          onclick={selectAllSafe}
        >
          Select All Safe
        </button>
        <button
          class="px-2.5 py-1 text-slate-400 hover:text-slate-200 transition-colors"
          onclick={selectAll}
        >
          All
        </button>
        <button
          class="px-2.5 py-1 text-slate-400 hover:text-slate-200 transition-colors"
          onclick={clearAll}
        >
          Clear
        </button>
      </div>
    </div>

    <button
      class="px-5 py-2.5 bg-emerald-600 hover:bg-emerald-500 active:scale-95 text-white font-semibold text-xs rounded-lg shadow-lg shadow-emerald-950/50 flex items-center justify-center gap-2 transition-all disabled:opacity-50 disabled:pointer-events-none"
      onclick={handleOpenPreview}
      disabled={selectedBytes === 0 || isScanning}
    >
      <Sparkles class="w-4 h-4" />
      <span>Preview & Clean ({formatBytes(selectedBytes)})</span>
    </button>
  </div>

  <!-- Tier 1: Safe Rules -->
  <div class="bg-slate-900/60 border border-slate-800 rounded-xl p-4">
    <div class="flex items-center justify-between mb-3 border-b border-slate-800 pb-2">
      <div class="flex items-center gap-2">
        <Badge variant="success">Tier 1: Safe</Badge>
        <span class="text-xs font-semibold text-slate-200">1-Click Purge (Zero Risk)</span>
      </div>
      <span class="text-xs text-slate-400">Scratch caches, crash dumps, and temp files</span>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
      {#each safeRules as rule}
        {@const scan = getRuleScan(rule.id)}
        <label
          class="flex items-start gap-3 p-3 rounded-lg border transition-all cursor-pointer {selectedRuleIds.has(rule.id) ? 'bg-cyan-950/30 border-cyan-800/60' : 'bg-slate-950/40 border-slate-800/80 hover:border-slate-700'}"
        >
          <input
            type="checkbox"
            checked={selectedRuleIds.has(rule.id)}
            onchange={() => toggleRule(rule.id)}
            class="mt-1 rounded bg-slate-800 border-slate-700 text-cyan-500 focus:ring-0 focus:ring-offset-0 cursor-pointer"
          />
          <div class="flex-1 min-w-0">
            <div class="flex items-center justify-between">
              <span class="text-xs font-semibold text-slate-100 truncate">{rule.name}</span>
              <span class="text-xs font-mono font-bold text-emerald-400 ml-2 whitespace-nowrap">
                {scan ? formatBytes(scan.total_bytes) : '...'}
              </span>
            </div>
            <p class="text-[11px] text-slate-400 line-clamp-1 mt-0.5">{rule.description}</p>
            <p class="text-[10px] text-slate-500 font-mono truncate mt-0.5">{rule.path}</p>
          </div>
        </label>
      {/each}
    </div>
  </div>

  <!-- Tier 2: Review Rules -->
  <div class="bg-slate-900/60 border border-slate-800 rounded-xl p-4">
    <div class="flex items-center justify-between mb-3 border-b border-slate-800 pb-2">
      <div class="flex items-center gap-2">
        <Badge variant="warning">Tier 2: Review Recommended</Badge>
        <span class="text-xs font-semibold text-slate-200">Requires User Verification</span>
      </div>
      <span class="text-xs text-slate-400">Old downloads and package manager tarball caches</span>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
      {#each reviewRules as rule}
        {@const scan = getRuleScan(rule.id)}
        <label
          class="flex items-start gap-3 p-3 rounded-lg border transition-all cursor-pointer {selectedRuleIds.has(rule.id) ? 'bg-amber-950/30 border-amber-800/60' : 'bg-slate-950/40 border-slate-800/80 hover:border-slate-700'}"
        >
          <input
            type="checkbox"
            checked={selectedRuleIds.has(rule.id)}
            onchange={() => toggleRule(rule.id)}
            class="mt-1 rounded bg-slate-800 border-slate-700 text-amber-500 focus:ring-0 focus:ring-offset-0 cursor-pointer"
          />
          <div class="flex-1 min-w-0">
            <div class="flex items-center justify-between">
              <span class="text-xs font-semibold text-slate-100 truncate">{rule.name}</span>
              <span class="text-xs font-mono font-bold text-amber-400 ml-2 whitespace-nowrap">
                {scan ? formatBytes(scan.total_bytes) : '...'}
              </span>
            </div>
            <p class="text-[11px] text-slate-400 line-clamp-1 mt-0.5">{rule.description}</p>
            <p class="text-[10px] text-slate-500 font-mono truncate mt-0.5">{rule.path}</p>
          </div>
        </label>
      {/each}
    </div>
  </div>

  <!-- Tier 3: Advanced Rules -->
  {#if advancedRules.length > 0}
    <div class="bg-slate-900/60 border border-slate-800 rounded-xl p-4">
      <div class="flex items-center justify-between mb-3 border-b border-slate-800 pb-2">
        <div class="flex items-center gap-2">
          <Badge variant="danger">Tier 3: Advanced</Badge>
          <span class="text-xs font-semibold text-slate-200">System Level Services</span>
        </div>
        <span class="text-xs text-slate-400">Delivery optimization and Windows system update store</span>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        {#each advancedRules as rule}
          {@const scan = getRuleScan(rule.id)}
          <label
            class="flex items-start gap-3 p-3 rounded-lg border transition-all cursor-pointer {selectedRuleIds.has(rule.id) ? 'bg-rose-950/30 border-rose-800/60' : 'bg-slate-950/40 border-slate-800/80 hover:border-slate-700'}"
          >
            <input
              type="checkbox"
              checked={selectedRuleIds.has(rule.id)}
              onchange={() => toggleRule(rule.id)}
              class="mt-1 rounded bg-slate-800 border-slate-700 text-rose-500 focus:ring-0 focus:ring-offset-0 cursor-pointer"
            />
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between">
                <span class="text-xs font-semibold text-slate-100 truncate">{rule.name}</span>
                <span class="text-xs font-mono font-bold text-rose-400 ml-2 whitespace-nowrap">
                  {scan ? formatBytes(scan.total_bytes) : '...'}
                </span>
              </div>
              <p class="text-[11px] text-slate-400 line-clamp-1 mt-0.5">{rule.description}</p>
              <p class="text-[10px] text-slate-500 font-mono truncate mt-0.5">{rule.path}</p>
            </div>
          </label>
        {/each}
      </div>
    </div>
  {/if}
</div>
