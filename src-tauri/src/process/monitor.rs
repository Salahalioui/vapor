use super::known_db::lookup_process;
use serde::{Deserialize, Serialize};
use std::sync::{LazyLock, Mutex};
use sysinfo::{Disks, ProcessesToUpdate, System};

static SYSTEM_STATE: LazyLock<Mutex<System>> = LazyLock::new(|| {
    let mut sys = System::new_all();
    sys.refresh_cpu_all();
    sys.refresh_memory();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    Mutex::new(sys)
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub memory_mb: f32,
    pub exe_path: Option<String>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub category: String,
    pub safety: String, // 'safe' | 'caution' | 'critical' | 'bloatware'
    pub can_kill: bool,
    pub is_known: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub free_percent: f32,
    pub file_system: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHardwareOverview {
    pub total_memory_bytes: u64,
    pub used_memory_bytes: u64,
    pub memory_percent: f32,
    pub global_cpu_percent: f32,
    pub cpu_core_count: usize,
    pub process_count: usize,
    pub top_processes: Vec<ProcessInfo>,
    pub disks: Vec<DiskInfo>,
}

pub fn get_hardware_overview() -> SystemHardwareOverview {
    let mut sys_guard = SYSTEM_STATE.lock().unwrap();
    let sys = &mut *sys_guard;

    sys.refresh_cpu_all();
    sys.refresh_memory();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let memory_percent = if total_memory > 0 {
        (used_memory as f32 / total_memory as f32) * 100.0
    } else {
        0.0
    };

    let global_cpu = sys.global_cpu_usage();
    let core_count = sys.cpus().len();

    let mut procs: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, proc_ref)| {
            let pid_u32 = pid.as_u32();
            let name = proc_ref.name().to_string_lossy().to_string();
            let exe_path = proc_ref.exe().map(|p| p.to_string_lossy().to_string());
            let mem = proc_ref.memory();
            let cpu = proc_ref.cpu_usage();

            let known = lookup_process(&name);
            let (publisher, description, category, safety, can_kill, is_known) = match known {
                Some(k) => (
                    Some(k.publisher),
                    Some(k.description),
                    k.category,
                    k.safety,
                    k.can_kill,
                    true,
                ),
                None => {
                    let is_system_path = exe_path
                        .as_ref()
                        .map(|p| p.to_lowercase().contains("windows\\system32"))
                        .unwrap_or(false);
                    (
                        None,
                        None,
                        if is_system_path {
                            "system".to_string()
                        } else {
                            "user".to_string()
                        },
                        if is_system_path {
                            "caution".to_string()
                        } else {
                            "safe".to_string()
                        },
                        !is_system_path,
                        false,
                    )
                }
            };

            ProcessInfo {
                pid: pid_u32,
                name,
                cpu_percent: cpu,
                memory_bytes: mem,
                memory_mb: (mem as f32) / (1024.0 * 1024.0),
                exe_path,
                publisher,
                description,
                category,
                safety,
                can_kill,
                is_known,
            }
        })
        .collect();

    // Sort by CPU percent descending, then memory descending
    procs.sort_by(|a, b| {
        b.cpu_percent
            .partial_cmp(&a.cpu_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.memory_bytes.cmp(&a.memory_bytes))
    });

    let process_count = procs.len();
    let top_processes = procs.into_iter().take(250).collect();

    // Disks
    let disks_obj = Disks::new_with_refreshed_list();
    let disks: Vec<DiskInfo> = disks_obj
        .iter()
        .map(|d| {
            let total = d.total_space();
            let avail = d.available_space();
            let free_pct = if total > 0 {
                (avail as f32 / total as f32) * 100.0
            } else {
                0.0
            };
            DiskInfo {
                name: d.name().to_string_lossy().to_string(),
                mount_point: d.mount_point().to_string_lossy().to_string(),
                total_bytes: total,
                available_bytes: avail,
                free_percent: free_pct,
                file_system: d.file_system().to_string_lossy().to_string(),
            }
        })
        .collect();

    SystemHardwareOverview {
        total_memory_bytes: total_memory,
        used_memory_bytes: used_memory,
        memory_percent,
        global_cpu_percent: global_cpu,
        cpu_core_count: core_count,
        process_count,
        top_processes,
        disks,
    }
}

pub fn kill_process_by_pid(pid: u32) -> Result<(), String> {
    let mut sys_guard = SYSTEM_STATE.lock().unwrap();
    let sys = &mut *sys_guard;
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let sysinfo_pid = sysinfo::Pid::from_u32(pid);

    let killed = if let Some(process) = sys.process(sysinfo_pid) {
        process.kill()
    } else {
        false
    };

    if killed {
        Ok(())
    } else {
        // Fallback to taskkill /F /PID
        let output = std::process::Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .output()
            .map_err(|e| format!("Failed to execute taskkill: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}

pub fn get_processes_for_audit() -> Vec<ProcessInfo> {
    let hw = get_hardware_overview();
    let mut procs = hw.top_processes;

    // Sort by prioritizing unknown processes, then high CPU, then high memory
    procs.sort_by(|a, b| {
        let a_score = (!a.is_known as u32 * 1000)
            + (a.cpu_percent * 10.0) as u32
            + (a.memory_mb as u32);
        let b_score = (!b.is_known as u32 * 1000)
            + (b.cpu_percent * 10.0) as u32
            + (b.memory_mb as u32);
        b_score.cmp(&a_score)
    });

    let mut unique_procs: Vec<ProcessInfo> = Vec::new();
    let mut seen_names = std::collections::HashSet::new();

    for p in procs {
        let name_lower = p.name.to_lowercase();
        if !seen_names.contains(&name_lower) {
            seen_names.insert(name_lower);
            unique_procs.push(p);
        }
        if unique_procs.len() >= 20 {
            break;
        }
    }

    unique_procs
}
