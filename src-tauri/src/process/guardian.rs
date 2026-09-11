use super::known_db::lookup_process;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignatureBadge {
    VerifiedMicrosoft,
    VerifiedPublisher,
    Unsigned,
    UnsignedSuspiciousLocation,
}

impl SignatureBadge {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignatureBadge::VerifiedMicrosoft => "Verified (Microsoft)",
            SignatureBadge::VerifiedPublisher => "Verified (Known Publisher)",
            SignatureBadge::Unsigned => "Unsigned",
            SignatureBadge::UnsignedSuspiciousLocation => "Unsigned in Temp/AppData",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessLocationHeuristics {
    pub is_in_temp: bool,
    pub is_in_appdata: bool,
    pub is_in_downloads: bool,
    pub is_suspicious_location: bool,
    pub location_label: String,
}

/// Analyzes executable path heuristics
pub fn analyze_path_heuristics(exe_path: Option<&str>) -> ProcessLocationHeuristics {
    let path_str = match exe_path {
        Some(p) if !p.trim().is_empty() => p.to_lowercase(),
        _ => {
            return ProcessLocationHeuristics {
                is_in_temp: false,
                is_in_appdata: false,
                is_in_downloads: false,
                is_suspicious_location: false,
                location_label: "Unknown / System".to_string(),
            };
        }
    };

    let is_in_temp = path_str.contains("\\temp\\")
        || path_str.contains("\\appdata\\local\\temp")
        || path_str.contains("/temp/")
        || path_str.contains("temporary internet files")
        || path_str.ends_with("\\temp")
        || path_str.ends_with("/temp");

    let is_in_local_programs = path_str.contains("\\appdata\\local\\programs\\")
        || path_str.contains("/appdata/local/programs/");

    // Legitimate user apps frequently install to AppData\Local\Programs (e.g., VS Code, Chrome).
    // Flag other AppData locations like Roaming root or Local root where droppers run.
    let is_in_appdata = (path_str.contains("\\appdata\\") || path_str.contains("/appdata/"))
        && !is_in_local_programs;

    let is_in_downloads = path_str.contains("\\downloads\\") || path_str.contains("/downloads/");

    let is_suspicious_location = is_in_temp || is_in_appdata || is_in_downloads;

    let location_label = if path_str.contains("\\windows\\system32") || path_str.contains("/windows/system32") {
        "Windows System32".to_string()
    } else if path_str.contains("\\windows\\") || path_str.contains("/windows/") {
        "Windows Directory".to_string()
    } else if path_str.contains("\\program files") || path_str.contains("/program files") {
        "Program Files".to_string()
    } else if is_in_local_programs {
        "User Programs (AppData)".to_string()
    } else if is_in_temp {
        "Temporary Directory".to_string()
    } else if is_in_downloads {
        "Downloads Directory".to_string()
    } else if is_in_appdata {
        "User AppData".to_string()
    } else {
        "Standard Directory".to_string()
    };

    ProcessLocationHeuristics {
        is_in_temp,
        is_in_appdata,
        is_in_downloads,
        is_suspicious_location,
        location_label,
    }
}

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

static SIGNATURE_CACHE: LazyLock<Mutex<HashMap<String, (bool, bool)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static PE_INFO_CACHE: LazyLock<Mutex<HashMap<String, (Option<String>, Option<String>)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[cfg(windows)]
type WinVerifyTrustFn = unsafe extern "system" fn(
    hwnd: *mut std::ffi::c_void,
    action_id: *const GUID,
    wvt_data: *mut WINTRUST_DATA,
) -> i32;

#[cfg(windows)]
#[repr(C)]
struct GUID {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

#[cfg(windows)]
#[repr(C)]
struct WINTRUST_FILE_INFO {
    cb_struct: u32,
    pcwsz_file_path: *const u16,
    h_file: *mut std::ffi::c_void,
    pg_known_subject: *const std::ffi::c_void,
}

#[cfg(windows)]
#[repr(C)]
struct WINTRUST_DATA {
    cb_struct: u32,
    p_policy_callback_data: *mut std::ffi::c_void,
    p_sip_client_data: *mut std::ffi::c_void,
    dw_ui_choice: u32,
    fdw_revocation_checks: u32,
    dw_union_choice: u32,
    p_file: *mut WINTRUST_FILE_INFO,
    dw_state_action: u32,
    h_wvt_state_data: *mut std::ffi::c_void,
    pwsz_url_reference: *const u16,
    dw_prov_flags: u32,
    dw_ui_context: u32,
    p_signature_settings: *mut std::ffi::c_void,
}

#[cfg(windows)]
static WINTRUST_FN: LazyLock<Option<WinVerifyTrustFn>> = LazyLock::new(|| unsafe {
    extern "system" {
        fn LoadLibraryA(lpLibFileName: *const u8) -> *mut std::ffi::c_void;
        fn GetProcAddress(
            hModule: *mut std::ffi::c_void,
            lpProcName: *const u8,
        ) -> *mut std::ffi::c_void;
    }
    let wintrust = LoadLibraryA(b"wintrust.dll\0".as_ptr());
    if wintrust.is_null() {
        None
    } else {
        let proc = GetProcAddress(wintrust, b"WinVerifyTrust\0".as_ptr());
        if proc.is_null() {
            None
        } else {
            Some(std::mem::transmute(proc))
        }
    }
});

/// Extracts CompanyName and FileDescription directly from PE version information
pub fn get_pe_version_info(exe_path: &str) -> (Option<String>, Option<String>) {
    let lower = exe_path.to_lowercase();
    if let Ok(guard) = PE_INFO_CACHE.lock() {
        if let Some(cached) = guard.get(&lower) {
            return cached.clone();
        }
    }

    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let path = Path::new(exe_path);
        if !path.exists() || !path.is_file() {
            return (None, None);
        }

        let wide_path: Vec<u16> = OsStr::new(exe_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        extern "system" {
            fn LoadLibraryA(lpLibFileName: *const u8) -> *mut std::ffi::c_void;
            fn GetProcAddress(
                hModule: *mut std::ffi::c_void,
                lpProcName: *const u8,
            ) -> *mut std::ffi::c_void;
        }

        type GetFileVersionInfoSizeWFn =
            unsafe extern "system" fn(lptstr_filename: *const u16, lpdw_handle: *mut u32) -> u32;
        type GetFileVersionInfoWFn = unsafe extern "system" fn(
            lptstr_filename: *const u16,
            dw_handle: u32,
            dw_len: u32,
            lp_data: *mut u8,
        ) -> i32;
        type VerQueryValueWFn = unsafe extern "system" fn(
            p_block: *const u8,
            lp_sub_block: *const u16,
            lplp_buffer: *mut *const u16,
            pu_len: *mut u32,
        ) -> i32;

        static VERSION_FNS: LazyLock<
            Option<(
                GetFileVersionInfoSizeWFn,
                GetFileVersionInfoWFn,
                VerQueryValueWFn,
            )>,
        > = LazyLock::new(|| unsafe {
            let ver_mod = LoadLibraryA(b"version.dll\0".as_ptr());
            if ver_mod.is_null() {
                None
            } else {
                let size_proc = GetProcAddress(ver_mod, b"GetFileVersionInfoSizeW\0".as_ptr());
                let get_proc = GetProcAddress(ver_mod, b"GetFileVersionInfoW\0".as_ptr());
                let query_proc = GetProcAddress(ver_mod, b"VerQueryValueW\0".as_ptr());

                if !size_proc.is_null() && !get_proc.is_null() && !query_proc.is_null() {
                    Some((
                        std::mem::transmute(size_proc),
                        std::mem::transmute(get_proc),
                        std::mem::transmute(query_proc),
                    ))
                } else {
                    None
                }
            }
        });

        if let Some((get_size, get_info, query_val)) = *VERSION_FNS {
            unsafe {
                let mut handle = 0u32;
                let size = get_size(wide_path.as_ptr(), &mut handle);
                if size > 0 && size < 65536 {
                    let mut buffer = vec![0u8; size as usize];
                    if get_info(wide_path.as_ptr(), 0, size, buffer.as_mut_ptr()) != 0 {
                        let trans_sub: Vec<u16> = OsStr::new("\\VarFileInfo\\Translation")
                            .encode_wide()
                            .chain(std::iter::once(0))
                            .collect();

                        let mut trans_ptr: *const u16 = std::ptr::null();
                        let mut trans_len = 0u32;

                        let (lang, code_page) = if query_val(
                            buffer.as_ptr(),
                            trans_sub.as_ptr(),
                            &mut trans_ptr,
                            &mut trans_len,
                        ) != 0
                            && trans_len >= 4
                            && !trans_ptr.is_null()
                        {
                            let words = std::slice::from_raw_parts(trans_ptr, 2);
                            (words[0], words[1])
                        } else {
                            (0x0409, 0x04b0) // US English, Unicode default
                        };

                        let query_string = |sub_block_name: &str| -> Option<String> {
                            let query_path = format!(
                                "\\StringFileInfo\\{:04x}{:04x}\\{}",
                                lang, code_page, sub_block_name
                            );
                            let sub_wide: Vec<u16> = OsStr::new(&query_path)
                                .encode_wide()
                                .chain(std::iter::once(0))
                                .collect();
                            let mut str_ptr: *const u16 = std::ptr::null();
                            let mut str_len = 0u32;
                            if query_val(
                                buffer.as_ptr(),
                                sub_wide.as_ptr(),
                                &mut str_ptr,
                                &mut str_len,
                            ) != 0
                                && str_len > 0
                                && !str_ptr.is_null()
                            {
                                let slice = std::slice::from_raw_parts(str_ptr, str_len as usize);
                                let end = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
                                let val = String::from_utf16_lossy(&slice[..end]).trim().to_string();
                                if !val.is_empty() {
                                    return Some(val);
                                }
                            }
                            None
                        };

                        let company = query_string("CompanyName");
                        let desc = query_string("FileDescription");
                        let res = (company, desc);

                        if let Ok(mut guard) = PE_INFO_CACHE.lock() {
                            guard.insert(lower, res.clone());
                        }
                        return res;
                    }
                }
            }
        }
    }

    (None, None)
}

/// Checks the cryptographic digital signature of an executable using Win32 WinVerifyTrust.
/// Results are persistently cached in memory to avoid repeated disk I/O and loader lock contention.
pub fn verify_executable_signature(path: &str) -> (bool, bool) {
    let norm_path = path.to_lowercase();
    if let Ok(guard) = SIGNATURE_CACHE.lock() {
        if let Some(&res) = guard.get(&norm_path) {
            return res;
        }
    }

    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let p = Path::new(path);
        if !p.exists() || !p.is_file() {
            return (false, false);
        }

        if let Some(win_verify_trust) = *WINTRUST_FN {
            let wide_path: Vec<u16> = OsStr::new(path)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            // WINTRUST_ACTION_GENERIC_VERIFY_V2: {00AAC56B-CD44-11d0-8CC2-00C04FC2AAE1}
            let action_generic_verify_v2 = GUID {
                data1: 0x00aac56b,
                data2: 0xcd44,
                data3: 0x11d0,
                data4: [0x8c, 0xc2, 0x00, 0xc0, 0x4f, 0xc2, 0xaa, 0xe1],
            };

            let mut file_info = WINTRUST_FILE_INFO {
                cb_struct: std::mem::size_of::<WINTRUST_FILE_INFO>() as u32,
                pcwsz_file_path: wide_path.as_ptr(),
                h_file: std::ptr::null_mut(),
                pg_known_subject: std::ptr::null_mut(),
            };

            let mut wvt_data = WINTRUST_DATA {
                cb_struct: std::mem::size_of::<WINTRUST_DATA>() as u32,
                p_policy_callback_data: std::ptr::null_mut(),
                p_sip_client_data: std::ptr::null_mut(),
                dw_ui_choice: 2,           // WTD_UI_NONE = 2
                fdw_revocation_checks: 0,  // WTD_REVOKE_NONE = 0
                dw_union_choice: 1,        // WTD_CHOICE_FILE = 1
                p_file: &mut file_info as *mut _,
                dw_state_action: 0,        // WTD_STATEACTION_IGNORE = 0
                h_wvt_state_data: std::ptr::null_mut(),
                pwsz_url_reference: std::ptr::null(),
                dw_prov_flags: 0x1000,     // WTD_CACHE_ONLY_URL_RETRIEVAL = 0x1000
                dw_ui_context: 0,
                p_signature_settings: std::ptr::null_mut(),
            };

            let hr = unsafe {
                win_verify_trust(
                    std::ptr::null_mut(),
                    &action_generic_verify_v2,
                    &mut wvt_data,
                )
            };

            let is_signed = hr == 0;
            // Verify path is in official non-writable Windows system directories
            let is_official_system_path = (norm_path.contains("\\windows\\system32\\")
                || norm_path.contains("\\windows\\syswow64\\")
                || norm_path.contains("\\windows\\systemapps\\")
                || norm_path.contains("\\windows\\winsxs\\")
                || (norm_path.starts_with("c:\\windows\\") && !norm_path.contains("\\temp\\") && !norm_path.contains("\\tasks\\")))
                && !norm_path.contains("\\temp\\");

            let res = (is_signed, is_official_system_path);
            if let Ok(mut guard) = SIGNATURE_CACHE.lock() {
                guard.insert(norm_path, res);
            }
            return res;
        }
    }

    let res = (false, false);
    if let Ok(mut guard) = SIGNATURE_CACHE.lock() {
        guard.insert(norm_path, res);
    }
    res
}

/// Determines the Guardian signature badge for a given process
pub fn evaluate_guardian_badge(
    exe_path: Option<&str>,
    publisher: Option<&str>,
    heuristics: &ProcessLocationHeuristics,
) -> SignatureBadge {
    let (is_signed, is_ms_system_path) = match exe_path {
        Some(p) => verify_executable_signature(p),
        None => (false, false),
    };

    let pub_is_ms = publisher
        .map(|p| p.to_lowercase().contains("microsoft"))
        .unwrap_or(false);

    // Official Microsoft validation requires either:
    // 1. A signed binary with explicit Microsoft publisher metadata, OR
    // 2. A signed binary residing in official Windows system paths that is not in a suspicious/temp folder.
    if is_signed && (pub_is_ms || (is_ms_system_path && !heuristics.is_suspicious_location)) {
        SignatureBadge::VerifiedMicrosoft
    } else if is_signed {
        SignatureBadge::VerifiedPublisher
    } else if heuristics.is_suspicious_location {
        SignatureBadge::UnsignedSuspiciousLocation
    } else {
        SignatureBadge::Unsigned
    }
}

/// Offline heuristic fallback explanation when Gemini API is offline or key is missing
pub fn generate_offline_process_explanation(
    process_name: &str,
    exe_path: Option<&str>,
    publisher: Option<&str>,
    description: Option<&str>,
    cpu_percent: f32,
    memory_mb: f32,
) -> crate::ai::gemini::GeminiProcessExplanation {
    generate_offline_process_explanation_with_category(
        process_name,
        exe_path,
        None,
        publisher,
        description,
        cpu_percent,
        memory_mb,
    )
}

/// Offline heuristic fallback explanation with optional sanitized path category support
pub fn generate_offline_process_explanation_with_category(
    process_name: &str,
    exe_path: Option<&str>,
    path_category: Option<&str>,
    publisher: Option<&str>,
    description: Option<&str>,
    cpu_percent: f32,
    memory_mb: f32,
) -> crate::ai::gemini::GeminiProcessExplanation {
    let mut heuristics = analyze_path_heuristics(exe_path);
    if let Some(cat) = path_category {
        let cat_lower = cat.to_lowercase();
        if cat_lower.contains("temp") {
            heuristics.is_in_temp = true;
            heuristics.is_suspicious_location = true;
            heuristics.location_label = "Temporary Directory".to_string();
        } else if cat_lower.contains("download") {
            heuristics.is_in_downloads = true;
            heuristics.is_suspicious_location = true;
            heuristics.location_label = "Downloads Directory".to_string();
        } else if cat_lower.contains("appdata") && !cat_lower.contains("programs") {
            heuristics.is_in_appdata = true;
            heuristics.is_suspicious_location = true;
            heuristics.location_label = "User AppData".to_string();
        }
    }

    let badge = evaluate_guardian_badge(exe_path, publisher, &heuristics);

    // 1. Check known process definitions
    if let Some(known) = lookup_process(process_name) {
        return crate::ai::gemini::GeminiProcessExplanation {
            summary: known.description.clone(),
            vendor: known.publisher.clone(),
            safety: known.safety.clone(),
            can_terminate: known.can_kill,
            why_high_usage: if cpu_percent > 15.0 || memory_mb > 300.0 {
                format!(
                    "Active tasks or background processing (currently {:.1}% CPU, {:.0} MB RAM).",
                    cpu_percent, memory_mb
                )
            } else {
                "Nominal background usage.".to_string()
            },
            recommendation: if known.can_kill {
                format!("Safe to close {} if not actively needed.", known.name)
            } else {
                "Windows core component. Do not terminate.".to_string()
            },
            sanitized_query: crate::ai::sanitize::sanitize_process_info(
                process_name,
                exe_path,
                publisher,
                description,
                cpu_percent,
                memory_mb,
            ),
        };
    }

    // 2. Dynamic heuristic explanation
    let (summary, vendor, safety, can_terminate, recommendation) = match badge {
        SignatureBadge::VerifiedMicrosoft => (
            format!(
                "{} is an official Microsoft executable located in {}.",
                process_name, heuristics.location_label
            ),
            "Microsoft Corporation".to_string(),
            if heuristics.location_label.contains("System32") { "critical" } else { "safe" }.to_string(),
            !heuristics.location_label.contains("System32"),
            "Cryptographically verified Microsoft component. Retain for smooth system functionality.".to_string(),
        ),
        SignatureBadge::VerifiedPublisher => {
            let pub_name = publisher.unwrap_or("Known Third-Party Vendor");
            (
                format!(
                    "{} is a digitally signed application from {}.",
                    process_name, pub_name
                ),
                pub_name.to_string(),
                "safe".to_string(),
                true,
                format!("Legitimate verified software. Can be terminated if you wish to reduce resource load."),
            )
        }
        SignatureBadge::UnsignedSuspiciousLocation => (
            format!(
                "{} is an unsigned binary running from a user location ({}). Adware and portable background utilities frequently run from here.",
                process_name, heuristics.location_label
            ),
            "Unsigned / Unknown".to_string(),
            "caution".to_string(),
            true,
            "Unsigned executable in temporary or user folders. Terminate and inspect if you did not deliberately run this application.".to_string(),
        ),
        SignatureBadge::Unsigned => (
            format!(
                "{} is an unsigned application executable located in {}.",
                process_name, heuristics.location_label
            ),
            publisher.unwrap_or("Unsigned").to_string(),
            "safe".to_string(),
            true,
            "Standard program without commercial code signing. Can be safely closed if not in active use.".to_string(),
        ),
    };

    let why_high_usage = if cpu_percent > 10.0 {
        format!(
            "Active CPU consumption ({:.1}%). May be performing background processing or indexing.",
            cpu_percent
        )
    } else if memory_mb > 250.0 {
        format!(
            "Active memory footprint ({:.0} MB). Normal for modern web browsers or development tools.",
            memory_mb
        )
    } else {
        "Nominal background resource consumption.".to_string()
    };

    crate::ai::gemini::GeminiProcessExplanation {
        summary,
        vendor,
        safety,
        can_terminate,
        why_high_usage,
        recommendation,
        sanitized_query: crate::ai::sanitize::sanitize_process_info(
            process_name,
            exe_path,
            publisher,
            description,
            cpu_percent,
            memory_mb,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_heuristics_classification() {
        let temp_heuristics = analyze_path_heuristics(Some("C:\\Users\\User\\AppData\\Local\\Temp\\update.exe"));
        assert!(temp_heuristics.is_in_temp);
        assert!(temp_heuristics.is_suspicious_location);

        let appdata_heuristics = analyze_path_heuristics(Some("C:\\Users\\User\\AppData\\Roaming\\miner\\miner.exe"));
        assert!(appdata_heuristics.is_in_appdata);
        assert!(appdata_heuristics.is_suspicious_location);

        let prog_heuristics = analyze_path_heuristics(Some("C:\\Program Files\\App\\app.exe"));
        assert!(!prog_heuristics.is_suspicious_location);
        assert_eq!(prog_heuristics.location_label, "Program Files");

        let local_programs = analyze_path_heuristics(Some("C:\\Users\\User\\AppData\\Local\\Programs\\VSCode\\Code.exe"));
        assert!(!local_programs.is_suspicious_location, "Local\\Programs is standard per-user app directory");
    }

    #[test]
    fn test_offline_explanation_fallback_known() {
        let exp = generate_offline_process_explanation(
            "explorer.exe",
            Some("C:\\Windows\\explorer.exe"),
            Some("Microsoft Corporation"),
            Some("Windows Explorer"),
            1.2,
            85.0,
        );
        assert_eq!(exp.vendor, "Microsoft Corporation");
        assert_eq!(exp.safety, "critical");
        assert!(!exp.can_terminate);
    }

    #[test]
    fn test_offline_explanation_suspicious_unsigned() {
        let exp = generate_offline_process_explanation(
            "unknown_miner.exe",
            Some("C:\\Users\\User\\AppData\\Local\\Temp\\unknown_miner.exe"),
            None,
            None,
            45.0,
            120.0,
        );
        assert_eq!(exp.safety, "caution");
        assert!(exp.can_terminate);
        assert!(exp.summary.contains("unsigned"));
        assert!(exp.recommendation.contains("Terminate"));
    }

    #[test]
    fn test_offline_explanation_with_path_category() {
        let exp = generate_offline_process_explanation_with_category(
            "temp_payload.exe",
            None,
            Some("User Temp Directory"),
            None,
            None,
            10.0,
            50.0,
        );
        assert_eq!(exp.safety, "caution");
        assert!(exp.summary.contains("unsigned"));
        assert!(exp.recommendation.contains("Terminate"));
    }

    #[test]
    fn test_signature_cache_consistency() {
        let res1 = verify_executable_signature("C:\\Windows\\explorer.exe");
        let res2 = verify_executable_signature("C:\\Windows\\explorer.exe");
        assert_eq!(res1, res2);
    }
}
