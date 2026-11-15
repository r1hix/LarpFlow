# LarpFlow Native Rust GUI (`larpflow-gui`)

> **High-performance native desktop GUI for LarpFlow developer log generator.**  
> Built with Rust, `eframe` / `egui`, and `rusqlite`.

---

## 🚀 Running the GUI

From the root of the repository:

```bash
cd gui
cargo run
```

Or from the workspace root directly:

```bash
cargo run --manifest-path gui/Cargo.toml
```

---

## 🛠 Features

1. **🚀 Generator View:**
   - Local git repository picker with native folder selection dialog (`rfd`).
   - Configurable commit inspection depth slider ($1-50$).
   - Narrative focus selector (`auto`, `bug`, `architecture`, `motivation`, `showcase`, `full_project`, `tech_stack`).
   - Media & screenshot attachment selector.
   - Selective platform checkboxes (`x`, `linkedin`, `reddit`, `discord`, `facebook`, `github`).
   - Asynchronous background LLM generation without GUI blocking.

2. **📝 Review & Gatekeeper View:**
   - Platform tabs for LinkedIn, X (Twitter Thread), Reddit, Discord, Facebook, and GitHub README.
   - Live constraint meters:
     - X: Character count per tweet with visual gauge ($\le 275$ limit warning/OK).
     - LinkedIn: Word count meter ($140-350$ words guideline indicator).
   - Editable drafts with "💾 Save Edits to SQLite" button.
   - One-click social dispatch with live / dry-run safety indicator.
   - Real-time audit log with post URLs and states (`OK`, `FAILED`, `SKIPPED`, `DRY RUN`).

3. **📜 History View:**
   - Direct embedded SQLite queries into `storage/history.sqlite` ($0\text{ms}$ latency).
   - Real-time repository search filter.
   - Status filtering (`posted`, `pending`, `partial`, `dry_run`).
   - "Load into Review" button to instantly inspect or re-dispatch past drafts.

4. **⚙ Settings & Doctor View:**
   - 1-click health diagnostics checking LLM keys, model, Telegram bot, LinkedIn profile, and social credentials.
   - Environment and binary path inspector.

---

## 📦 Building a Release Binary

```bash
cargo build --release
```

Binary is saved at `gui/target/release/larpflow-gui`.

