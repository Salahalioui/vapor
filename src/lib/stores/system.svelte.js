import {
  fetchSystemHealth,
  fetchHardwareStats,
  onOperationProgress,
  fetchLargeFiles,
  fetchDevDiet,
  auditProcessesBatch,
  fetchPcSpecs,
  fetchPcTweaks,
  applyPcTweak,
  fetchSponsoredBloatware,
  removeSponsoredApp,
  createSystemRestorePoint,
  fetchCleanupRules,
  stageCleanupRules
} from '../api.js';
import { settings } from './settings.svelte.js';

class SystemStore {
  health = $state(null);
  hardware = $state(null);
  activeTab = $state('dashboard');
  
  isRefreshingHealth = $state(false);
  isRefreshingHardware = $state(false);
  lastUpdated = $state(Date.now());
  
  activeRescueBinDrawer = $state(false);
  activeExplainProcess = $state(null);
  previewCleanup = $state(null);

  // Storage tab cache & scan states
  selectedDriveRoot = $state(null);
  largeFiles = $state(null);
  isScanningLargeFiles = $state(false);

  devDietItems = $state(null);
  isScanningDevDiet = $state(false);

  // Gemini Fleet AI audit state & caching
  batchFleetReport = $state(null);
  isAuditingFleet = $state(false);
  cachedProcessExplanations = $state({});

  // v0.3.0 PC Specs, Capabilities, Tweaks, and Debloat Center
  pcSpecs = $state(null);
  isRefreshingSpecs = $state(false);

  pcTweaks = $state([]);
  isRefreshingTweaks = $state(false);

  sponsoredApps = $state([]);
  isRefreshingSponsored = $state(false);

  isCreatingRestorePoint = $state(false);
  restorePointMessage = $state(null);

  isSmartCleaning = $state(false);
  smartCleanSuccess = $state(null);

  // Live operation progress
  operationProgress = $state(null);

  pollingTimer = null;
  unlistenProgress = null;

  setSelectedDriveRoot(root) {
    this.selectedDriveRoot = root;
  }

  async scanStorageForDrive(driveMount) {
    this.setSelectedDriveRoot(driveMount);
    this.setIsScanningLargeFiles(true);
    this.setIsScanningDevDiet(true);
    try {
      const roots = driveMount ? [driveMount] : null;
      const [largeResults, devResults] = await Promise.all([
        fetchLargeFiles(roots, settings.largeFileSizeThresholdMb, 60),
        fetchDevDiet(roots, settings.devDietThresholdDays)
      ]);
      this.setLargeFiles(largeResults);
      this.setDevDietItems(devResults);
      await this.refreshHealth();
    } catch (e) {
      console.error('Failed targeted drive scan:', e);
    } finally {
      this.setIsScanningLargeFiles(false);
      this.setIsScanningDevDiet(false);
    }
  }

  cacheProcessExplanation(processName, explanation) {
    if (!processName) return;
    const lower = processName.toLowerCase();
    const noExt = lower.endsWith('.exe') ? lower.slice(0, -4) : lower;
    const withExt = lower.endsWith('.exe') ? lower : `${lower}.exe`;
    this.cachedProcessExplanations = {
      ...this.cachedProcessExplanations,
      [lower]: explanation,
      [noExt]: explanation,
      [withExt]: explanation
    };
  }

  getProcessExplanationFromCache(processName) {
    if (!processName) return null;
    const lower = processName.toLowerCase();
    const noExt = lower.endsWith('.exe') ? lower.slice(0, -4) : lower;
    const withExt = lower.endsWith('.exe') ? lower : `${lower}.exe`;
    return this.cachedProcessExplanations[lower] 
      ?? this.cachedProcessExplanations[noExt] 
      ?? this.cachedProcessExplanations[withExt] 
      ?? null;
  }

  async runFleetAiAudit(apiKey = '', processes = null) {
    this.isAuditingFleet = true;
    try {
      const report = await auditProcessesBatch(apiKey || '', processes);
      this.batchFleetReport = report;
      if (report?.items) {
        const updated = { ...this.cachedProcessExplanations };
        for (const item of report.items) {
          const entry = {
            summary: item.summary,
            vendor: item.vendor,
            safety: item.safety,
            can_terminate: item.can_terminate,
            why_high_usage: item.why_high_usage,
            recommendation: item.recommendation,
            isFromFleetAudit: true,
          };
          const lower = item.process_name.toLowerCase();
          const noExt = lower.endsWith('.exe') ? lower.slice(0, -4) : lower;
          const withExt = lower.endsWith('.exe') ? lower : `${lower}.exe`;
          updated[lower] = entry;
          updated[noExt] = entry;
          updated[withExt] = entry;
        }
        this.cachedProcessExplanations = updated;
      }
      return report;
    } finally {
      this.isAuditingFleet = false;
    }
  }

  setActiveTab(tab) {
    this.activeTab = tab;
  }

  openRescueBin() {
    this.activeRescueBinDrawer = true;
  }

  closeRescueBin() {
    this.activeRescueBinDrawer = false;
  }

  openExplainProcess(proc) {
    this.activeExplainProcess = proc;
  }

  closeExplainProcess() {
    this.activeExplainProcess = null;
  }

  setPreviewCleanup(data) {
    this.previewCleanup = data;
  }

  setIsScanningLargeFiles(val) {
    this.isScanningLargeFiles = val;
  }

  setIsScanningDevDiet(val) {
    this.isScanningDevDiet = val;
  }

  setLargeFiles(files) {
    this.largeFiles = files;
  }

  setDevDietItems(items) {
    this.devDietItems = items;
  }

  setOperationProgress(progress) {
    this.operationProgress = progress;
  }

  clearOperationProgress() {
    this.operationProgress = null;
  }

  async refreshHealth() {
    if (this.isRefreshingHealth) return;
    this.isRefreshingHealth = true;
    try {
      const devDietTotal = this.devDietItems?.filter((i) => i.is_dormant).reduce((acc, i) => acc + i.size_bytes, 0) ?? null;
      const largeFilesTotal = this.largeFiles?.reduce((acc, f) => acc + f.size_bytes, 0) ?? null;
      this.health = await fetchSystemHealth(devDietTotal, largeFilesTotal);
      this.lastUpdated = Date.now();
    } catch (e) {
      console.error('Failed to fetch system health:', e);
    } finally {
      this.isRefreshingHealth = false;
    }
  }

  async refreshHardware() {
    if (this.isRefreshingHardware) return;
    this.isRefreshingHardware = true;
    try {
      this.hardware = await fetchHardwareStats();
    } catch (e) {
      console.error('Failed to fetch hardware stats:', e);
    } finally {
      this.isRefreshingHardware = false;
    }
  }

  async refreshPcSpecs() {
    if (this.isRefreshingSpecs) return;
    this.isRefreshingSpecs = true;
    try {
      this.pcSpecs = await fetchPcSpecs();
    } catch (e) {
      console.error('Failed to fetch PC specs:', e);
    } finally {
      this.isRefreshingSpecs = false;
    }
  }

  async refreshPcTweaks() {
    if (this.isRefreshingTweaks) return;
    this.isRefreshingTweaks = true;
    try {
      this.pcTweaks = await fetchPcTweaks();
    } catch (e) {
      console.error('Failed to fetch PC tweaks:', e);
    } finally {
      this.isRefreshingTweaks = false;
    }
  }

  async togglePcTweak(id, enable) {
    try {
      await applyPcTweak(id, enable);
      await this.refreshPcTweaks();
    } catch (e) {
      console.error(`Failed to toggle PC tweak ${id}:`, e);
      throw e;
    }
  }

  async refreshSponsoredApps() {
    if (this.isRefreshingSponsored) return;
    this.isRefreshingSponsored = true;
    try {
      this.sponsoredApps = await fetchSponsoredBloatware();
    } catch (e) {
      console.error('Failed to fetch sponsored bloatware:', e);
    } finally {
      this.isRefreshingSponsored = false;
    }
  }

  async uninstallSponsored(packageFullName) {
    try {
      await removeSponsoredApp(packageFullName);
      this.sponsoredApps = (this.sponsoredApps || []).filter(
        (a) => a.package_full_name !== packageFullName
      );
    } catch (e) {
      console.error(`Failed to uninstall sponsored app ${packageFullName}:`, e);
      throw e;
    }
  }

  async createRestorePoint(description = '') {
    this.isCreatingRestorePoint = true;
    this.restorePointMessage = null;
    try {
      const msg = await createSystemRestorePoint(description);
      this.restorePointMessage = { type: 'success', text: msg };
      return msg;
    } catch (e) {
      const errText = typeof e === 'string' ? e : e?.message || 'Failed to create restore point';
      this.restorePointMessage = { type: 'error', text: errText };
      throw e;
    } finally {
      this.isCreatingRestorePoint = false;
    }
  }

  async runSmartClean() {
    if (this.isSmartCleaning) return;
    this.isSmartCleaning = true;
    this.smartCleanSuccess = null;
    try {
      const rules = await fetchCleanupRules();
      const safeRules = (rules || []).filter((r) => r.risk === 'Safe');
      const ruleIds = safeRules.map((r) => r.id);
      if (ruleIds.length > 0) {
        const summary = await stageCleanupRules(ruleIds);
        this.smartCleanSuccess = {
          fileCount: summary.file_count,
          bytesSaved: summary.total_bytes,
          stageId: summary.stage_id,
        };
      }
      await this.refreshHealth();
    } catch (e) {
      console.error('Failed to run smart clean:', e);
      throw e;
    } finally {
      this.isSmartCleaning = false;
    }
  }

  async initialize() {
    try {
      this.unlistenProgress = await onOperationProgress((payload) => {
        this.setOperationProgress(payload);
      });
    } catch (e) {
      console.warn('Failed to register operation-progress listener:', e);
    }

    await Promise.all([
      this.refreshHealth(),
      this.refreshHardware(),
      this.refreshPcSpecs(),
      this.refreshPcTweaks(),
      this.refreshSponsoredApps(),
    ]);
    this.startPolling();
  }

  startPolling() {
    this.stopPolling();
    this.pollingTimer = setInterval(() => {
      if (settings.autoRefreshEnabled) {
        this.refreshHardware();
      }
    }, 2000);
  }

  stopPolling() {
    if (this.pollingTimer) {
      clearInterval(this.pollingTimer);
      this.pollingTimer = null;
    }
  }
}

export const system = new SystemStore();
