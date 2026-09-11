use super::prefetch::get_executable_activity_map;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::time::{Duration, SystemTime};
use winreg::enums::*;
use winreg::RegKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledApp {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub publisher: Option<String>,
    pub install_date: Option<String>,
    pub size_bytes: u64,
    pub install_location: Option<String>,
    pub uninstall_string: Option<String>,
    pub estimated_last_used_secs: Option<u64>,
    pub days_since_last_used: Option<u32>,
    pub is_zombie: bool,
    pub is_bloatware: bool,
}

pub fn get_installed_applications() -> Vec<InstalledApp> {
    let activity_map = get_executable_activity_map();
    let now = SystemTime::now();

    let mut apps: Vec<InstalledApp> = Vec::new();
    let mut seen_names = HashSet::new();

    let registry_targets = [
        (HKEY_LOCAL_MACHINE, "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        (HKEY_LOCAL_MACHINE, "Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        (HKEY_CURRENT_USER, "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
    ];

    for (root_hive, subkey_path) in registry_targets {
        let root = RegKey::predef(root_hive);
        if let Ok(uninstall_key) = root.open_subkey_with_flags(subkey_path, KEY_READ) {
            for subkey_name in uninstall_key.enum_keys().flatten() {
                if let Ok(app_key) = uninstall_key.open_subkey_with_flags(&subkey_name, KEY_READ) {
                    let display_name: String = match app_key.get_value("DisplayName") {
                        Ok(name) => name,
                        Err(_) => continue, // Skip if no display name
                    };

                    let clean_name = display_name.trim();
                    if clean_name.is_empty() || seen_names.contains(clean_name) {
                        continue;
                    }

                    // Check if system component / Windows update
                    let is_system_component: u32 = app_key.get_value("SystemComponent").unwrap_or(0);
                    let parent_key: String = app_key.get_value("ParentKeyName").unwrap_or_default();
                    if is_system_component != 0 || !parent_key.is_empty() {
                        continue;
                    }

                    seen_names.insert(clean_name.to_string());

                    let version: Option<String> = app_key.get_value("DisplayVersion").ok();
                    let publisher: Option<String> = app_key.get_value("Publisher").ok();
                    let install_date: Option<String> = app_key.get_value("InstallDate").ok();
                    let uninstall_string: Option<String> = app_key.get_value("UninstallString").ok();
                    let install_location: Option<String> = app_key.get_value("InstallLocation").ok();
                    let display_icon: Option<String> = app_key.get_value("DisplayIcon").ok();

                    // Size in registry is in KB
                    let size_kb: u32 = app_key.get_value("EstimatedSize").unwrap_or(0);
                    let mut size_bytes = (size_kb as u64) * 1024;

                    // If registry size is 0 and install_location exists, try reading folder size if reasonable
                    if size_bytes == 0 {
                        if let Some(ref loc) = install_location {
                            let p = Path::new(loc);
                            if p.exists() && p.is_dir() {
                                if let Ok(entries) = std::fs::read_dir(p) {
                                    let mut folder_sum = 0u64;
                                    for f in entries.flatten().take(100) {
                                        if let Ok(m) = f.metadata() {
                                            folder_sum += m.len();
                                        }
                                    }
                                    size_bytes = folder_sum;
                                }
                            }
                        }
                    }

                    let last_used_secs = match_app_last_used(
                        clean_name,
                        display_icon.as_deref(),
                        install_location.as_deref(),
                        &activity_map,
                    );

                    let days_since_last_used = last_used_secs.map(|secs| {
                        let then = SystemTime::UNIX_EPOCH + Duration::from_secs(secs);
                        let diff = now.duration_since(then).unwrap_or(Duration::ZERO);
                        (diff.as_secs() / 86400) as u32
                    });

                    let is_zombie = is_app_zombie(size_bytes, days_since_last_used, install_date.as_deref());

                    // Bloatware heuristic
                    let lower_name = clean_name.to_lowercase();
                    let lower_pub = publisher.as_deref().unwrap_or("").to_lowercase();
                    let is_bloatware = lower_name.contains("telemetry")
                        || lower_name.contains("assistant")
                        || lower_name.contains("toolbar")
                        || lower_name.contains("updater helper")
                        || lower_name.contains("customer experience")
                        || lower_pub.contains("wildtangent")
                        || lower_pub.contains("mywebsearch")
                        || lower_pub.contains("conduit");

                    apps.push(InstalledApp {
                        id: subkey_name,
                        name: clean_name.to_string(),
                        version,
                        publisher,
                        install_date,
                        size_bytes,
                        install_location,
                        uninstall_string,
                        estimated_last_used_secs: last_used_secs,
                        days_since_last_used,
                        is_zombie,
                        is_bloatware,
                    });
                }
            }
        }
    }

    apps.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    apps
}

pub fn match_app_last_used(
    clean_name: &str,
    display_icon: Option<&str>,
    install_location: Option<&str>,
    activity_map: &std::collections::HashMap<String, u64>,
) -> Option<u64> {
    let lower_name = clean_name.to_lowercase();

    // 1. Try matching against executable extracted from DisplayIcon
    let icon_stem = display_icon.and_then(|icon| {
        let clean = icon.trim_matches('"').split(',').next()?.trim();
        let p = Path::new(clean);
        p.file_stem().map(|s| s.to_string_lossy().to_lowercase())
    });

    if let Some(ref stem) = icon_stem {
        if let Some(&secs) = activity_map.get(stem) {
            return Some(secs);
        }
    }

    // 2. Try matching against executable / folder name in InstallLocation
    if let Some(loc) = install_location {
        let p = Path::new(loc.trim_matches('"'));
        if let Some(folder_stem) = p.file_name().map(|s| s.to_string_lossy().to_lowercase()) {
            if let Some(&secs) = activity_map.get(&folder_stem) {
                return Some(secs);
            }
        }
    }

    // 3. Substring / word matching with activity map keys
    let words: Vec<&str> = lower_name
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3)
        .collect();

    let mut best_time: Option<u64> = None;
    for (exe_key, &secs) in activity_map {
        let clean_key = exe_key.trim_end_matches(".exe");
        if clean_key.len() < 3 {
            continue;
        }
        let matches = words.contains(&clean_key)
            || lower_name.contains(clean_key)
            || (clean_key.len() >= 4 && lower_name.starts_with(clean_key));
        if matches {
            best_time = Some(best_time.map_or(secs, |prev| prev.max(secs)));
        }
    }

    best_time
}

pub fn is_app_zombie(
    size_bytes: u64,
    days_since_last_used: Option<u32>,
    install_date: Option<&str>,
) -> bool {
    // Implementation Plan: [Zombie App] (>60 days since estimated launch, size > 500MB)
    if size_bytes < 500 * 1024 * 1024 {
        return false;
    }

    match days_since_last_used {
        Some(days) => days >= 60,
        None => {
            if let Some(idate) = install_date {
                if idate.len() == 8 && idate.chars().all(|c| c.is_ascii_digit()) {
                    if let (Ok(y), Ok(m), Ok(d)) = (
                        idate[0..4].parse::<i32>(),
                        idate[4..6].parse::<u32>(),
                        idate[6..8].parse::<u32>(),
                    ) {
                        if let Some(date) = chrono::NaiveDate::from_ymd_opt(y, m, d) {
                            let today = chrono::Local::now().date_naive();
                            (today - date).num_days() >= 60
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_match_app_last_used_chrome_to_prefetch() {
        let mut map = HashMap::new();
        map.insert("chrome".to_string(), 1700000000);
        map.insert("chrome.exe".to_string(), 1700000000);

        let res = match_app_last_used("Google Chrome", None, None, &map);
        assert_eq!(res, Some(1700000000), "Google Chrome must match chrome in prefetch");
    }

    #[test]
    fn test_is_app_zombie_active_app_not_zombie() {
        // App used 2 days ago, size 1GB -> NOT zombie
        let zombie = is_app_zombie(1024 * 1024 * 1024, Some(2), None);
        assert!(!zombie, "App used 2 days ago must not be a zombie");

        // App used 75 days ago, size 1GB -> ZOMBIE
        let old_zombie = is_app_zombie(1024 * 1024 * 1024, Some(75), None);
        assert!(old_zombie, "App unused for 75 days must be marked as zombie");

        // Small app (100MB) unused for 100 days -> NOT zombie (< 500MB)
        let small = is_app_zombie(100 * 1024 * 1024, Some(100), None);
        assert!(!small, "App smaller than 500MB must not be marked as zombie");
    }
}
