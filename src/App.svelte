<script>
  import { onMount } from 'svelte';
  import { system } from './lib/stores/system.svelte.js';

  // Dashboard components
  import HealthGauge from './lib/components/dashboard/HealthGauge.svelte';
  import TopActionsCard from './lib/components/dashboard/TopActionsCard.svelte';
  import QuickStats from './lib/components/dashboard/QuickStats.svelte';

  // Storage components
  import DriveCards from './lib/components/storage/DriveCards.svelte';
  import LargeFilesTable from './lib/components/storage/LargeFilesTable.svelte';
  import DevDietView from './lib/components/storage/DevDietView.svelte';

  // Cleanup components
  import RuleChecklist from './lib/components/cleanup/RuleChecklist.svelte';
  import CleanupPreviewModal from './lib/components/cleanup/CleanupPreviewModal.svelte';
  import RescueBinDrawer from './lib/components/cleanup/RescueBinDrawer.svelte';

  // Process components
  import ProcessTable from './lib/components/processes/ProcessTable.svelte';
  import GeminiExplainModal from './lib/components/processes/GeminiExplainModal.svelte';

  // Apps components
  import InstalledAppsTable from './lib/components/apps/InstalledAppsTable.svelte';
  import StartupRadar from './lib/components/apps/StartupRadar.svelte';

  // Settings
  import SettingsView from './lib/components/settings/SettingsView.svelte';

  // Icons
  import {
    LayoutDashboard,
    HardDrive,
    Sparkles,
    Activity,
    Package,
    Settings,
    ShieldCheck,
    Wind
  } from 'lucide-svelte';

  onMount(() => {
    system.initialize();
  });

  const tabs = [
    { id: 'dashboard', label: 'Dashboard', icon: LayoutDashboard },
    { id: 'storage', label: 'Storage', icon: HardDrive },
    { id: 'cleanup', label: 'Safe Cleanup', icon: Sparkles },
    { id: 'processes', label: 'Process Radar', icon: Activity },
    { id: 'apps', label: 'Apps & Zombies', icon: Package },
    { id: 'settings', label: 'Settings', icon: Settings },
  ];
</script>

<main class="h-screen w-screen flex flex-col bg-slate-950 text-slate-100 font-sans select-none overflow-hidden">
  <!-- Top Navigation Header -->
  <header class="h-14 bg-slate-900/90 border-b border-slate-800/90 px-6 flex items-center justify-between shrink-0 backdrop-blur-md z-30">
    <!-- Brand Logo -->
    <div class="flex items-center gap-2.5">
      <div class="p-1.5 rounded-lg bg-cyan-500/10 border border-cyan-500/30 text-cyan-400">
        <Wind class="w-5 h-5" />
      </div>
      <div>
        <div class="flex items-center gap-2">
          <span class="text-sm font-black tracking-widest text-slate-100 uppercase">Vapor</span>
          <span class="text-[10px] uppercase font-mono px-1.5 py-0.2 bg-slate-800 text-cyan-400 rounded border border-slate-700">ClearDeck</span>
        </div>
      </div>
    </div>

    <!-- Center Tab Navigation -->
    <nav class="flex items-center gap-1 bg-slate-950/60 p-1 rounded-xl border border-slate-800">
      {#each tabs as tab}
        {@const Icon = tab.icon}
        {@const isActive = system.activeTab === tab.id}
        <button
          class="flex items-center gap-2 px-3.5 py-1.5 text-xs font-semibold rounded-lg transition-all {isActive ? 'bg-cyan-950 text-cyan-300 border border-cyan-700/60 shadow-sm' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60'}"
          onclick={() => system.setActiveTab(tab.id)}
        >
          <Icon class="w-3.5 h-3.5 {isActive ? 'text-cyan-400' : 'text-slate-400'}" />
          <span>{tab.label}</span>
        </button>
      {/each}
    </nav>

    <!-- Top Right Quick Actions -->
    <div class="flex items-center gap-3">
      <!-- Rescue Bin Trigger -->
      <button
        class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-slate-300 bg-slate-800/80 hover:bg-slate-700 active:scale-95 border border-slate-700 rounded-lg transition-all"
        onclick={() => system.openRescueBin()}
        title="Open Reversible Rescue Bin"
      >
        <ShieldCheck class="w-3.5 h-3.5 text-emerald-400" />
        <span>Rescue Bin</span>
      </button>

      <!-- Health Pill -->
      {#if system.health}
        <div class="hidden sm:flex items-center gap-1.5 px-2.5 py-1 bg-slate-900 border border-slate-800 rounded-lg text-xs font-mono">
          <span class="text-slate-400">Score:</span>
          <span class="font-bold text-cyan-400">{system.health.overall_score}</span>
        </div>
      {/if}
    </div>
  </header>

  <!-- Main View Area -->
  <section class="flex-1 overflow-y-auto p-6 bg-gradient-to-b from-slate-950 to-slate-900/50">
    <div class="max-w-7xl mx-auto">
      {#if system.activeTab === 'dashboard'}
        <HealthGauge />
        <TopActionsCard />
        <QuickStats />
      {:else if system.activeTab === 'storage'}
        <DriveCards />
        <LargeFilesTable />
        <DevDietView />
      {:else if system.activeTab === 'cleanup'}
        <RuleChecklist />
      {:else if system.activeTab === 'processes'}
        <ProcessTable />
      {:else if system.activeTab === 'apps'}
        <InstalledAppsTable />
        <StartupRadar />
      {:else if system.activeTab === 'settings'}
        <SettingsView />
      {/if}
    </div>
  </section>
</main>

<!-- Global Modals & Drawers -->
<CleanupPreviewModal />
<RescueBinDrawer />
<GeminiExplainModal />
