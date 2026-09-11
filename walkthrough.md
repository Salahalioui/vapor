# Vapor v0.2.0 Walkthrough & Feature Guide

## Overview

Vapor v0.2.0 introduces the **Batch Gemini Fleet AI Audit**, non-destructive Recycle Bin developer diet purges, reversible startup management, native Windows Explorer integration, granular exclusion cleanup, and custom in-app confirmation dialogs.

---

## 1. Batch Gemini Fleet AI Audit & Process Radar

### What's New
- **Batch Fleet AI Audit**: Rather than sending individual requests for every single running process, Vapor deduplicates and prioritizes the top 15–20 active unknown and heavy processes, analyzing them in a single structured prompt to Google Gemini 2.5 Flash.
- **In-Memory & UI Caching**: Explanations returned by the fleet audit are automatically indexed into an in-memory and frontend cache (normalized with and without `.exe`). Process table rows display safety badges (`AI: safe`, `AI: bloatware`, `AI: caution`), and clicking "AI Insight" on any row instantly renders the cached analysis without additional network requests.
- **Freeze on Hover**: The 2-second real-time process table sampler automatically freezes when hovering over the table, preventing rows from shifting while inspecting or clicking actions.
- **Manual Live Feed Toggle**: Use the "Pause Feed / Resume Feed" toggle icon button for manual control over live polling.
- **Immediate Offline Dictionary + Optional Deep Dive**: The process explanation modal immediately renders offline Windows metadata (description, publisher, and known safety rating) upon opening, with "Deep Dive with Gemini" as an optional button. Opening the modal never consumes API quota prematurely.
- **Inline Gemini API Key**: If an API key is not yet configured in Settings, users can enter and save it directly in the modal via "Save & Analyze".
- **Deduplicated Fleet Audit Selection**: Automatically excludes duplicate multi-instance child processes (e.g. 15 instances of Chrome/Edge) so 20 distinct active software packages are audited.

---

## 2. Reversible Startup Radar & Apps Actions

### What's New
- **Reversible Startup Archival**: When a startup application is toggled off, Vapor archives its registry path and command line to %LOCALAPPDATA%\Vapor\disabled_startup.json. Toggling it back on writes the entry back to the Windows registry.
- **Elevation Detection**: Detects whether Vapor is running with administrative elevation. Non-elevated instances display an "Admin Required" shield badge on HKLM (system-wide) autostart entries.
- **Installed Apps Actions**:
  - **Uninstall**: Invokes the registered Windows uninstaller or opens Windows Apps & Features.
  - **Open Folder**: Reveals the app installation folder in Windows File Explorer via explorer /select,"<path>".
  - **Search Web**: Opens Google Search in the default browser to quickly look up unfamiliar software.

---

## 3. Storage Explorer & Non-Destructive Developer Diet

### What's New
- **Recycle Bin Staging for Build Artifacts**: Purging build folders (`node_modules`, `target`, `.venv`, `.next`) now utilizes the native Windows Recycle Bin (`trash::delete`). Files are not immediately destroyed and can be restored from the Windows Recycle Bin if needed.
- **Reveal in File Explorer**: Added "Open in Explorer" buttons to large files and developer directories.
- **Targeted Drive Filtering**: Click any drive card in the storage overview to set the target root mount and run scans specific to that partition.
- **Standardized Color Bands**: Partition headroom and health gauge follow consistent color codes:
  - < 10% Free: Rose / Critical
  - < 20% Free: Amber / Caution
  - ≥ 20% Free: Emerald / Healthy

---

## 4. Safe Cleanup & Rescue Bin Drawer

### What's New
- **Granular Exclusion Filtering**: Interactive checkboxes on each item in the Cleanup Preview modal allow excluding individual files. Reclaimable space dynamically updates in real time.
- **Sample Limit Banner**: If more than 500 files match cleanup rules, a clear banner is displayed noting that the preview shows the first 500 sample files for review while all matching non-excluded files are staged safely.
- **Manifest Inspector Accordion**: View the exact list of staged files directly within the Rescue Bin drawer without restoring.
- **Human-Readable Timestamps**: Staged cleanup runs display relative and absolute dates ("Cleanup Run • Today / Xd ago").
- **Partial Restore Reporting**: If any files fail to restore (e.g. locked by another program), Vapor reports the exact failed file paths while successfully restoring the rest.

---

## 5. Universal In-App Confirmation Dialogs

All native, blocking window.confirm() and window.alert() calls have been replaced with the accessible ConfirmModal.svelte component:
- Terminating processes
- Purging developer build artifacts
- Restoring or permanently purging Rescue Bin stages
- Enabling or disabling startup items
