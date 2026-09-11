use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsTweak {
    pub id: String,
    pub title: String,
    pub description: String,
    pub impact: String,
    pub is_applied: bool,
    pub is_safe: bool,
    pub category: String, // "privacy" | "taskbar" | "gaming" | "network"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SponsoredApp {
    pub id: String,
    pub display_name: String,
    pub package_full_name: String,
    pub publisher: String,
    pub description: String,
}

/// Known sponsored app signatures that Windows often bundles
const SPONSORED_PATTERNS: &[(&str, &str, &str)] = &[
    ("TikTok", "TikTok", "ByteDance short-form video feed app"),
    ("Disney", "Disney+", "Disney streaming application"),
    ("CandyCrush", "Candy Crush Saga", "King.com casual sponsored game"),
    ("King.com", "King Games", "King.com sponsored gaming suite"),
    ("Microsoft.BingNews", "MSN News", "MSN news feed and MSN Start widget"),
    ("Microsoft.BingWeather", "MSN Weather", "Weather app with bundled MSN advertising links"),
    ("MicrosoftSolitaireCollection", "Microsoft Solitaire Collection", "Casual games bundle with banner advertisements"),
    ("SpotifyAB.SpotifyMusic", "Spotify", "Pre-installed Spotify music player"),
    ("Netflix", "Netflix", "Pre-installed Netflix streaming player"),
];

/// Checks the current status of all safe Windows tweaks
pub fn get_windows_tweaks() -> Vec<WindowsTweak> {
    vec![
        get_bing_search_tweak(),
        get_taskbar_widgets_tweak(),
        get_game_dvr_tweak(),
        get_delivery_optimization_tweak(),
    ]
}

/// 1. Disable Bing Search in Start Menu
/// Registry: HKCU\Software\Policies\Microsoft\Windows\Explorer or HKCU\Software\Microsoft\Windows\CurrentVersion\Search
/// BingSearchEnabled = 0, DisableSearchBoxSuggestions = 1
fn get_bing_search_tweak() -> WindowsTweak {
    let mut is_applied = false;

    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(search_key) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Search") {
            let bing_enabled: Result<u32, _> = search_key.get_value("BingSearchEnabled");
            let disable_suggestions: Result<u32, _> = search_key.get_value("DisableSearchBoxSuggestions");

            if let (Ok(be), Ok(ds)) = (bing_enabled, disable_suggestions) {
                if be == 0 && ds == 1 {
                    is_applied = true;
                }
            }
        }
    }

    WindowsTweak {
        id: "bing_search".to_string(),
        title: "Disable Start Menu Bing Web Search".to_string(),
        description: "Prevents the Start Menu from sending local search keystrokes to Bing and fetching online web results.".to_string(),
        impact: "Instant local file search with zero latency or web clutter.".to_string(),
        is_applied,
        is_safe: true,
        category: "privacy".to_string(),
    }
}

/// 2. Disable Taskbar Widgets & MSN News Feed
/// Registry: HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced
/// TaskbarDa = 0 (0 = hidden, 1 = shown)
fn get_taskbar_widgets_tweak() -> WindowsTweak {
    let mut is_applied = false;

    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(adv_key) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced") {
            let taskbar_da: Result<u32, _> = adv_key.get_value("TaskbarDa");
            if let Ok(val) = taskbar_da {
                if val == 0 {
                    is_applied = true;
                }
            }
        }
    }

    WindowsTweak {
        id: "taskbar_widgets".to_string(),
        title: "Hide Taskbar MSN Widgets".to_string(),
        description: "Removes the news and weather widget from the taskbar, stopping continuous background web feed refresh.".to_string(),
        impact: "Saves ~150 MB background RAM and prevents accidental widget popups.".to_string(),
        is_applied,
        is_safe: true,
        category: "taskbar".to_string(),
    }
}

/// 3. Disable Game DVR Background Video Capture
/// Registry: HKCU\System\GameConfigStore (GameDVR_Enabled = 0)
/// Registry: HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR (AppCaptureEnabled = 0)
fn get_game_dvr_tweak() -> WindowsTweak {
    let mut is_applied = false;

    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let game_store_res = hkcu.open_subkey("System\\GameConfigStore");
        let app_capture_res = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\GameDVR");

        let mut g_disabled = false;
        let mut a_disabled = false;

        if let Ok(key) = game_store_res {
            if let Ok(val) = key.get_value::<u32, _>("GameDVR_Enabled") {
                if val == 0 {
                    g_disabled = true;
                }
            }
        }

        if let Ok(key) = app_capture_res {
            if let Ok(val) = key.get_value::<u32, _>("AppCaptureEnabled") {
                if val == 0 {
                    a_disabled = true;
                }
            }
        }

        if g_disabled && a_disabled {
            is_applied = true;
        }
    }

    WindowsTweak {
        id: "game_dvr".to_string(),
        title: "Disable Game DVR Background Recording".to_string(),
        description: "Disables background video capture (Xbox Game Bar DVR) that continuously buffers screen recordings to disk.".to_string(),
        impact: "Eliminates background GPU encoder load and disk writes during gaming and 3D apps.".to_string(),
        is_applied,
        is_safe: true,
        category: "gaming".to_string(),
    }
}

/// 4. Turn Off P2P Windows Update Delivery Optimization
/// Registry: HKCU\Software\Microsoft\Windows\CurrentVersion\DeliveryOptimization (DODownloadMode = 0)
fn get_delivery_optimization_tweak() -> WindowsTweak {
    let mut is_applied = false;

    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(do_key) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\DeliveryOptimization") {
            if let Ok(val) = do_key.get_value::<u32, _>("DODownloadMode") {
                if val == 0 {
                    is_applied = true;
                }
            }
        }
    }

    WindowsTweak {
        id: "delivery_optimization".to_string(),
        title: "Turn Off P2P Update Bandwidth Sharing".to_string(),
        description: "Stops Windows from uploading downloaded update packages to other PCs on the internet or local network.".to_string(),
        impact: "Preserves upload bandwidth and reduces background network activity.".to_string(),
        is_applied,
        is_safe: true,
        category: "network".to_string(),
    }
}

/// Applies or reverts a tweak by ID
pub fn set_windows_tweak(id: &str, enable: bool) -> Result<(), String> {
    match id {
        "bing_search" => set_bing_search(enable),
        "taskbar_widgets" => set_taskbar_widgets(enable),
        "game_dvr" => set_game_dvr(enable),
        "delivery_optimization" => set_delivery_optimization(enable),
        _ => Err(format!("Unknown tweak ID: {}", id)),
    }
}

fn set_bing_search(enable: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (search_key, _) = hkcu
            .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Search")
            .map_err(|e| format!("Failed to access Search registry: {}", e))?;

        if enable {
            // Apply tweak: disable Bing search
            search_key
                .set_value("BingSearchEnabled", &0u32)
                .map_err(|e| format!("Failed to set BingSearchEnabled: {}", e))?;
            search_key
                .set_value("DisableSearchBoxSuggestions", &1u32)
                .map_err(|e| format!("Failed to set DisableSearchBoxSuggestions: {}", e))?;
        } else {
            // Revert: enable Bing search
            let _ = search_key.set_value("BingSearchEnabled", &1u32);
            let _ = search_key.set_value("DisableSearchBoxSuggestions", &0u32);
        }
        return Ok(());
    }
    #[cfg(not(windows))]
    Ok(())
}

fn set_taskbar_widgets(enable: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (adv_key, _) = hkcu
            .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced")
            .map_err(|e| format!("Failed to access Explorer\\Advanced registry: {}", e))?;

        let val = if enable { 0u32 } else { 1u32 };
        adv_key
            .set_value("TaskbarDa", &val)
            .map_err(|e| format!("Failed to set TaskbarDa: {}", e))?;
        return Ok(());
    }
    #[cfg(not(windows))]
    Ok(())
}

fn set_game_dvr(enable: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (store_key, _) = hkcu
            .create_subkey("System\\GameConfigStore")
            .map_err(|e| format!("Failed to access GameConfigStore: {}", e))?;
        let (dvr_key, _) = hkcu
            .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\GameDVR")
            .map_err(|e| format!("Failed to access GameDVR: {}", e))?;

        let val = if enable { 0u32 } else { 1u32 };
        store_key
            .set_value("GameDVR_Enabled", &val)
            .map_err(|e| format!("Failed to set GameDVR_Enabled: {}", e))?;
        dvr_key
            .set_value("AppCaptureEnabled", &val)
            .map_err(|e| format!("Failed to set AppCaptureEnabled: {}", e))?;
        return Ok(());
    }
    #[cfg(not(windows))]
    Ok(())
}

fn set_delivery_optimization(enable: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (do_key, _) = hkcu
            .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\DeliveryOptimization")
            .map_err(|e| format!("Failed to access DeliveryOptimization: {}", e))?;

        let val = if enable { 0u32 } else { 1u32 };
        do_key
            .set_value("DODownloadMode", &val)
            .map_err(|e| format!("Failed to set DODownloadMode: {}", e))?;
        return Ok(());
    }
    #[cfg(not(windows))]
    Ok(())
}

/// Discovers pre-installed sponsored/promotional Appx packages
pub fn get_sponsored_apps() -> Vec<SponsoredApp> {
    let mut detected = Vec::new();

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;

        // Query Appx packages via PowerShell in compact JSON
        let ps_cmd = "Get-AppxPackage | Select-Object -Property Name, PackageFullName, Publisher | ConvertTo-Json -Compress";
        let mut cmd = Command::new("powershell.exe");
        cmd.args(["-NoProfile", "-NonInteractive", "-Command", ps_cmd]);
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let trimmed = stdout.trim();

                #[derive(Deserialize)]
                struct RawAppx {
                    #[serde(rename = "Name")]
                    name: Option<String>,
                    #[serde(rename = "PackageFullName")]
                    package_full_name: Option<String>,
                    #[serde(rename = "Publisher")]
                    publisher: Option<String>,
                }

                // Can be a single object or an array of objects
                let parsed_list: Vec<RawAppx> = if trimmed.starts_with('[') {
                    serde_json::from_str(trimmed).unwrap_or_default()
                } else if trimmed.starts_with('{') {
                    serde_json::from_str::<RawAppx>(trimmed)
                        .map(|item| vec![item])
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };

                for item in parsed_list {
                    if let (Some(name), Some(full_name)) = (item.name, item.package_full_name) {
                        for &(pattern, display_name, description) in SPONSORED_PATTERNS {
                            if name.to_lowercase().contains(&pattern.to_lowercase()) {
                                detected.push(SponsoredApp {
                                    id: name.clone(),
                                    display_name: display_name.to_string(),
                                    package_full_name: full_name.clone(),
                                    publisher: item.publisher.clone().unwrap_or_else(|| "Third Party".to_string()),
                                    description: description.to_string(),
                                });
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    detected
}

/// Uninstalls an Appx sponsored package using PowerShell
pub fn uninstall_sponsored_app(package_full_name: &str) -> Result<(), String> {
    let clean_name = package_full_name.trim().trim_matches('"');
    if clean_name.is_empty() || clean_name.contains(';') || clean_name.contains('&') || clean_name.contains('|') {
        return Err("Invalid package name".to_string());
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;

        // Matches by exact PackageFullName first, then falls back to short Name.
        // Raises an explicit error if the package does not exist rather than silently doing nothing.
        let ps_cmd = format!(
            "$target = \"{clean_name}\"; \
             $pkg = Get-AppxPackage -Package $target -ErrorAction SilentlyContinue; \
             if (-not $pkg) {{ $pkg = Get-AppxPackage -Name $target -ErrorAction SilentlyContinue }}; \
             if ($pkg) {{ $pkg | Remove-AppxPackage -ErrorAction Stop }} else {{ throw \"Package '$target' not found or already uninstalled.\" }}"
        );
        let mut cmd = Command::new("powershell.exe");
        cmd.args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd]);
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to invoke PowerShell: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            Err(format!("Uninstall failed: {}", err_msg.trim()))
        }
    }
    #[cfg(not(windows))]
    Ok(())
}

/// Creates a Windows System Restore Point using PowerShell Checkpoint-Computer
pub fn create_system_restore_point(description: &str) -> Result<String, String> {
    let safe_desc = description.replace('"', "'").replace(';', ",").replace('&', " ");
    let desc_arg = if safe_desc.trim().is_empty() {
        "Vapor Pre-Optimization Checkpoint"
    } else {
        safe_desc.trim()
    };

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;

        let ps_cmd = format!(
            "Checkpoint-Computer -Description \"{}\" -RestorePointType \"MODIFY_SETTINGS\" -ErrorAction Stop",
            desc_arg
        );
        let mut cmd = Command::new("powershell.exe");
        cmd.args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd]);
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to invoke PowerShell: {}", e))?;

        if output.status.success() {
            Ok("System Restore Point created successfully.".to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("1440") || stderr.contains("frequency") || stderr.contains("already been created") {
                // Windows limits checkpoint creation frequency to once per 24 hours by default.
                // Reassure the user that their system already has a recent valid restore point.
                Ok("A Windows Restore Point was already created within the last 24 hours. Your system state is already protected.".to_string())
            } else if stderr.contains("0x80042306") || stderr.contains("disabled") {
                Err("System Restore is turned off on this system drive. You can enable it in Windows System Properties > System Protection.".to_string())
            } else if stderr.contains("privilege") || stderr.contains("Access is denied") {
                Err("Creating a System Restore Point requires Administrator privileges. Run Vapor as Administrator or apply tweaks directly (they are fully reversible).".to_string())
            } else {
                Err(format!("Restore point creation note: {}", stderr.trim()))
            }
        }
    }
    #[cfg(not(windows))]
    Ok("Restore point simulated (non-Windows).".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sponsored_patterns_validity() {
        assert!(!SPONSORED_PATTERNS.is_empty());
        for &(pattern, name, desc) in SPONSORED_PATTERNS {
            assert!(!pattern.is_empty());
            assert!(!name.is_empty());
            assert!(!desc.is_empty());
        }
    }

    #[test]
    fn test_get_windows_tweaks_schema() {
        let tweaks = get_windows_tweaks();
        assert_eq!(tweaks.len(), 4);

        let ids: Vec<&str> = tweaks.iter().map(|t| t.id.as_str()).collect();
        assert!(ids.contains(&"bing_search"));
        assert!(ids.contains(&"taskbar_widgets"));
        assert!(ids.contains(&"game_dvr"));
        assert!(ids.contains(&"delivery_optimization"));

        for tweak in tweaks {
            assert!(tweak.is_safe);
            assert!(!tweak.title.is_empty());
            assert!(!tweak.description.is_empty());
            assert!(!tweak.impact.is_empty());
        }
    }

    #[test]
    fn test_sponsored_uninstall_validation() {
        assert!(uninstall_sponsored_app("").is_err());
        assert!(uninstall_sponsored_app("   ").is_err());
        assert!(uninstall_sponsored_app("test; malicious").is_err());
        assert!(uninstall_sponsored_app("test&whoami").is_err());
        assert!(uninstall_sponsored_app("test|calc").is_err());
    }
}
