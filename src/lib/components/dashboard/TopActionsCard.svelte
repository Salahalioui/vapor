<script>
  import { system } from '../../stores/system.svelte.js';
  import Badge from '../common/Badge.svelte';
  import { Zap, HardDrive, Cpu, ArrowRight } from 'lucide-svelte';

  const defaultActions = [
    {
      action_type: 'fastest',
      title: 'Fastest Win: Safe Cache Cleanup',
      description: 'Instant 1-click purge of temporary files, crash dumps, and caches. Reversible in Rescue Bin.',
      impact: 'Up to 3.5 GB Reclaimable',
      badge: 'Zero Risk',
      target_tab: 'cleanup'
    },
    {
      action_type: 'biggest',
      title: 'Biggest Win: Clean Dormant Dev Caches',
      description: 'Prune dormant node_modules, Rust targets, and Python venvs unused for over 30 days.',
      impact: '12.4 GB Potential',
      badge: 'High Impact',
      target_tab: 'storage'
    },
    {
      action_type: 'smartest',
      title: 'Smartest Win: Startup & Zombie Optimization',
      description: 'Disable autostart apps that run on boot to accelerate boot time and reclaim active RAM.',
      impact: '4 Apps Reviewed',
      badge: 'Performance',
      target_tab: 'apps'
    }
  ];

  const actions = $derived(system.health?.top_actions?.length ? system.health.top_actions : defaultActions);

  function getActionIcon(type) {
    switch (type) {
      case 'fastest': return Zap;
      case 'biggest': return HardDrive;
      case 'smartest': return Cpu;
      default: return Zap;
    }
  }

  function getBadgeVariant(type) {
    switch (type) {
      case 'fastest': return 'success';
      case 'biggest': return 'accent';
      case 'smartest': return 'warning';
      default: return 'default';
    }
  }

  function handleNavigate(targetTab) {
    system.setActiveTab(targetTab);
  }
</script>

<div class="mt-6">
  <div class="flex items-center justify-between mb-3.5">
    <div>
      <h3 class="text-base font-semibold text-slate-100 flex items-center gap-2">
        <Zap class="w-4 h-4 text-cyan-400" />
        Top 3 Recommended Actions
      </h3>
      <p class="text-xs text-slate-400">
        Decision engine: curated fastest wins to boost responsiveness and reclaim drive capacity.
      </p>
    </div>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
    {#each actions as action}
      {@const IconComponent = getActionIcon(action.action_type)}
      <div
        class="bg-slate-900/70 hover:bg-slate-900 border border-slate-800 hover:border-slate-700/80 rounded-xl p-4 transition-all duration-200 flex flex-col justify-between group shadow-md"
      >
        <div>
          <div class="flex items-center justify-between mb-2.5">
            <div class="p-2 rounded-lg bg-slate-800/80 text-cyan-400 group-hover:bg-cyan-950/60 group-hover:text-cyan-300 transition-colors">
              <IconComponent class="w-4 h-4" />
            </div>
            <Badge variant={getBadgeVariant(action.action_type)}>
              {action.badge}
            </Badge>
          </div>

          <h4 class="text-sm font-semibold text-slate-100 mb-1 line-clamp-1">
            {action.title}
          </h4>
          <p class="text-xs text-slate-400 line-clamp-2 leading-relaxed mb-3">
            {action.description}
          </p>
        </div>

        <div class="pt-3 border-t border-slate-800/80 flex items-center justify-between mt-auto">
          <span class="text-xs font-mono font-medium {action.impact?.includes('Pending') ? 'text-amber-400' : 'text-emerald-400'}">
            {action.impact}
          </span>
          <button
            class="flex items-center gap-1.5 text-xs font-medium text-cyan-400 hover:text-cyan-300 group/btn"
            onclick={() => handleNavigate(action.target_tab)}
          >
            <span>{action.impact?.includes('Pending') ? 'Scan' : 'Resolve'}</span>
            <ArrowRight class="w-3.5 h-3.5 group-hover/btn:translate-x-0.5 transition-transform" />
          </button>
        </div>
      </div>
    {/each}
  </div>
</div>
