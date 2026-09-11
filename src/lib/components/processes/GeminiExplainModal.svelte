<script>
  import { system } from '../../stores/system.svelte.js';
  import { settings } from '../../stores/settings.svelte.js';
  import { explainProcess, killProcess } from '../../api.js';
  import Modal from '../common/Modal.svelte';
  import Badge from '../common/Badge.svelte';
  import ConfirmModal from '../common/ConfirmModal.svelte';
  import {
    Sparkles,
    Shield,
    Cpu,
    MemoryStick,
    AlertTriangle,
    CheckCircle2,
    Lock,
    Key,
    BookOpen,
    RefreshCw,
    Folder
  } from 'lucide-svelte';

  let explanation = $state(null);
  let isLoading = $state(false);
  let errorMsg = $state('');
  let killSuccessMsg = $state('');
  let isFromCache = $state(false);
  let inlineApiKey = $state('');

  let confirmState = $state({
    isOpen: false,
    title: '',
    message: '',
    confirmText: '',
    isDanger: false,
    action: () => {}
  });

  const proc = $derived(system.activeExplainProcess);
  const isOpen = $derived(proc !== null);

  async function requestAiExplanation(forceRefresh = false) {
    if (!proc) return;
    
    if (!forceRefresh) {
      const cached = system.getProcessExplanationFromCache(proc.name);
      if (cached) {
        explanation = cached;
        isFromCache = true;
        return;
      }
    }

    isLoading = true;
    errorMsg = '';
    isFromCache = false;

    try {
      // explainProcess checks persistent disk cache first (succeeds even if key is empty when cached)
      const res = await explainProcess(
        settings.geminiApiKey || '',
        proc.name,
        proc.exe_path,
        proc.publisher,
        proc.description,
        proc.cpu_percent,
        proc.memory_mb
      );
      explanation = res;
      isFromCache = true;
      system.cacheProcessExplanation(proc.name, res);
    } catch (e) {
      if (!settings.geminiApiKey) {
        // Not in local cache and no API key entered yet; prompt for key without red error
        explanation = null;
      } else {
        errorMsg = `${e}`;
      }
    } finally {
      isLoading = false;
    }
  }

  async function handleSaveKeyAndAnalyze() {
    if (!inlineApiKey.trim()) {
      errorMsg = 'Please enter a valid Gemini API key.';
      return;
    }
    settings.setGeminiKey(inlineApiKey.trim());
    errorMsg = '';
    await requestAiExplanation(true);
  }

  function handleKill() {
    if (!proc) return;
    confirmState = {
      isOpen: true,
      title: `Terminate "${proc.name}"?`,
      message: `Are you sure you want to terminate process "${proc.name}" (PID ${proc.pid})?\n\nCPU: ${proc.cpu_percent.toFixed(1)}% | RAM: ${proc.memory_mb.toFixed(0)} MB\n\nUnsaved progress in this process will be lost.`,
      confirmText: 'Terminate Process',
      isDanger: true,
      action: async () => {
        try {
          await killProcess(proc.pid);
          killSuccessMsg = `Process ${proc.name} (PID ${proc.pid}) has been terminated.`;
          system.refreshHardware();
          setTimeout(() => {
            system.closeExplainProcess();
          }, 1400);
        } catch (e) {
          confirmState = {
            isOpen: true,
            title: 'Kill Failed',
            message: `Failed to terminate process: ${e}`,
            confirmText: 'OK',
            isDanger: false,
            action: () => {}
          };
        }
      }
    };
  }

  $effect(() => {
    if (proc) {
      killSuccessMsg = '';
      errorMsg = '';
      inlineApiKey = settings.geminiApiKey || '';
      const cached = system.getProcessExplanationFromCache(proc.name);
      if (cached) {
        explanation = cached;
        isFromCache = true;
        isLoading = false;
      } else {
        explanation = null;
        isFromCache = false;
        // Check persistent local disk cache silently
        explainProcess(
          settings.geminiApiKey || '',
          proc.name,
          proc.exe_path,
          proc.publisher,
          proc.description,
          proc.cpu_percent,
          proc.memory_mb
        ).then((res) => {
          if (res && proc && res.sanitized_query?.process_name?.toLowerCase() === proc.name.toLowerCase()) {
            explanation = res;
            isFromCache = true;
            system.cacheProcessExplanation(proc.name, res);
          }
        }).catch(() => {
          // If not in cache and no key or network error, silently ignore until user explicitly asks
        });
      }
    }
  });

  function getSafetyBadgeVariant(safety) {
    switch (safety?.toLowerCase()) {
      case 'safe': return 'success';
      case 'bloatware': return 'danger';
      case 'caution': return 'warning';
      case 'critical': return 'purple';
      default: return 'default';
    }
  }
</script>

<Modal
  {isOpen}
  title={`Process Diagnostics: ${proc?.name || 'Process'}`}
  onClose={() => system.closeExplainProcess()}
  maxWidth="max-w-2xl"
>
  {#if proc}
    <!-- Top Process Stats Header -->
    <div class="bg-slate-950/60 border border-slate-800 rounded-xl p-4 mb-4 flex items-center justify-between">
      <div>
        <div class="flex items-center gap-2">
          <h4 class="text-base font-bold font-mono text-slate-100">{proc.name}</h4>
          <span class="text-xs text-slate-400 font-mono">PID {proc.pid}</span>
          {#if proc.is_known}
            <span class="text-[10px] text-cyan-400 px-1.5 py-0.5 bg-cyan-950/60 rounded border border-cyan-800/50">
              Verified Windows Task
            </span>
          {/if}
        </div>
        <p class="text-xs text-slate-400 mt-0.5 truncate max-w-[380px]">
          {proc.exe_path || 'Location: System Image'}
        </p>
      </div>

      <div class="flex items-center gap-3 text-right">
        <div>
          <div class="text-xs text-slate-400">CPU</div>
          <div class="text-sm font-bold font-mono text-cyan-400">{proc.cpu_percent.toFixed(1)}%</div>
        </div>
        <div>
          <div class="text-xs text-slate-400">RAM</div>
          <div class="text-sm font-bold font-mono text-emerald-400">{proc.memory_mb.toFixed(0)} MB</div>
        </div>
      </div>
    </div>

    {#if killSuccessMsg}
      <div class="mb-4 p-3 bg-emerald-950/80 border border-emerald-800 text-emerald-300 text-xs rounded-lg flex items-center gap-2">
        <CheckCircle2 class="w-4 h-4" />
        <span>{killSuccessMsg}</span>
      </div>
    {/if}

    <!-- 1. Immediate Offline Dictionary Summary -->
    <div class="mb-4 p-3.5 bg-slate-900/60 border border-slate-800 rounded-xl space-y-2">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-1.5 text-xs font-semibold text-slate-300">
          <BookOpen class="w-3.5 h-3.5 text-cyan-400" />
          <span>Offline Knowledge Base Summary</span>
        </div>
        <Badge variant={proc.category === 'system' ? 'purple' : 'accent'}>
          {proc.category.toUpperCase()}
        </Badge>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-3 gap-2 text-xs text-slate-300 pt-1">
        <div>
          <span class="text-slate-500">Publisher:</span>
          <span class="font-medium text-slate-200 ml-1 truncate block" title={proc.publisher || 'Unknown'}>{proc.publisher || 'Unknown / Third-party'}</span>
        </div>
        <div>
          <span class="text-slate-500">Classification:</span>
          <span class="font-medium text-slate-200 ml-1 capitalize block">{proc.category}</span>
        </div>
        <div>
          <span class="text-slate-500">Known Safety:</span>
          <span class="font-semibold ml-1 capitalize block {proc.safety === 'safe' ? 'text-emerald-400' : proc.safety === 'critical' ? 'text-purple-400' : proc.safety === 'bloatware' ? 'text-rose-400' : 'text-amber-400'}">
            {proc.safety || 'Unrated'}
          </span>
        </div>
      </div>

      {#if proc.description}
        <p class="text-xs text-slate-300 bg-slate-950/40 p-2.5 rounded-lg border border-slate-800/80 leading-relaxed">
          {proc.description}
        </p>
      {/if}
    </div>

    <!-- 2. Privacy Guardrail Banner -->
    <div class="mb-4 px-3 py-2 bg-slate-950/40 border border-slate-800/80 rounded-lg flex items-center justify-between text-[11px] text-slate-400">
      <div class="flex items-center gap-2">
        <Lock class="w-3.5 h-3.5 text-cyan-400 shrink-0" />
        <span>Privacy Guard: Usernames, hostnames, and private directory paths are sanitized prior to AI analysis.</span>
      </div>
    </div>

    <!-- 3. Gemini AI Section -->
    <div class="mb-4">
      {#if isLoading}
        <div class="py-8 flex flex-col items-center justify-center gap-2 text-slate-400 bg-slate-950/30 rounded-xl border border-slate-800">
          <RefreshCw class="w-6 h-6 text-purple-400 animate-spin" />
          <span class="text-xs font-medium text-purple-300">Consulting Gemini with Google Search for deep process insights...</span>
        </div>
      {:else if !settings.geminiApiKey}
        <!-- Inline API Key Entry -->
        <div class="p-4 bg-purple-950/20 border border-purple-800/50 rounded-xl space-y-3">
          <div class="flex items-center gap-2">
            <Key class="w-4 h-4 text-purple-400" />
            <h5 class="text-xs font-bold text-slate-200 uppercase tracking-wider">Unlock Gemini Deep AI Analysis</h5>
          </div>
          <p class="text-xs text-slate-400">
            Enter your Google Gemini API key below to get real-time security diagnostics, resource utilization breakdown, and termination recommendations.
          </p>
          <div class="flex items-center gap-2">
            <input
              type="password"
              placeholder="Paste Gemini API key (AIzaSy...)"
              bind:value={inlineApiKey}
              class="flex-1 bg-slate-900 text-xs text-slate-200 px-3 py-2 rounded-lg border border-slate-700 focus:outline-none focus:border-purple-500 font-mono"
            />
            <button
              class="px-4 py-2 bg-purple-600 hover:bg-purple-500 active:scale-95 text-white text-xs font-semibold rounded-lg shadow-sm transition-all whitespace-nowrap"
              onclick={handleSaveKeyAndAnalyze}
            >
              Save & Analyze
            </button>
          </div>
          {#if errorMsg}
            <p class="text-xs text-rose-400">{errorMsg}</p>
          {/if}
        </div>
      {:else if errorMsg}
        <!-- Error Banner -->
        <div class="p-4 bg-rose-950/50 border border-rose-800/80 rounded-xl text-xs text-rose-300 space-y-2">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2 font-semibold">
              <AlertTriangle class="w-4 h-4 text-rose-400" />
              <span>AI Analysis Error</span>
            </div>
            <button
              class="px-2.5 py-1 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded text-xs transition-colors"
              onclick={() => requestAiExplanation(true)}
            >
              Retry
            </button>
          </div>
          <p class="leading-relaxed">{errorMsg}</p>
        </div>
      {:else if explanation}
        <!-- Detailed AI Output -->
        <div class="space-y-3">
          <div class="p-3.5 bg-slate-950/50 border border-slate-800 rounded-xl space-y-2">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <Sparkles class="w-3.5 h-3.5 text-purple-400" />
                <span class="text-xs font-semibold text-slate-300">Gemini AI Assessment</span>
                {#if isFromCache}
                  <span class="text-[10px] text-purple-300 bg-purple-950/80 px-1.5 py-0.2 rounded border border-purple-800/60 font-mono">
                    Pre-cached by Fleet Audit
                  </span>
                {/if}
              </div>
              <div class="flex items-center gap-2">
                <Badge variant={getSafetyBadgeVariant(explanation.safety)}>
                  {explanation.safety.toUpperCase()}
                </Badge>
                {#if explanation.can_terminate}
                  <Badge variant="success">Safe to Terminate</Badge>
                {:else}
                  <Badge variant="danger">System Critical</Badge>
                {/if}
              </div>
            </div>

            <p class="text-xs text-slate-200 leading-relaxed font-medium">
              {explanation.summary}
            </p>
          </div>

          <!-- Why High Usage -->
          {#if explanation.why_high_usage}
            <div class="p-3 bg-slate-950/50 border border-slate-800 rounded-xl space-y-1">
              <span class="text-[11px] font-semibold text-slate-400 uppercase tracking-wider">Resource Utilization</span>
              <p class="text-xs text-slate-300 leading-relaxed">
                {explanation.why_high_usage}
              </p>
            </div>
          {/if}

          <!-- Recommended Action -->
          <div class="p-3 bg-cyan-950/20 border border-cyan-800/40 rounded-xl space-y-1">
            <div class="flex items-center justify-between">
              <span class="text-[11px] font-semibold text-cyan-400 uppercase tracking-wider">Recommended Action</span>
              <button
                class="text-[11px] text-slate-400 hover:text-cyan-300 underline flex items-center gap-1"
                onclick={() => requestAiExplanation(true)}
                title="Force refresh analysis with Gemini"
              >
                <RefreshCw class="w-2.5 h-2.5" />
                <span>Re-analyze</span>
              </button>
            </div>
            <p class="text-xs text-cyan-200 leading-relaxed">
              {explanation.recommendation}
            </p>
          </div>
        </div>
      {:else}
        <!-- Ready to Analyze -->
        <div class="p-4 bg-slate-950/40 border border-slate-800 rounded-xl flex items-center justify-between">
          <div>
            <div class="text-xs font-semibold text-slate-200">Cloud Intelligence Available</div>
            <div class="text-[11px] text-slate-400">Perform deep cloud reasoning with Gemini Flash & Google Search grounding for real-time telemetry analysis and advice.</div>
          </div>
          <button
            class="px-3.5 py-2 bg-purple-600 hover:bg-purple-500 active:scale-95 text-white text-xs font-semibold rounded-lg transition-all flex items-center gap-1.5 shadow-sm shadow-purple-950/40 shrink-0 ml-3"
            onclick={() => requestAiExplanation(true)}
          >
            <Sparkles class="w-3.5 h-3.5" />
            <span>Deep Dive with Gemini</span>
          </button>
        </div>
      {/if}
    </div>

    <!-- Bottom Actions Footer -->
    <div class="pt-3 border-t border-slate-800 flex items-center justify-between">
      <button
        class="px-4 py-2 text-xs text-slate-400 hover:text-slate-200 transition-colors"
        onclick={() => system.closeExplainProcess()}
      >
        Close
      </button>

      {#if proc.can_kill}
        <button
          class="px-4 py-2 bg-rose-600 hover:bg-rose-500 active:scale-95 text-white text-xs font-semibold rounded-lg shadow-lg shadow-rose-950/40 transition-all"
          onclick={handleKill}
        >
          Terminate Process
        </button>
      {/if}
    </div>
  {/if}
</Modal>

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
