<script>
  import { system } from '../../stores/system.svelte.js';
  import { formatBytes } from '../../utils.js';
  import ProgressBar from '../common/ProgressBar.svelte';
  import Badge from '../common/Badge.svelte';
  import { HardDrive, RefreshCw, CheckCircle2 } from 'lucide-svelte';

  const disks = $derived(system.hardware?.disks ?? []);
  const selectedRoot = $derived(system.selectedDriveRoot);
  const isScanning = $derived(system.isScanningLargeFiles || system.isScanningDevDiet);

  function handleSelectDrive(mountPoint) {
    system.scanStorageForDrive(mountPoint);
  }
</script>

<div class="mb-6">
  <div class="flex items-center justify-between mb-2 px-1">
    <span class="text-xs text-slate-400 font-medium">
      Mounted Storage Volumes {selectedRoot ? `• Targeted: ${selectedRoot}` : '• All Drives'}
    </span>
    {#if selectedRoot}
      <button
        class="text-xs text-cyan-400 hover:text-cyan-300 font-medium transition-colors"
        onclick={() => system.scanStorageForDrive(null)}
      >
        Reset to All Drives
      </button>
    {/if}
  </div>

  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
    {#if disks.length === 0}
      <div class="col-span-full p-6 text-center text-slate-500 bg-slate-900/40 rounded-xl border border-slate-800">
        Scanning physical volumes...
      </div>
    {:else}
      {#each disks as disk}
        {@const usedBytes = disk.total_bytes - disk.available_bytes}
        {@const usedPercent = 100 - disk.free_percent}
        {@const isTargeted = selectedRoot === disk.mount_point}
        {@const isCurrentScanning = isScanning && isTargeted}

        <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
        <div
          class="cursor-pointer transition-all duration-200 rounded-xl p-4 shadow-md flex flex-col justify-between border select-none {isTargeted ? 'bg-slate-900 border-cyan-500/80 ring-2 ring-cyan-500/30 shadow-cyan-950/40' : 'bg-slate-900/70 border-slate-800 hover:border-slate-700/80 hover:bg-slate-900'}"
          onclick={() => handleSelectDrive(disk.mount_point)}
          role="button"
          tabindex="0"
          title={`Click to audit large files and dev caches on ${disk.mount_point}`}
        >
          <div>
            <div class="flex items-center justify-between mb-3">
              <div class="flex items-center gap-2.5">
                <div class="p-2 rounded-lg {isTargeted ? 'bg-cyan-950 text-cyan-400 border border-cyan-800/60' : 'bg-slate-800 text-emerald-400'}">
                  {#if isCurrentScanning}
                    <RefreshCw class="w-4 h-4 animate-spin text-cyan-400" />
                  {:else}
                    <HardDrive class="w-4 h-4" />
                  {/if}
                </div>
                <div>
                  <div class="flex items-center gap-1.5">
                    <span class="text-sm font-bold text-slate-100">{disk.name || disk.mount_point}</span>
                    {#if isTargeted}
                      <Badge variant="accent" class="text-[10px] py-0 px-1.5">Target</Badge>
                    {/if}
                  </div>
                  <span class="text-[11px] text-slate-500 font-mono uppercase">{disk.file_system} • {disk.mount_point}</span>
                </div>
              </div>
              <span class="text-xs font-mono font-medium text-slate-300">
                {formatBytes(disk.available_bytes)} free
              </span>
            </div>

            <!-- Standardized disk health color bands: <10% rose, <20% amber, >=20% emerald -->
            <ProgressBar
              value={usedPercent}
              color={disk.free_percent < 10 ? 'rose' : disk.free_percent < 20 ? 'amber' : 'emerald'}
              size="md"
              label={`${formatBytes(usedBytes)} used of ${formatBytes(disk.total_bytes)}`}
              sublabel={`${usedPercent.toFixed(0)}%`}
            />
          </div>

          <div class="pt-2.5 mt-3 border-t border-slate-800/60 flex items-center justify-between text-[11px] text-slate-400">
            <span>{isTargeted ? 'Active target for storage tables' : 'Click to filter storage tables'}</span>
            <span class="text-cyan-400 font-medium">{isTargeted ? 'Selected' : 'Filter'}</span>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</div>
