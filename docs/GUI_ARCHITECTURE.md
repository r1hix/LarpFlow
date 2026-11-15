# LarpFlow Desktop GUI — Architecture & Technical Specification

> **A native, zero-latency desktop frontend for LarpFlow written in Rust.**  
> Powered by `eframe` / `egui`, direct SQLite history querying, and an asynchronous headless Python CLI bridge.

---

## 1. System Overview & Philosophy

LarpFlow's desktop GUI provides developers with a visual studio to inspect local git repositories, auto-infer narrative context from commits and diffs, preview and fine-tune platform-specific dev-logs, and safely dispatch to developer social networks.

### Design Principles:
1. **Zero Electron / Web Overhead:** Built 100% in pure Rust using `egui`/`eframe`. Starts in $<150\text{ms}$, consumes $<30\text{MB}$ RAM, and renders natively at 60+ FPS via OpenGL / Metal.
2. **Dual-Channel Inter-Process Communication:**
   - **Direct SQLite Ingestion (0ms):** Reads and mutates `storage/history.sqlite` directly using `rusqlite`, eliminating Python process startup overhead when browsing past drafts or saving edits.
   - **Asynchronous Subprocess Bridge:** Heavy LLM generation and network social dispatches run in isolated background threads via `larpflow`'s structured JSON CLI API (`--json`), keeping the UI responsive.
3. **Strict Human-in-the-Loop Safety:** Drafts are never published without explicit review. Live indicators alert users when `DRY_RUN` is active vs. live mode.
4. **Live Constraint Validation:** Real-time character counters ($\le 275$ for X tweets) and word counters ($140-350$ words for LinkedIn) prevent social network API rejections before dispatch.

---

## 2. Architecture Diagram

```
┌────────────────────────────────────────────────────────────────────────┐
│                   LarpFlow Rust GUI (eframe / egui)                    │
│                                                                        │
│   ┌────────────────┐  ┌──────────────────┐  ┌───────────────────────┐  │
│   │ 🚀 Generator   │  │ 📝 Review & Gate │  │ 📜 History & Doctor   │  │
│   │   - Git picker │  │   - X (chars/275)│  │   - SQLite Table      │  │
│   │   - Focus type │  │   - LinkedIn (W) │  │   - Health badges     │  │
│   │   - Media pick │  │   - Reddit MD    │  │   - Diagnostic report │  │
│   └───────┬────────┘  └────────┬─────────┘  └───────────┬───────────┘  │
└───────────┼────────────────────┼────────────────────────┼──────────────┘
            │                    │                        │
       Async MPSC           Async MPSC              Direct Rusqlite
       (Non-blocking)       (Non-blocking)          (Zero latency)
            │                    │                        │
            ▼                    ▼                        ▼
 ┌──────────────────────────────────────────────┐  ┌─────────────────────┐
 │       Python Engine Bridge (`backend.rs`)    │  │ storage/            │
 │                                              │  │ history.sqlite      │
 │  `python -m larpflow.core.cli run ... --json`│  │ (drafts, posts,     │
 │  `python -m larpflow.core.cli send ... --json`  │ status, timestamps) │
 └──────────────────────┬───────────────────────┘  └─────────────────────┘
                        │
                        ▼
 ┌──────────────────────────────────────────────┐
 │             LarpFlow Core Pipeline           │
 │                                              │
 │  1. Ingestor (Git diffs, commits, AST)       │
 │  2. Narrative Stylist (Gemini/OpenAI LLM)    │
 │  3. Modular Rule Checker & Surgic Retries    │
 │  4. Dispatcher (X, LinkedIn, Reddit, Discord)│
 └──────────────────────────────────────────────┘
```

---

## 3. Core Component Hierarchy

The GUI crate lives in the `gui/` directory and is organized cleanly:

```
gui/
├── Cargo.toml               # Native dependencies (eframe, egui, rusqlite, rfd, serde)
├── README.md                # Quickstart instructions
└── src/
    ├── main.rs              # Application entrypoint & native window options
    ├── app.rs               # Central event loop, navigation tabs, async channel polling
    ├── models.rs            # Strongly typed Rust models matching LarpFlow schema
    ├── backend.rs           # Python CLI bridge + direct rusqlite data layer
    └── views/
        ├── mod.rs           # View module declarations
        ├── generator.rs     # Local git picking, commit slider, focus selector, platform toggles
        ├── preview.rs       # Tabbed platform editors, rule verification gauges, dispatch gate
        ├── history.rs       # Filterable SQLite history grid, instant draft re-loader
        └── settings.rs      # System health doctor (API keys, models, dry-run mode, paths)
```

### Detailed Component Roles:

#### 1. `GeneratorView` (`src/views/generator.rs`)
- **Local Git Ingestion:** Browse and select local git repositories via native OS file dialog (`rfd`) or target remote GitHub `owner/repo`.
- **Commit Depth:** Configurable slider ($1$ to $50$ commits, defaulting to $10$).
- **Narrative Focus:** Mode selector:
  - `auto`: Inferred directly from recent diffs and commit messages.
  - `bug`: Deep technical post-mortem, edge-case failure, and root cause fix.
  - `architecture`: Trade-offs, design patterns, and structural choices.
  - `motivation`: Problem context and mission behind the project.
  - `showcase`: Visual demo and feature highlights.
  - `full_project`: End-to-end holistic narrative.
  - `tech_stack`: In-depth breakdown of dependencies and tools used.
- **Media Attachment:** Browse and attach gameplay captures, architecture diagrams, or screenshots.
- **Selective Platform Routing:** Checkboxes to select which channels to generate for (`x`, `linkedin`, `reddit`, `discord`, `facebook`, `github`).

#### 2. `PreviewView` (`src/views/preview.rs`)
- **Live Rule Validation:**
  - **X (Twitter):** Visual character count gauge per tweet. Warns in red if $>275$ characters; confirms in green when compliant. Supports adding and removing tweets from thread.
  - **LinkedIn:** Word counter validating against the optimal $140–350$ word guideline.
  - **Reddit:** Subreddit selector, title input, markdown body preview, and auto-suggested subreddits.
  - **Discord:** Embed title and description cards with webhook media preview.
  - **Facebook & GitHub README:** Formatted showcase post and architecture section.
- **Draft Persistence:** "Save Edits to SQLite" button updates the database directly without re-running the LLM.
- **Gatekeeper Dispatch:**
  - Mode badge clearly states `⚠️ DRY RUN` or `● LIVE DISPATCH`.
  - Target platform selectors for granular channel control.
  - Audit log table displaying dispatch states (`OK`, `FAILED`, `SKIPPED`, `DRY RUN`) and live post URLs.

#### 3. `HistoryView` (`src/views/history.rs`)
- Queries SQLite directly via `rusqlite` for instantaneous display.
- Search filter for repository names.
- Status filter combo (`all`, `posted`, `pending`, `partial`, `dry_run`).
- "Load into Review" button pulls any past draft directly into the active review workspace.

#### 4. `SettingsView` (`src/views/settings.rs`)
- Executes `larpflow doctor --json` asynchronously.
- Visual badges for:
  - Dry Run mode state.
  - LLM API key & selected model (`gemini-3.8-flash`, `gpt-4o-mini`, etc.).
  - GitHub token status.
  - Social platform configurations (X, LinkedIn, Reddit, Discord, Facebook, Telegram).
  - LinkedIn profile background markdown status (`config/linkedin_profile.md`).
- Environment paths (Python binary, project root, SQLite database).

---

## 4. Communication Protocol & Data Models

### 1. Generation Request (`RunParams`)
```json
{
  "repo": "/Users/developer/my-project",
  "commits": 10,
  "focus_type": "bug",
  "problem": "High memory consumption on large replay buffers",
  "solution": "Replaced dynamic Vec with a pre-allocated ring buffer",
  "image_path": "/path/to/screenshot.png",
  "target_platforms": ["x", "linkedin", "reddit", "discord"]
}
```

### 2. Generation Output (`RunResponse`)
```json
{
  "draft_id": 16,
  "repo": "my-project",
  "drafts": {
    "tweets": ["Tweet 1...", "Tweet 2..."],
    "linkedin": "Structured post-mortem...",
    "reddit_subreddit": "SideProject",
    "reddit_title": "How we eliminated memory leaks in our replay system",
    "reddit_body": "### The Bottleneck\n...\n### The Fix\n...",
    "discord_title": "Dev-Log #16: Ring Buffer Optimization",
    "discord_body": "...",
    "github_readme_section": "### Architecture & Trade-offs\n...",
    "visual_asset": "/path/to/screenshot.png"
  },
  "file": "storage/drafts/draft_16_my-project.md"
}
```

### 3. Dispatch Result (`DispatchResult`)
```json
{
  "draft_id": 16,
  "results": [
    { "platform": "linkedin", "state": "ok", "detail": "urn:li:share:718291..." },
    { "platform": "discord", "state": "ok", "detail": "Webhook 204 No Content" },
    { "platform": "x", "state": "dry", "detail": "would post (DRY_RUN is on)" }
  ]
}
```

---

## 5. Security & Safety Model

1. **Explicit Human Confirmation:** Live dispatch is impossible without clicking the visual dispatch button and confirming the target channels.
2. **Safe Defaults:** If `DRY_RUN=true` is set in `.env`, the GUI shows a prominent amber badge and performs full simulation runs without publishing.
3. **No Credential Exposure:** Secrets and tokens in `.env` are never logged, echoed, or displayed in plaintext in the UI; only configuration status booleans (`CONFIGURED` / `MISSING`) are rendered.
4. **Offline Local Git Ingestion:** Local repositories are inspected strictly via git subprocess or pygit without sending codebase files over the network.

