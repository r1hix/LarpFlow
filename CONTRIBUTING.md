# Contributing to LarpFlow

Thank you for your interest in contributing to LarpFlow! We welcome bug fixes, documentation improvements, new social syndication channels, and feature proposals.

---

## 🛠️ Local Development Setup

1. **Fork and Clone the Repository**:
   ```bash
   git clone https://github.com/<your-username>/larpflow.git
   cd larpflow
   ```

2. **Create a Virtual Environment**:
   ```bash
   python3 -m venv venv
   source venv/bin/activate    # Linux / macOS
   # venv\Scripts\activate     # Windows
   ```

3. **Install Package in Editable Mode with Dev Dependencies**:
   ```bash
   pip install -e ".[dev]"
   ```

4. **Verify Installation**:
   ```bash
   larpflow doctor
   ```

---

## 🧪 Running the Test Suite

Always ensure the test suite passes before submitting changes:

```bash
pytest tests/ -v
```

If adding new features (such as rule checks or platform helpers), please add corresponding unit tests in `tests/`.

---

## 🔌 Adding a New Syndication Channel

To add a new platform channel (e.g. Mastodon, Bluesky, Threads):

1. **Create the Tool Module** in `larpflow/tools/tool_<name>.py`:
   - Implement `configured() -> bool`: Returns `True` if necessary API credentials exist in `config.py`.
   - Implement `post(...) -> tuple[str, str]`: Publishes content and returns `(post_id, url)`.
2. **Add Environment Settings** in `larpflow/core/config.py` and `.env.example`.
3. **Register Platform Dispatch** in `larpflow/core/pipeline.py`:
   - Add your platform to `PLATFORMS`.
4. **Define Draft Guidelines & Formatting** in `larpflow/core/drafts.py`:
   - Add validation rules in `platform_problems()`.
   - Update `Drafts` schema model and `build_user_prompt()`.
5. **Add Unit Tests** in `tests/test_drafts.py`.

---

## 📝 Pull Request Guidelines

- Create a feature branch with a descriptive name (e.g., `feat/bluesky-syndication` or `fix/linkedin-media-upload`).
- Follow [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, `test:`, `chore:`).
- Verify all unit tests pass locally on your branch.
- Document any new environment variables in `.env.example` and `README.md`.
