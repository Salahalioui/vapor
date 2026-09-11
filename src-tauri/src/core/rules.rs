use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RiskTier {
    Safe,
    Review,
    Advanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: String,
    pub risk: RiskTier,
    pub category: String,
    #[serde(default)]
    pub age_days: Option<u32>,
    #[serde(default)]
    pub pattern: Option<String>,
}

impl CleanupRule {
    pub fn resolve_paths(&self) -> Vec<PathBuf> {
        let expanded = expand_env_vars(&self.path);
        let path = Path::new(&expanded);

        if expanded.contains('*') {
            // Handle simple globbing for parent directories (e.g., Firefox profile paths)
            let mut results = Vec::new();
            if let Some(parent) = path.parent() {
                let parent_str = parent.to_string_lossy();
                if parent_str.contains('*') {
                    if let Some(grandparent) = parent.parent() {
                        if grandparent.exists() {
                            if let Ok(entries) = std::fs::read_dir(grandparent) {
                                for entry in entries.flatten() {
                                    let candidate = entry.path();
                                    if candidate.is_dir() {
                                        if let Some(sub) = path.file_name() {
                                            let sub_path = candidate.join(sub);
                                            if sub_path.exists() {
                                                results.push(sub_path);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !results.is_empty() {
                return results;
            }
        }

        if path.exists() {
            vec![path.to_path_buf()]
        } else {
            Vec::new()
        }
    }
}

pub fn expand_env_vars(raw: &str) -> String {
    let mut result = raw.to_string();
    let vars = [
        "TEMP",
        "TMP",
        "LOCALAPPDATA",
        "APPDATA",
        "USERPROFILE",
        "SYSTEMROOT",
        "SYSTEMDRIVE",
        "PROGRAMDATA",
        "PROGRAMFILES",
        "PROGRAMFILES(X86)",
        "WINDIR",
        "PUBLIC",
    ];

    for var in vars {
        let token = format!("%{}%", var);
        if result.contains(&token) {
            if let Ok(val) = env::var(var) {
                result = result.replace(&token, &val);
            }
        }
    }

    result
}

pub fn matches_wildcard(filename: &str, pattern: &str) -> bool {
    let fname = filename.to_lowercase();
    let pat = pattern.to_lowercase();

    if pat == "*" || pat.is_empty() {
        return true;
    }

    if !pat.contains('*') {
        return fname == pat;
    }

    let parts: Vec<&str> = pat.split('*').collect();
    if parts.len() == 2 {
        let prefix = parts[0];
        let suffix = parts[1];
        return fname.starts_with(prefix)
            && fname.ends_with(suffix)
            && fname.len() >= prefix.len() + suffix.len();
    }

    let mut cursor = 0;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            if !fname.starts_with(part) {
                return false;
            }
            cursor = part.len();
        } else if i == parts.len() - 1 {
            return fname[cursor..].ends_with(part);
        } else {
            match fname[cursor..].find(part) {
                Some(pos) => cursor += pos + part.len(),
                None => return false,
            }
        }
    }
    true
}

pub fn load_default_rules() -> Vec<CleanupRule> {
    const EMBEDDED_RULES: &str = include_str!("../../resources/cleanup_rules.json");
    let cleaned = EMBEDDED_RULES.trim_start_matches('\u{feff}');
    serde_json::from_str(cleaned).unwrap_or_else(|e| {
        eprintln!("Failed to parse cleanup_rules.json: {}", e);
        Vec::new()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default_rules() {
        let rules = load_default_rules();
        assert!(!rules.is_empty(), "Default rules should not be empty");
        assert!(rules.iter().any(|r| r.id == "user_temp"));
        assert!(rules.iter().any(|r| r.risk == RiskTier::Safe));
        assert!(rules.iter().any(|r| r.risk == RiskTier::Review));
    }

    #[test]
    fn test_expand_env_vars() {
        let expanded = expand_env_vars("%TEMP%\\sample");
        assert!(!expanded.contains("%TEMP%"));
    }

    #[test]
    fn test_matches_wildcard() {
        assert!(matches_wildcard("thumbcache_256.db", "thumbcache_*.db"));
        assert!(matches_wildcard("thumbcache_idx.db", "thumbcache_*.db"));
        assert!(!matches_wildcard("iconcache_256.db", "thumbcache_*.db"));
        assert!(matches_wildcard("crash.log", "*.log"));
        assert!(!matches_wildcard("crash.txt", "*.log"));
        assert!(matches_wildcard("my_cache_data.tmp", "*cache*"));
        assert!(matches_wildcard("everything.anything", "*"));
    }
}
