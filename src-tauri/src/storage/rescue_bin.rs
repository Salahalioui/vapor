use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RescueItem {
    pub original_path: String,
    pub relative_staged_path: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationProgress {
    pub stage: String,           // "scanning" | "staging" | "deleting"
    pub current_item: String,    // Current filename being processed
    pub processed_count: usize,  // Number of items completed
    pub total_count: usize,      // Total items discovered
    pub percent: f32,            // 0.0 - 100.0
    pub skipped_locked: usize,   // Count of busy/locked files skipped
    pub bytes_processed: u64,    // Bytes cleaned/staged so far
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageManifest {
    pub stage_id: String,
    pub created_at_secs: u64,
    pub total_bytes: u64,
    pub items: Vec<RescueItem>,
    #[serde(default)]
    pub skipped_locked: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RescueStageSummary {
    pub stage_id: String,
    pub created_at_secs: u64,
    pub total_bytes: u64,
    pub file_count: usize,
    pub days_old: u32,
    #[serde(default)]
    pub skipped_locked: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreSummary {
    pub stage_id: String,
    pub restored_count: usize,
    pub failed_count: usize,
    pub restored_bytes: u64,
    pub errors: Vec<String>,
}

pub fn get_rescue_root() -> PathBuf {
    if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
        PathBuf::from(local_appdata).join("Vapor").join("RescueBin")
    } else {
        env::temp_dir().join("Vapor_RescueBin")
    }
}

pub fn stage_cleanup_files(file_paths: &[String]) -> Result<RescueStageSummary, String> {
    stage_cleanup_files_with_progress(file_paths, None::<fn(OperationProgress)>)
}

pub fn stage_cleanup_files_with_progress<F>(
    file_paths: &[String],
    mut progress_callback: Option<F>,
) -> Result<RescueStageSummary, String>
where
    F: FnMut(OperationProgress) + Send,
{
    if file_paths.is_empty() {
        return Err("No files selected for cleanup".to_string());
    }

    let rescue_root = get_rescue_root();
    let now = SystemTime::now();
    let timestamp = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let stage_id = format!("stage_{}_{}", timestamp, rand_suffix());
    let stage_dir = rescue_root.join(&stage_id);
    let staged_files_dir = stage_dir.join("files");

    fs::create_dir_all(&staged_files_dir)
        .map_err(|e| format!("Failed to create rescue bin stage directory: {}", e))?;

    let total_count = file_paths.len();
    let mut staged_items = Vec::new();
    let mut total_bytes = 0u64;
    let mut skipped_locked = 0usize;
    let mut processed_count = 0usize;
    let mut last_emit = std::time::Instant::now();
    let emit_interval = Duration::from_millis(50);

    for (idx, path_str) in file_paths.iter().enumerate() {
        processed_count += 1;
        let orig_path = Path::new(path_str);
        let mut file_skipped = false;

        let file_name = orig_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("file_{}", idx));

        if orig_path.is_file() {
            if let Ok(meta) = orig_path.metadata() {
                let size = meta.len();
                let target_name = format!("{}_{}", idx, file_name);
                let dest_path = staged_files_dir.join(&target_name);

                // Fast rename move. If locked, access denied, or sharing violation,
                // skip immediately (< 1ms) and record in skipped_locked count.
                // Never attempt slow byte-by-byte copies on full drives.
                match fs::rename(orig_path, &dest_path) {
                    Ok(_) => {
                        total_bytes += size;
                        staged_items.push(RescueItem {
                            original_path: path_str.clone(),
                            relative_staged_path: format!("files/{}", target_name),
                            size_bytes: size,
                        });
                    }
                    Err(_) => {
                        skipped_locked += 1;
                        file_skipped = true;
                    }
                }
            }
        }

        if let Some(ref mut cb) = progress_callback {
            let is_first = processed_count == 1;
            let is_last = processed_count == total_count;
            let time_due = last_emit.elapsed() >= emit_interval;

            if is_first || is_last || time_due || file_skipped {
                last_emit = std::time::Instant::now();
                let percent = if total_count > 0 {
                    ((processed_count as f32) / (total_count as f32)) * 100.0
                } else {
                    100.0
                };
                cb(OperationProgress {
                    stage: "staging".to_string(),
                    current_item: file_name,
                    processed_count,
                    total_count,
                    percent,
                    skipped_locked,
                    bytes_processed: total_bytes,
                });
            }
        }
    }

    let count = staged_items.len();
    if count == 0 {
        let _ = fs::remove_dir_all(&stage_dir);
        if skipped_locked > 0 {
            return Err(format!(
                "All {} selected files are currently locked or in-use by active applications and were skipped.",
                skipped_locked
            ));
        }
        return Err("No files could be moved into Rescue Bin (files may be in use or already removed)".to_string());
    }

    let manifest = StageManifest {
        stage_id: stage_id.clone(),
        created_at_secs: timestamp,
        total_bytes,
        items: staged_items,
        skipped_locked,
    };

    let manifest_path = stage_dir.join("manifest.json");
    if let Ok(json) = serde_json::to_string_pretty(&manifest) {
        let _ = fs::write(manifest_path, json);
    }

    Ok(RescueStageSummary {
        stage_id,
        created_at_secs: timestamp,
        total_bytes,
        file_count: count,
        days_old: 0,
        skipped_locked,
    })
}

pub fn stage_cleanup_rules(rule_ids: &[String]) -> Result<RescueStageSummary, String> {
    stage_cleanup_rules_with_exclusions_and_progress(rule_ids, &[], None::<fn(OperationProgress)>)
}

pub fn stage_cleanup_rules_with_progress<F>(
    rule_ids: &[String],
    progress_callback: Option<F>,
) -> Result<RescueStageSummary, String>
where
    F: FnMut(OperationProgress) + Send,
{
    stage_cleanup_rules_with_exclusions_and_progress(rule_ids, &[], progress_callback)
}

pub fn stage_cleanup_rules_with_exclusions_and_progress<F>(
    rule_ids: &[String],
    excluded_paths: &[String],
    mut progress_callback: Option<F>,
) -> Result<RescueStageSummary, String>
where
    F: FnMut(OperationProgress) + Send,
{
    if let Some(ref mut cb) = progress_callback {
        cb(OperationProgress {
            stage: "scanning".to_string(),
            current_item: "Preparing target files...".to_string(),
            processed_count: 0,
            total_count: 0,
            percent: 0.0,
            skipped_locked: 0,
            bytes_processed: 0,
        });
    }

    let all_rules = crate::core::rules::load_default_rules();
    let mut all_paths = Vec::new();

    let exclusions: std::collections::HashSet<String> = excluded_paths
        .iter()
        .map(|p| p.to_lowercase())
        .collect();

    for rule_id in rule_ids {
        if let Some(rule) = all_rules.iter().find(|r| &r.id == rule_id) {
            let paths = super::scanner::collect_all_rule_file_paths(rule);
            for p in paths {
                if !exclusions.contains(&p.to_lowercase()) {
                    all_paths.push(p);
                }
            }
        }
    }

    if all_paths.is_empty() {
        return Err("No files found matching the selected rules (or all matching files were excluded)".to_string());
    }

    stage_cleanup_files_with_progress(&all_paths, progress_callback)
}

pub fn list_rescue_stages() -> Vec<RescueStageSummary> {
    let rescue_root = get_rescue_root();
    if !rescue_root.exists() {
        return Vec::new();
    }

    let mut stages = Vec::new();
    let now = SystemTime::now();

    if let Ok(entries) = fs::read_dir(&rescue_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("manifest.json");
                if manifest_path.exists() {
                    if let Ok(content) = fs::read_to_string(&manifest_path) {
                        if let Ok(manifest) = serde_json::from_str::<StageManifest>(&content) {
                            let created_time = SystemTime::UNIX_EPOCH
                                + Duration::from_secs(manifest.created_at_secs);
                            let age_secs = now
                                .duration_since(created_time)
                                .unwrap_or(Duration::ZERO)
                                .as_secs();
                            let days_old = (age_secs / 86400) as u32;

                            stages.push(RescueStageSummary {
                                stage_id: manifest.stage_id,
                                created_at_secs: manifest.created_at_secs,
                                total_bytes: manifest.total_bytes,
                                file_count: manifest.items.len(),
                                days_old,
                                skipped_locked: manifest.skipped_locked,
                            });
                        }
                    }
                }
            }
        }
    }

    stages.sort_by(|a, b| b.created_at_secs.cmp(&a.created_at_secs));
    stages
}

pub fn get_stage_manifest(stage_id: &str) -> Result<StageManifest, String> {
    let rescue_root = get_rescue_root();
    let stage_dir = rescue_root.join(stage_id);
    let manifest_path = stage_dir.join("manifest.json");

    if !manifest_path.exists() {
        return Err(format!("Stage manifest not found for stage {}", stage_id));
    }

    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read stage manifest: {}", e))?;
    let manifest: StageManifest = serde_json::from_str(&content)
        .map_err(|e| format!("Corrupted stage manifest: {}", e))?;

    Ok(manifest)
}

pub fn restore_stage(stage_id: &str) -> Result<RestoreSummary, String> {
    let rescue_root = get_rescue_root();
    let stage_dir = rescue_root.join(stage_id);
    let manifest_path = stage_dir.join("manifest.json");

    if !manifest_path.exists() {
        return Err(format!("Stage {} manifest not found", stage_id));
    }

    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read stage manifest: {}", e))?;
    let manifest: StageManifest = serde_json::from_str(&content)
        .map_err(|e| format!("Corrupted stage manifest: {}", e))?;

    let mut restored_count = 0;
    let mut failed_count = 0;
    let mut restored_bytes = 0u64;
    let mut errors = Vec::new();

    for item in &manifest.items {
        let staged_file = stage_dir.join(&item.relative_staged_path);
        let orig_path = Path::new(&item.original_path);

        if !staged_file.exists() {
            failed_count += 1;
            errors.push(format!("Missing staged file: {}", item.relative_staged_path));
            continue;
        }

        if let Some(parent) = orig_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let move_result = fs::rename(&staged_file, orig_path)
            .or_else(|_| fs::copy(&staged_file, orig_path).map(|_| ()));

        match move_result {
            Ok(_) => {
                restored_count += 1;
                restored_bytes += item.size_bytes;
                let _ = fs::remove_file(&staged_file);
            }
            Err(e) => {
                failed_count += 1;
                errors.push(format!("Failed restoring {}: {}", item.original_path, e));
            }
        }
    }

    // Clean up stage folder if all restored
    if failed_count == 0 {
        let _ = fs::remove_dir_all(&stage_dir);
    }

    Ok(RestoreSummary {
        stage_id: stage_id.to_string(),
        restored_count,
        failed_count,
        restored_bytes,
        errors,
    })
}

pub fn purge_stage(stage_id: &str) -> Result<(), String> {
    let rescue_root = get_rescue_root();
    let stage_dir = rescue_root.join(stage_id);
    if stage_dir.exists() {
        fs::remove_dir_all(&stage_dir)
            .map_err(|e| format!("Failed to delete stage directory: {}", e))?;
    }
    Ok(())
}

pub fn auto_purge_expired(retention_days: u32) -> Result<usize, String> {
    let stages = list_rescue_stages();
    let mut purged = 0;
    for s in stages {
        if s.days_old >= retention_days {
            if purge_stage(&s.stage_id).is_ok() {
                purged += 1;
            }
        }
    }
    Ok(purged)
}

pub fn recycle_via_trash(path_str: &str) -> Result<(), String> {
    let path = Path::new(path_str);
    if !path.exists() {
        return Err("File does not exist".to_string());
    }
    trash::delete(path).map_err(|e| format!("Failed to move to Windows Recycle Bin: {}", e))
}

fn rand_suffix() -> u32 {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    nanos % 100_000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_manifest_serialization() {
        let manifest = StageManifest {
            stage_id: "stage_test_123".to_string(),
            created_at_secs: 1700000000,
            total_bytes: 4096,
            items: vec![RescueItem {
                original_path: "C:\\test\\file.tmp".to_string(),
                relative_staged_path: "files/0_file.tmp".to_string(),
                size_bytes: 4096,
            }],
            skipped_locked: 0,
        };

        let json = serde_json::to_string(&manifest).expect("Must serialize");
        let deserialized: StageManifest = serde_json::from_str(&json).expect("Must deserialize");
        assert_eq!(deserialized.stage_id, "stage_test_123");
        assert_eq!(deserialized.items.len(), 1);
        assert_eq!(deserialized.items[0].size_bytes, 4096);
        assert_eq!(deserialized.skipped_locked, 0);
    }

    #[test]
    fn test_stage_cleanup_progress_callback() {
        let temp_dir = env::temp_dir().join("vapor_test_stage_progress");
        let _ = fs::create_dir_all(&temp_dir);
        let f1 = temp_dir.join("progress_test_1.txt");
        let f2 = temp_dir.join("progress_test_2.txt");
        let _ = fs::write(&f1, "hello 1");
        let _ = fs::write(&f2, "hello 2");

        let mut progress_events = Vec::new();
        let paths = vec![f1.to_string_lossy().to_string(), f2.to_string_lossy().to_string()];

        let res = stage_cleanup_files_with_progress(&paths, Some(|p: OperationProgress| {
            progress_events.push(p);
        })).expect("Staging should succeed");

        assert_eq!(res.file_count, 2);
        assert_eq!(res.skipped_locked, 0);
        assert_eq!(progress_events.len(), 2);
        assert_eq!(progress_events[1].processed_count, 2);
        assert_eq!(progress_events[1].percent, 100.0);

        // Cleanup
        let _ = purge_stage(&res.stage_id);
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_stage_cleanup_with_missing_files() {
        let temp_dir = env::temp_dir().join("vapor_test_stage_missing");
        let _ = fs::create_dir_all(&temp_dir);
        let f1 = temp_dir.join("existing_file.txt");
        let f_missing = temp_dir.join("non_existent_file.txt");
        let _ = fs::write(&f1, "existing");

        let mut progress_events = Vec::new();
        let paths = vec![
            f1.to_string_lossy().to_string(),
            f_missing.to_string_lossy().to_string(),
        ];

        let res = stage_cleanup_files_with_progress(&paths, Some(|p: OperationProgress| {
            progress_events.push(p);
        })).expect("Staging should succeed for existing file");

        assert_eq!(res.file_count, 1);
        assert_eq!(res.skipped_locked, 0);
        // Ensure final event reached 100% despite missing file
        let last_event = progress_events.last().expect("Must have progress events");
        assert_eq!(last_event.processed_count, 2);
        assert_eq!(last_event.percent, 100.0);

        let _ = purge_stage(&res.stage_id);
        let _ = fs::remove_dir_all(&temp_dir);
    }
}

