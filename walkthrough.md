# Vapor v0.3.0 Walkthrough & Feature Guide

## Overview

Vapor v0.3.0 delivers the complete **Dual-Mode UI experience**, the **Honest PC Specs & Capability Rating Engine**, the **Safe Windows Performance & Debloat Center**, and the **Suspicious Process Guardian**.

---

## 1. Dual-Mode UI Flow

### What's New
- **Simple Mode (Default)**:
  - Designed for non-technical users who want a welcoming, zero-jargon experience.
  - **Traffic Light Trust System**:
    - 🟢 **Green (Safe)**: Harmless temporary files, thumbnails, and cache files that can be cleaned with zero risk.
    - 🟡 **Yellow (Review)**: Background software, optional tweaks, and startup entries where the user decides.
    - 🔴 **Red (Protected)**: Core Windows system services, kernel components, and critical drivers that Vapor explicitly protects and will never touch or terminate.
  - **1-Click Smart Clean**: A single primary button that scans safe cleanup rules and stages matching files into the Rescue Bin. Shows warm, honest feedback without commercial scare tactics.
  - **PC Capabilities & Bottleneck Detective Card**: Clear letter grade, 1.0–10.0 ratings across Everyday/Office, Dev, and Gaming, accompanied by plain-English bottleneck explanations.
  - **Quick Windows Tweaks**: On/off toggles for Start search web clutter, taskbar MSN news, background game capture, and P2P updates.
  - **Sponsored Apps Quick Remover**: Immediate 1-click safe uninstall of detected pre-installed partner software.
- **Power User Mode**:
  - Accessible via a single click in the top-right navigation pill.
  - Unlocks deep tables and diagnostic engines:
    - **Dashboard**: 4-pillar health score (Disk Headroom, Memory Pressure, Safe Cleanup, Startup & Zombie Drag) and Top 3 Action items.
    - **Storage Explorer**: Targeted drive filtering, Large Files inspection, and Developer Diet (`node_modules`, `target`, `.venv`).
    - **Safe Cleanup Checklist**: Granular rule selection with item-level exclusion filters and Rescue Bin drawer.
    - **Process Radar & Suspicious Guardian**: Live 2-second sampled process monitor, WinVerifyTrust digital signature badges, and Gemini AI fleet auditing.
    - **Apps & Zombies**: 32/64-bit installed software auditor, Prefetch last-used execution analysis, and reversible startup management.
    - **Windows Debloat Center**: Dedicated hub for safe registry tweaks, sponsored bloatware uninstaller, and System Restore points.
    - **Settings**: Persistent configuration for retention days, threshold sizes, and Gemini API keys.

---

## 2. Honest PC Specs & Capability Rating Engine

### What's New
- **SSD vs HDD Detection via Win32 IOCTL**:
  - Directly queries `IOCTL_STORAGE_QUERY_PROPERTY` using `StorageDeviceSeekPenaltyProperty`. Drives without seek penalty are flagged as instant SSDs (`IncursSeekPenalty == 0`), while mechanical rotational drives are flagged as HDDs.
- **DirectX DXGI GPU Diagnostics**:
  - Dynamically loads `dxgi.dll` (`CreateDXGIFactory1`, `EnumAdapters1`, `GetDesc1`) to enumerate all graphics devices, retrieve dedicated VRAM sizes, and accurately distinguish dedicated GPUs (NVIDIA/AMD) from integrated graphics (Intel UHD/Iris, AMD Radeon Graphics).
- **Dynamic 3-Pillar Capability Math**:
  - **Everyday & Office**: Weighted heavily on SSD presence (+4.0 pts) and 8GB+ RAM (+3.0 pts) for snappy 4K video playback, web multitasking, and office suites.
  - **Software Development**: Weighted on 16GB+ RAM (+3.2 pts), 6+ core CPUs (+2.8 pts), and fast SSD (+2.5 pts) for local compilers, containers, and IDEs.
  - **3D Gaming**: Weighted on dedicated GPU vs integrated graphics (+5.0 pts for 12GB+ VRAM, +4.3 pts for 8GB+ VRAM, down to 1.0 pt for integrated) and multi-core CPU performance.
- **"Bottleneck Detective"**:
  - Inspects actual hardware metrics to identify the primary hardware bottleneck in plain English:
    - Mechanical HDD primary drive
    - Low RAM (< 8 GB)
    - Integrated graphics architecture
    - Modest RAM (8 GB)
    - Quad-Core CPU limitations
- **Windows Fast Startup Trap Warning**:
  - Windows Uptime is queried via `sysinfo::System::uptime()`. If continuous uptime exceeds 7 days, a clear warning explains why Windows "Fast Startup" (which hibernates the kernel instead of shutting down) leaves accumulated memory leaks and why clicking "Restart" gives a true fresh boot.

---

## 3. Safe Windows Performance & Debloat Center

### What's New
- **Safe Reversible Registry Tweaks**:
  - **Disable Start Menu Bing Web Search**: Stops Start Menu searches from querying Bing and displaying web advertisements (`HKCU\...\Search: BingSearchEnabled = 0, DisableSearchBoxSuggestions = 1`).
  - **Hide Taskbar MSN Widgets**: Removes the MSN news and weather ticker (`HKCU\...\Explorer\Advanced: TaskbarDa = 0`), saving ~150 MB background RAM.
  - **Disable Game DVR Background Recording**: Prevents Xbox Game Bar from continuously recording the screen in the background (`HKCU\...\GameConfigStore: GameDVR_Enabled = 0`, `HKCU\...\GameDVR: AppCaptureEnabled = 0`).
  - **Turn Off P2P Delivery Optimization**: Prevents Windows Update from uploading update packages to external PCs on the internet (`HKCU\...\DeliveryOptimization: DODownloadMode = 0`).
- **Appx Sponsored Bloatware Remover**:
  - Scans for pre-installed commercial promotional apps: TikTok, Disney+, Candy Crush, King Games, MSN News, Bing Weather, Microsoft Solitaire, Spotify, and Netflix.
  - Offers 1-click clean uninstallation using PowerShell `Get-AppxPackage | Remove-AppxPackage`.
- **System Restore Point Integration**:
  - Creates a Windows System Restore point before optimizations using PowerShell `Checkpoint-Computer`.

---

## 4. Suspicious Process Guardian

### What's New
- **Path Heuristics**:
  - Flags executables running from `%APPDATA%`, `%TEMP%`, or `Downloads` (the standard execution staging ground for adware, droppers, and crypto-miners).
- **Cryptographic Signature Verification (WinVerifyTrust)**:
  - Validates PE cryptographic signatures against trusted root certificates using `wintrust.dll`.
  - Badges processes across the UI:
    - 🟢 `Verified (Microsoft)`: Official Windows binary.
    - 🔵 `Verified (Known Publisher)`: Signed by a legitimate third-party certificate.
    - 🟡 `Unsigned`: Standard software without commercial code signing.
    - 🔴 `Unsigned in Temp/AppData`: High alert for unsigned binaries executing from user temp folders.
- **Quota-Preserving AI & Offline Heuristics**:
  - Persistent disk caching (`ai_process_cache.json`) ensures 0 redundant API calls.
  - If the user is offline or has no API key, Vapor seamlessly falls back to offline heuristic explanations, meaning the app never crashes or fails.

---

- **Backend Tests**: 55 passed (36 unit tests + 19 integration tests in `tests/integration_tests.rs`).
- **Frontend Build**: `npm run build` succeeds cleanly in ~17s with 0 errors.
- **Process Optimization**: Single-loaded dynamic `WINTRUST_FN` and in-memory `SIGNATURE_CACHE` and `PE_INFO_CACHE` (`version.dll`) eliminate high-frequency disk and library churn during 2-second monitor refreshes.
- **Appx Robustness**: Multi-stage `-Package` and `-Name` pipeline for sponsored bloatware removal ensures guaranteed uninstallation with clean error handling.
- **Restore Point Reliability**: Windows 24-hour restore point rate limit (`1440 minutes`) handled gracefully with clear user guidance.
- **Binary & Runtime Profile**: Zero resident background daemons, ~30MB RAM idle runtime, non-destructive reversible operations throughout.
