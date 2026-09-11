<script>
  import { settings } from '../../stores/settings.svelte.js';
  import Badge from '../common/Badge.svelte';
  import {
    Settings,
    Key,
    ShieldCheck,
    HardDrive,
    Lock,
    Info,
    Check,
    Heart,
    Coffee,
    Coins,
    QrCode,
    Copy,
    ExternalLink,
    X
  } from 'lucide-svelte';
  import { openExternalUrl } from '../../api.js';

  let apiKeyInput = $state(settings.geminiApiKey);
  let retentionDaysInput = $state(settings.retentionDays);
  let devDietDaysInput = $state(settings.devDietThresholdDays);
  let largeFileMbInput = $state(settings.largeFileSizeThresholdMb);
  let savedBanner = $state(false);

  // Support Modal State
  let showCryptoModal = $state(false);
  let showPaypalModal = $state(false);
  let copiedAddress = $state(false);

  const USDT_TRC20_ADDRESS = 'TBQv6e9SixpNDqympDGX3LMRdmKdEj3BHm';

  function handleSave() {
    settings.setGeminiKey(apiKeyInput);
    settings.setRetentionDays(retentionDaysInput);
    settings.setDevDietThreshold(devDietDaysInput);
    settings.setLargeFileSizeThreshold(largeFileMbInput);

    savedBanner = true;
    setTimeout(() => (savedBanner = false), 3000);
  }

  async function copyToClipboard(text) {
    try {
      await navigator.clipboard.writeText(text);
      copiedAddress = true;
      setTimeout(() => (copiedAddress = false), 2500);
    } catch (e) {
      console.error('Failed to copy address:', e);
    }
  }

  function handleOpenLink(url) {
    openExternalUrl(url).catch(() => window.open(url, '_blank'));
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

  <!-- Support the Project & Development -->
  <div class="bg-gradient-to-r from-slate-900/90 via-slate-900/80 to-purple-950/30 border border-slate-800/90 rounded-2xl p-6 shadow-xl space-y-4">
    <div class="flex items-start justify-between gap-4">
      <div>
        <div class="flex items-center gap-2">
          <Heart class="w-5 h-5 text-rose-400 fill-rose-500/20" />
          <h3 class="text-sm font-bold text-slate-100">Support Vapor Development</h3>
          <Badge variant="accent">100% Free & Open Source</Badge>
        </div>
        <p class="text-xs text-slate-300 mt-1 leading-relaxed">
          Vapor has zero telemetry, no commercial popups, and no paywalls. If this tool helped speed up your PC or freed gigabytes of space, consider supporting ongoing independent development!
        </p>
      </div>
    </div>

    <!-- Donation / Support Actions -->
    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 pt-2">
      <!-- Ko-fi / PayPal Button -->
      <button
        onclick={() => handleOpenLink('https://ko-fi.com/salahalioui')}
        class="flex items-center justify-between p-3.5 bg-slate-950/70 hover:bg-slate-800/80 active:scale-[0.98] border border-slate-700/80 hover:border-cyan-500/50 rounded-xl transition-all group text-left cursor-pointer"
      >
        <div class="flex items-center gap-3">
          <div class="p-2 rounded-lg bg-amber-500/10 border border-amber-500/30 text-amber-400">
            <Coffee class="w-4 h-4" />
          </div>
          <div>
            <span class="text-xs font-bold text-slate-200 block group-hover:text-cyan-300 transition-colors">
              Support on Ko-fi / Card
            </span>
            <span class="text-[11px] text-slate-400 block">Donate or buy a coffee ($3 / $5 / $10)</span>
          </div>
        </div>
        <ExternalLink class="w-4 h-4 text-slate-500 group-hover:text-cyan-400 transition-colors" />
      </button>

      <!-- Bybit USDT Crypto Button -->
      <button
        onclick={() => (showCryptoModal = true)}
        class="flex items-center justify-between p-3.5 bg-slate-950/70 hover:bg-slate-800/80 active:scale-[0.98] border border-slate-700/80 hover:border-emerald-500/50 rounded-xl transition-all group text-left cursor-pointer"
      >
        <div class="flex items-center gap-3">
          <div class="p-2 rounded-lg bg-emerald-500/10 border border-emerald-500/30 text-emerald-400">
            <Coins class="w-4 h-4" />
          </div>
          <div>
            <span class="text-xs font-bold text-slate-200 block group-hover:text-emerald-300 transition-colors">
              Pay with Crypto (USDT)
            </span>
            <span class="text-[11px] text-slate-400 block">TRON (TRC20) via Bybit Wallet</span>
          </div>
        </div>
        <QrCode class="w-4 h-4 text-slate-500 group-hover:text-emerald-400 transition-colors" />
      </button>
    </div>

    <div class="flex items-center justify-between pt-2 border-t border-slate-800/80 text-[11px] text-slate-400">
      <span>Prefer direct PayPal?</span>
      <button
        onclick={() => (showPaypalModal = true)}
        class="text-cyan-400 hover:text-cyan-300 hover:underline font-medium inline-flex items-center gap-1 cursor-pointer"
      >
        <span>View Direct PayPal QR Code</span>
        <QrCode class="w-3 h-3" />
      </button>
    </div>
  </div>

  <!-- About Box -->
  <div class="bg-slate-950/40 border border-slate-800/80 rounded-xl p-4 flex items-start gap-3 text-xs text-slate-400">
    <Info class="w-4 h-4 text-cyan-400 shrink-0 mt-0.5" />
    <div>
      <strong class="text-slate-200">Vapor v0.3.0</strong>
      <p class="mt-1 text-slate-400 leading-relaxed">
        High-performance native Windows triage utility built with Tauri 2.0 (Rust engine) and Svelte 5 (Runes). Zero background resident daemons, sub-30MB memory footprint, parallel directory walking with Rayon & Jwalk.
      </p>
    </div>
  </div>
</div>

<!-- Bybit USDT (TRC-20) Modal -->
{#if showCryptoModal}
  <div
    class="fixed inset-0 z-50 bg-slate-950/80 backdrop-blur-sm flex items-center justify-center p-4"
    role="dialog"
    aria-modal="true"
  >
    <div class="bg-slate-900 border border-slate-800 rounded-3xl max-w-md w-full p-6 shadow-2xl space-y-4 animate-in fade-in zoom-in-95 duration-150">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="p-2 rounded-xl bg-emerald-500/10 border border-emerald-500/30 text-emerald-400">
            <Coins class="w-5 h-5" />
          </div>
          <div>
            <h3 class="text-base font-bold text-slate-100">USDT Support (Bybit)</h3>
            <p class="text-[11px] text-slate-400">Network: TRON (TRC20)</p>
          </div>
        </div>
        <button
          onclick={() => (showCryptoModal = false)}
          class="p-1.5 text-slate-400 hover:text-slate-200 hover:bg-slate-800 rounded-xl transition-all cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- QR Code Display -->
      <div class="bg-white p-3 rounded-2xl flex flex-col items-center justify-center max-w-[220px] mx-auto shadow-inner">
        <img
          src="/bybit_usdt_trc20.jpg"
          alt="Bybit USDT TRC20 QR Code"
          class="w-48 h-48 object-contain rounded-lg"
        />
      </div>

      <!-- Address Box & Copy Button -->
      <div class="space-y-1.5">
        <span class="text-[11px] text-slate-400 font-semibold block">Wallet Address (TRC-20):</span>
        <div class="flex items-center gap-2 bg-slate-950 p-2.5 rounded-xl border border-slate-800">
          <span class="text-xs font-mono text-emerald-400 break-all select-all flex-1">
            {USDT_TRC20_ADDRESS}
          </span>
          <button
            onclick={() => copyToClipboard(USDT_TRC20_ADDRESS)}
            class="shrink-0 flex items-center gap-1 px-3 py-1.5 bg-emerald-600 hover:bg-emerald-500 active:scale-95 text-slate-950 text-xs font-bold rounded-lg transition-all cursor-pointer"
          >
            {#if copiedAddress}
              <Check class="w-3.5 h-3.5" />
              <span>Copied!</span>
            {:else}
              <Copy class="w-3.5 h-3.5" />
              <span>Copy</span>
            {/if}
          </button>
        </div>
      </div>

      <div class="text-[11px] text-slate-400 space-y-1 bg-slate-950/40 p-3 rounded-xl border border-slate-800/80">
        <p>&bull; <strong>Minimum Deposit:</strong> 0.005 USDT</p>
        <p>&bull; <strong>Accepted Network:</strong> TRON (TRC20) only</p>
        <p>&bull; <strong>Destination:</strong> Bybit Exchange Wallet</p>
      </div>
    </div>
  </div>
{/if}

<!-- PayPal QR Modal -->
{#if showPaypalModal}
  <div
    class="fixed inset-0 z-50 bg-slate-950/80 backdrop-blur-sm flex items-center justify-center p-4"
    role="dialog"
    aria-modal="true"
  >
    <div class="bg-slate-900 border border-slate-800 rounded-3xl max-w-sm w-full p-6 shadow-2xl space-y-4 animate-in fade-in zoom-in-95 duration-150">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div class="p-2 rounded-xl bg-cyan-500/10 border border-cyan-500/30 text-cyan-400">
            <QrCode class="w-5 h-5" />
          </div>
          <div>
            <h3 class="text-base font-bold text-slate-100">Direct PayPal</h3>
            <p class="text-[11px] text-slate-400">Salah Alioui</p>
          </div>
        </div>
        <button
          onclick={() => (showPaypalModal = false)}
          class="p-1.5 text-slate-400 hover:text-slate-200 hover:bg-slate-800 rounded-xl transition-all cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <div class="bg-white p-3 rounded-2xl flex flex-col items-center justify-center max-w-[240px] mx-auto shadow-inner">
        <img
          src="/paypal_qr.jpg"
          alt="PayPal Salah Alioui QR Code"
          class="w-52 h-auto object-contain rounded-lg"
        />
      </div>

      <p class="text-center text-xs text-slate-300">
        Scan with your PayPal app or camera to send support directly.
      </p>

      <div class="pt-2 text-center">
        <button
          onclick={() => handleOpenLink('https://paypal.me/salahalioui')}
          class="text-xs text-cyan-400 hover:underline font-semibold inline-flex items-center gap-1 cursor-pointer"
        >
          <span>Open paypal.me/salahalioui</span>
          <ExternalLink class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  </div>
{/if}
