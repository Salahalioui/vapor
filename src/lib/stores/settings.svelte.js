class SettingsStore {
  geminiApiKey = $state(localStorage.getItem('vapor_gemini_key') || '');
  retentionDays = $state(parseInt(localStorage.getItem('vapor_retention_days') || '14', 10));
  devDietThresholdDays = $state(parseInt(localStorage.getItem('vapor_dev_threshold') || '30', 10));
  largeFileSizeThresholdMb = $state(parseInt(localStorage.getItem('vapor_large_file_mb') || '100', 10));
  autoRefreshEnabled = $state(localStorage.getItem('vapor_auto_refresh') !== 'false');
  uiMode = $state(localStorage.getItem('vapor_ui_mode') || 'simple');

  setUiMode(mode) {
    this.uiMode = mode === 'expert' ? 'expert' : 'simple';
    localStorage.setItem('vapor_ui_mode', this.uiMode);
  }

  toggleUiMode() {
    this.setUiMode(this.uiMode === 'simple' ? 'expert' : 'simple');
  }

  setGeminiKey(key) {
    this.geminiApiKey = key.trim();
    localStorage.setItem('vapor_gemini_key', this.geminiApiKey);
  }

  setGeminiApiKey(key) {
    this.setGeminiKey(key);
  }

  setRetentionDays(days) {
    this.retentionDays = Math.max(1, days);
    localStorage.setItem('vapor_retention_days', this.retentionDays.toString());
  }

  setDevDietThreshold(days) {
    this.devDietThresholdDays = Math.max(1, days);
    localStorage.setItem('vapor_dev_threshold', this.devDietThresholdDays.toString());
  }

  setLargeFileSizeThreshold(mb) {
    this.largeFileSizeThresholdMb = Math.max(10, mb);
    localStorage.setItem('vapor_large_file_mb', this.largeFileSizeThresholdMb.toString());
  }

  toggleAutoRefresh() {
    this.autoRefreshEnabled = !this.autoRefreshEnabled;
    localStorage.setItem('vapor_auto_refresh', this.autoRefreshEnabled.toString());
  }
}

export const settings = new SettingsStore();
