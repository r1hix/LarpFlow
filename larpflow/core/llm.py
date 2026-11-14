"""Tiny wrapper around an OpenAI-compatible API (OpenAI, Gemini, Ollama, and others)."""
import json
import os
import re
import time

from openai import OpenAI

from . import config


def _client():
    if not config.LLM_API_KEY:
        raise RuntimeError("LLM_API_KEY is empty. Put it in .env first.")
    kwargs = {
        "api_key": config.LLM_API_KEY,
        "timeout": config.LLM_TIMEOUT,
    }
    if config.LLM_BASE_URL:
        kwargs["base_url"] = config.LLM_BASE_URL
    return OpenAI(**kwargs)


def get_fallback_models():
    """Returns sensible fallback models tailored to the active LLM provider."""
    # 1. Explicit user override via environment variable
    custom = config.env("LLM_FALLBACK_MODELS")
    if custom:
        return [m.strip() for m in custom.split(",") if m.strip()]

    base_url = (config.LLM_BASE_URL or "").lower()
    model = (config.LLM_MODEL or "").lower()

    # 2. Google Gemini via OpenAI compatibility endpoint
    if "googleapis.com" in base_url or "generativelanguage" in base_url or "gemini" in model:
        return [
            "gemini-3.8-flash",
            "gemini-3.7-flash",
            "gemini-2.5-flash",
            "gemini-flash-latest",
        ]

    # 3. OpenAI official API
    if not base_url or "api.openai.com" in base_url or model.startswith("gpt-"):
        return ["gpt-4o-mini", "gpt-4o"]

    # 4. Custom / Local / OpenRouter / Groq / vLLM: retry user model only without assuming third-party names
    return []


def chat(system, user, json_mode=False, temperature=0.7):
    client = _client()
    fallbacks = get_fallback_models()
    models_to_try = [config.LLM_MODEL] + [m for m in fallbacks if m != config.LLM_MODEL]
    last_err: Exception | None = None

    for model in models_to_try:
        args = dict(
            model=model,
            temperature=temperature,
            timeout=config.LLM_TIMEOUT,
            messages=[
                {"role": "system", "content": system},
                {"role": "user", "content": user},
            ],
        )
        for attempt in range(2):
            try:
                if json_mode:
                    try:
                        resp = client.chat.completions.create(response_format={"type": "json_object"}, **args)
                    except Exception as inner_e:
                        err_s = str(inner_e)
                        # Do not burn an attempt with non-json if it is an infrastructure / quota / auth error
                        if any(s in err_s for s in ["503", "429", "401", "403", "RESOURCE_EXHAUSTED", "Quota exceeded"]) or "timeout" in err_s.lower():
                            raise
                        resp = client.chat.completions.create(**args)
                else:
                    resp = client.chat.completions.create(**args)
                return resp.choices[0].message.content or ""
            except Exception as e:
                last_err = e
                err_str = str(e)
                # If model is not found (404) or daily quota is exhausted (RESOURCE_EXHAUSTED), fail over to next model immediately
                if "404" in err_str or "RESOURCE_EXHAUSTED" in err_str or "Quota exceeded" in err_str:
                    break
                # If 503 (high demand) or transient 429, pause briefly and retry once
                if ("503" in err_str or "429" in err_str) and attempt < 1:
                    time.sleep(1.5)
                    continue
                break  # try next model on other errors or after retry

    if last_err is not None:
        raise last_err
    raise RuntimeError("All LLM attempts failed without an exception.")


def parse_json(raw):
    """Finds and parses the JSON object in the LLM answer, handling markdown fences and extraneous text."""
    clean = raw.strip()
    match = re.search(r"```(?:json)?\s*(\{.*?\})\s*```", clean, re.DOTALL)
    if match:
        clean = match.group(1).strip()
    else:
        start = clean.find("{")
        end = clean.rfind("}")
        if start != -1 and end != -1:
            clean = clean[start : end + 1]
        else:
            raise ValueError(f"No JSON object found in model response: {raw[:200]}")
    return json.loads(clean)
