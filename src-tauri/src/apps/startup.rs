use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupItem {
    pub name: String,
    pub command: String,
    pub hive: String, // 'HKCU' | 'HKLM'
    pub impact: String, // 'High' | 'Medium' | 'Low'
    pub can_disable: bool,
    pub enabled: bool,
    pub requires_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisabledStartupEntry {
    pub name: String,
    pub command: String,
    pub hive: String,
    pub impact: String,
    pub disabled_at: String,
    #[serde(default = "default_run_subkey")]
    pub subkey: String,
}

fn default_run_subkey() -> String {
    "Software\\Microsoft\\Windows\\CurrentVersion\\Run".to_string()
}

pub fn is_elevated() -> bool {
    RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_WRITE)
        .is_ok()
}

pub fn get_disabled_startup_path() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    let dir = base.join("Vapor");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("disabled_startup.json")
}

pub fn load_disabled_startup_entries() -> Vec<DisabledStartupEntry> {
    let p = get_disabled_startup_path();
    if !p.exists() {
        return Vec::new();
    }
    match std::fs::read_to_string(&p) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn save_disabled_startup_entries(entries: &[DisabledStartupEntry]) -> Result<(), String> {
    let p = get_disabled_startup_path();
    let json = serde_json::to_string_pretty(entries)
        .map_err(|e| format!("Failed to serialize disabled startup items: {}", e))?;
    std::fs::write(&p, json)
        .map_err(|e| format!("Failed to write disabled startup registry archive: {}", e))?;
    Ok(())
}

pub fn get_startup_items() -> Vec<StartupItem> {
    let mut items = Vec::new();
    let elevated = is_elevated();

    let targets = [
        (HKEY_CURRENT_USER, "Software\\Microsoft\\Windows\\CurrentVersion\\Run", "HKCU"),
        (HKEY_LOCAL_MACHINE, "Software\\Microsoft\\Windows\\CurrentVersion\\Run", "HKLM"),
        (HKEY_LOCAL_MACHINE, "Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Run", "HKLM"),
    ];

    let mut seen = std::collections::HashSet::new();

    for (root_hive, subkey_path, hive_str) in targets {
        let root = RegKey::predef(root_hive);
        if let Ok(run_key) = root.open_subkey_with_flags(subkey_path, KEY_READ) {
            for val_result in run_key.enum_values().flatten() {
                let name = val_result.0;
                let cmd_str: String = run_key.get_value(&name).unwrap_or_default();

                let lower_cmd = cmd_str.to_lowercase();
                let impact = if lower_cmd.contains("electron")
                    || lower_cmd.contains("teams")
                    || lower_cmd.contains("discord")
                    || lower_cmd.contains("spotify")
                    || lower_cmd.contains("steam")
                {
                    "High".to_string()
                } else if lower_cmd.contains("updater") || lower_cmd.contains("helper") {
                    "Low".to_string()
                } else {
                    "Medium".to_string()
                };

                let requires_admin = hive_str == "HKLM" && !elevated;
                seen.insert((name.clone(), hive_str.to_string()));

                items.push(StartupItem {
                    name,
                    command: cmd_str,
                    hive: hive_str.to_string(),
                    impact,
                    can_disable: true,
                    enabled: true,
                    requires_admin,
                });
            }
        }
    }

    // Merge disabled items from disabled_startup.json
    let disabled = load_disabled_startup_entries();
    for d in disabled {
        if !seen.contains(&(d.name.clone(), d.hive.clone())) {
            let requires_admin = d.hive == "HKLM" && !elevated;
            items.push(StartupItem {
                name: d.name,
                command: d.command,
                hive: d.hive,
                impact: d.impact,
                can_disable: true,
                enabled: false,
                requires_admin,
            });
        }
    }

    items
}

pub fn toggle_startup_item(name: &str, hive: &str, enable: bool) -> Result<(), String> {
    let elevated = is_elevated();
    if hive == "HKLM" && !elevated {
        return Err(
            "Administrator permission required to modify System-wide (HKLM) startup entries. Please restart Vapor as Administrator.".to_string()
        );
    }

    let root_hive = if hive == "HKLM" {
        HKEY_LOCAL_MACHINE
    } else {
        HKEY_CURRENT_USER
    };

    let candidate_subkeys: &[&str] = if hive == "HKLM" {
        &[
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            "Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Run",
        ]
    } else {
        &["Software\\Microsoft\\Windows\\CurrentVersion\\Run"]
    };

    let mut disabled_entries = load_disabled_startup_entries();

    if enable {
        // Re-enable an archived entry
        let pos = disabled_entries.iter().position(|e| e.name == name && e.hive == hive);
        let entry = match pos {
            Some(idx) => disabled_entries.remove(idx),
            None => {
                return Err(format!("Entry '{}' not found in disabled startup archive", name));
            }
        };

        let subkey_to_use = if entry.subkey.is_empty() {
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run"
        } else {
            &entry.subkey
        };

        let root = RegKey::predef(root_hive);
        let run_key = root
            .open_subkey_with_flags(subkey_to_use, KEY_SET_VALUE)
            .map_err(|e| format!("Failed to open registry key '{}' with write permissions: {}", subkey_to_use, e))?;

        run_key
            .set_value(name, &entry.command)
            .map_err(|e| format!("Failed to restore startup entry '{}': {}", name, e))?;

        save_disabled_startup_entries(&disabled_entries)?;
    } else {
        // Disable an active entry: check candidates to find which subkey has the value
        let root = RegKey::predef(root_hive);
        let mut found_data: Option<(String, String)> = None; // (command, subkey)

        for &subkey_path in candidate_subkeys {
            if let Ok(read_key) = root.open_subkey_with_flags(subkey_path, KEY_READ) {
                if let Ok(cmd) = read_key.get_value::<String, _>(name) {
                    // Try to delete it
                    if let Ok(write_key) = root.open_subkey_with_flags(subkey_path, KEY_SET_VALUE) {
                        if write_key.delete_value(name).is_ok() {
                            found_data = Some((cmd, subkey_path.to_string()));
                            break;
                        }
                    }
                }
            }
        }

        let (command, subkey_used) = match found_data {
            Some(d) => d,
            None => return Err(format!("Startup entry '{}' not found or could not be removed from registry", name)),
        };

        let lower_cmd = command.to_lowercase();
        let impact = if lower_cmd.contains("electron")
            || lower_cmd.contains("teams")
            || lower_cmd.contains("discord")
            || lower_cmd.contains("spotify")
            || lower_cmd.contains("steam")
        {
            "High".to_string()
        } else if lower_cmd.contains("updater") || lower_cmd.contains("helper") {
            "Low".to_string()
        } else {
            "Medium".to_string()
        };

        disabled_entries.retain(|e| !(e.name == name && e.hive == hive));
        disabled_entries.push(DisabledStartupEntry {
            name: name.to_string(),
            command,
            hive: hive.to_string(),
            impact,
            disabled_at: chrono::Utc::now().to_rfc3339(),
            subkey: subkey_used,
        });

        save_disabled_startup_entries(&disabled_entries)?;
    }

    Ok(())
}

pub fn remove_startup_item(name: &str, hive: &str) -> Result<(), String> {
    toggle_startup_item(name, hive, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_startup_serialization() {
        let entries = vec![DisabledStartupEntry {
            name: "TestApp".to_string(),
            command: "C:\\TestApp\\test.exe --background".to_string(),
            hive: "HKCU".to_string(),
            impact: "High".to_string(),
            disabled_at: "2026-09-10T12:00:00Z".to_string(),
            subkey: "Software\\Microsoft\\Windows\\CurrentVersion\\Run".to_string(),
        }];

        let json = serde_json::to_string(&entries).unwrap();
        let deserialized: Vec<DisabledStartupEntry> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.len(), 1);
        assert_eq!(deserialized[0].name, "TestApp");
        assert_eq!(deserialized[0].hive, "HKCU");
        assert_eq!(deserialized[0].subkey, "Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    }
}
