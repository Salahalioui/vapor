use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopAction {
    pub action_type: String, // 'fastest' | 'biggest' | 'smartest'
    pub title: String,
    pub description: String,
    pub impact: String,
    pub badge: String,
    pub reclaimable_bytes: u64,
    pub target_tab: String, // 'cleanup' | 'storage' | 'apps' | 'processes'
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PillarScores {
    pub disk_score: u32,
    pub memory_score: u32,
    pub cleanup_score: u32,
    pub startup_score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_score: u32, // 0 - 100
    pub grade: String,       // 'Optimal' | 'Good' | 'Attention Needed' | 'Critical'
    pub pillars: PillarScores,
    pub top_actions: Vec<TopAction>,
    pub total_memory_bytes: u64,
    pub used_memory_bytes: u64,
    pub memory_percent: f32,
    pub cpu_percent: f32,
    pub primary_disk_total_bytes: u64,
    pub primary_disk_free_bytes: u64,
    pub primary_disk_free_percent: f32,
    pub safe_cleanup_bytes: u64,
    pub dev_diet_bytes: u64,
    pub startup_count: usize,
    pub zombie_count: usize,
}

pub fn calculate_health_score(
    disk_free_pct: f32,
    ram_used_pct: f32,
    safe_cleanup_bytes: u64,
    startup_and_zombie_count: usize,
) -> (u32, PillarScores) {
    // 1. Disk Pillar (35%)
    let disk_score = if disk_free_pct >= 25.0 {
        100
    } else if disk_free_pct >= 15.0 {
        85
    } else if disk_free_pct >= 10.0 {
        60
    } else if disk_free_pct >= 5.0 {
        35
    } else {
        15
    };

    // 2. Memory Pillar (25%)
    let memory_score = if ram_used_pct <= 55.0 {
        100
    } else if ram_used_pct <= 70.0 {
        85
    } else if ram_used_pct <= 80.0 {
        70
    } else if ram_used_pct <= 90.0 {
        45
    } else {
        20
    };

    // 3. Cleanup Pillar (20%)
    let safe_mb = safe_cleanup_bytes / (1024 * 1024);
    let cleanup_score = if safe_mb < 500 {
        100
    } else if safe_mb < 2000 {
        85
    } else if safe_mb < 5000 {
        70
    } else if safe_mb < 15000 {
        45
    } else {
        25
    };

    // 4. Startup & Zombie Pillar (20%)
    let startup_score = if startup_and_zombie_count <= 3 {
        100
    } else if startup_and_zombie_count <= 6 {
        85
    } else if startup_and_zombie_count <= 10 {
        65
    } else if startup_and_zombie_count <= 15 {
        45
    } else {
        20
    };

    let overall = (disk_score as f32 * 0.35
        + memory_score as f32 * 0.25
        + cleanup_score as f32 * 0.20
        + startup_score as f32 * 0.20)
        .round() as u32;

    (
        overall.clamp(0, 100),
        PillarScores {
            disk_score,
            memory_score,
            cleanup_score,
            startup_score,
        },
    )
}

pub fn generate_top_actions(
    safe_cleanup_bytes: u64,
    dev_diet_bytes: u64,
    large_files_bytes: u64,
    startup_count: usize,
    zombie_count: usize,
) -> Vec<TopAction> {
    let mut actions = Vec::new();

    // 1. Fastest Win: Safe Cleanup
    let fastest_desc = if safe_cleanup_bytes > 0 {
        format!(
            "Instant 1-click purge of temporary files, crash dumps, and caches. Staged in Rescue Bin.",
        )
    } else {
        "Temporary and scratch caches are currently clean.".to_string()
    };
    actions.push(TopAction {
        action_type: "fastest".to_string(),
        title: "Fastest Win: Safe Cache Cleanup".to_string(),
        description: fastest_desc,
        impact: format!("{:.2} GB Reclaimable", safe_cleanup_bytes as f64 / 1_073_741_824.0),
        badge: "Zero Risk".to_string(),
        reclaimable_bytes: safe_cleanup_bytes,
        target_tab: "cleanup".to_string(),
    });

    // 2. Biggest Win: Dev Diet or Large Files
    let (biggest_title, biggest_desc, biggest_bytes, biggest_impact) = if dev_diet_bytes > large_files_bytes && dev_diet_bytes > 0 {
        (
            "Biggest Win: Clean Dormant Dev Caches".to_string(),
            "Prune dormant node_modules, Rust targets, or Python venvs unused for over 30 days.".to_string(),
            dev_diet_bytes,
            format!("{:.2} GB Potential", dev_diet_bytes as f64 / 1_073_741_824.0),
        )
    } else if large_files_bytes > 0 {
        (
            "Biggest Win: Large Files & Old Downloads".to_string(),
            "Review large ISOs, installers, and downloads older than 90 days.".to_string(),
            large_files_bytes,
            format!("{:.2} GB Potential", large_files_bytes as f64 / 1_073_741_824.0),
        )
    } else {
        (
            "Biggest Win: Storage Optimization".to_string(),
            "Run a storage scan on your drives to discover reclaimable caches and oversized files.".to_string(),
            0,
            "Storage Audit Pending - Scan needed".to_string(),
        )
    };
    actions.push(TopAction {
        action_type: "biggest".to_string(),
        title: biggest_title,
        description: biggest_desc,
        impact: biggest_impact,
        badge: "High Impact".to_string(),
        reclaimable_bytes: biggest_bytes,
        target_tab: "storage".to_string(),
    });

    // 3. Smartest Win: Startup & Zombie reduction
    let smartest_desc = if startup_count > 4 {
        format!(
            "{} autostart apps launch on boot. Disabling heavy items accelerates boot time and saves RAM.",
            startup_count
        )
    } else if zombie_count > 0 {
        format!(
            "{} heavy installed applications have not been launched in over 60 days.",
            zombie_count
        )
    } else {
        "Startup applications and background autostarts are well tuned.".to_string()
    };
    actions.push(TopAction {
        action_type: "smartest".to_string(),
        title: "Smartest Win: Startup & Zombie Optimization".to_string(),
        description: smartest_desc,
        impact: format!("{} Apps Reviewed", startup_count + zombie_count),
        badge: "Performance".to_string(),
        reclaimable_bytes: 0,
        target_tab: "apps".to_string(),
    });

    actions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_score_calculation() {
        let (score, pillars) = calculate_health_score(30.0, 45.0, 100_000, 2);
        assert_eq!(score, 100);
        assert_eq!(pillars.disk_score, 100);
        assert_eq!(pillars.memory_score, 100);

        let (low_score, _) = calculate_health_score(4.0, 95.0, 20_000_000_000, 18);
        assert!(low_score < 30);
    }

    #[test]
    fn test_top_actions_generation() {
        let actions = generate_top_actions(1024 * 1024 * 1024, 2 * 1024 * 1024 * 1024, 500 * 1024 * 1024, 8, 3);
        assert_eq!(actions.len(), 3);
        assert_eq!(actions[0].action_type, "fastest");
        assert_eq!(actions[1].action_type, "biggest");
        assert_eq!(actions[2].action_type, "smartest");
    }
}
