use crate::core::rules::{matches_wildcard, CleanupRule, RiskTier};
use crate::storage::rescue_bin::OperationProgress;
use jwalk::WalkDir;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedFileItem {
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
    pub modified_epoch_secs: u64,
    pub rule_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleScanResult {
    pub rule_id: String,
    pub rule_name: String,
    pub risk: RiskTier,
    pub total_bytes: u64,
    pub file_count: usize,
    pub items: Vec<ScannedFileItem>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalScanSummary {
    pub results: Vec<RuleScanResult>,
    pub total_bytes: u64,
    pub safe_bytes: u64,
    pub review_bytes: u64,
    pub advanced_bytes: u64,
    pub total_files: usize,
}

pub fn scan_single_rule(rule: &CleanupRule) -> RuleScanResult {
    let resolved_paths = rule.resolve_paths();
    if resolved_paths.is_empty() {
        return RuleScanResult {
            rule_id: rule.id.clone(),
            rule_name: rule.name.clone(),
            risk: rule.risk.clone(),
            total_bytes: 0,
            file_count: 0,
            items: Vec::new(),
            error: None,
        };
    }

    let min_age_cutoff = rule.age_days.map(|days| {
        SystemTime::now()
            .checked_sub(Duration::from_secs((days as u64) * 86400))
            .unwrap_or(SystemTime::UNIX_EPOCH)
    });

    let mut all_items: Vec<ScannedFileItem> = Vec::new();
    let mut total_bytes = 0u64;
    let mut total_files = 0usize;

    for base_path in resolved_paths {
        if !base_path.exists() {
            continue;
        }

        // If it's a file directly
        if base_path.is_file() {
            if let Ok(metadata) = base_path.metadata() {
                let size = metadata.len();
                let modified = metadata
                    .modified()
                    .unwrap_or(SystemTime::UNIX_EPOCH);

                let is_old_enough = match min_age_cutoff {
                    Some(cutoff) => modified <= cutoff,
                    None => true,
                };

                if is_old_enough {
                    total_bytes += size;
                    total_files += 1;
                    if all_items.len() < 500 {
                        all_items.push(ScannedFileItem {
                            path: base_path.to_string_lossy().to_string(),
                            name: base_path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string(),
                            size_bytes: size,
                            modified_epoch_secs: modified
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                            rule_id: rule.id.clone(),
                        });
                    }
                }
            }
            continue;
        }

        // Walk directory
        for entry in WalkDir::new(&base_path)
            .skip_hidden(false)
            .follow_links(false)
            .into_iter()
            .flatten()
        {
            if entry.file_type().is_file() {
                let path = entry.path();

                // Check wildcard pattern if provided
                if let Some(ref pat) = rule.pattern {
                    let file_name = path
                        .file_name()
                        .map(|s| s.to_string_lossy())
                        .unwrap_or_default();
                    if !matches_wildcard(&file_name, pat) {
                        continue;
                    }
                }

                if let Ok(metadata) = entry.metadata() {
                    let size = metadata.len();
                    let modified = metadata
                        .modified()
                        .unwrap_or(SystemTime::UNIX_EPOCH);

                    let is_old_enough = match min_age_cutoff {
                        Some(cutoff) => modified <= cutoff,
                        None => true,
                    };

                    if is_old_enough {
                        total_bytes += size;
                        total_files += 1;
                        if all_items.len() < 500 {
                            all_items.push(ScannedFileItem {
                                path: path.to_string_lossy().to_string(),
                                name: path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string(),
                                size_bytes: size,
                                modified_epoch_secs: modified
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs(),
                                rule_id: rule.id.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    RuleScanResult {
        rule_id: rule.id.clone(),
        rule_name: rule.name.clone(),
        risk: rule.risk.clone(),
        total_bytes,
        file_count: total_files,
        items: all_items,
        error: None,
    }
}

pub fn collect_all_rule_file_paths(rule: &CleanupRule) -> Vec<String> {
    let resolved_paths = rule.resolve_paths();
    let min_age_cutoff = rule.age_days.map(|days| {
        SystemTime::now()
            .checked_sub(Duration::from_secs((days as u64) * 86400))
            .unwrap_or(SystemTime::UNIX_EPOCH)
    });

    let mut paths = Vec::new();

    for base_path in resolved_paths {
        if !base_path.exists() {
            continue;
        }

        if base_path.is_file() {
            let is_old = match min_age_cutoff {
                Some(cutoff) => base_path
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .map_or(true, |m| m <= cutoff),
                None => true,
            };
            if is_old {
                paths.push(base_path.to_string_lossy().to_string());
            }
            continue;
        }

        for entry in WalkDir::new(&base_path)
            .skip_hidden(false)
            .follow_links(false)
            .into_iter()
            .flatten()
        {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(ref pat) = rule.pattern {
                    let fname = path.file_name().map(|s| s.to_string_lossy()).unwrap_or_default();
                    if !matches_wildcard(&fname, pat) {
                        continue;
                    }
                }

                let is_old = match min_age_cutoff {
                    Some(cutoff) => entry
                        .metadata()
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .map_or(true, |m| m <= cutoff),
                    None => true,
                };

                if is_old {
                    paths.push(path.to_string_lossy().to_string());
                }
            }
        }
    }

    paths
}

pub fn scan_all_rules_with_progress<F>(
    rules: &[CleanupRule],
    mut progress_callback: Option<F>,
) -> TotalScanSummary
where
    F: FnMut(OperationProgress) + Send,
{
    let total_rules = rules.len();
    let mut results = Vec::new();
    let mut total_bytes = 0u64;
    let mut safe_bytes = 0u64;
    let mut review_bytes = 0u64;
    let mut advanced_bytes = 0u64;
    let mut total_files = 0usize;

    for (idx, rule) in rules.iter().enumerate() {
        let rule_result = scan_single_rule(rule);

        total_bytes += rule_result.total_bytes;
        total_files += rule_result.file_count;
        match rule_result.risk {
            RiskTier::Safe => safe_bytes += rule_result.total_bytes,
            RiskTier::Review => review_bytes += rule_result.total_bytes,
            RiskTier::Advanced => advanced_bytes += rule_result.total_bytes,
        }

        results.push(rule_result);

        if let Some(ref mut cb) = progress_callback {
            let processed_count = idx + 1;
            let percent = if total_rules > 0 {
                ((processed_count as f32) / (total_rules as f32)) * 100.0
            } else {
                100.0
            };
            cb(OperationProgress {
                stage: "scanning".to_string(),
                current_item: rule.name.clone(),
                processed_count,
                total_count: total_rules,
                percent,
                skipped_locked: 0,
                bytes_processed: total_bytes,
            });
        }
    }

    TotalScanSummary {
        results,
        total_bytes,
        safe_bytes,
        review_bytes,
        advanced_bytes,
        total_files,
    }
}

pub fn scan_all_rules(rules: &[CleanupRule]) -> TotalScanSummary {
    let results: Vec<RuleScanResult> = rules.par_iter().map(scan_single_rule).collect();

    let mut total_bytes = 0u64;
    let mut safe_bytes = 0u64;
    let mut review_bytes = 0u64;
    let mut advanced_bytes = 0u64;
    let mut total_files = 0usize;

    for r in &results {
        total_bytes += r.total_bytes;
        total_files += r.file_count;
        match r.risk {
            RiskTier::Safe => safe_bytes += r.total_bytes,
            RiskTier::Review => review_bytes += r.total_bytes,
            RiskTier::Advanced => advanced_bytes += r.total_bytes,
        }
    }

    TotalScanSummary {
        results,
        total_bytes,
        safe_bytes,
        review_bytes,
        advanced_bytes,
        total_files,
    }
}

pub fn compute_directory_size(dir: &Path) -> u64 {
    WalkDir::new(dir)
        .skip_hidden(false)
        .into_iter()
        .flatten()
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}
