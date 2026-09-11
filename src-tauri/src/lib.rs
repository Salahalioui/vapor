pub mod ai;
pub mod apps;
pub mod core;
pub mod process;
pub mod storage;
pub mod system;

use ai::gemini::{
    audit_processes_batch_with_gemini, explain_process_with_gemini, FleetAuditReport,
    GeminiProcessExplanation,
};
use system::specs::{get_system_specs, SystemSpecsOverview};
use system::tweaks::{
    create_system_restore_point, get_sponsored_apps, get_windows_tweaks, set_windows_tweak,
    uninstall_sponsored_app, SponsoredApp, WindowsTweak,
};
use ai::sanitize::sanitize_process_info;
use apps::installed::{get_installed_applications, InstalledApp};
use apps::startup::{get_startup_items, remove_startup_item, toggle_startup_item, StartupItem};
use core::health::{calculate_health_score, generate_top_actions, SystemHealth};
use core::rules::{load_default_rules, CleanupRule};
use process::monitor::{get_hardware_overview, kill_process_by_pid, ProcessInfo, SystemHardwareOverview};
use storage::dev_diet::{purge_dev_directory, scan_dev_diet, DevDietItem};
use storage::large_files::{scan_large_files, LargeFileItem};
use storage::rescue_bin::{
    auto_purge_expired, get_stage_manifest, list_rescue_stages, purge_stage, recycle_via_trash,
    restore_stage, stage_cleanup_files_with_progress,
    OperationProgress, RescueStageSummary, RestoreSummary, StageManifest,
};
use storage::scanner::{scan_all_rules, scan_single_rule, RuleScanResult, TotalScanSummary};
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::Emitter;

static LAST_DEV_DIET_BYTES: AtomicU64 = AtomicU64::new(0);
static LAST_LARGE_FILES_BYTES: AtomicU64 = AtomicU64::new(0);

#[tauri::command]
async fn get_system_health(
    cached_dev_diet: Option<u64>,
    cached_large_files: Option<u64>,
) -> SystemHealth {
    tokio::task::spawn_blocking(move || {
        let hw = get_hardware_overview();
        let rules = load_default_rules();
        let safe_rules: Vec<CleanupRule> = rules
            .into_iter()
            .filter(|r| r.risk == core::rules::RiskTier::Safe)
            .collect();

        // Fast scan on safe rules
        let scan_sum = scan_all_rules(&safe_rules);
        let safe_cleanup_bytes = scan_sum.safe_bytes;

        let primary_disk = hw.disks.first();
        let disk_free_pct = primary_disk.map(|d| d.free_percent).unwrap_or(25.0);
        let disk_total = primary_disk.map(|d| d.total_bytes).unwrap_or(0);
        let disk_free = primary_disk.map(|d| d.available_bytes).unwrap_or(0);

        let startups = get_startup_items();
        let installed = get_installed_applications();
        let zombie_count = installed.iter().filter(|a| a.is_zombie).count();
        let startup_count = startups.len();

        let (score, pillars) = calculate_health_score(
            disk_free_pct,
            hw.memory_percent,
            safe_cleanup_bytes,
            startup_count + zombie_count,
        );

        let grade = if score >= 85 {
            "Optimal".to_string()
        } else if score >= 70 {
            "Good".to_string()
        } else if score >= 50 {
            "Attention Needed".to_string()
        } else {
            "Critical".to_string()
        };

        let dev_diet_bytes = cached_dev_diet.unwrap_or_else(|| LAST_DEV_DIET_BYTES.load(Ordering::Relaxed));
        let large_files_bytes = cached_large_files.unwrap_or_else(|| LAST_LARGE_FILES_BYTES.load(Ordering::Relaxed));

        let top_actions = generate_top_actions(
            safe_cleanup_bytes,
            dev_diet_bytes,
            large_files_bytes,
            startup_count,
            zombie_count,
        );

        SystemHealth {
            overall_score: score,
            grade,
            pillars,
            top_actions,
            total_memory_bytes: hw.total_memory_bytes,
            used_memory_bytes: hw.used_memory_bytes,
            memory_percent: hw.memory_percent,
            cpu_percent: hw.global_cpu_percent,
            primary_disk_total_bytes: disk_total,
            primary_disk_free_bytes: disk_free,
            primary_disk_free_percent: disk_free_pct,
            safe_cleanup_bytes,
            dev_diet_bytes,
            startup_count,
            zombie_count,
        }
    })
    .await
    .expect("get_system_health task panicked")
}

#[tauri::command]
async fn get_hardware_stats() -> SystemHardwareOverview {
    tokio::task::spawn_blocking(get_hardware_overview)
        .await
        .unwrap_or_else(|_| get_hardware_overview())
}

#[tauri::command]
fn get_cleanup_rules() -> Vec<CleanupRule> {
    load_default_rules()
}

#[tauri::command]
async fn scan_rules(
    app: tauri::AppHandle,
    rules: Option<Vec<CleanupRule>>,
) -> TotalScanSummary {
    tokio::task::spawn_blocking(move || {
        let targets = rules.unwrap_or_else(load_default_rules);
        storage::scanner::scan_all_rules_with_progress(&targets, Some(|p: OperationProgress| {
            let _ = app.emit("operation-progress", p);
        }))
    })
    .await
    .unwrap_or_else(|_| TotalScanSummary {
        results: Vec::new(),
        total_bytes: 0,
        safe_bytes: 0,
        review_bytes: 0,
        advanced_bytes: 0,
        total_files: 0,
    })
}

#[tauri::command]
async fn scan_single_cleanup_rule(rule: CleanupRule) -> RuleScanResult {
    let rule_clone = rule.clone();
    tokio::task::spawn_blocking(move || scan_single_rule(&rule))
        .await
        .unwrap_or_else(|_| scan_single_rule(&rule_clone))
}

#[tauri::command]
async fn stage_cleanup(
    app: tauri::AppHandle,
    paths: Vec<String>,
) -> Result<RescueStageSummary, String> {
    tokio::task::spawn_blocking(move || {
        stage_cleanup_files_with_progress(&paths, Some(|p: OperationProgress| {
            let _ = app.emit("operation-progress", p);
        }))
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

#[tauri::command]
async fn stage_cleanup_rules(
    app: tauri::AppHandle,
    rule_ids: Vec<String>,
    excluded_paths: Option<Vec<String>>,
) -> Result<RescueStageSummary, String> {
    tokio::task::spawn_blocking(move || {
        let exclusions = excluded_paths.unwrap_or_default();
        storage::rescue_bin::stage_cleanup_rules_with_exclusions_and_progress(
            &rule_ids,
            &exclusions,
            Some(|p: OperationProgress| {
                let _ = app.emit("operation-progress", p);
            }),
        )
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

#[tauri::command]
async fn get_stage_manifest_details(stage_id: String) -> Result<StageManifest, String> {
    tokio::task::spawn_blocking(move || get_stage_manifest(&stage_id))
        .await
        .map_err(|e| format!("Task error: {}", e))?
}

#[tauri::command]
async fn get_rescue_stages() -> Vec<RescueStageSummary> {
    tokio::task::spawn_blocking(list_rescue_stages)
        .await
        .unwrap_or_default()
}

#[tauri::command]
async fn restore_rescue_stage(stage_id: String) -> Result<RestoreSummary, String> {
    tokio::task::spawn_blocking(move || {
        restore_stage(&stage_id)
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

#[tauri::command]
async fn purge_rescue_stage(stage_id: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        purge_stage(&stage_id)
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

#[tauri::command]
async fn auto_purge_rescue_stages(retention_days: u32) -> Result<usize, String> {
    tokio::task::spawn_blocking(move || auto_purge_expired(retention_days))
        .await
        .map_err(|e| format!("Task error: {}", e))?
}

#[tauri::command]
async fn recycle_file(path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || recycle_via_trash(&path))
        .await
        .map_err(|e| format!("Task error: {}", e))?
}

#[tauri::command]
async fn get_large_files(
    custom_roots: Option<Vec<String>>,
    min_size_mb: Option<u64>,
    limit: Option<usize>,
) -> Vec<LargeFileItem> {
    tokio::task::spawn_blocking(move || {
        let min_bytes = min_size_mb.unwrap_or(100) * 1024 * 1024;
        let max_count = limit.unwrap_or(50);
        let items = scan_large_files(custom_roots, min_bytes, max_count);
        let total: u64 = items.iter().map(|i| i.size_bytes).sum();
        LAST_LARGE_FILES_BYTES.store(total, Ordering::Relaxed);
        items
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
async fn get_dev_diet(
    custom_roots: Option<Vec<String>>,
    dormant_days: Option<u32>,
) -> Vec<DevDietItem> {
    tokio::task::spawn_blocking(move || {
        let items = scan_dev_diet(custom_roots, dormant_days.unwrap_or(30));
        let total: u64 = items.iter().filter(|i| i.is_dormant).map(|i| i.size_bytes).sum();
        LAST_DEV_DIET_BYTES.store(total, Ordering::Relaxed);
        items
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
async fn purge_dev_target(path: String) -> Result<u64, String> {
    tokio::task::spawn_blocking(move || {
        purge_dev_directory(&path)
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

#[tauri::command]
fn kill_process(pid: u32) -> Result<(), String> {
    kill_process_by_pid(pid)
}

#[tauri::command]
async fn explain_process(
    api_key: String,
    process_name: String,
    exe_path: Option<String>,
    publisher: Option<String>,
    description: Option<String>,
    cpu_percent: f32,
    memory_mb: f32,
) -> Result<GeminiProcessExplanation, String> {
    let sanitized = sanitize_process_info(
        &process_name,
        exe_path.as_deref(),
        publisher.as_deref(),
        description.as_deref(),
        cpu_percent,
        memory_mb,
    );
    explain_process_with_gemini(&api_key, sanitized).await
}

#[tauri::command]
async fn get_installed_software() -> Vec<InstalledApp> {
    tokio::task::spawn_blocking(get_installed_applications)
        .await
        .unwrap_or_default()
}

#[tauri::command]
async fn get_startup_software() -> Vec<StartupItem> {
    tokio::task::spawn_blocking(get_startup_items)
        .await
        .unwrap_or_default()
}

#[tauri::command]
async fn disable_startup_software(name: String, hive: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || remove_startup_item(&name, &hive))
        .await
        .map_err(|e| format!("Task error: {}", e))?
}

#[tauri::command]
async fn toggle_startup_software(name: String, hive: String, enabled: bool) -> Result<(), String> {
    tokio::task::spawn_blocking(move || toggle_startup_item(&name, &hive, enabled))
        .await
        .map_err(|e| format!("Task error: {}", e))?
}

#[tauri::command]
async fn show_in_folder(path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let trimmed = path.trim().trim_matches('"');
        if trimmed.is_empty() {
            return Err("Path cannot be empty".to_string());
        }
        let clean = trimmed.replace('/', "\\");
        let p = std::path::Path::new(&clean);
        if !p.exists() {
            return Err(format!("Target path does not exist: {}", clean));
        }

        let mut cmd = std::process::Command::new("explorer");
        if p.is_file() {
            cmd.arg(format!("/select,{}", clean));
        } else {
            cmd.arg(&clean);
        }
        cmd.spawn().map_err(|e| format!("Failed to open Explorer: {}", e))?;
        Ok(())
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

#[tauri::command]
async fn launch_uninstaller(uninstall_string: Option<String>) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        if let Some(cmd_str) = uninstall_string.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                let mut cmd = std::process::Command::new("cmd");
                cmd.raw_arg(format!("/s /c \"{}\"", cmd_str));
                if cmd.spawn().is_ok() {
                    return Ok(());
                }
            }
            #[cfg(not(windows))]
            {
                if std::process::Command::new("sh").args(["-c", cmd_str]).spawn().is_ok() {
                    return Ok(());
                }
            }
        }
        // Fallback: Open Windows Installed Apps Settings
        std::process::Command::new("explorer")
            .arg("ms-settings:appsfeatures")
            .spawn()
            .map_err(|e| format!("Failed to open settings: {}", e))?;
        Ok(())
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

#[tauri::command]
async fn open_external_url(url: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let trimmed = url.trim();
        if trimmed.is_empty() {
            return Err("URL cannot be empty".to_string());
        }
        if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
            return Err("Invalid URL protocol: must start with http:// or https://".to_string());
        }
        // Launch via rundll32 url.dll,FileProtocolHandler to safely open default browser
        // without invoking cmd.exe shell which breaks on & ampersands and special characters
        std::process::Command::new("rundll32.exe")
            .args(["url.dll,FileProtocolHandler", trimmed])
            .spawn()
            .map_err(|e| format!("Failed to launch browser: {}", e))?;
        Ok(())
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

#[tauri::command]
async fn audit_processes_batch(
    api_key: String,
    processes: Option<Vec<ProcessInfo>>,
) -> Result<FleetAuditReport, String> {
    let procs = match processes {
        Some(p) if !p.is_empty() => p,
        _ => crate::process::monitor::get_processes_for_audit(),
    };
    audit_processes_batch_with_gemini(&api_key, &procs).await
}

#[tauri::command]
async fn get_pc_specs() -> SystemSpecsOverview {
    tokio::task::spawn_blocking(get_system_specs)
        .await
        .unwrap_or_else(|_| get_system_specs())
}

#[tauri::command]
async fn get_pc_tweaks() -> Vec<WindowsTweak> {
    tokio::task::spawn_blocking(get_windows_tweaks)
        .await
        .unwrap_or_default()
}

#[tauri::command]
async fn apply_pc_tweak(id: String, enable: bool) -> Result<(), String> {
    tokio::task::spawn_blocking(move || set_windows_tweak(&id, enable))
        .await
        .map_err(|e| format!("Task error: {}", e))?
}

#[tauri::command]
async fn get_sponsored_bloatware() -> Vec<SponsoredApp> {
    tokio::task::spawn_blocking(get_sponsored_apps)
        .await
        .unwrap_or_default()
}

#[tauri::command]
async fn remove_sponsored_app(package_full_name: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || uninstall_sponsored_app(&package_full_name))
        .await
        .map_err(|e| format!("Task error: {}", e))?
}

#[tauri::command]
async fn trigger_system_restore_point(description: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || create_system_restore_point(&description))
        .await
        .map_err(|e| format!("Task error: {}", e))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            let cached_count = ai::gemini::init_ai_cache();
            log::info!("Vapor startup: initialized AI cache ({} entries loaded)", cached_count);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_system_health,
            get_hardware_stats,
            get_cleanup_rules,
            scan_rules,
            scan_single_cleanup_rule,
            stage_cleanup,
            stage_cleanup_rules,
            get_stage_manifest_details,
            get_rescue_stages,
            restore_rescue_stage,
            purge_rescue_stage,
            auto_purge_rescue_stages,
            recycle_file,
            get_large_files,
            get_dev_diet,
            purge_dev_target,
            kill_process,
            explain_process,
            audit_processes_batch,
            get_installed_software,
            get_startup_software,
            disable_startup_software,
            toggle_startup_software,
            show_in_folder,
            launch_uninstaller,
            open_external_url,
            get_pc_specs,
            get_pc_tweaks,
            apply_pc_tweak,
            get_sponsored_bloatware,
            remove_sponsored_app,
            trigger_system_restore_point,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
