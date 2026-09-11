<script>
  import { settings } from '../../stores/settings.svelte.js';
  import Badge from '../common/Badge.svelte';
  import { Settings, Key, ShieldCheck, HardDrive, Lock, Info, Check } from 'lucide-svelte';

  let apiKeyInput = $state(settings.geminiApiKey);
  let retentionDaysInput = $state(settings.retentionDays);
  let devDietDaysInput = $state(settings.devDietThresholdDays);
  let largeFileMbInput = $state(settings.largeFileSizeThresholdMb);
  let savedBanner = $state(false);

  function handleSave() {
    settings.setGeminiKey(apiKeyInput);
    settings.setRetentionDays(retentionDaysInput);
    settings.setDevDietThreshold(devDietDaysInput);
    settings.setLargeFileSizeThreshold(largeFileMbInput);

    savedBanner = true;
    setTimeout(() => (savedBanner = false), 3000);
  }
</script>

<div class="max-w-3xl mx-auto space-y-6">
  <div class="bg-slate-900/70 border border-slate-800 rounded-xl p-5 shadow-lg">
    <div class="flex items-center gap-2 mb-1">
      <Settings class="w-5 h-5 text-cyan-400" />
      <h2 class="text-lg font-bold text-slate-100">Preferences & Configuration</h2>
    </div>
    <p class="text-xs text-slate-400">
      Configure your Gemini API credentials, Rescue Bin safe staging duration, and scan thresholds.
    </p>
  </div>

  {#if savedBanner}
    <div class="p-3.5 bg-emerald-950/80 border border-emerald-800 text-emerald-300 text-xs rounded-xl flex items-center gap-2 shadow-lg animate-in fade-in duration-200">
      <Check class="w-4 h-4" />
      <span class="font-semibold">Preferences successfully saved!</span>
    </div>
  {/if}

  <!-- BYOK Gemini API Key -->
  <div class="bg-slate-900/70 border border-slate-800 rounded-xl p-5 shadow-lg space-y-4">
    <div class="flex items-start justify-between">
      <div>
        <div class="flex items-center gap-2">
          <Key class="w-4 h-4 text-cyan-400" />
          <h3 class="text-sm font-bold text-slate-100">Google Gemini API Key (BYOK)</h3>
          <Badge variant="accent">Gemini 3.8 Flash + Search</Badge>
        </div>
        <p class="text-xs text-slate-400 mt-1">
          Used exclusively on-demand when you click "Ask AI" to explain unfamiliar Windows processes.
        </p>
      </div>
    </div>

    <!-- Privacy Guarantee Callout -->
    <div class="p-3.5 bg-cyan-950/20 border border-cyan-800/40 rounded-lg text-xs text-cyan-200 flex items-start gap-2.5">
      <Lock class="w-4 h-4 text-cyan-400 shrink-0 mt-0.5" />
      <div>
        <strong class="font-semibold">Zero Telemetry & Strict Path Sanitization:</strong>
        Vapor never sends telemetry. Outbound requests to Gemini are scrubbed of usernames, personal user directories, computer names, and IP addresses. Only sanitized executable names and categorized path types are transmitted.
      </div>
    </div>

    <div>
      <label class="block text-xs font-semibold text-slate-300 mb-1.5" for="apiKey">
        API Key
      </label>
      <input
        id="apiKey"
        type="password"
        placeholder="AIzaSy..."
        bind:value={apiKeyInput}
        class="w-full bg-slate-950/80 text-xs text-slate-100 font-mono px-3 py-2 rounded-lg border border-slate-700 focus:outline-none focus:border-cyan-500 placeholder:text-slate-600"
      />
      <p class="text-[11px] text-slate-500 mt-1">
        Stored securely in local application storage. Obtain a free key at <a href="https://aistudio.google.com" target="_blank" rel="noreferrer" class="text-cyan-400 hover:underline">aistudio.google.com</a>.
      </p>
    </div>
  </div>

  <!-- Rescue Bin & Retention Preferences -->
  <div class="bg-slate-900/70 border border-slate-800 rounded-xl p-5 shadow-lg space-y-5">
    <div class="flex items-center gap-2">
      <ShieldCheck class="w-4 h-4 text-emerald-400" />
      <h3 class="text-sm font-bold text-slate-100">Safety & Rescue Bin Settings</h3>
    </div>

    <!-- Retention slider -->
    <div class="space-y-2">
      <div class="flex justify-between items-center text-xs">
        <label for="retention-slider" class="text-slate-300 font-medium">Rescue Bin Retention Period</label>
        <span class="font-mono text-cyan-400 font-bold">{retentionDaysInput} days</span>
      </div>
      <input
        id="retention-slider"
        type="range"
        min="3"
        max="60"
        step="1"
        bind:value={retentionDaysInput}
        class="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-cyan-500"
      />
      <p class="text-[11px] text-slate-500">
        Staged files older than this will be automatically pruned during maintenance.
      </p>
    </div>

    <!-- Dev Diet dormancy threshold -->
    <div class="space-y-2 pt-3 border-t border-slate-800/80">
      <div class="flex justify-between items-center text-xs">
        <label for="dormancy-slider" class="text-slate-300 font-medium">Developer Diet Dormancy Threshold</label>
        <span class="font-mono text-emerald-400 font-bold">{devDietDaysInput} days</span>
      </div>
      <input
        id="dormancy-slider"
        type="range"
        min="7"
        max="180"
        step="7"
        bind:value={devDietDaysInput}
        class="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-emerald-500"
      />
      <p class="text-[11px] text-slate-500">
        Build targets and node_modules folders untouched for this many days will be flagged for review.
      </p>
    </div>

    <!-- Large file threshold -->
    <div class="space-y-2 pt-3 border-t border-slate-800/80">
      <div class="flex justify-between items-center text-xs">
        <label for="filesize-slider" class="text-slate-300 font-medium">Large File Minimum Size</label>
        <span class="font-mono text-amber-400 font-bold">{largeFileMbInput} MB</span>
      </div>
      <input
        id="filesize-slider"
        type="range"
        min="25"
        max="1024"
        step="25"
        bind:value={largeFileMbInput}
        class="w-full h-1.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-amber-500"
      />
    </div>
  </div>

  <!-- Save Button -->
  <div class="flex justify-end">
    <button
      class="px-6 py-2.5 bg-cyan-600 hover:bg-cyan-500 active:scale-95 text-white text-xs font-bold rounded-lg shadow-lg shadow-cyan-950/40 transition-all"
      onclick={handleSave}
    >
      Save Preferences
    </button>
  </div>

  <!-- About Box -->
  <div class="bg-slate-950/40 border border-slate-800/80 rounded-xl p-4 flex items-start gap-3 text-xs text-slate-400">
    <Info class="w-4 h-4 text-cyan-400 shrink-0 mt-0.5" />
    <div>
      <strong class="text-slate-200">Vapor (ClearDeck Engine) v0.1.0</strong>
      <p class="mt-1 text-slate-400 leading-relaxed">
        High-performance native Windows triage utility built with Tauri 2.0 (Rust engine) and Svelte 5 (Runes). Zero background resident daemons, sub-40MB memory footprint, parallel directory walking with Rayon & Jwalk.
      </p>
    </div>
  </div>
</div>
