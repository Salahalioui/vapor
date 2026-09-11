use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn get_executable_activity_map() -> HashMap<String, u64> {
    let mut map = HashMap::new();

    // 1. Try Windows Prefetch if accessible (C:\Windows\Prefetch)
    let prefetch_dir = Path::new("C:\\Windows\\Prefetch");
    if prefetch_dir.exists() {
        if let Ok(entries) = fs::read_dir(prefetch_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext.to_string_lossy().eq_ignore_ascii_case("pf") {
                        if let Some(stem) = path.file_stem() {
                            let stem_str = stem.to_string_lossy().to_string();
                            // Prefetch filenames are formatted like: EXENAME.EXE-HASH.pf
                            if let Some(dash_idx) = stem_str.rfind('-') {
                                let exe_name = stem_str[..dash_idx].to_lowercase();
                                let exe_stem = exe_name.trim_end_matches(".exe").to_string();
                                if let Ok(meta) = entry.metadata() {
                                    if let Ok(mod_time) = meta.modified() {
                                        let secs = mod_time
                                            .duration_since(SystemTime::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_secs();

                                        map.entry(exe_name)
                                            .and_modify(|e: &mut u64| *e = (*e).max(secs))
                                            .or_insert(secs);

                                        if !exe_stem.is_empty() {
                                            map.entry(exe_stem)
                                                .and_modify(|e: &mut u64| *e = (*e).max(secs))
                                                .or_insert(secs);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Fallback / supplement: Recent shortcuts and Start menu links
    if let Ok(appdata) = env::var("APPDATA") {
        let recent = PathBuf::from(appdata)
            .join("Microsoft")
            .join("Windows")
            .join("Recent");

        if recent.exists() {
            if let Ok(entries) = fs::read_dir(recent) {
                for entry in entries.flatten().take(200) {
                    if let Ok(meta) = entry.metadata() {
                        if let Ok(mod_time) = meta.modified() {
                            let secs = mod_time
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            let name = entry.file_name().to_string_lossy().to_lowercase();
                            let clean_name = name.trim_end_matches(".lnk").to_string();
                            map.entry(clean_name)
                                .and_modify(|e| *e = (*e).max(secs))
                                .or_insert(secs);
                        }
                    }
                }
            }
        }
    }

    map
}
