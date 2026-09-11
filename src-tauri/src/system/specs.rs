use serde::{Deserialize, Serialize};
use sysinfo::{Disks, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuSpecs {
    pub model: String,
    pub physical_cores: usize,
    pub logical_threads: usize,
    pub frequency_mhz: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySpecs {
    pub total_gb: f32,
    pub available_gb: f32,
    pub used_gb: f32,
    pub used_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSpecs {
    pub drive_letter: String,
    pub is_ssd: bool,
    pub media_type: String, // "NVMe/SATA SSD" or "Mechanical HDD"
    pub total_gb: f32,
    pub free_gb: f32,
    pub free_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuSpecs {
    pub name: String,
    pub dedicated_vram_mb: u64,
    pub is_dedicated: bool,
    pub vendor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityScore {
    pub score: f32,          // 1.0 - 10.0
    pub letter_grade: String, // "S", "A+", "A", "B", "C", "D"
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRatings {
    pub office_everyday: CapabilityScore,
    pub software_development: CapabilityScore,
    pub gaming_3d: CapabilityScore,
    pub overall_grade: String,
    pub bottleneck_headline: String,
    pub bottleneck_explanation: String,
    pub uptime_days: f32,
    pub fast_startup_warning: bool,
    pub fast_startup_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSpecsOverview {
    pub cpu: CpuSpecs,
    pub memory: MemorySpecs,
    pub primary_storage: StorageSpecs,
    pub gpus: Vec<GpuSpecs>,
    pub primary_gpu: GpuSpecs,
    pub capabilities: CapabilityRatings,
    pub os_version: String,
    pub uptime_seconds: u64,
}

/// Detects whether a Windows drive is an SSD vs HDD using Win32
/// IOCTL_STORAGE_QUERY_PROPERTY with StorageDeviceSeekPenaltyProperty.
/// IncursSeekPenalty == 0 indicates solid-state drive (SSD/NVMe).
/// IncursSeekPenalty != 0 indicates rotational mechanical drive (HDD).
pub fn detect_is_ssd(drive_letter: &str) -> bool {
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let trimmed = drive_letter.trim().trim_end_matches('\\').trim_end_matches('/');
        let device_path = format!("\\\\.\\{}", trimmed);
        let wide: Vec<u16> = OsStr::new(&device_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        #[repr(C)]
        struct StoragePropertyQuery {
            property_id: u32,
            query_type: u32,
            additional_parameters: [u8; 1],
        }

        #[repr(C)]
        struct DeviceSeekPenaltyDescriptor {
            version: u32,
            size: u32,
            incurs_seek_penalty: u8,
        }

        extern "system" {
            fn CreateFileW(
                lpFileName: *const u16,
                dwDesiredAccess: u32,
                dwShareMode: u32,
                lpSecurityAttributes: *mut std::ffi::c_void,
                dwCreationDisposition: u32,
                dwFlagsAndAttributes: u32,
                hTemplateFile: *mut std::ffi::c_void,
            ) -> *mut std::ffi::c_void;

            fn DeviceIoControl(
                hDevice: *mut std::ffi::c_void,
                dwIoControlCode: u32,
                lpInBuffer: *const std::ffi::c_void,
                nInBufferSize: u32,
                lpOutBuffer: *mut std::ffi::c_void,
                nOutBufferSize: u32,
                lpBytesReturned: *mut u32,
                lpOverlapped: *mut std::ffi::c_void,
            ) -> i32;

            fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
        }

        const FILE_SHARE_READ: u32 = 1;
        const FILE_SHARE_WRITE: u32 = 2;
        const OPEN_EXISTING: u32 = 3;
        const IOCTL_STORAGE_QUERY_PROPERTY: u32 = 0x002D1400;
        const INVALID_HANDLE_VALUE: *mut std::ffi::c_void = -1isize as *mut std::ffi::c_void;

        unsafe {
            let handle = CreateFileW(
                wide.as_ptr(),
                0, // 0 desired access allows querying device attributes without administrator elevation
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                std::ptr::null_mut(),
                OPEN_EXISTING,
                0,
                std::ptr::null_mut(),
            );

            if handle != INVALID_HANDLE_VALUE && !handle.is_null() {
                let query = StoragePropertyQuery {
                    property_id: 7, // StorageDeviceSeekPenaltyProperty
                    query_type: 0,  // PropertyStandardQuery
                    additional_parameters: [0],
                };
                let mut descriptor: DeviceSeekPenaltyDescriptor = std::mem::zeroed();
                let mut bytes_returned = 0u32;

                let success = DeviceIoControl(
                    handle,
                    IOCTL_STORAGE_QUERY_PROPERTY,
                    &query as *const _ as *const _,
                    std::mem::size_of::<StoragePropertyQuery>() as u32,
                    &mut descriptor as *mut _ as *mut _,
                    std::mem::size_of::<DeviceSeekPenaltyDescriptor>() as u32,
                    &mut bytes_returned,
                    std::ptr::null_mut(),
                );

                CloseHandle(handle);

                if success != 0 {
                    return descriptor.incurs_seek_penalty == 0;
                }
            }
        }
    }

    // Default to true (SSD) for modern platforms if query cannot be executed
    true
}

/// Enumerates GPUs via DXGI (DirectX Graphics Infrastructure)
pub fn detect_gpus() -> Vec<GpuSpecs> {
    let mut gpus = Vec::new();

    #[cfg(windows)]
    {
        type HRESULT = i32;
        const S_OK: HRESULT = 0;
        const DXGI_ADAPTER_FLAG_SOFTWARE: u32 = 2;

        #[repr(C)]
        struct DXGI_ADAPTER_DESC1 {
            description: [u16; 128],
            vendor_id: u32,
            device_id: u32,
            sub_sys_id: u32,
            revision: u32,
            dedicated_video_memory: usize,
            dedicated_system_memory: usize,
            shared_system_memory: usize,
            adapter_luid: [u32; 2],
            flags: u32,
        }

        #[repr(C)]
        struct IDXGIAdapter1Vtbl {
            query_interface: usize,
            add_ref: unsafe extern "system" fn(this: *mut std::ffi::c_void) -> u32,
            release: unsafe extern "system" fn(this: *mut std::ffi::c_void) -> u32,
            set_private_data: usize,
            set_private_data_interface: usize,
            get_private_data: usize,
            get_parent: usize,
            enum_outputs: usize,
            get_desc: usize,
            check_interface_support: usize,
            get_desc1: unsafe extern "system" fn(
                this: *mut std::ffi::c_void,
                p_desc: *mut DXGI_ADAPTER_DESC1,
            ) -> HRESULT,
        }

        #[repr(C)]
        struct IDXGIAdapter1 {
            vtbl: *const IDXGIAdapter1Vtbl,
        }

        #[repr(C)]
        struct IDXGIFactory1Vtbl {
            query_interface: usize,
            add_ref: unsafe extern "system" fn(this: *mut std::ffi::c_void) -> u32,
            release: unsafe extern "system" fn(this: *mut std::ffi::c_void) -> u32,
            set_private_data: usize,
            set_private_data_interface: usize,
            get_private_data: usize,
            get_parent: usize,
            enum_adapters: usize,
            make_window_association: usize,
            get_window_association: usize,
            create_swap_chain: usize,
            create_software_adapter: usize,
            enum_adapters1: unsafe extern "system" fn(
                this: *mut std::ffi::c_void,
                adapter: u32,
                pp_adapter: *mut *mut IDXGIAdapter1,
            ) -> HRESULT,
        }

        #[repr(C)]
        struct IDXGIFactory1 {
            vtbl: *const IDXGIFactory1Vtbl,
        }

        extern "system" {
            fn LoadLibraryA(lpLibFileName: *const u8) -> *mut std::ffi::c_void;
            fn GetProcAddress(
                hModule: *mut std::ffi::c_void,
                lpProcName: *const u8,
            ) -> *mut std::ffi::c_void;
            fn FreeLibrary(hLibModule: *mut std::ffi::c_void) -> i32;
        }

        unsafe {
            let dxgi_module = LoadLibraryA(b"dxgi.dll\0".as_ptr());
            if !dxgi_module.is_null() {
                let create_factory_proc =
                    GetProcAddress(dxgi_module, b"CreateDXGIFactory1\0".as_ptr());
                if !create_factory_proc.is_null() {
                    // IID_IDXGIFactory1 = {770aae78-f26f-4dba-a829-253c83d1b387}
                    #[repr(C)]
                    struct GUID {
                        data1: u32,
                        data2: u16,
                        data3: u16,
                        data4: [u8; 8],
                    }
                    let iid_factory1 = GUID {
                        data1: 0x770aae78,
                        data2: 0xf26f,
                        data3: 0x4dba,
                        data4: [0xa8, 0x29, 0x25, 0x3c, 0x83, 0xd1, 0xb3, 0x87],
                    };

                    type CreateDXGIFactory1Fn = unsafe extern "system" fn(
                        riid: *const GUID,
                        pp_factory: *mut *mut IDXGIFactory1,
                    ) -> HRESULT;

                    let create_factory: CreateDXGIFactory1Fn =
                        std::mem::transmute(create_factory_proc);
                    let mut factory: *mut IDXGIFactory1 = std::ptr::null_mut();

                    if create_factory(&iid_factory1, &mut factory) == S_OK && !factory.is_null() {
                        let mut adapter_index = 0u32;
                        loop {
                            let mut adapter: *mut IDXGIAdapter1 = std::ptr::null_mut();
                            let hr = ((*(*factory).vtbl).enum_adapters1)(
                                factory as *mut _,
                                adapter_index,
                                &mut adapter,
                            );
                            if hr != S_OK || adapter.is_null() {
                                break;
                            }

                            let mut desc: DXGI_ADAPTER_DESC1 = std::mem::zeroed();
                            if ((*(*adapter).vtbl).get_desc1)(adapter as *mut _, &mut desc) == S_OK
                            {
                                // Skip software emulation adapters (WARP)
                                if (desc.flags & DXGI_ADAPTER_FLAG_SOFTWARE) == 0 {
                                    let desc_len = desc
                                        .description
                                        .iter()
                                        .position(|&c| c == 0)
                                        .unwrap_or(desc.description.len());
                                    let name = String::from_utf16_lossy(&desc.description[..desc_len])
                                        .trim()
                                        .to_string();

                                    let vram_mb = (desc.dedicated_video_memory / (1024 * 1024)) as u64;

                                    let vendor = match desc.vendor_id {
                                        0x10DE => "NVIDIA",
                                        0x1002 => "AMD",
                                        0x8086 => "Intel",
                                        0x1414 => "Microsoft",
                                        _ => "Unknown",
                                    }
                                    .to_string();

                                    let name_lower = name.to_lowercase();
                                    let is_integrated_name = name_lower.contains("intel(r) uhd")
                                        || name_lower.contains("intel(r) hd")
                                        || name_lower.contains("intel(r) iris")
                                        || name_lower.contains("radeon graphics")
                                        || name_lower.contains("radeon(tm) graphics")
                                        || name_lower.contains("vega")
                                        || name_lower.contains("microsoft basic")
                                        || name_lower.contains("standard vga")
                                        || name_lower.contains("virtualbox")
                                        || name_lower.contains("vmware");

                                    // Accurate dedicated GPU criteria:
                                    // True discrete cards have dedicated VRAM >= 2048 MB (or NVIDIA >= 1024 MB),
                                    // avoiding false classification of integrated APUs with BIOS-shared memory apertures.
                                    let is_dedicated = if is_integrated_name {
                                        name_lower.contains("arc") && vram_mb >= 4096
                                    } else if desc.vendor_id == 0x10DE {
                                        vram_mb >= 1024
                                    } else {
                                        vram_mb >= 2048
                                    };

                                    gpus.push(GpuSpecs {
                                        name,
                                        dedicated_vram_mb: vram_mb,
                                        is_dedicated,
                                        vendor,
                                    });
                                }
                            }

                            ((*(*adapter).vtbl).release)(adapter as *mut _);
                            adapter_index += 1;
                        }

                        ((*(*factory).vtbl).release)(factory as *mut _);
                    }
                }
                FreeLibrary(dxgi_module);
            }
        }
    }

    if gpus.is_empty() {
        // Fallback default
        gpus.push(GpuSpecs {
            name: "Standard Graphics Adapter".to_string(),
            dedicated_vram_mb: 512,
            is_dedicated: false,
            vendor: "Generic".to_string(),
        });
    }

    gpus
}

/// Evaluates hardware capability ratings based on dynamic system metrics
pub fn calculate_capabilities(
    cpu: &CpuSpecs,
    memory: &MemorySpecs,
    storage: &StorageSpecs,
    primary_gpu: &GpuSpecs,
    uptime_seconds: u64,
) -> CapabilityRatings {
    // 1. Everyday & Office (Web, 4K video, Office)
    // Heavily weighted on SSD presence + 8GB+ RAM
    let storage_office_pts: f32 = if storage.is_ssd { 4.0 } else { 1.5 };
    let ram_office_pts: f32 = if memory.total_gb >= 16.0 {
        3.5
    } else if memory.total_gb >= 8.0 {
        3.0
    } else if memory.total_gb >= 4.0 {
        2.0
    } else {
        1.0
    };
    let cpu_office_pts: f32 = if cpu.physical_cores >= 6 {
        2.5
    } else if cpu.physical_cores >= 4 {
        2.2
    } else if cpu.physical_cores >= 2 {
        1.6
    } else {
        0.8
    };
    let office_score_val = (storage_office_pts + ram_office_pts + cpu_office_pts).clamp(1.0, 10.0);
    let office_score_rounded = (office_score_val * 10.0).round() / 10.0;

    let office_grade = score_to_letter(office_score_rounded);
    let office_summary = format!(
        "{}: Smooth web browsing, 4K streaming, and multi-document office productivity.",
        office_grade
    );

    // 2. Software Development (Compiling, IDEs, Docker, VMs)
    // Heavily weighted on 16GB+ RAM, multi-core CPU, fast SSD
    let ram_dev_pts: f32 = if memory.total_gb >= 32.0 {
        4.0
    } else if memory.total_gb >= 16.0 {
        3.2
    } else if memory.total_gb >= 12.0 {
        2.4
    } else if memory.total_gb >= 8.0 {
        1.5
    } else {
        0.6
    };
    let cpu_dev_pts: f32 = if cpu.logical_threads >= 16 || cpu.physical_cores >= 8 {
        3.5
    } else if cpu.physical_cores >= 6 {
        2.8
    } else if cpu.physical_cores >= 4 {
        2.0
    } else {
        0.8
    };
    let storage_dev_pts: f32 = if storage.is_ssd { 2.5 } else { 0.5 };
    let dev_score_val = (ram_dev_pts + cpu_dev_pts + storage_dev_pts).clamp(1.0, 10.0);
    let dev_score_rounded = (dev_score_val * 10.0).round() / 10.0;

    let dev_grade = score_to_letter(dev_score_rounded);
    let dev_summary = format!(
        "{}: Local compilation, multi-container workflows, and developer tooling responsiveness.",
        dev_grade
    );

    // 3. 3D Gaming (DirectX 12, modern titles, VRAM)
    // Heavily weighted on dedicated GPU vs integrated graphics and VRAM amount
    let gpu_gaming_pts: f32 = if primary_gpu.is_dedicated {
        if primary_gpu.dedicated_vram_mb >= 12000 {
            5.0
        } else if primary_gpu.dedicated_vram_mb >= 8000 {
            4.3
        } else if primary_gpu.dedicated_vram_mb >= 6000 {
            3.6
        } else if primary_gpu.dedicated_vram_mb >= 4000 {
            2.8
        } else if primary_gpu.dedicated_vram_mb >= 2000 {
            1.8
        } else {
            1.2
        }
    } else {
        1.0 // Integrated graphics
    };
    let cpu_gaming_pts: f32 = if cpu.physical_cores >= 8 {
        2.5
    } else if cpu.physical_cores >= 6 {
        2.2
    } else if cpu.physical_cores >= 4 {
        1.6
    } else {
        0.7
    };
    let ram_gaming_pts: f32 = if memory.total_gb >= 16.0 {
        1.5
    } else if memory.total_gb >= 8.0 {
        1.0
    } else {
        0.4
    };
    let storage_gaming_pts: f32 = if storage.is_ssd { 1.0 } else { 0.2 };
    let gaming_score_val = if primary_gpu.is_dedicated {
        (gpu_gaming_pts + cpu_gaming_pts + ram_gaming_pts + storage_gaming_pts).clamp(1.0, 10.0)
    } else {
        // Without dedicated VRAM / discrete GPU, modern 3D AAA gaming is strictly hardware-constrained.
        // Cap score to casual/light 3D gaming (max 4.0, letter grade C/D).
        (gpu_gaming_pts + (cpu_gaming_pts + ram_gaming_pts + storage_gaming_pts) * 0.4).clamp(1.0, 4.0)
    };
    let gaming_score_rounded = (gaming_score_val * 10.0).round() / 10.0;

    let gaming_grade = score_to_letter(gaming_score_rounded);
    let gaming_summary = if primary_gpu.is_dedicated {
        format!(
            "{}: Dedicated {} ({:.1} GB VRAM) for 3D gaming and graphics rendering.",
            gaming_grade,
            primary_gpu.name,
            primary_gpu.dedicated_vram_mb as f32 / 1024.0
        )
    } else {
        format!(
            "{}: Integrated graphics. Ideal for casual/2D titles and video playback.",
            gaming_grade
        )
    };

    // Overall grade
    let avg_score = (office_score_rounded + dev_score_rounded + gaming_score_rounded) / 3.0;
    let overall_grade = score_to_letter((avg_score * 10.0).round() / 10.0);

    // Bottleneck Detective
    let (bottleneck_headline, bottleneck_explanation) = if !storage.is_ssd {
        (
            "Mechanical Hard Drive (HDD)".to_string(),
            "Your Windows primary drive is a mechanical hard drive. Upgrading to an NVMe or SATA SSD is the single highest-impact improvement for boot times, application loading, and overall responsiveness.".to_string(),
        )
    } else if memory.total_gb < 8.0 {
        (
            "Low System Memory (< 8 GB)".to_string(),
            format!(
                "System memory is only {:.1} GB. Modern Windows and web browsers frequently exceed this, causing virtual memory paging lag. Upgrading to at least 16 GB will significantly reduce stutters.",
                memory.total_gb
            ),
        )
    } else if !primary_gpu.is_dedicated {
        (
            "Integrated Graphics Architecture".to_string(),
            format!(
                "Graphics processing relies on integrated {} with shared system RAM. While completely capable for 4K video and office work, this is the primary bottleneck for 3D gaming and GPU acceleration.",
                primary_gpu.name
            ),
        )
    } else if memory.total_gb < 16.0 {
        (
            "Modest System Memory (8 GB)".to_string(),
            "8 GB RAM handles single-task office workloads comfortably, but multitasking across heavy browser sessions, IDEs, or modern games will quickly consume available headroom.".to_string(),
        )
    } else if cpu.physical_cores <= 4 {
        (
            "Quad-Core CPU Limitation".to_string(),
            format!(
                "CPU has {} physical cores. While fast for everyday tasks, modern heavy multitasking, code compilation, and gaming engines benefit substantially from 6+ cores.",
                cpu.physical_cores
            ),
        )
    } else {
        (
            "Balanced Hardware Profile".to_string(),
            "No single hardware bottleneck detected. CPU cores, RAM capacity, SSD storage speed, and graphics capabilities are well-balanced for power-user workloads.".to_string(),
        )
    };

    // Uptime & Fast Startup Trap
    let uptime_days = uptime_seconds as f32 / 86400.0;
    let fast_startup_warning = uptime_days >= 7.0;
    let fast_startup_message = if fast_startup_warning {
        Some(format!(
            "Windows has been running continuously for {:.1} days without a fresh boot. Windows 'Fast Startup' saves kernel memory to disk instead of fully clearing it during standard shutdowns. A true 'Restart' (not Shutdown) flushes leaked memory, driver caches, and pending OS routines.",
            uptime_days
        ))
    } else {
        None
    };

    CapabilityRatings {
        office_everyday: CapabilityScore {
            score: office_score_rounded,
            letter_grade: office_grade,
            summary: office_summary,
        },
        software_development: CapabilityScore {
            score: dev_score_rounded,
            letter_grade: dev_grade,
            summary: dev_summary,
        },
        gaming_3d: CapabilityScore {
            score: gaming_score_rounded,
            letter_grade: gaming_grade,
            summary: gaming_summary,
        },
        overall_grade,
        bottleneck_headline,
        bottleneck_explanation,
        uptime_days: (uptime_days * 10.0).round() / 10.0,
        fast_startup_warning,
        fast_startup_message,
    }
}

fn score_to_letter(score: f32) -> String {
    if score >= 9.0 {
        "S".to_string()
    } else if score >= 8.0 {
        "A+".to_string()
    } else if score >= 7.0 {
        "A".to_string()
    } else if score >= 5.5 {
        "B".to_string()
    } else if score >= 4.0 {
        "C".to_string()
    } else {
        "D".to_string()
    }
}

/// Retrieves complete hardware specs, capabilities, and uptime
pub fn get_system_specs() -> SystemSpecsOverview {
    let mut sys = System::new_all();
    sys.refresh_cpu_all();
    sys.refresh_memory();

    let cpus = sys.cpus();
    let cpu_model = cpus
        .first()
        .map(|c| c.brand().trim().to_string())
        .unwrap_or_else(|| "Unknown CPU".to_string());
    let cpu_freq = cpus.first().map(|c| c.frequency()).unwrap_or(0);
    let physical_cores = sys.physical_core_count().unwrap_or(cpus.len());
    let logical_threads = cpus.len();

    let cpu = CpuSpecs {
        model: cpu_model,
        physical_cores,
        logical_threads,
        frequency_mhz: cpu_freq,
    };

    let total_bytes = sys.total_memory();
    let used_bytes = sys.used_memory();
    let avail_bytes = sys.available_memory();

    let total_gb = total_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
    let used_gb = used_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
    let available_gb = avail_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
    let used_percent = if total_bytes > 0 {
        (used_bytes as f32 / total_bytes as f32) * 100.0
    } else {
        0.0
    };

    let memory = MemorySpecs {
        total_gb: (total_gb * 10.0).round() / 10.0,
        available_gb: (available_gb * 10.0).round() / 10.0,
        used_gb: (used_gb * 10.0).round() / 10.0,
        used_percent: (used_percent * 10.0).round() / 10.0,
    };

    // Primary disk (usually %SystemDrive% or mount point with system root)
    let sys_drive = std::env::var("SystemDrive")
        .unwrap_or_else(|_| "C:".to_string())
        .to_uppercase();
    let sys_prefix = sys_drive.trim_end_matches('\\');

    let disks = Disks::new_with_refreshed_list();
    let primary_disk = disks.iter().find(|d| {
        let mount = d.mount_point().to_string_lossy().to_uppercase();
        mount.starts_with(sys_prefix) || mount == "/"
    }).or_else(|| disks.first());

    let (drive_letter, total_disk_gb, free_disk_gb, free_disk_pct) = if let Some(d) = primary_disk {
        let mount = d.mount_point().to_string_lossy().to_string();
        let total = d.total_space() as f32 / (1024.0 * 1024.0 * 1024.0);
        let avail = d.available_space() as f32 / (1024.0 * 1024.0 * 1024.0);
        let pct = if d.total_space() > 0 {
            (d.available_space() as f32 / d.total_space() as f32) * 100.0
        } else {
            0.0
        };
        (mount, total, avail, pct)
    } else {
        ("C:".to_string(), 256.0, 100.0, 39.0)
    };

    let is_ssd = detect_is_ssd(&drive_letter);
    let media_type = if is_ssd {
        "NVMe / SATA Solid State Drive (SSD)".to_string()
    } else {
        "Rotational Mechanical Hard Drive (HDD)".to_string()
    };

    let storage = StorageSpecs {
        drive_letter,
        is_ssd,
        media_type,
        total_gb: (total_disk_gb * 10.0).round() / 10.0,
        free_gb: (free_disk_gb * 10.0).round() / 10.0,
        free_percent: (free_disk_pct * 10.0).round() / 10.0,
    };

    let gpus = detect_gpus();
    // Choose primary GPU: prefer dedicated GPU with largest VRAM, otherwise first GPU
    let primary_gpu = gpus
        .iter()
        .filter(|g| g.is_dedicated)
        .max_by_key(|g| g.dedicated_vram_mb)
        .cloned()
        .unwrap_or_else(|| gpus[0].clone());

    let uptime_seconds = System::uptime();
    let os_version = System::long_os_version().unwrap_or_else(|| "Windows 11".to_string());

    let capabilities = calculate_capabilities(
        &cpu,
        &memory,
        &storage,
        &primary_gpu,
        uptime_seconds,
    );

    SystemSpecsOverview {
        cpu,
        memory,
        primary_storage: storage,
        gpus,
        primary_gpu,
        capabilities,
        os_version,
        uptime_seconds,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specs_calculation_and_bottleneck_detection() {
        let cpu = CpuSpecs {
            model: "Intel Core i7-12700H".to_string(),
            physical_cores: 14,
            logical_threads: 20,
            frequency_mhz: 2300,
        };
        let memory = MemorySpecs {
            total_gb: 16.0,
            available_gb: 8.0,
            used_gb: 8.0,
            used_percent: 50.0,
        };
        let storage = StorageSpecs {
            drive_letter: "C:".to_string(),
            is_ssd: true,
            media_type: "NVMe/SATA SSD".to_string(),
            total_gb: 512.0,
            free_gb: 200.0,
            free_percent: 39.0,
        };
        let gpu = GpuSpecs {
            name: "NVIDIA GeForce RTX 3060".to_string(),
            dedicated_vram_mb: 6144,
            is_dedicated: true,
            vendor: "NVIDIA".to_string(),
        };

        let ratings = calculate_capabilities(&cpu, &memory, &storage, &gpu, 86400 * 2);

        assert!(ratings.office_everyday.score >= 8.0);
        assert!(ratings.software_development.score >= 7.5);
        assert!(ratings.gaming_3d.score >= 6.5);
        assert_eq!(ratings.bottleneck_headline, "Balanced Hardware Profile");
        assert!(!ratings.fast_startup_warning);
    }

    #[test]
    fn test_bottleneck_detection_hdd() {
        let cpu = CpuSpecs {
            model: "Test CPU".to_string(),
            physical_cores: 8,
            logical_threads: 16,
            frequency_mhz: 3000,
        };
        let memory = MemorySpecs {
            total_gb: 32.0,
            available_gb: 20.0,
            used_gb: 12.0,
            used_percent: 37.5,
        };
        let storage = StorageSpecs {
            drive_letter: "C:".to_string(),
            is_ssd: false, // Mechanical HDD!
            media_type: "HDD".to_string(),
            total_gb: 1000.0,
            free_gb: 500.0,
            free_percent: 50.0,
        };
        let gpu = GpuSpecs {
            name: "RTX 4080".to_string(),
            dedicated_vram_mb: 16384,
            is_dedicated: true,
            vendor: "NVIDIA".to_string(),
        };

        let ratings = calculate_capabilities(&cpu, &memory, &storage, &gpu, 100);
        assert_eq!(ratings.bottleneck_headline, "Mechanical Hard Drive (HDD)");
        assert!(ratings.bottleneck_explanation.contains("SSD"));
    }

    #[test]
    fn test_fast_startup_trap_warning() {
        let cpu = CpuSpecs {
            model: "Test CPU".to_string(),
            physical_cores: 4,
            logical_threads: 8,
            frequency_mhz: 2500,
        };
        let memory = MemorySpecs {
            total_gb: 8.0,
            available_gb: 3.0,
            used_gb: 5.0,
            used_percent: 62.5,
        };
        let storage = StorageSpecs {
            drive_letter: "C:".to_string(),
            is_ssd: true,
            media_type: "SSD".to_string(),
            total_gb: 256.0,
            free_gb: 50.0,
            free_percent: 19.5,
        };
        let gpu = GpuSpecs {
            name: "Intel UHD Graphics".to_string(),
            dedicated_vram_mb: 128,
            is_dedicated: false,
            vendor: "Intel".to_string(),
        };

        // 10 days uptime = 864,000 seconds
        let ratings = calculate_capabilities(&cpu, &memory, &storage, &gpu, 864000);
        assert!(ratings.fast_startup_warning);
        assert!(ratings.fast_startup_message.is_some());
        assert!(ratings.fast_startup_message.unwrap().contains("Restart"));
    }

    #[test]
    fn test_gpu_capability_scoring_integrated_vs_dedicated() {
        let cpu = CpuSpecs {
            model: "Core i7".to_string(),
            physical_cores: 8,
            logical_threads: 16,
            frequency_mhz: 3200,
        };
        let memory = MemorySpecs {
            total_gb: 16.0,
            available_gb: 8.0,
            used_gb: 8.0,
            used_percent: 50.0,
        };
        let storage = StorageSpecs {
            drive_letter: "C:".to_string(),
            is_ssd: true,
            media_type: "SSD".to_string(),
            total_gb: 512.0,
            free_gb: 200.0,
            free_percent: 39.0,
        };

        // Integrated GPU
        let igpu = GpuSpecs {
            name: "Intel(R) UHD Graphics 620".to_string(),
            dedicated_vram_mb: 512, // BIOS aperture
            is_dedicated: false,
            vendor: "Intel".to_string(),
        };
        let igpu_ratings = calculate_capabilities(&cpu, &memory, &storage, &igpu, 100);
        assert!(igpu_ratings.gaming_3d.score < 6.0);
        assert!(igpu_ratings.gaming_3d.summary.contains("Integrated"));
        assert_eq!(igpu_ratings.bottleneck_headline, "Integrated Graphics Architecture");

        // Dedicated GPU
        let dgpu = GpuSpecs {
            name: "NVIDIA GeForce RTX 4070".to_string(),
            dedicated_vram_mb: 12288,
            is_dedicated: true,
            vendor: "NVIDIA".to_string(),
        };
        let dgpu_ratings = calculate_capabilities(&cpu, &memory, &storage, &dgpu, 100);
        assert!(dgpu_ratings.gaming_3d.score >= 9.0);
        assert!(dgpu_ratings.gaming_3d.summary.contains("Dedicated"));
        assert_eq!(dgpu_ratings.bottleneck_headline, "Balanced Hardware Profile");
    }
}
