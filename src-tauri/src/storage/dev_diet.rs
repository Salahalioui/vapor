use jwalk::WalkDir;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

pub const IGNORED_SYSTEM_DIRS: &[&str] = &[
    ".git",
    ".svn",
    ".hg",
    ".vs",
    ".vscode",
    ".idea",
    "appdata",
    "$recycle.bin",
];

pub const DEV_TARGET_NAMES: &[(&str, &str)] = &[
    ("node_modules", "node_modules"),
    ("target", "rust_target"),
    (".venv", "python_venv"),
    ("venv", "python_venv"),
    (".next", "next_cache"),
    ("dist", "build_dist"),
    ("build", "build_dist"),
];

pub fn is_ignored_system_dir(name_lower: &str) -> bool {
    IGNORED_SYSTEM_DIRS.iter().any(|&d| d == name_lower)
}

pub fn is_dev_target_dir(name_lower: &str) -> bool {
    DEV_TARGET_NAMES.iter().any(|(n, _)| *n == name_lower)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevDietItem {
    pub path: String,
    pub name: String,
    pub kind: String, // 'node_modules' | 'rust_target' | 'python_venv' | 'next_cache' | 'build_dist'
    pub size_bytes: u64,
    pub last_modified_secs: u64,
    pub days_dormant: u32,
    pub is_dormant: bool,
}

pub fn get_default_dev_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(up) = env::var("USERPROFILE") {
        let base = PathBuf::from(&up);
        for sub in &[
            "Documents",
            "Projects",
            "source",
            "repos",
            "Developer",
            "Desktop",
        ] {
            let candidate = base.join(sub);
            if candidate.exists() {
                roots.push(candidate);
            }
        }
    }
    roots
}

pub fn scan_dev_diet(
    custom_roots: Option<Vec<String>>,
    dormant_days_threshold: u32,
) -> Vec<DevDietItem> {
    let roots: Vec<PathBuf> = match custom_roots {
        Some(paths) if !paths.is_empty() => paths.into_iter().map(PathBuf::from).collect(),
        _ => get_default_dev_roots(),
    };

    let mut matched_dirs: Vec<(PathBuf, String)> = Vec::new();

    for root in roots {
        if !root.exists() {
            continue;
        }

        for entry in WalkDir::new(&root)
            .max_depth(7)
            .skip_hidden(false)
            .process_read_dir(|_depth, _dir_path, _state, children| {
                // 1. Drop system / VCS directories completely so WalkDir neither yields nor enters them
                children.retain(|res| {
                    if let Ok(dir_entry) = res {
                        if dir_entry.file_type.is_dir() {
                            let name_lower = dir_entry.file_name.to_string_lossy().to_lowercase();
                            if is_ignored_system_dir(&name_lower) {
                                return false;
                            }
                        }
                    }
                    true
                });

                // 2. For dev target directories (node_modules, target, .venv), prune WalkDir descent:
                // Yield the directory itself so it can be recorded, but do NOT traverse its nested subdirectories.
                for res in children.iter_mut() {
                    if let Ok(dir_entry) = res {
                        if dir_entry.file_type.is_dir() {
                            let name_lower = dir_entry.file_name.to_string_lossy().to_lowercase();
                            if is_dev_target_dir(&name_lower) {
                                dir_entry.read_children_path = None;
                            }
                        }
                    }
                }
            })
            .into_iter()
            .flatten()
        {
            if entry.file_type().is_dir() {
                let dir_name = entry
                    .file_name()
                    .to_string_lossy()
                    .to_lowercase();

                for (target_name, kind) in DEV_TARGET_NAMES {
                    if dir_name == *target_name {
                        let path = entry.path();
                        // Verify this is likely a dev folder:
                        // e.g. for target, parent should have Cargo.toml
                        // for node_modules, parent should have package.json
                        // for .venv, parent has pyproject.toml or requirements.txt
                        let should_include = match *kind {
                            "rust_target" => {
                                path.parent().map_or(false, |p| p.join("Cargo.toml").exists())
                            }
                            "node_modules" => {
                                path.parent().map_or(false, |p| p.join("package.json").exists())
                            }
                            "python_venv" => {
                                path.join("pyvenv.cfg").exists()
                                    || path.parent().map_or(false, |p| {
                                        p.join("pyproject.toml").exists()
                                            || p.join("requirements.txt").exists()
                                            || p.join("setup.py").exists()
                                            || p.join("Pipfile").exists()
                                    })
                            }
                            "next_cache" => {
                                path.parent().map_or(false, |p| {
                                    p.join("package.json").exists()
                                        || p.join("next.config.js").exists()
                                        || p.join("next.config.mjs").exists()
                                        || p.join("next.config.ts").exists()
                                })
                            }
                            "build_dist" => {
                                path.parent().map_or(false, |p| {
                                    p.join("package.json").exists()
                                        || p.join("Cargo.toml").exists()
                                        || p.join("tsconfig.json").exists()
                                        || p.join("CMakeLists.txt").exists()
                                        || p.join("Makefile").exists()
                                        || p.join(".git").exists()
                                })
                            }
                            _ => true,
                        };

                        if should_include && !matched_dirs.iter().any(|(p, _)| p == &path) {
                            matched_dirs.push((path, kind.to_string()));
                        }
                        break;
                    }
                }
            }
        }
    }

    let now = SystemTime::now();

    // Bound Rayon concurrency to 4 worker threads to prevent saturating disk queues
    let pool = rayon::ThreadPoolBuilder::new().num_threads(4).build();
    let compute_fn = || {
        matched_dirs
            .into_par_iter()
            .map(|(path, kind)| {
                let mut total_size = 0u64;
                let mut newest_time = SystemTime::UNIX_EPOCH;

                for file_entry in WalkDir::new(&path)
                    .skip_hidden(false)
                    .follow_links(false)
                    .parallelism(jwalk::Parallelism::Serial)
                    .into_iter()
                    .flatten()
                {
                    if file_entry.file_type().is_file() {
                        if let Ok(meta) = file_entry.metadata() {
                            total_size += meta.len();
                            if let Ok(mod_time) = meta.modified() {
                                if mod_time > newest_time {
                                    newest_time = mod_time;
                                }
                            }
                        }
                    }
                }

                let age_duration = now.duration_since(newest_time).unwrap_or(Duration::ZERO);
                let days_dormant = (age_duration.as_secs() / 86400) as u32;
                let is_dormant = days_dormant >= dormant_days_threshold;

                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                DevDietItem {
                    path: path.to_string_lossy().to_string(),
                    name,
                    kind,
                    size_bytes: total_size,
                    last_modified_secs: newest_time
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    days_dormant,
                    is_dormant,
                }
            })
            .collect::<Vec<DevDietItem>>()
    };

    let results = match pool {
        Ok(p) => p.install(compute_fn),
        Err(_) => compute_fn(),
    };

    let mut sorted = results;
    sorted.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    sorted
}

pub fn purge_dev_directory(path_str: &str) -> Result<u64, String> {
    let path = Path::new(path_str);
    if !path.exists() {
        return Err("Directory does not exist".to_string());
    }
    // Safety verification: make sure folder name is one of the dev targets
    let dir_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let allowed = ["node_modules", "target", ".venv", "venv", ".next", "dist", "build", "bin", "obj"];
    if !allowed.contains(&dir_name.as_str()) {
        return Err("Target folder is not a recognized developer build/cache directory".to_string());
    }

    let size = super::scanner::compute_directory_size(path);
    trash::delete(path).map_err(|e| format!("Failed to move directory to Recycle Bin: {}", e))?;
    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_jwalk_pruning_and_vcs_skipping() {
        let temp_dir = env::temp_dir().join("vapor_test_jwalk_prune_vcs");
        let git_dir = temp_dir.join(".git").join("objects");
        let project_dir = temp_dir.join("my_proj");
        let node_modules = project_dir.join("node_modules");
        let nested_nm = node_modules.join("sub_pkg").join("node_modules");
        let _ = fs::create_dir_all(&git_dir);
        let _ = fs::create_dir_all(&nested_nm);
        let _ = fs::write(project_dir.join("package.json"), "{}");
        let _ = fs::write(node_modules.join("foo.js"), "foo");

        let items = scan_dev_diet(Some(vec![temp_dir.to_string_lossy().to_string()]), 0);
        let _ = fs::remove_dir_all(&temp_dir);

        // node_modules is found
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, "node_modules");
        // .git is not included
        assert!(!items.iter().any(|i| i.path.contains(".git")));
    }

    #[test]
    fn test_non_dev_build_folder_excluded() {
        let temp_dir = env::temp_dir().join("vapor_test_non_dev_build");
        let non_dev_build = temp_dir.join("build");
        let _ = fs::create_dir_all(&non_dev_build);
        let _ = fs::write(non_dev_build.join("notes.txt"), "regular notes, not a dev build");

        let items = scan_dev_diet(Some(vec![temp_dir.to_string_lossy().to_string()]), 0);
        let _ = fs::remove_dir_all(&temp_dir);

        // Standalone 'build' folder without package.json, Cargo.toml, etc. should not be matched
        assert!(items.is_empty(), "Non-developer build directory must not be included");
    }
}

