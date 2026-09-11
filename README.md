# Vapor (ClearDeck Engine) ⚡

> **Ultra-lightweight, performant, native Windows PC triage and cleanup assistant.**  
> Built with **Tauri 2.0 (Rust backend)** and **Svelte 5 (Runes) + Tailwind CSS**.

![Rust](https://img.shields.io/badge/Rust-1.77%2B-orange.svg)
![Tauri](https://img.shields.io/badge/Tauri-2.0-blue.svg)
![Svelte](https://img.shields.io/badge/Svelte-5-red.svg)
![TailwindCSS](https://img.shields.io/badge/Tailwind-3.4-cyan.svg)
![License](https://img.shields.io/badge/License-MIT-emerald.svg)

---

## 🌟 Core Philosophy: Zero-Bloat Performance

Traditional Windows optimizers often defeat their own purpose: they install persistent background resident daemons, consume 150MB+ RAM, trigger unnecessary notifications, and indiscriminately delete files without safeguards.

**Vapor takes a fundamentally different approach:**
- **Zero Resident Daemons**: When closed, Vapor exits completely. No background services, no telemetry workers.
- **Ultra-Low Memory Footprint**: Runs at **~30MB to 40MB RAM** at idle.
- **Rayon & Jwalk Concurrency**: Multi-threaded parallel file walking across NVMe and SSD drives.
- **Non-Destructive Cleanup**: Every deletion passes through an interactive preview modal and stages into a reversible **Rescue Bin** before permanent purge.
- **Privacy-First AI**: Outbound calls to Google Gemini 2.5 Flash strip usernames, personal paths, and hostnames prior to network dispatch.

---

## 🚀 Key Features (v0.2.0)

### 1. 0–100 System Health Score & "Top 3 Actions"
- **Dynamic 4-Pillar Composite Score**:
  - **Disk Headroom (35%)**: Storage margin on primary volume with standardized color bands (<10% rose, <20% amber, >=20% emerald) and contextual live sublabels.
  - **Active Memory Pressure (25%)**: Real-time RAM consumption load (`X GB used (Y%)`).
  - **Safe Cleanup Volume (20%)**: Accumulated temp, scratch, and crash logs (`X MB cleanable`).
  - **Startup & Zombie Drag (20%)**: Combined boot drag and dormant applications (`X apps (Y boot, Z dormant)`).
- **Decision Engine**: Eliminates decision fatigue by curating:
  - ⚡ **Fastest Win**: 1-click safe cleanup.
  - 💾 **Biggest Win**: Dormant build directory (`node_modules`, `target`, `.venv`) or large file cleanup. Shows "Storage Audit Pending" when un-scanned.
  - 🧠 **Smartest Win**: Disabling autostart registry items to accelerate boot time.

### 2. Developer Diet & Storage Explorer
- **Non-Destructive Recycle Bin Purge**: Dev directory purges now use `trash::delete` to move folders to the native Windows Recycle Bin rather than destructive permanent unlinking.
- **Developer Diet Scanner**: Specifically hunts dormant `node_modules`, Rust `target/`, Python `.venv`, Next.js `.next`, and build outputs untouched for >30 days.
- **Reveal in File Explorer**: 1-click "Open Folder" (`show_in_folder`) buttons for build directories, large files, and installed applications.
- **Targeted Drive Filtering**: Clickable drive cards trigger scoped scans for selected drive mount roots.
- **Large Files & Old Downloads**: Configurable threshold (bound to user settings) across user profiles with age badges.

### 3. Safe Cleanup Engine & Reversible Rescue Bin
- **Interactive Exclusion Filtering**: Checkboxes per file with "Select All" toggle and dynamic live reclaim calculations.
- **Large Sample Cap Banner**: Clear notification when review results exceed 500 files, capping rendered DOM elements while staging all matching rule paths safely.
- **Reversible Rescue Bin**: Cleaned files are isolated into `%LOCALAPPDATA%\Vapor\RescueBin\<Stage_ID>\` with a JSON manifest.
- **Manifest Inspector Accordion**: Inspect staged file paths and sizes directly inside the Rescue Bin drawer without needing to restore first.
- **Human-Readable Timestamps & Partial Restore Diagnostics**: Stages display clean dates ("Cleanup Run • Sept 10 (2 hours ago)") and surface itemized path errors if any locked files fail restoration.

### 4. Process Radar & Gemini Batch Fleet AI Audit
- **Batch Gemini Fleet Audit**: Analyzes top 15–20 active unknown and heavy processes (with multi-instance deduplication) in a single token-optimized prompt, returning safe/caution/bloatware/critical breakdowns, overall fleet summary, and prioritized recommendations.
- **In-Memory & UI Caching**: Explanations from fleet audits are indexed in memory and frontend stores (normalized with and without `.exe`), instantly populating row badges and individual modal queries without redundant API calls.
- **Freeze on Hover & Live Feed Pause**: Hovering over the process table automatically freezes rows to prevent jumping during 2s sampling; dedicated Pause/Resume button gives full manual control.
- **Immediate Offline Dictionary + Optional Deep Dive**: Built-in verification (`known_processes.json`) displays publisher, description, and safety classification instantly upon modal open; optional "Deep Dive with Gemini" button performs cloud reasoning on demand.
- **Inline Gemini API Key Entry**: If an API key is not yet configured, users can paste and save it directly inside the diagnostics modal.

### 5. Installed Apps & Reversible Startup Radar
- **Reversible Startup Management**: Disabling autostart software safely archives entries to `%LOCALAPPDATA%\Vapor\disabled_startup.json` (supporting both 64-bit and `WOW6432Node` branches) so they can be re-enabled anytime.
- **Elevation Detection**: Detects non-elevated permissions and displays an "Admin Required" shield badge for HKLM startup items.
- **Installed Apps Actions**: Direct "Uninstall" invocation (with shell escaping guardrails), "Open Folder" in Explorer, and "Search Web" (Google search via `rundll32` URL dispatch) for unknown packages.
- **Zombie App Detection**: Flags heavy apps (>300MB) untouched for >60 days using Prefetch execution traces.

### 6. Universal Safety & Custom Modal Dialogs
- **In-App Confirmation Modals**: Every destructive or elevated action (kill process, purge build cache, restore rescue stage, enable/disable startup) uses an accessible in-app `ConfirmModal.svelte` dialog, eliminating all native blocking `window.confirm()` and `window.alert()` popups.

---

## 🛠️ Project Structure

```
jolly-raman/
├── src-tauri/                     # Native Rust Backend
│   ├── Cargo.toml                 # tauri, sysinfo, jwalk, rayon, winreg, reqwest
│   ├── tauri.conf.json            # Window styling & security config
│   ├── resources/
│   │   ├── cleanup_rules.json     # Extensible JSON rules catalog
│   │   └── known_processes.json   # Offline dictionary of Windows binaries
│   ├── tests/
│   │   └── integration_tests.rs   # End-to-end integration test suite
│   └── src/
│       ├── main.rs                # Windows entrypoint
│       ├── lib.rs                 # Tauri commands & plugin registry
│       ├── core/
│       │   ├── health.rs          # 0-100 composite health formula
│       │   └── rules.rs           # cleanup_rules.json loader & path evaluator
│       ├── storage/
│       │   ├── scanner.rs         # Non-blocking parallel directory walker
│       │   ├── large_files.rs     # Oversized files filter
│       │   ├── dev_diet.rs        # Developer cache hunter (node_modules, target)
│       │   └── rescue_bin.rs      # Reversible staging & restore engine
│       ├── process/
│       │   ├── monitor.rs         # Sampled CPU/RAM monitor
│       │   └── known_db.rs        # Local lookup dictionary
│       ├── apps/
│       │   ├── installed.rs       # 32/64-bit registry auditor
│       │   ├── prefetch.rs        # Execution traces for "Estimated Last Used"
│       │   └── startup.rs         # Autostart registry items & impact detector
│       └── ai/
│           ├── sanitize.rs        # Privacy path & username scrubber
│           └── gemini.rs          # Gemini 3.8 / 3.7 / 2.5 Flash client with Google Search grounding & persistent disk cache
├── src/                           # Svelte 5 Frontend
│   ├── main.js                    # Vite entrypoint
│   ├── app.css                    # Tailwind CSS + Fluent Dark design tokens
│   ├── App.svelte                 # Primary tab shell & global modals
│   └── lib/
│       ├── api.js                 # Tauri IPC invoke wrappers
│       ├── utils.js               # Byte formatters, timeAgo, date helpers
│       ├── stores/
│       │   ├── system.svelte.js   # Svelte 5 rune store for system & hardware
│       │   └── settings.svelte.js # Svelte 5 rune store for preferences
│       └── components/
│           ├── common/            # Modal, Badge, ProgressBar, Card
│           ├── dashboard/         # HealthGauge, TopActionsCard, QuickStats
│           ├── storage/           # DriveCards, LargeFilesTable, DevDietView
│           ├── cleanup/           # RuleChecklist, CleanupPreviewModal, RescueBinDrawer
│           ├── processes/         # ProcessTable, GeminiExplainModal
│           ├── apps/              # InstalledAppsTable, StartupRadar
│           └── settings/          # SettingsView (BYOK Gemini API key)
├── package.json
├── tailwind.config.js
└── vite.config.js
```

---

## 📦 Building & Development

### Prerequisites
- [Node.js](https://nodejs.org) (v18+)
- [Rust & Cargo](https://rustup.rs) (1.77+)
- Windows 10/11 64-bit

### 1. Install Dependencies
```bash
npm install
```

### 2. Run Tests
```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

### 3. Build Web Assets
```bash
npm run build
```

### 4. Run Development Application
```bash
npx @tauri-apps/cli dev
```

### 5. Build Native Windows Executable
```bash
npx @tauri-apps/cli build
```

---

## 🔒 Security & Privacy

1. **No Outbound Network Connections by Default**: Vapor does not connect to any servers unless the user explicitly triggers "Ask AI" on a process.
2. **Metadata Sanitizer**: Outbound AI queries strip all username paths (`C:\Users\<Name>\...` is anonymized to generic category like `AppData/Local/...`), hostnames, and IP addresses.
3. **Reversible Rescue Bin**: Cleanup operations do not immediately call `std::fs::remove_file`. Files are staged with original paths in `manifest.json` for 1-click restore.
