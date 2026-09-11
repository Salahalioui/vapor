# Vapor (ClearDeck Engine) ⚡ v0.3.0

> **Ultra-lightweight, performant, native Windows PC triage, capability rating, and safe debloat assistant.**  
> Built with **Tauri 2.0 (Rust backend)** and **Svelte 5 (Runes) + Tailwind CSS**.

![Rust](https://img.shields.io/badge/Rust-1.77%2B-orange.svg)
![Tauri](https://img.shields.io/badge/Tauri-2.0-blue.svg)
![Svelte](https://img.shields.io/badge/Svelte-5-red.svg)
![TailwindCSS](https://img.shields.io/badge/Tailwind-3.4-cyan.svg)
![License](https://img.shields.io/badge/License-MIT-emerald.svg)
[![Ko-fi](https://img.shields.io/badge/Ko--fi-Donate-F16061?style=flat&logo=ko-fi&logoColor=white)](https://ko-fi.com/salahalioui)
[![PayPal](https://img.shields.io/badge/PayPal-Support-00457C?style=flat&logo=paypal&logoColor=white)](https://paypal.me/salahalioui)

---

## 🌟 Core Philosophy: Honest Optimization & Zero Bloat

Traditional Windows optimizers frequently resemble the very problems they claim to solve: they install persistent background resident daemons, trigger scareware notifications ("You have 3,450 critical errors! Upgrade to Pro now!"), and make destructive, irreversible changes to the system.

**Vapor takes a fundamentally different, transparent approach:**
- **Zero Resident Daemons**: When closed, Vapor exits completely. No persistent background services, no autostart daemons, zero telemetry.
- **Ultra-Low Memory Footprint**: Runs at **~30MB to 40MB RAM** runtime and compiles to a lightweight **<15MB native binary**.
- **Transparent & Honest Communication**: NO commercial scareware language, NO hardcoded fake feedback, NO paywalls. Everything is calculated dynamically from real hardware and system APIs.
- **Max Non-Destructive Reversibility**: File cleanups are staged into a reversible **Rescue Bin**, tweaks modify standard user registry values with instant one-click revert, and optional **Windows System Restore Points** can be created beforehand.
- **Gemini Free Quota Preservation**: Persistent disk caching (`ai_process_cache.json`), offline heuristic rule fallback, and Google Search grounding when online.

---

## 🚀 What's New in v0.3.0

### 1. Dual-Mode UI Flow (Simple Mode & Power User Mode)
- **Simple Mode (Default)**:
  - Clean, welcoming, friendly interface with **zero technical jargon**.
  - **Traffic Light Trust System**:
    - 🟢 **Green (Safe)**: Safe temporary caches, thumbnail databases, and log files.
    - 🟡 **Yellow (Review)**: Optional tweaks and background software that require user choice.
    - 🔴 **Red (Protected)**: Core Windows kernel, essential drivers, and critical services that Vapor will never touch.
  - **1-Click Smart Clean**: Aggregates safe cleanable files and stages them into the Rescue Bin with friendly, honest confirmation.
  - **PC Capabilities & Bottleneck Detective**: Instant plain-English capability grades and hardware bottlenecks.
- **Power User Mode**:
  - Deep system telemetry across 7 dedicated views: **Dashboard**, **Storage Explorer** (Drive cards, Large files, Dev Diet), **Safe Cleanup Checklist**, **Process Radar & Suspicious Guardian**, **Apps & Zombies**, **Windows Debloat Center**, and **Settings**.

### 2. Honest PC Specs & Capability Rating Engine
- **Instant SSD vs HDD Detection**:
  - Utilizes Win32 `IOCTL_STORAGE_QUERY_PROPERTY` querying `StorageDeviceSeekPenaltyProperty`. Drives with `IncursSeekPenalty == 0` are identified as solid-state drives (SSD/NVMe); rotational drives are identified as HDDs.
- **Hardware GPU Enumeration via DXGI**:
  - Direct integration with DirectX Graphics Infrastructure (`dxgi.dll`) to enumerate graphics adapters, VRAM sizes, and distinguish dedicated GPUs from integrated graphics.
- **Dynamic 3-Pillar Capability Scoring (1.0 to 10.0 scale + Letter Grade)**:
  - **Everyday & Office**: Weighted heavily on SSD presence and 8GB+ RAM for 4K video streaming and web responsiveness.
  - **Software Development**: Weighted on 16GB+ RAM, multi-core CPU threads, and fast SSD for compilers, IDEs, and local containers.
  - **3D Gaming**: Weighted on dedicated GPU vs integrated graphics and dedicated VRAM capacity.
- **"Bottleneck Detective"**:
  - Identifies the #1 real hardware bottleneck in plain English (e.g., Mechanical HDD primary drive, low RAM, or integrated graphics).
- **Windows Uptime & "Fast Startup" Trap Alert**:
  - Detects Windows continuous uptime exceeding 7 days and explains why a true "Restart" (not Shutdown) flushes leaked memory and stale driver caches.

### 3. Safe Windows Performance & Debloat Center
- **Safe Reversible Registry Tweaks**:
  - **Disable Start Menu Bing Web Search**: Stops the Start Menu from sending keystrokes to Bing (`BingSearchEnabled = 0`, `DisableSearchBoxSuggestions = 1`).
  - **Hide Taskbar MSN Widgets**: Removes the MSN news and weather ticker (`TaskbarDa = 0`), saving ~150MB RAM.
  - **Disable Game DVR Background Video Capture**: Stops continuous background GPU screen recording (`GameDVR_Enabled = 0`, `AppCaptureEnabled = 0`).
  - **Turn Off P2P Delivery Optimization**: Prevents Windows Update from uploading update packages to external PCs (`DODownloadMode = 0`).
- **Sponsored Appx Bloatware Remover**:
  - Detects pre-installed commercial promotional apps (TikTok, Disney+, Candy Crush, MSN News, Bing Weather, Solitaire) with descriptions and 1-click clean uninstallation via PowerShell.
- **Optional System Restore Point Integration**:
  - Allows generating an instant Windows System Restore checkpoint (`Checkpoint-Computer`) before optimization.

### 4. Suspicious Process Guardian
- **Path Heuristics**:
  - Flags executables running from `%APPDATA%`, `%TEMP%`, or `Downloads` (primary indicators of adware, droppers, and unauthorized miners).
- **Digital Signature Verification (Win32 WinVerifyTrust)**:
  - Cryptographically verifies PE executable signatures using `wintrust.dll`.
  - Process Badges:
    - 🟢 `Verified (Microsoft)`: Cryptographically verified official Windows component.
    - 🔵 `Verified (Known Publisher)`: Digitally signed by a recognized third-party developer.
    - 🟡 `Unsigned`: Unsigned program running from standard application directories.
    - 🔴 `Unsigned in Temp/AppData`: High suspicion alert for unsigned binaries in user temp/data folders.
- **Offline Heuristic & Persistent Cache Explainer**:
  - Transparently falls back to local rules and dictionary when offline or without an API key.

---

## 🛠️ Project Structure

```
jolly-raman/
├── src-tauri/                     # Native Rust Backend
│   ├── Cargo.toml                 # Dependencies: tauri, sysinfo, jwalk, rayon, winreg, reqwest
│   ├── tauri.conf.json            # Window styling & security config (v0.3.0)
│   ├── resources/
│   │   ├── cleanup_rules.json     # Safe cleanup rules catalog
│   │   └── known_processes.json   # Offline dictionary of Windows binaries
│   ├── tests/
│   │   └── integration_tests.rs   # 19 comprehensive end-to-end integration tests
│   └── src/
│       ├── main.rs                # Windows entrypoint
│       ├── lib.rs                 # Tauri commands & plugin registry
│       ├── system/
│       │   ├── specs.rs           # Hardware specs, SSD seek penalty, DXGI GPU, capability math
│       │   └── tweaks.rs          # Reversible registry tweaks, sponsored apps, restore points
│       ├── process/
│       │   ├── monitor.rs         # Live sampled CPU/RAM monitor with Guardian badges
│       │   ├── guardian.rs        # WinVerifyTrust digital signatures & path heuristics
│       │   └── known_db.rs        # Local lookup dictionary
│       ├── storage/
│       │   ├── scanner.rs         # Parallel multi-threaded directory walker
│       │   ├── large_files.rs     # Oversized files filter
│       │   ├── dev_diet.rs        # Developer cache hunter (node_modules, target, .venv)
│       │   └── rescue_bin.rs      # Reversible staging & restore engine
│       ├── apps/
│       │   ├── installed.rs       # Installed software auditor
│       │   └── startup.rs         # Reversible startup items manager
│       ├── core/
│       │   ├── health.rs          # 0-100 composite health formula
│       │   └── rules.rs           # Path evaluator & environment variable expansion
│       └── ai/
│           ├── gemini.rs          # Google Gemini 3.8/3.7/2.5 Flash, search grounding & disk cache
│           └── sanitize.rs        # Privacy scrubber (removes usernames, paths, hostnames)
└── src/                           # Modern Svelte 5 + Tailwind Frontend
    ├── App.svelte                 # Main layout with Dual-Mode toggle
    └── lib/
        ├── api.js                 # Tauri invoke wrapper functions
        ├── stores/
        │   ├── system.svelte.js   # Global reactive state for specs, tweaks, health, processes
        │   └── settings.svelte.js # Settings (UI mode, API key, thresholds)
        └── components/
            ├── simple/            # Friendly Simple Mode components
            │   └── SimpleModeView.svelte
            ├── tweaks/            # Windows Debloat & Performance Center
            │   └── WindowsDebloatView.svelte
            ├── dashboard/         # Health gauge, pillars, top actions
            ├── storage/           # Drive cards, large files, Dev Diet
            ├── cleanup/           # Rules checklist, preview modal, Rescue Bin
            ├── processes/         # Process radar, Guardian badges, Gemini modal
            ├── apps/              # Installed apps, startup radar
            └── settings/          # Configuration & API key management
```

---

## 🧪 Testing & Verification

Vapor is rigorously verified with automated unit and integration tests:

```bash
# Run all backend unit and integration tests (55 passed)
cd src-tauri
cargo test

# Build frontend production assets cleanly
npm run build
```

---

## 📦 Releases & GitHub Actions Automated Builds

Pre-compiled standalone Windows executables and installers are automatically generated via GitHub Actions:

- **Latest Releases**: Available on the [GitHub Releases Page](https://github.com/Salahalioui/vapor/releases).
- **Automated CI/CD**: Every release tag triggers an automated matrix build compiling an optimized standalone `vapor.exe` and modern NSIS installer (`.exe`).
- **Standalone Portable**: Runs with zero installer overhead directly from any folder or USB drive.

---

## ☕ Support the Project

Vapor is 100% free, open-source software built with **zero ads, zero tracking, and zero commercial scareware**. If Vapor helped speed up your PC, freed gigabytes of disk space, or made your Windows experience cleaner, consider supporting independent open-source development!

### 💳 Card / PayPal (via Ko-fi)
- **Ko-fi**: [ko-fi.com/salahalioui](https://ko-fi.com/salahalioui) *(Accepts Debit/Credit Cards & PayPal balance with 0% platform cut on donations)*
- **Direct PayPal**: [paypal.me/salahalioui](https://paypal.me/salahalioui)

### 🪙 Cryptocurrency (USDT - TRON TRC-20)
Support directly via the developer's Bybit exchange wallet:
- **Network**: `TRON (TRC20)`
- **Wallet Address**: `TBQv6e9SixpNDqympDGX3LMRdmKdEj3BHm`
- **Minimum Deposit**: `0.005 USDT`

---

## 📄 License

MIT License &bull; Built with pride for clean, honest Windows computing.
