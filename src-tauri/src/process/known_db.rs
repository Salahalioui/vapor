use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownProcess {
    pub name: String,
    pub publisher: String,
    pub description: String,
    pub category: String,
    pub safety: String, // 'safe' | 'caution' | 'critical' | 'bloatware'
    pub can_kill: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnownProcessDatabase {
    pub system: Vec<KnownProcess>,
    pub apps: Vec<KnownProcess>,
}

use std::sync::LazyLock;

static KNOWN_DB: LazyLock<KnownProcessDatabase> = LazyLock::new(|| {
    const EMBEDDED: &str = include_str!("../../resources/known_processes.json");
    let cleaned = EMBEDDED.trim_start_matches('\u{feff}');
    serde_json::from_str(cleaned).unwrap_or_default()
});

pub fn get_known_database() -> KnownProcessDatabase {
    KNOWN_DB.clone()
}

pub fn lookup_process(name: &str) -> Option<KnownProcess> {
    let lower = name.to_lowercase();
    for p in KNOWN_DB.system.iter().chain(KNOWN_DB.apps.iter()) {
        if p.name.to_lowercase() == lower {
            return Some(p.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_known_process() {
        let explorer = lookup_process("explorer.exe");
        assert!(explorer.is_some());
        assert_eq!(explorer.unwrap().safety, "critical");

        let chrome = lookup_process("chrome.exe");
        assert!(chrome.is_some());
        assert_eq!(chrome.unwrap().category, "browser");
    }
}
