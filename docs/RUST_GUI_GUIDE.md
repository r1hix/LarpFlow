# LarpFlow Desktop GUI — Developer & User Guide

> **A native Rust GUI for inspecting code, generating authentic dev-logs, and managing multi-platform social syndication.**

---

## ⚡ Quickstart

### 1. Prerequisites

Ensure you have the required toolchains installed:
- **Rust & Cargo:** 1.75 or newer (`cargo --version`, `rustc --version`).  
  If not installed, install via:
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Python:** 3.10 or newer with LarpFlow virtual environment configured in `venv/`:
  ```bash
  # In repository root:
  python3 -m venv venv
  source venv/bin/activate
  pip install -e .
  ```

### 2. Run the Desktop App (Development Mode)

From the project root:

```bash
cd gui
cargo run
```

Or from the workspace root directly:

```bash
cargo run --manifest-path gui/Cargo.toml
```

---

## 🖥 Features Walkthrough

### 1. 🚀 Generator Tab
- **Local Git Ingestion:** Enter `.` for the current repository, type an absolute path, or click **📁 Browse Local** to open a native directory picker. LarpFlow inspects commits, diffs, and codebase structure locally without network calls.
- **Commit Slider:** Configure how many commits back to inspect ($1$ to $50$, default $10$).
- **Narrative Focus Selector:** Select narrative angle (`auto`, `bug`, `architecture`, `motivation`, `showcase`, `full_project`, `tech_stack`).
- **Media / Screenshot:** Attach gameplay images, architecture diagrams, or UI screenshots via **🖼 Select File**.
- **Selective Platforms:** Toggle checkboxes for platforms you want drafts generated for (X, LinkedIn, Reddit, Discord, Facebook, GitHub).
- **Generate Drafts:** Fires an asynchronous LLM worker and automatically routes to the **Review & Gatekeeper** tab upon completion.

### 2. 📝 Review & Gatekeeper Tab
- **Platform-Specific Tabs:**
  - **💼 LinkedIn:** Live word count counter ($140-350$ optimal words indicator).
  - **🐦 X (Twitter):** Thread view showing character count per tweet with visual gauge ($\le 275$ chars limit). Supports adding/removing tweets.
  - **🤖 Reddit:** Subreddit selector with auto-suggested subreddits, title field, and structured markdown post-mortem editor.
  - **💬 Discord:** Rich embed title and description cards with webhook preview.
  - **👥 Facebook & 🐙 GitHub:** Clean showcase post and README architecture section editors.
- **Direct SQLite Persistence:** Click **💾 Save Edits to SQLite** to persist text adjustments.
- **Social Dispatcher:**
  - Mode badge shows `⚠️ DRY RUN` or `● LIVE DISPATCH`.
  - Select which platforms to post to.
  - Click **🚀 Dispatch to Socials** to publish.
  - View real-time **Audit Log** with status badges (`OK`, `FAILED`, `SKIPPED`, `DRY RUN`) and direct live URLs.

### 3. 📜 History Tab
- Instantaneous reading of `storage/history.sqlite` via direct embedded SQLite driver.
- Real-time search filter by repository name.
- Status filter: `All`, `Posted`, `Pending`, `Partial`, `Dry Run`.
- Click **🔍 Load into Review** on any historical draft to inspect or re-dispatch.

### 4. ⚙ Settings & Doctor Tab
- Run system health diagnostics with 1 click (**🩺 Run Diagnostics**).
- Verifies API keys, selected LLM model, GitHub tokens, and connected social platform credentials.
- Displays detected Python executable, project root, and database paths.

---

## 📦 Building & Distribution

### 1. Optimized Release Binary

Compile an optimized native binary:

```bash
cd gui
cargo build --release
```

The compiled standalone executable will be located at:
- **macOS / Linux:** `gui/target/release/larpflow-gui`
- **Windows:** `gui/target/release/larpflow-gui.exe`

### 2. Creating a macOS `.app` Bundle

You can package `larpflow-gui` into a native macOS Application bundle:

```bash
# Create bundle structure
mkdir -p "LarpFlow.app/Contents/MacOS"
mkdir -p "LarpFlow.app/Contents/Resources"

# Copy binary
cp gui/target/release/larpflow-gui "LarpFlow.app/Contents/MacOS/LarpFlow"

# Create Info.plist
cat << 'EOF' > "LarpFlow.app/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>LarpFlow</string>
    <key>CFBundleIdentifier</key>
    <string>com.larpflow.studio</string>
    <key>CFBundleName</key>
    <string>LarpFlow</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
</dict>
</plist>
EOF
```

---

## 🛠 Troubleshooting

| Issue | Cause | Solution |
|---|---|---|
| `Doctor command failed: ModuleNotFoundError: No module named 'openai'` | Python path points to system Python instead of venv. | Run `python3 -m venv venv && pip install -e .` in project root. The GUI automatically detects `venv/bin/python`. |
| Black screen or OpenGL crash on Linux | Missing OpenGL/Wayland development libraries. | Install `libx11-dev libxcursor-dev libxrandr-dev libxi-dev libgl1-mesa-dev` on Debian/Ubuntu. |
| macOS Gatekeeper prompt | Standalone unsigned binary. | In Terminal, run `xattr -d com.apple.quarantine gui/target/release/larpflow-gui`. |
