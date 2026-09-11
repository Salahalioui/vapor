use jwalk::WalkDir;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeFileItem {
    pub path: String,
    pub name: String,
    pub extension: String,
    pub size_bytes: u64,
    pub modified_epoch_secs: u64,
    pub age_days: u32,
    pub is_in_downloads: bool,
}

pub fn get_default_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(user_profile) = env::var("USERPROFILE") {
        let p = PathBuf::from(&user_profile);
        for sub in &["Downloads", "Documents", "Desktop", "Videos", "Music", "Pictures"] {
            let target = p.join(sub);
            if target.exists() {
                roots.push(target);
            }
        }
    }
    roots
}

const IGNORED_DIRS: &[&str] = &[
    ".git",
    ".svn",
    ".hg",
    ".vs",
    ".vscode",
    ".idea",
    "appdata",
    "$recycle.bin",
    "node_modules",
    "target",
    ".venv",
    "venv",
    ".next",
    "dist",
];

fn is_ignored_dir(name_lower: &str) -> bool {
    IGNORED_DIRS.iter().any(|&d| d == name_lower)
}

pub fn scan_large_files(
    custom_roots: Option<Vec<String>>,
    min_size_bytes: u64,
    limit: usize,
) -> Vec<LargeFileItem> {
    let roots: Vec<PathBuf> = match custom_roots {
        Some(paths) if !paths.is_empty() => paths.into_iter().map(PathBuf::from).collect(),
        _ => get_default_search_roots(),
    };

    let now = SystemTime::now();
    let downloads_dir = env::var("USERPROFILE")
        .map(|up| PathBuf::from(up).join("Downloads"))
        .unwrap_or_default();

    let mut items: Vec<LargeFileItem> = Vec::new();

    for root in roots {
        if !root.exists() {
            continue;
        }

        for entry in WalkDir::new(&root)
            .max_depth(10)
            .skip_hidden(true)
            .process_read_dir(|_depth, _dir_path, _state, children| {
                children.retain(|res| {
                    if let Ok(entry) = res {
                        if entry.file_type.is_dir() {
                            let name_lower = entry.file_name.to_string_lossy().to_lowercase();
                            if is_ignored_dir(&name_lower) {
                                return false;
                            }
                        }
                    }
                    true
                });
            })
            .into_iter()
            .flatten()
        {
            if entry.file_type().is_file() {
                if let Ok(meta) = entry.metadata() {
                    let size = meta.len();
                    if size >= min_size_bytes {
                        let path = entry.path();
                        let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                        let age_duration = now.duration_since(modified).unwrap_or(Duration::ZERO);
                        let age_days = (age_duration.as_secs() / 86400) as u32;

                        let is_in_downloads = path.starts_with(&downloads_dir);
                        let ext = path
                            .extension()
                            .map(|e| e.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();

                        items.push(LargeFileItem {
                            path: path.to_string_lossy().to_string(),
                            name,
                            extension: ext,
                            size_bytes: size,
                            modified_epoch_secs: modified
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                            age_days,
                            is_in_downloads,
                        });
                    }
                }
            }
        }
    }

    // Sort descending by size
    items.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    items.truncate(limit);
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_large_files_prunes_ignored_dirs() {
        let temp_dir = env::temp_dir().join("vapor_test_large_files_pruning");
        let git_dir = temp_dir.join(".git");
        let target_dir = temp_dir.join("target");
        let valid_dir = temp_dir.join("valid_files");
        let _ = fs::create_dir_all(&git_dir);
        let _ = fs::create_dir_all(&target_dir);
        let _ = fs::create_dir_all(&valid_dir);

        let fake_git_blob = git_dir.join("huge_pack.pack");
        let fake_target_bin = target_dir.join("huge_build.exe");
        let valid_file = valid_dir.join("large_archive.zip");

        let fake_data = vec![0u8; 10_000];
        let _ = fs::write(&fake_git_blob, &fake_data);
        let _ = fs::write(&fake_target_bin, &fake_data);
        let _ = fs::write(&valid_file, &fake_data);

        let res = scan_large_files(Some(vec![temp_dir.to_string_lossy().to_string()]), 5000, 10);
        let _ = fs::remove_dir_all(&temp_dir);

        assert_eq!(res.len(), 1);
        assert_eq!(res[0].name, "large_archive.zip");
        assert!(!res.iter().any(|i| i.path.contains(".git")));
        assert!(!res.iter().any(|i| i.path.contains("target")));
    }
}
