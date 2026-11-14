"""Loads settings from .env and the config/ folder with Unix XDG and Windows cross-platform compatibility."""
import json
import os
import platform
import re
from pathlib import Path

from dotenv import load_dotenv

# Base paths
PACKAGE_DIR = Path(__file__).resolve().parent.parent  # larpflow/
REPO_ROOT = PACKAGE_DIR.parent


def _resolve_config_dir() -> Path:
    """Resolves config directory across environment, CWD, OS-standard user directories, and package fallbacks."""
    # 1. Explicit environment override
    if os.getenv("LARPFLOW_CONFIG_DIR"):
        p = Path(os.getenv("LARPFLOW_CONFIG_DIR")).expanduser()
        if p.exists():
            return p

    # 2. Local working directory config/ (if running from repo or custom workspace)
    cwd_config = Path.cwd() / "config"
    if cwd_config.is_dir() and ((cwd_config / "system_instructions.md").exists() or (cwd_config / "persona.json").exists()):
        return cwd_config

    # 3. Standard user configuration directory (Unix XDG or Windows APPDATA)
    if platform.system() == "Windows":
        appdata = os.getenv("APPDATA")
        base = Path(appdata) if appdata else Path.home() / "AppData" / "Roaming"
        user_config = base / "larpflow"
    else:
        xdg_config = os.getenv("XDG_CONFIG_HOME")
        base = Path(xdg_config) if xdg_config else Path.home() / ".config"
        user_config = base / "larpflow"

    if user_config.is_dir():
        return user_config

    # 4. Repo root config/ (when running from git clone)
    repo_config = REPO_ROOT / "config"
    if repo_config.is_dir():
        return repo_config

    # 5. Bundled package templates
    pkg_config = PACKAGE_DIR / "config"
    if pkg_config.is_dir():
        return pkg_config

    return user_config


def _resolve_storage_dir() -> Path:
    """Resolves writable storage directory across environment, local repo, and OS data directories."""
    # 1. Explicit environment override
    if os.getenv("LARPFLOW_STORAGE_DIR"):
        p = Path(os.getenv("LARPFLOW_STORAGE_DIR")).expanduser()
        p.mkdir(parents=True, exist_ok=True)
        return p

    # 2. Local working directory storage/ (if running inside repo checkout)
    cwd_storage = Path.cwd() / "storage"
    if cwd_storage.is_dir() or (Path.cwd() / "pyproject.toml").is_file():
        cwd_storage.mkdir(parents=True, exist_ok=True)
        return cwd_storage

    # 3. Standard user data directory (Unix XDG or Windows LOCALAPPDATA)
    if platform.system() == "Windows":
        local_appdata = os.getenv("LOCALAPPDATA")
        base = Path(local_appdata) if local_appdata else Path.home() / "AppData" / "Local"
        user_storage = base / "larpflow" / "storage"
    else:
        xdg_data = os.getenv("XDG_DATA_HOME")
        base = Path(xdg_data) if xdg_data else Path.home() / ".local" / "share"
        user_storage = base / "larpflow" / "storage"

    user_storage.mkdir(parents=True, exist_ok=True)
    return user_storage


def _load_env_files(cfg_dir: Path):
    """Loads environment variables from custom override, CWD, user config dir, or repo root."""
    if os.getenv("LARPFLOW_ENV_FILE"):
        custom = Path(os.getenv("LARPFLOW_ENV_FILE")).expanduser()
        if custom.is_file():
            load_dotenv(custom)
            return

    # 1. CWD .env
    cwd_env = Path.cwd() / ".env"
    if cwd_env.is_file():
        load_dotenv(cwd_env)
        return

    # 2. User config dir .env
    user_env = cfg_dir / ".env"
    if user_env.is_file():
        load_dotenv(user_env)
        return

    # 3. Repo root .env
    repo_env = REPO_ROOT / ".env"
    if repo_env.is_file():
        load_dotenv(repo_env)
        return


CONFIG_DIR = _resolve_config_dir()
STORAGE_DIR = _resolve_storage_dir()
_load_env_files(CONFIG_DIR)


def env(name, default=""):
    return os.getenv(name, default).strip()


def env_bool(name, default=False):
    value = env(name)
    if not value:
        return default
    return value.lower() in ("1", "true", "yes", "y", "on")


# safety switch. True means nothing really gets posted.
DRY_RUN = env_bool("DRY_RUN", True)

LLM_API_KEY = env("LLM_API_KEY")
LLM_BASE_URL = env("LLM_BASE_URL")
LLM_MODEL = env("LLM_MODEL", "gpt-4o-mini")
LLM_TIMEOUT = float(env("LLM_TIMEOUT", "30.0"))

GITHUB_USER = env("GITHUB_USER")
GITHUB_TOKEN = env("GITHUB_TOKEN")
if not GITHUB_TOKEN or not GITHUB_USER:
    try:
        import subprocess
        if not GITHUB_TOKEN:
            res_token = subprocess.run(["gh", "auth", "token"], capture_output=True, text=True, timeout=2)
            if res_token.returncode == 0 and res_token.stdout.strip():
                GITHUB_TOKEN = res_token.stdout.strip()
        if not GITHUB_USER:
            res_user = subprocess.run(["gh", "api", "user", "--jq", ".login"], capture_output=True, text=True, timeout=2)
            if res_user.returncode == 0 and res_user.stdout.strip():
                GITHUB_USER = res_user.stdout.strip()
    except Exception:
        pass
if not GITHUB_USER:
    GITHUB_USER = "user"


TELEGRAM_BOT_TOKEN = env("TELEGRAM_BOT_TOKEN")
TELEGRAM_OWNER_ID = int(env("TELEGRAM_OWNER_ID", "0") or 0)

X_API_KEY = env("X_API_KEY")
X_API_SECRET = env("X_API_SECRET")
X_ACCESS_TOKEN = env("X_ACCESS_TOKEN")
X_ACCESS_TOKEN_SECRET = env("X_ACCESS_TOKEN_SECRET")

REDDIT_CLIENT_ID = env("REDDIT_CLIENT_ID")
REDDIT_CLIENT_SECRET = env("REDDIT_CLIENT_SECRET")
REDDIT_USERNAME = env("REDDIT_USERNAME")
REDDIT_PASSWORD = env("REDDIT_PASSWORD")
REDDIT_USER_AGENT = env("REDDIT_USER_AGENT", "larpflow")
REDDIT_ALLOWED_SUBS = [
    s.strip().replace("r/", "")
    for s in env("REDDIT_ALLOWED_SUBS", "SideProject").split(",")
    if s.strip()
]

LINKEDIN_ACCESS_TOKEN = env("LINKEDIN_ACCESS_TOKEN")
LINKEDIN_PERSON_URN = env("LINKEDIN_PERSON_URN")

DISCORD_WEBHOOK_URL = env("DISCORD_WEBHOOK_URL")

FACEBOOK_PAGE_ID = env("FACEBOOK_PAGE_ID")
FACEBOOK_ACCESS_TOKEN = env("FACEBOOK_ACCESS_TOKEN")


def load_persona():
    candidates = [
        CONFIG_DIR / "persona.json",
        CONFIG_DIR / "persona.example.json",
        PACKAGE_DIR / "config" / "persona.example.json",
        REPO_ROOT / "config" / "persona.example.json",
    ]
    for path in candidates:
        if path.is_file():
            try:
                return json.loads(path.read_text(encoding="utf-8"))
            except Exception:
                pass
    return {"developer": {"tone_guardrails": {"do_not_use": []}}}


def load_system_prompt():
    candidates = [
        CONFIG_DIR / "system_instructions.md",
        PACKAGE_DIR / "config" / "system_instructions.md",
        REPO_ROOT / "config" / "system_instructions.md",
    ]
    for path in candidates:
        if path.is_file():
            return path.read_text(encoding="utf-8")
    raise FileNotFoundError("Could not find system_instructions.md in config directory or package resources.")


def load_linkedin_profile():
    """Returns the pasted LinkedIn text, or fallback example if available."""
    candidates = [
        CONFIG_DIR / "linkedin_profile.md",
        CONFIG_DIR / "linkedin_profile.example.md",
        PACKAGE_DIR / "config" / "linkedin_profile.example.md",
        REPO_ROOT / "config" / "linkedin_profile.example.md",
    ]
    for path in candidates:
        if path.is_file():
            text = re.sub(r"<!--.*?-->", "", path.read_text(encoding="utf-8"), flags=re.S)
            body = "\n".join(
                line for line in text.splitlines() if line.strip() and not line.startswith("#")
            )
            if body.strip():
                return text.strip()
    return ""
