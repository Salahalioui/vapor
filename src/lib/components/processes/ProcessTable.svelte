<script>
  import { system } from '../../stores/system.svelte.js';
  import { settings } from '../../stores/settings.svelte.js';
  import { killProcess } from '../../api.js';
  import Badge from '../common/Badge.svelte';
  import ConfirmModal from '../common/ConfirmModal.svelte';
  import {
    Activity,
    Search,
    Sparkles,
    Trash2,
    RefreshCw,
    Shield,
    AlertOctagon,
    SlidersHorizontal,
    Pause,
    Play,
    AlertTriangle,
    CheckCircle2,
    Info,
    X
  } from 'lucide-svelte';

  let searchQuery = $state('');
  let sortBy = $state('cpu'); // 'cpu' | 'memory' | 'name'
  let sortAsc = $state(false);
  let categoryFilter = $state('all'); // 'all' | 'system' | 'browser' | 'developer' | 'user'

  // Live feed freeze and pause controls
  let isFeedPaused = $state(false);
  let isTableHovered = $state(false);
  let frozenProcesses = $state(null);
  let fleetAuditError = $state('');
  let showFleetCard = $state(true);

  $effect(() => {
    const procs = system.hardware?.top_processes ?? [];
    if (!isFeedPaused && !isTableHovered) {
      frozenProcesses = procs;
    }
  });

  const allProcesses = $derived(frozenProcesses ?? system.hardware?.top_processes ?? []);

  const filteredProcesses = $derived(
    allProcesses.filter((p) => {
      if (categoryFilter !== 'all' && p.category !== categoryFilter) return false;
      if (!searchQuery.trim()) return true;
      const q = searchQuery.toLowerCase();
      return (
        p.name.toLowerCase().includes(q) ||
        (p.publisher && p.publisher.toLowerCase().includes(q)) ||
        (p.description && p.description.toLowerCase().includes(q)) ||
        p.pid.toString().includes(q)
      );
    })
  );

  const sortedProcesses = $derived(
    [...filteredProcesses].sort((a, b) => {
      let cmp = 0;
      if (sortBy === 'cpu') {
        cmp = a.cpu_percent - b.cpu_percent;
      } else if (sortBy === 'memory') {
        cmp = a.memory_bytes - b.memory_bytes;
      } else {
        cmp = a.name.localeCompare(b.name);
      }
      return sortAsc ? cmp : -cmp;
    })
  );

  let confirmState = $state({
    isOpen: false,
    title: '',
    message: '',
    confirmText: '',
    isDanger: false,
    action: () => {}
  });

  function handleKill(proc) {
    if (!proc.can_kill) {
      confirmState = {
        isOpen: true,
        title: 'Protected System Process',
        message: `Safety Guardrail: "${proc.name}" (PID ${proc.pid}) is a critical Windows system process and cannot be terminated to prevent system instability.`,
        confirmText: 'Understood',
        isDanger: false,
        action: () => {}
      };
      return;
    }

    confirmState = {
      isOpen: true,
      title: `Terminate "${proc.name}"?`,
      message: `Are you sure you want to kill process "${proc.name}" (PID ${proc.pid})?\n\nCPU: ${proc.cpu_percent.toFixed(1)}% | RAM: ${proc.memory_mb.toFixed(0)} MB\n\nUnsaved work in this application may be lost.`,
      confirmText: 'Kill Process',
      isDanger: true,
      action: async () => {
        try {
          await killProcess(proc.pid);
          system.refreshHardware();
        } catch (e) {
          confirmState = {
            isOpen: true,
            title: 'Kill Failed',
            message: `Failed to terminate process "${proc.name}": ${e}`,
            confirmText: 'OK',
            isDanger: false,
            action: () => {}
          };
        }
      }
    };
  }

  function handleExplain(proc) {
    system.openExplainProcess(proc);
  }

  async function handleFleetAudit() {
    fleetAuditError = '';
    showFleetCard = true;
    try {
      await system.runFleetAiAudit(settings.geminiApiKey || '');
    } catch (e) {
      if (!settings.geminiApiKey) {
        fleetAuditError = 'Gemini API key is required to audit uncached processes. Click any "Ask AI" button or go to Settings to add your key.';
      } else {
        fleetAuditError = `Fleet AI Audit failed: ${e}`;
      }
    }
  }

  function toggleSort(field) {
    if (sortBy === field) {
      sortAsc = !sortAsc;
    } else {
      sortBy = field;
      sortAsc = false;
    }
  }
</script>

<div class="bg-slate-900/70 border border-slate-800 rounded-xl p-5 shadow-lg flex flex-col h-[calc(100vh-140px)]">
  <!-- Top Bar -->
  <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-4">
    <div>
      <h2 class="text-lg font-bold text-slate-100 flex items-center gap-2">
        <Activity class="w-5 h-5 text-cyan-400" />
        Process Radar & System Tasks
      </h2>
      <p class="text-xs text-slate-400 mt-0.5">
        Sampled 2s CPU/RAM monitoring with offline Windows dictionary and privacy-scrubbed Gemini AI explanation.
      </p>
    </div>

    <!-- Search and Controls -->
    <div class="flex items-center gap-2 flex-wrap">
      <div class="relative">
        <Search class="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
        <input
          type="text"
          placeholder="Filter processes, PIDs..."
          bind:value={searchQuery}
          class="bg-slate-800 text-xs text-slate-200 pl-8 pr-3 py-1.5 rounded-lg border border-slate-700 focus:outline-none focus:border-cyan-500 w-48 sm:w-56 placeholder:text-slate-500 font-medium"
        />
      </div>

      <select
        bind:value={categoryFilter}
        class="bg-slate-800 text-xs text-slate-200 border border-slate-700 rounded-lg px-2.5 py-1.5 focus:outline-none focus:border-cyan-500 font-medium"
      >
        <option value="all">All Categories</option>
        <option value="system">System Core</option>
        <option value="browser">Web Browsers</option>
        <option value="developer">Developer Tools</option>
        <option value="user">User Apps</option>
      </select>

      <!-- Pause / Resume Live Feed Toggle Button -->
      <button
        class="flex items-center gap-1.5 text-xs px-2.5 py-1.5 rounded-lg border font-medium transition-colors {isFeedPaused ? 'bg-amber-950/70 text-amber-300 border-amber-700' : 'bg-slate-800 text-slate-300 border-slate-700 hover:text-slate-100'}"
        onclick={() => (isFeedPaused = !isFeedPaused)}
        title={isFeedPaused ? "Resume Live Feed" : "Pause Live Feed"}
      >
        {#if isFeedPaused}
          <Play class="w-3.5 h-3.5 text-amber-400" />
          <span>Resume Feed</span>
        {:else}
          <Pause class="w-3.5 h-3.5 text-slate-400" />
          <span>Pause Feed</span>
        {/if}
      </button>

      <!-- Full Fleet AI Audit Button -->
      <button
        class="flex items-center gap-1.5 text-xs px-2.5 py-1.5 rounded-lg border font-semibold transition-all shadow-sm {system.isAuditingFleet ? 'bg-purple-950/80 text-purple-300 border-purple-800' : 'bg-purple-600 hover:bg-purple-500 text-white border-purple-500 active:scale-95'}"
        onclick={handleFleetAudit}
        disabled={system.isAuditingFleet}
        title="Batch analyze top 20 active processes with Gemini in a single prompt"
      >
        {#if system.isAuditingFleet}
          <RefreshCw class="w-3.5 h-3.5 animate-spin text-purple-300" />
          <span>Auditing Fleet...</span>
        {:else}
          <Sparkles class="w-3.5 h-3.5 text-purple-200" />
          <span>Full Fleet AI Audit</span>
        {/if}
      </button>

      <!-- Refresh Button -->
      <button
        class="p-1.5 bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 rounded-lg transition-colors"
        onclick={() => system.refreshHardware()}
        disabled={system.isRefreshingHardware}
        title="Refresh Processes"
      >
        <RefreshCw class="w-4 h-4 {system.isRefreshingHardware ? 'animate-spin text-cyan-400' : ''}" />
      </button>
    </div>
  </div>

  <!-- Fleet Audit Error Banner -->
  {#if fleetAuditError}
    <div class="mb-3 px-3 py-2 bg-rose-950/80 border border-rose-800 text-rose-300 text-xs rounded-lg flex items-center justify-between">
      <div class="flex items-center gap-2">
        <AlertTriangle class="w-4 h-4 shrink-0 text-rose-400" />
        <span>{fleetAuditError}</span>
      </div>
      <button class="p-1 text-rose-400 hover:text-rose-200 rounded" onclick={() => (fleetAuditError = '')}>
        <X class="w-3.5 h-3.5" />
      </button>
    </div>
  {/if}

  <!-- Fleet Overview Card -->
  {#if system.batchFleetReport && showFleetCard}
    {@const report = system.batchFleetReport}
    <div class="mb-4 p-3.5 bg-purple-950/25 border border-purple-800/50 rounded-xl relative shadow-md">
      <button
        class="absolute top-2.5 right-2.5 p-1 text-slate-400 hover:text-slate-200 rounded-lg transition-colors"
        onclick={() => (showFleetCard = false)}
        title="Dismiss Fleet Overview"
      >
        <X class="w-4 h-4" />
      </button>

      <div class="flex items-center gap-2 mb-2">
        <Sparkles class="w-4 h-4 text-purple-400" />
        <h4 class="text-xs font-bold text-slate-100 uppercase tracking-wider">
          Gemini Fleet AI Audit Report ({report.total_analyzed} Processes Evaluated)
        </h4>
        <span class="text-[10px] text-purple-300 bg-purple-900/50 px-1.5 py-0.5 rounded border border-purple-700/60 font-mono">
          Cached Offline
        </span>
      </div>

      <!-- Quick Metrics -->
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-2 mb-2.5">
        <div class="px-2.5 py-1.5 bg-slate-900/80 border border-emerald-900/50 rounded-lg">
          <div class="text-[10px] text-slate-400">Safe Processes</div>
          <div class="text-sm font-bold text-emerald-400 font-mono">{report.safe_count}</div>
        </div>
        <div class="px-2.5 py-1.5 bg-slate-900/80 border border-amber-900/50 rounded-lg">
          <div class="text-[10px] text-slate-400">Caution / Background</div>
          <div class="text-sm font-bold text-amber-400 font-mono">{report.caution_count}</div>
        </div>
        <div class="px-2.5 py-1.5 bg-slate-900/80 border border-rose-900/50 rounded-lg">
          <div class="text-[10px] text-slate-400">Bloatware / Heavy</div>
          <div class="text-sm font-bold text-rose-400 font-mono">{report.bloatware_count}</div>
        </div>
        <div class="px-2.5 py-1.5 bg-slate-900/80 border border-cyan-900/50 rounded-lg">
          <div class="text-[10px] text-slate-400">Core Windows</div>
          <div class="text-sm font-bold text-cyan-400 font-mono">{report.critical_count}</div>
        </div>
      </div>

      <!-- Fleet Summary & Recommendations -->
      <p class="text-xs text-slate-300 leading-relaxed mb-2">
        {report.fleet_summary}
      </p>

      {#if report.recommendations && report.recommendations.length > 0}
        <div class="text-[11px] text-slate-400 bg-slate-900/60 p-2 rounded-lg border border-slate-800/80">
          <span class="font-semibold text-purple-300">Actionable Suggestions:</span>
          <ul class="list-disc list-inside mt-1 space-y-0.5 text-slate-300">
            {#each report.recommendations as rec}
              <li>{rec}</li>
            {/each}
          </ul>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Summary Counters & Live Feed State -->
  <div class="mb-3 px-3 py-2 bg-slate-950/40 border border-slate-800/80 rounded-lg flex items-center justify-between text-xs">
    <div class="flex items-center gap-4">
      <span class="text-slate-400">
        Total System Tasks: <strong class="text-slate-200">{system.hardware?.process_count ?? allProcesses.length}</strong>
      </span>
      <span class="text-slate-400">
        Active Matches: <strong class="text-cyan-400 font-mono">{sortedProcesses.length}</strong>
      </span>
    </div>
    <div class="flex items-center gap-2 text-[11px]">
      {#if isFeedPaused}
        <span class="inline-block w-2 h-2 rounded-full bg-amber-500"></span>
        <span class="text-amber-400 font-medium">Live Feed Paused</span>
      {:else if isTableHovered}
        <span class="inline-block w-2 h-2 rounded-full bg-cyan-400"></span>
        <span class="text-cyan-300 font-medium">Feed Frozen (Hover)</span>
      {:else}
        <span class="inline-block w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
        <span class="text-slate-400">2s Live Feed Active</span>
      {/if}
    </div>
  </div>

  <!-- Table Container (Freezes updates when cursor hovers over it) -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="flex-1 overflow-x-auto overflow-y-auto border border-slate-800 rounded-lg bg-slate-950/30"
    role="region"
    aria-label="Process list"
    onmouseenter={() => (isTableHovered = true)}
    onmouseleave={() => (isTableHovered = false)}
  >
    <table class="w-full text-left text-xs text-slate-300 border-collapse">
      <thead class="sticky top-0 bg-slate-950 text-slate-400 uppercase tracking-wider font-semibold border-b border-slate-800 z-10 text-[11px]">
        <tr>
          <th class="py-2.5 px-3 cursor-pointer select-none" onclick={() => toggleSort('name')}>
            Process Name {sortBy === 'name' ? (sortAsc ? '↑' : '↓') : ''}
          </th>
          <th class="py-2.5 px-3">PID</th>
          <th class="py-2.5 px-3 cursor-pointer select-none" onclick={() => toggleSort('cpu')}>
            CPU % {sortBy === 'cpu' ? (sortAsc ? '↑' : '↓') : ''}
          </th>
          <th class="py-2.5 px-3 cursor-pointer select-none" onclick={() => toggleSort('memory')}>
            RAM {sortBy === 'memory' ? (sortAsc ? '↑' : '↓') : ''}
          </th>
          <th class="py-2.5 px-3">Identity / Category</th>
          <th class="py-2.5 px-3 text-right">Actions</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-slate-800/60 font-sans">
        {#if sortedProcesses.length === 0}
          <tr>
            <td colspan="6" class="py-8 text-center text-slate-500 text-xs">
              No matching processes found.
            </td>
          </tr>
        {:else}
          {#each sortedProcesses as proc (proc.pid)}
            {@const cachedAi = system.getProcessExplanationFromCache(proc.name)}
            <tr class="hover:bg-slate-800/40 transition-colors">
              <td class="py-2 px-3 font-semibold text-slate-100 max-w-[240px] truncate" title={proc.name}>
                <div class="flex items-center gap-1.5 flex-wrap">
                  <span class="truncate">{proc.name}</span>

                  <!-- Suspicious Process Guardian Badges -->
                  {#if proc.signature_badge === 'Verified (Microsoft)'}
                    <span class="text-[9px] font-mono px-1 py-0.2 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/30" title="Cryptographically signed by Microsoft">
                      🟢 MS Verified
                    </span>
                  {:else if proc.signature_badge === 'Verified (Known Publisher)'}
                    <span class="text-[9px] font-mono px-1 py-0.2 rounded bg-blue-500/10 text-blue-400 border border-blue-500/30" title="Cryptographically signed by verified publisher">
                      🔵 Signed
                    </span>
                  {:else if proc.signature_badge === 'Unsigned in Temp/AppData'}
                    <span class="text-[9px] font-mono px-1 py-0.2 rounded bg-rose-500/10 text-rose-400 border border-rose-500/30 font-bold" title="Unsigned binary running from user temp/appdata folder">
                      🔴 Unsigned Temp
                    </span>
                  {:else if proc.signature_badge === 'Unsigned'}
                    <span class="text-[9px] font-mono px-1 py-0.2 rounded bg-amber-500/10 text-amber-400 border border-amber-500/30" title="Unsigned executable">
                      🟡 Unsigned
                    </span>
                  {/if}

                  {#if cachedAi}
                    <span
                      class="text-[9px] px-1 py-0.2 rounded border font-semibold uppercase {cachedAi.safety === 'safe' ? 'text-emerald-400 bg-emerald-950/60 border-emerald-800/50' : cachedAi.safety === 'bloatware' ? 'text-rose-400 bg-rose-950/60 border-rose-800/50' : 'text-amber-400 bg-amber-950/60 border-amber-800/50'}"
                      title="Fleet AI: {cachedAi.summary}"
                    >
                      AI: {cachedAi.safety}
                    </span>
                  {/if}
                </div>
              </td>
              <td class="py-2 px-3 font-mono text-slate-400 text-[11px]">
                {proc.pid}
              </td>
              <td class="py-2 px-3 font-mono font-semibold whitespace-nowrap">
                <span class={proc.cpu_percent > 15 ? 'text-rose-400 font-bold' : proc.cpu_percent > 5 ? 'text-amber-400' : 'text-slate-300'}>
                  {proc.cpu_percent.toFixed(1)}%
                </span>
              </td>
              <td class="py-2 px-3 font-mono font-semibold text-emerald-400 whitespace-nowrap">
                {proc.memory_mb.toFixed(0)} MB
              </td>
              <td class="py-2 px-3 text-slate-400 max-w-[240px] truncate" title={proc.exe_path || proc.description || proc.publisher}>
                <div class="flex items-center gap-1.5 truncate">
                  {#if proc.location_category}
                    <span class="text-[10px] font-mono text-slate-500 bg-slate-900 px-1 rounded border border-slate-800">
                      {proc.location_category}
                    </span>
                  {/if}
                  {#if proc.description}
                    <span class="text-slate-300 truncate">{proc.description}</span>
                  {:else if proc.publisher}
                    <span class="text-slate-400 truncate">{proc.publisher}</span>
                  {:else}
                    <span class="text-slate-500 capitalize">{proc.category}</span>
                  {/if}
                </div>
              </td>
              <td class="py-2 px-3 text-right whitespace-nowrap space-x-1">
                <!-- Explain with Gemini button -->
                <button
                  class="px-2 py-1 {cachedAi ? 'bg-purple-950/80 hover:bg-purple-900 text-purple-300 border-purple-800/60' : 'bg-cyan-950/80 hover:bg-cyan-900 text-cyan-300 border-cyan-800/60'} border rounded text-[11px] font-medium transition-colors inline-flex items-center gap-1"
                  onclick={() => handleExplain(proc)}
                  title={cachedAi ? "View Cached AI Explanation" : "Privacy-Sanitized Gemini AI Explanation"}
                >
                  <Sparkles class="w-3 h-3 {cachedAi ? 'text-purple-400' : 'text-cyan-400'}" />
                  <span>{cachedAi ? 'AI Insight' : 'Ask AI'}</span>
                </button>

                <!-- Kill Process button -->
                {#if proc.can_kill}
                  <button
                    class="px-2 py-1 bg-rose-950/60 hover:bg-rose-900/80 text-rose-300 border border-rose-800/60 rounded text-[11px] font-medium transition-colors inline-flex items-center gap-1"
                    onclick={() => handleKill(proc)}
                    title="Terminate Process"
                  >
                    <Trash2 class="w-3 h-3 text-rose-400" />
                    <span>Kill</span>
                  </button>
                {:else}
                  <span
                    class="px-2 py-1 text-slate-600 border border-slate-800 rounded text-[11px] font-medium inline-flex items-center gap-1 cursor-not-allowed"
                    title="Protected Windows system process"
                  >
                    <Shield class="w-3 h-3" />
                    <span>Protected</span>
                  </span>
                {/if}
              </td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>
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
