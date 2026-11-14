"""Unit tests for LLM utilities and JSON response parsing."""
import pytest

from larpflow.core import config, llm


def test_parse_json_plain():
    raw = '{"name": "larpflow", "version": "1.1.0"}'
    parsed = llm.parse_json(raw)
    assert parsed["name"] == "larpflow"
    assert parsed["version"] == "1.1.0"


def test_parse_json_markdown_fences():
    raw = """
    Here is the requested output:
    ```json
    {
        "status": "ok",
        "items": [1, 2, 3]
    }
    ```
    Hope this helps!
    """
    parsed = llm.parse_json(raw)
    assert parsed["status"] == "ok"
    assert parsed["items"] == [1, 2, 3]


def test_parse_json_raw_code_block():
    raw = "```\n{\"key\": \"val\"}\n```"
    parsed = llm.parse_json(raw)
    assert parsed["key"] == "val"


def test_parse_json_invalid():
    with pytest.raises(ValueError):
        llm.parse_json("No json here at all.")


def test_get_fallback_models_env_override(monkeypatch):
    monkeypatch.setenv("LLM_FALLBACK_MODELS", "custom-model-1, custom-model-2")
    fallbacks = llm.get_fallback_models()
    assert fallbacks == ["custom-model-1", "custom-model-2"]


def test_get_fallback_models_gemini(monkeypatch):
    monkeypatch.delenv("LLM_FALLBACK_MODELS", raising=False)
    monkeypatch.setattr(config, "LLM_BASE_URL", "https://generativelanguage.googleapis.com/v1beta/openai/")
    monkeypatch.setattr(config, "LLM_MODEL", "gemini-3.8-flash")
    fallbacks = llm.get_fallback_models()
    assert any("gemini" in m for m in fallbacks)


def test_get_fallback_models_openai(monkeypatch):
    monkeypatch.delenv("LLM_FALLBACK_MODELS", raising=False)
    monkeypatch.setattr(config, "LLM_BASE_URL", "")
    monkeypatch.setattr(config, "LLM_MODEL", "gpt-4o-mini")
    fallbacks = llm.get_fallback_models()
    assert "gpt-4o-mini" in fallbacks or "gpt-4o" in fallbacks
