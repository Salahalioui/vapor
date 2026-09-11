let invokeFn;

async function getInvoke() {
  if (!invokeFn) {
    try {
      const core = await import('@tauri-apps/api/core');
      invokeFn = core.invoke;
    } catch (e) {
      console.warn('Tauri core API not available, falling back to mock mode:', e);
      invokeFn = null;
    }
  }
  return invokeFn;
}

export async function invokeCommand(cmd, args = {}) {
  const inv = await getInvoke();
  if (inv) {
    return await inv(cmd, args);
  }
  throw new Error(`Tauri invoke not available for command: ${cmd}`);
}

export async function onOperationProgress(handler) {
  try {
    const event = await import('@tauri-apps/api/event');
    return await event.listen('operation-progress', (e) => handler(e.payload));
  } catch (err) {
    console.warn('Tauri event API not available:', err);
    return () => {};
  }
}

// 1. System Health & Hardware
export async function fetchSystemHealth(cachedDevDiet = null, cachedLargeFiles = null) {
  return await invokeCommand('get_system_health', {
    cachedDevDiet,
    cachedLargeFiles,
  });
}

export async function fetchHardwareStats() {
  return await invokeCommand('get_hardware_stats');
}

// 2. Rules & Cleanup
export async function fetchCleanupRules() {
  return await invokeCommand('get_cleanup_rules');
}

export async function scanRules(rules = null) {
  return await invokeCommand('scan_rules', { rules });
}

export async function scanSingleRule(rule) {
  return await invokeCommand('scan_single_cleanup_rule', { rule });
}

export async function stageCleanup(paths) {
  return await invokeCommand('stage_cleanup', { paths });
}

export async function stageCleanupRules(ruleIds, excludedPaths = null) {
  return await invokeCommand('stage_cleanup_rules', { ruleIds, excludedPaths });
}

// 3. Rescue Bin
export async function fetchRescueStages() {
  return await invokeCommand('get_rescue_stages');
}

export async function getStageManifest(stageId) {
  return await invokeCommand('get_stage_manifest_details', { stageId });
}

export async function restoreRescueStage(stageId) {
  return await invokeCommand('restore_rescue_stage', { stageId });
}

export async function purgeRescueStage(stageId) {
  return await invokeCommand('purge_rescue_stage', { stageId });
}

export async function autoPurgeRescueStages(retentionDays = 14) {
  return await invokeCommand('auto_purge_rescue_stages', { retentionDays });
}

export async function recycleFile(path) {
  return await invokeCommand('recycle_file', { path });
}

// 4. Large Files & Dev Diet
export async function fetchLargeFiles(customRoots = null, minSizeMb = 100, limit = 50) {
  return await invokeCommand('get_large_files', { customRoots, minSizeMb, limit });
}

export async function fetchDevDiet(customRoots = null, dormantDays = 30) {
  return await invokeCommand('get_dev_diet', { customRoots, dormantDays });
}

export async function purgeDevTarget(path) {
  return await invokeCommand('purge_dev_target', { path });
}

// 5. Processes
export async function killProcess(pid) {
  return await invokeCommand('kill_process', { pid });
}

export async function explainProcess(apiKey, processName, exePath, publisher, description, cpuPercent, memoryMb) {
  return await invokeCommand('explain_process', {
    apiKey,
    processName,
    exePath,
    publisher,
    description,
    cpuPercent,
    memoryMb
  });
}

export async function auditProcessesBatch(apiKey, processes = null) {
  return await invokeCommand('audit_processes_batch', {
    apiKey,
    processes
  });
}

// 6. Apps & Startup
export async function fetchInstalledSoftware() {
  return await invokeCommand('get_installed_software');
}

export async function fetchStartupSoftware() {
  return await invokeCommand('get_startup_software');
}

export async function disableStartupSoftware(name, hive) {
  return await invokeCommand('disable_startup_software', { name, hive });
}

export async function toggleStartupSoftware(name, hive, enabled) {
  return await invokeCommand('toggle_startup_software', { name, hive, enabled });
}

// 7. System Explorer & External
export async function showInFolder(path) {
  return await invokeCommand('show_in_folder', { path });
}

export async function launchUninstaller(uninstallString = null) {
  return await invokeCommand('launch_uninstaller', { uninstallString });
}

export async function openExternalUrl(url) {
  return await invokeCommand('open_external_url', { url });
}

