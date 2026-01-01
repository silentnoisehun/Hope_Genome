# ============================================================
# HOPE GENOME - WATCHDOG CLIENT FACTORY
# ============================================================
#
# Unified factory for creating Watchdog-protected API clients.
# Automatically detects provider from API key format!
#
# Usage:
#   from hope_genome.integrations import create_watchdog_client
#
#   # Auto-detect from key
#   client = create_watchdog_client(api_key="sk-...")
#
#   # Or specify provider
#   client = create_watchdog_client(
#       provider="openai",
#       api_key="sk-...",
#       rules=["No harm"]
#   )
#
# Mate Robert + Claude
# VAS SZIGORA - 2026.01.01.
# ============================================================

"""
Watchdog Client Factory
=======================

Unified factory for creating Watchdog-protected API clients.

Features:
- Auto-detection from API key format
- Unified configuration
- Easy provider switching

Example:
    >>> from hope_genome.integrations import create_watchdog_client
    >>>
    >>> # Auto-detect from key format
    >>> client = create_watchdog_client(api_key="sk-...")
    >>>
    >>> # Explicit provider
    >>> client = create_watchdog_client(
    ...     provider="anthropic",
    ...     api_key="sk-ant-...",
    ...     rules=["No harmful content"]
    ... )
    >>>
    >>> # Use same API across providers!
    >>> response = client.chat("Hello!")
"""

from typing import Optional, List, Union
from .base import WatchdogClient, WatchdogConfig


def create_watchdog_client(
    provider: Optional[str] = None,
    api_key: Optional[str] = None,
    rules: Optional[List[str]] = None,
    config: Optional[WatchdogConfig] = None,
    **kwargs
) -> WatchdogClient:
    """
    Create a Watchdog-protected API client.

    This factory function automatically detects the provider from the API key
    format, or you can specify it explicitly.

    Args:
        provider: Provider name or None for auto-detect
        api_key: API key for the service (not needed for ollama)
        rules: Ethical rules to enforce
        config: WatchdogConfig object
        **kwargs: Provider-specific options

    Supported Providers:
        - openai: GPT-4, GPT-3.5
        - anthropic: Claude 3.5, Claude 3
        - gemini/google: Gemini 2.0, 1.5
        - grok/xai: Grok-2
        - perplexity: Sonar
        - mistral: Mistral Large, Codestral
        - deepseek: DeepSeek-V3, DeepSeek-R1
        - kimi/moonshot: Moonshot-v1
        - ollama: Llama, Qwen, Mistral (local)

    Returns:
        WatchdogClient instance

    Raises:
        ValueError: If provider cannot be detected or is unsupported

    Example:
        >>> # Auto-detect from key
        >>> client = create_watchdog_client(api_key="sk-...")
        >>>
        >>> # Local Ollama
        >>> client = create_watchdog_client(provider="ollama", model="llama3.2")
    """
    # Ollama doesn't need API key
    if provider and provider.lower() == "ollama":
        from .ollama_watchdog import OllamaWatchdog
        return OllamaWatchdog(
            rules=rules,
            config=config,
            **kwargs
        )

    if not api_key:
        raise ValueError("api_key is required (except for ollama)")

    # Auto-detect provider if not specified
    if provider is None:
        provider = _detect_provider(api_key)

    provider = provider.lower()

    # Import lazily to avoid circular imports
    if provider == "openai":
        from .openai_watchdog import OpenAIWatchdog
        return OpenAIWatchdog(api_key=api_key, rules=rules, config=config, **kwargs)

    elif provider == "anthropic":
        from .anthropic_watchdog import AnthropicWatchdog
        return AnthropicWatchdog(api_key=api_key, rules=rules, config=config, **kwargs)

    elif provider in ("gemini", "google"):
        from .gemini_watchdog import GeminiWatchdog
        return GeminiWatchdog(api_key=api_key, rules=rules, config=config, **kwargs)

    elif provider in ("grok", "xai"):
        from .grok_watchdog import GrokWatchdog
        return GrokWatchdog(api_key=api_key, rules=rules, config=config, **kwargs)

    elif provider in ("perplexity", "pplx"):
        from .perplexity_watchdog import PerplexityWatchdog
        return PerplexityWatchdog(api_key=api_key, rules=rules, config=config, **kwargs)

    elif provider == "mistral":
        from .mistral_watchdog import MistralWatchdog
        return MistralWatchdog(api_key=api_key, rules=rules, config=config, **kwargs)

    elif provider == "deepseek":
        from .deepseek_watchdog import DeepSeekWatchdog
        return DeepSeekWatchdog(api_key=api_key, rules=rules, config=config, **kwargs)

    elif provider in ("kimi", "moonshot"):
        from .kimi_watchdog import KimiWatchdog
        return KimiWatchdog(api_key=api_key, rules=rules, config=config, **kwargs)

    else:
        raise ValueError(
            f"Unknown provider: {provider}. "
            f"Supported: openai, anthropic, gemini, grok, perplexity, mistral, deepseek, kimi, ollama"
        )


def _detect_provider(api_key: str) -> str:
    """
    Detect provider from API key format.

    Args:
        api_key: The API key to analyze

    Returns:
        Provider name

    Raises:
        ValueError: If provider cannot be detected
    """
    key = api_key.strip()

    # OpenAI: starts with "sk-" but not "sk-ant-"
    if key.startswith("sk-") and not key.startswith("sk-ant-") and not key.startswith("sk-or-"):
        return "openai"

    # Anthropic: starts with "sk-ant-"
    if key.startswith("sk-ant-"):
        return "anthropic"

    # Google/Gemini: starts with "AIza"
    if key.startswith("AIza"):
        return "gemini"

    # xAI/Grok: starts with "xai-"
    if key.startswith("xai-"):
        return "grok"

    # Perplexity: starts with "pplx-"
    if key.startswith("pplx-"):
        return "perplexity"

    # Mistral: various formats, check for common patterns
    # Mistral keys are typically UUIDs or similar
    # DeepSeek and Kimi also use generic formats

    raise ValueError(
        "Cannot auto-detect provider from API key format. "
        "Please specify provider explicitly: "
        "create_watchdog_client(provider='openai', api_key=...)"
    )


# ============================================================
# PROVIDER REGISTRY
# ============================================================

PROVIDERS = {
    "openai": {
        "name": "OpenAI",
        "key_prefix": "sk-",
        "models": ["gpt-4o", "gpt-4o-mini", "gpt-4-turbo", "gpt-3.5-turbo"],
        "features": ["chat", "streaming", "function_calling", "vision", "embeddings"]
    },
    "anthropic": {
        "name": "Anthropic",
        "key_prefix": "sk-ant-",
        "models": ["claude-3-5-sonnet", "claude-3-opus", "claude-3-haiku"],
        "features": ["chat", "streaming", "tool_use", "vision"]
    },
    "gemini": {
        "name": "Google Gemini",
        "key_prefix": "AIza",
        "models": ["gemini-2.0-flash-exp", "gemini-1.5-pro", "gemini-1.5-flash"],
        "features": ["chat", "streaming", "function_calling", "vision"]
    }
}


def list_providers() -> List[str]:
    """Get list of supported providers."""
    return list(PROVIDERS.keys())


def get_provider_info(provider: str) -> dict:
    """
    Get information about a provider.

    Args:
        provider: Provider name

    Returns:
        Provider info dict
    """
    provider = provider.lower()
    if provider not in PROVIDERS:
        raise ValueError(f"Unknown provider: {provider}")
    return PROVIDERS[provider]


def list_models(provider: str) -> List[str]:
    """
    Get list of available models for a provider.

    Args:
        provider: Provider name

    Returns:
        List of model names
    """
    return get_provider_info(provider)["models"]
