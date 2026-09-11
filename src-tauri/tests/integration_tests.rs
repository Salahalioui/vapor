use std::fs;
use vapor_lib::ai::sanitize::sanitize_process_info;
use vapor_lib::apps::installed::get_installed_applications;
use vapor_lib::apps::startup::get_startup_items;
use vapor_lib::core::health::calculate_health_score;
use vapor_lib::core::rules::load_default_rules;
use vapor_lib::process::known_db::lookup_process;
use vapor_lib::process::monitor::get_hardware_overview;
use vapor_lib::storage::dev_diet::{purge_dev_directory, scan_dev_diet};
use vapor_lib::storage::rescue_bin::{
    list_rescue_stages, restore_stage, stage_cleanup_files, stage_cleanup_rules,
};

#[test]
fn test_health_score_weights_and_bounds() {
    // Perfect conditions
    let (perfect, pillars) = calculate_health_score(40.0, 30.0, 0, 1);
    assert_eq!(perfect, 100);
    assert_eq!(pillars.disk_score, 100);
    assert_eq!(pillars.memory_score, 100);
    assert_eq!(pillars.cleanup_score, 100);
    assert_eq!(pillars.startup_score, 100);

    // Stressed conditions
    let (stressed, p_stressed) = calculate_health_score(2.0, 96.0, 30_000_000_000, 20);
    assert!(stressed < 30);
    assert_eq!(p_stressed.disk_score, 15);
    assert_eq!(p_stressed.memory_score, 20);
    assert_eq!(p_stressed.cleanup_score, 25);
    assert_eq!(p_stressed.startup_score, 20);
}

#[test]
fn test_live_hardware_overview() {
    let hw = get_hardware_overview();
    assert!(hw.total_memory_bytes > 0, "Total RAM must be > 0");
    assert!(hw.cpu_core_count > 0, "Core count must be > 0");
    assert!(!hw.disks.is_empty(), "Must detect at least one disk");
    assert!(!hw.top_processes.is_empty(), "Must detect running processes");
}

#[test]
fn test_rules_env_expansion() {
    let rules = load_default_rules();
    assert!(!rules.is_empty());

    let temp_rule = rules.iter().find(|r| r.id == "user_temp").unwrap();
    let paths = temp_rule.resolve_paths();
    assert!(!paths.is_empty(), "User temp path must resolve");
    assert!(paths[0].exists(), "Resolved temp path must exist on disk");
}

#[test]
fn test_rescue_bin_full_lifecycle() {
    let temp_dir = std::env::temp_dir().join("vapor_test_staging");
    let _ = fs::create_dir_all(&temp_dir);

    let dummy_file = temp_dir.join("test_dummy_document.txt");
    let dummy_content = "Vapor ClearDeck Staging Test Content 12345";
    fs::write(&dummy_file, dummy_content).expect("Must write dummy file");
    assert!(dummy_file.exists());

    // 1. Stage cleanup
    let dummy_str = dummy_file.to_string_lossy().to_string();
    let stage_res = stage_cleanup_files(&[dummy_str.clone()]).expect("Stage must succeed");
    assert_eq!(stage_res.file_count, 1);
    assert!(!dummy_file.exists(), "Original file should be moved into stage");

    // 2. Verify stage is in list
    let stages = list_rescue_stages();
    assert!(stages.iter().any(|s| s.stage_id == stage_res.stage_id));

    // 3. Restore stage
    let restore_res = restore_stage(&stage_res.stage_id).expect("Restore must succeed");
    assert_eq!(restore_res.restored_count, 1);
    assert!(dummy_file.exists(), "Restored file must exist at original path");

    let restored_content = fs::read_to_string(&dummy_file).expect("Must read restored file");
    assert_eq!(restored_content, dummy_content);

    // 4. Clean up test file and test dir
    let _ = fs::remove_file(&dummy_file);
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_dev_diet_scanner_and_purge() {
    let temp_base = std::env::temp_dir().join("vapor_test_dev_diet");
    let fake_project = temp_base.join("my_fake_web_app");
    let fake_node_modules = fake_project.join("node_modules");
    let fake_pkg_json = fake_project.join("package.json");

    let _ = fs::create_dir_all(&fake_node_modules);
    fs::write(&fake_pkg_json, "{}").expect("Must write fake package.json");

    let dummy_dep = fake_node_modules.join("react.js");
    fs::write(&dummy_dep, "console.log('react')").expect("Must write dummy dep");

    // Scan
    let items = scan_dev_diet(Some(vec![temp_base.to_string_lossy().to_string()]), 0);
    assert!(!items.is_empty(), "Must detect fake node_modules");
    assert_eq!(items[0].kind, "node_modules");

    // Purge
    let reclaimed = purge_dev_directory(&items[0].path).expect("Purge must succeed");
    assert!(reclaimed > 0);
    assert!(!fake_node_modules.exists(), "node_modules must be deleted");

    // Clean up base
    let _ = fs::remove_dir_all(&temp_base);
}

#[test]
fn test_privacy_sanitizer_rules() {
    let sanitized = sanitize_process_info(
        "slack.exe",
        Some("C:\\Users\\SecretUser123\\AppData\\Local\\Slack\\app-4.36.140\\slack.exe"),
        Some("Slack Technologies, LLC"),
        Some("Slack Desktop Communication Platform"),
        2.4,
        185.2,
    );

    assert_eq!(sanitized.process_name, "slack.exe");
    assert!(!sanitized.path_category.contains("SecretUser123"));
    assert_eq!(sanitized.path_category, "AppData/Local/Slack");
}

#[test]
fn test_offline_dictionary_lookup() {
    let dwm = lookup_process("dwm.exe");
    assert!(dwm.is_some());
    let dwm_info = dwm.unwrap();
    assert_eq!(dwm_info.safety, "critical");
    assert!(!dwm_info.can_kill);

    let code = lookup_process("code.exe");
    assert!(code.is_some());
    assert_eq!(code.unwrap().category, "developer");
}

#[test]
fn test_live_windows_installed_software() {
    let apps = get_installed_applications();
    assert!(!apps.is_empty(), "Should audit Windows registry and find installed applications");
}

#[test]
fn test_live_windows_startup_items() {
    // Should run safely without erroring
    let startup_items = get_startup_items();
    println!("Found {} startup items in Windows registry", startup_items.len());
}

#[test]
fn test_stage_cleanup_rules_lifecycle() {
    use vapor_lib::core::rules::matches_wildcard;
    // Verify wildcard matcher handles real cleanup rule patterns
    assert!(matches_wildcard("thumbcache_256.db", "thumbcache_*.db"));
    assert!(matches_wildcard("thumbcache_idx.db", "thumbcache_*.db"));
    assert!(!matches_wildcard("iconcache_256.db", "thumbcache_*.db"));
    assert!(matches_wildcard("temp_file.tmp", "*.tmp"));
    assert!(!matches_wildcard("temp_file.log", "*.tmp"));

    // Stage non-existent rule should gracefully err
    let res = stage_cleanup_rules(&["non_existent_rule_xyz".to_string()]);
    assert!(res.is_err());
}

#[test]
fn test_locked_file_immediate_skip_and_progress() {
    use vapor_lib::storage::rescue_bin::{stage_cleanup_files_with_progress, purge_stage, OperationProgress};
    use std::fs::OpenOptions;
    #[cfg(windows)]
    use std::os::windows::fs::OpenOptionsExt;

    let temp_dir = std::env::temp_dir().join("vapor_test_locked_skip");
    let _ = fs::create_dir_all(&temp_dir);

    let normal_file = temp_dir.join("normal.txt");
    let locked_file = temp_dir.join("locked.txt");
    fs::write(&normal_file, "unlocked content").expect("Must write normal file");
    fs::write(&locked_file, "locked content").expect("Must write locked file");

    // Hold an exclusive handle with share_mode(0) to simulate a running process holding a sharing lock
    #[cfg(windows)]
    let _lock_handle = OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(0)
        .open(&locked_file)
        .expect("Must open locked file handle");

    let paths = vec![
        normal_file.to_string_lossy().to_string(),
        locked_file.to_string_lossy().to_string(),
    ];

    let mut events = Vec::new();
    let res = stage_cleanup_files_with_progress(&paths, Some(|p: OperationProgress| {
        events.push(p);
    })).expect("Staging must succeed for partially locked batch");

    // Normal file was staged
    assert_eq!(res.file_count, 1);
    // Locked file was recorded as skipped_locked
    assert_eq!(res.skipped_locked, 1);
    // Locked file still exists at original path
    assert!(locked_file.exists());

    // Progress events should be recorded
    assert!(!events.is_empty());
    let last = events.last().unwrap();
    assert_eq!(last.processed_count, 2);
    assert_eq!(last.percent, 100.0);
    assert_eq!(last.skipped_locked, 1);

    drop(_lock_handle);
    let _ = purge_stage(&res.stage_id);
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_get_stage_manifest_and_exclusions() {
    use vapor_lib::storage::rescue_bin::{get_stage_manifest, stage_cleanup_files, purge_stage};

    let temp_dir = std::env::temp_dir().join("vapor_test_manifest_inspect");
    let _ = fs::create_dir_all(&temp_dir);

    let file_a = temp_dir.join("file_a.txt");
    let file_b = temp_dir.join("file_b.txt");
    fs::write(&file_a, "alpha").unwrap();
    fs::write(&file_b, "beta").unwrap();

    let paths = vec![
        file_a.to_string_lossy().to_string(),
        file_b.to_string_lossy().to_string(),
    ];

    let stage_res = stage_cleanup_files(&paths).expect("Must stage files");
    assert_eq!(stage_res.file_count, 2);

    // Retrieve manifest
    let manifest = get_stage_manifest(&stage_res.stage_id).expect("Must read stage manifest");
    assert_eq!(manifest.items.len(), 2);
    assert!(manifest.items.iter().any(|i| i.original_path.contains("file_a.txt")));
    assert!(manifest.items.iter().any(|i| i.original_path.contains("file_b.txt")));

    let _ = purge_stage(&stage_res.stage_id);
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_disabled_startup_archive_roundtrip() {
    use vapor_lib::apps::startup::{
        load_disabled_startup_entries, save_disabled_startup_entries, DisabledStartupEntry,
    };

    let mut entries = load_disabled_startup_entries();
    let initial_len = entries.len();

    let test_entry = DisabledStartupEntry {
        name: "VaporUnitTestEntry_XYZ".to_string(),
        command: "C:\\VaporTest\\test.exe".to_string(),
        hive: "HKCU".to_string(),
        impact: "Medium".to_string(),
        disabled_at: "2026-09-10T21:00:00Z".to_string(),
        subkey: "Software\\Microsoft\\Windows\\CurrentVersion\\Run".to_string(),
    };

    entries.push(test_entry.clone());
    save_disabled_startup_entries(&entries).expect("Must save disabled entries");

    let loaded = load_disabled_startup_entries();
    assert_eq!(loaded.len(), initial_len + 1);
    assert!(loaded.iter().any(|e| e.name == "VaporUnitTestEntry_XYZ"));

    // Clean up
    let cleaned: Vec<DisabledStartupEntry> = loaded
        .into_iter()
        .filter(|e| e.name != "VaporUnitTestEntry_XYZ")
        .collect();
    save_disabled_startup_entries(&cleaned).expect("Must clean up test entry");
}

#[test]
fn test_gemini_explanation_caching() {
    use vapor_lib::ai::gemini::{
        clear_explanation_cache, get_cached_explanation, insert_cached_explanation,
        GeminiProcessExplanation,
    };
    use vapor_lib::ai::sanitize::SanitizedProcessMetadata;

    clear_explanation_cache();

    let meta = SanitizedProcessMetadata {
        process_name: "custom_worker.exe".to_string(),
        publisher: Some("DeepMind".to_string()),
        description: Some("Agent worker".to_string()),
        path_category: "ProgramFiles/Vapor".to_string(),
        cpu_percent: 0.8,
        memory_mb: 85.0,
    };

    let exp = GeminiProcessExplanation {
        summary: "Autonomous triage worker".to_string(),
        vendor: "DeepMind".to_string(),
        safety: "safe".to_string(),
        can_terminate: true,
        why_high_usage: "None".to_string(),
        recommendation: "Allow background execution".to_string(),
        sanitized_query: meta,
    };

    insert_cached_explanation("custom_worker.exe", exp);
    assert!(get_cached_explanation("custom_worker.exe").is_some());
    assert!(get_cached_explanation("CUSTOM_WORKER.EXE").is_some());
    assert_eq!(
        get_cached_explanation("custom_worker.exe").unwrap().summary,
        "Autonomous triage worker"
    );
}

#[test]
fn test_toggle_startup_item_roundtrip() {
    use vapor_lib::apps::startup::{toggle_startup_item, load_disabled_startup_entries, save_disabled_startup_entries};
    use winreg::enums::*;
    use winreg::RegKey;

    let test_name = "VaporLiveToggleUnitTest_ABC";
    let test_cmd = "C:\\VaporTest\\toggle_test.exe --run";

    // 1. Write dummy key into HKCU Run
    let root = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(run_key) = root.open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_SET_VALUE) {
        let _ = run_key.set_value(test_name, &test_cmd);

        // 2. Disable: should remove from registry and archive to JSON
        let disable_res = toggle_startup_item(test_name, "HKCU", false);
        assert!(disable_res.is_ok(), "Disabling must succeed: {:?}", disable_res);

        let disabled = load_disabled_startup_entries();
        assert!(disabled.iter().any(|d| d.name == test_name), "Disabled entry must be in JSON archive");

        // Verify removed from registry
        let read_key = root.open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_READ).unwrap();
        assert!(read_key.get_value::<String, _>(test_name).is_err(), "Must be deleted from registry");

        // 3. Enable: should restore to registry and remove from JSON
        let enable_res = toggle_startup_item(test_name, "HKCU", true);
        assert!(enable_res.is_ok(), "Enabling must succeed: {:?}", enable_res);

        let restored_cmd: Result<String, _> = read_key.get_value(test_name);
        assert!(restored_cmd.is_ok(), "Must be restored to registry");
        assert_eq!(restored_cmd.unwrap(), test_cmd);

        // 4. Clean up registry and archive
        let _ = run_key.delete_value(test_name);
        let cleaned: Vec<_> = load_disabled_startup_entries().into_iter().filter(|d| d.name != test_name).collect();
        let _ = save_disabled_startup_entries(&cleaned);
    }
}

#[test]
fn test_get_processes_for_audit() {
    use vapor_lib::process::monitor::get_processes_for_audit;
    let audit_procs = get_processes_for_audit();
    assert!(!audit_procs.is_empty(), "Should return candidates for audit");
    assert!(audit_procs.len() <= 20, "Should cap candidates at 20");

    // Verify all process names are unique (deduplicated)
    let mut names = std::collections::HashSet::new();
    for p in &audit_procs {
        assert!(names.insert(p.name.to_lowercase()), "Process names in audit batch must be deduplicated: {}", p.name);
    }
}

#[test]
fn test_live_pc_specs_and_capabilities() {
    use vapor_lib::system::specs::get_system_specs;
    let specs = get_system_specs();

    assert!(!specs.cpu.model.is_empty(), "CPU model must be detected");
    assert!(specs.cpu.physical_cores > 0, "Physical cores must be > 0");
    assert!(specs.memory.total_gb > 0.0, "Total RAM must be > 0 GB");
    assert!(!specs.primary_storage.drive_letter.is_empty(), "Primary storage must be detected");
    assert!(specs.primary_storage.total_gb > 0.0, "Storage total capacity must be > 0");

    // Dynamic scoring bounds
    assert!(specs.capabilities.office_everyday.score >= 1.0 && specs.capabilities.office_everyday.score <= 10.0);
    assert!(specs.capabilities.software_development.score >= 1.0 && specs.capabilities.software_development.score <= 10.0);
    assert!(specs.capabilities.gaming_3d.score >= 1.0 && specs.capabilities.gaming_3d.score <= 10.0);
    assert!(!specs.capabilities.overall_grade.is_empty());
    assert!(!specs.capabilities.bottleneck_headline.is_empty());
    assert!(!specs.capabilities.bottleneck_explanation.is_empty());
}

#[test]
fn test_live_windows_tweaks_query() {
    use vapor_lib::system::tweaks::get_windows_tweaks;
    let tweaks = get_windows_tweaks();
    assert_eq!(tweaks.len(), 4, "Must expose 4 safe Windows debloat tweaks");

    for tweak in tweaks {
        assert!(tweak.is_safe, "All tweaks must be marked safe and reversible");
        assert!(!tweak.title.is_empty());
        assert!(!tweak.description.is_empty());
    }
}

#[test]
fn test_guardian_process_badges_on_live_system() {
    use vapor_lib::process::monitor::get_hardware_overview;
    let hw = get_hardware_overview();
    assert!(!hw.top_processes.is_empty());

    for proc in &hw.top_processes {
        assert!(!proc.signature_badge.is_empty(), "Every process must have a Guardian signature badge: {}", proc.name);
        assert!(!proc.location_category.is_empty(), "Every process must have a location category: {}", proc.name);
    }
}
