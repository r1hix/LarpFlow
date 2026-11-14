"""Unit tests for configuration loading and directory resolution."""
from pathlib import Path
from larpflow.core import config


def test_config_dir_exists():
    assert config.CONFIG_DIR.exists()


def test_storage_dir_exists():
    assert config.STORAGE_DIR.exists()


def test_load_system_prompt():
    prompt = config.load_system_prompt()
    assert isinstance(prompt, str)
    assert len(prompt) > 50


def test_load_persona():
    persona = config.load_persona()
    assert isinstance(persona, dict)
    assert "developer" in persona


def test_load_linkedin_profile():
    profile = config.load_linkedin_profile()
    assert isinstance(profile, str)
