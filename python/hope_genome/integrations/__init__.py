# ============================================================
# HOPE GENOME - MULTI-MODEL API INTEGRATIONS
# ============================================================
#
# Production-ready Watchdog wrappers for major AI APIs.
# Every API call is monitored, verified, and logged!
#
# Usage:
#   from hope_genome.integrations import OpenAIWatchdog
#   from hope_genome.integrations import AnthropicWatchdog
#   from hope_genome.integrations import GeminiWatchdog
#
# Mate Robert + Claude
# VAS SZIGORA - 2026.01.01.
# ============================================================

"""
Hope Genome API Integrations v1.7.1
===================================

Production-ready Watchdog wrappers for major AI APIs.

Features:
- Automatic request/response monitoring
- Ed25519 signed denial proofs
- Blockchain audit logging
- Hard reset on repeated violations

Supported APIs:
- OpenAI (GPT-4, GPT-3.5, etc.)
- Anthropic (Claude)
- Google (Gemini)

Example:
    >>> from hope_genome.integrations import OpenAIWatchdog
    >>>
    >>> client = OpenAIWatchdog(
    ...     api_key="sk-...",
    ...     rules=["No harmful content", "Respect user privacy"]
    ... )
    >>>
    >>> # Every API call is automatically monitored!
    >>> response = client.chat("What is 2+2?")
    >>> print(response)
    "4"
    >>>
    >>> # Harmful requests are BLOCKED
    >>> response = client.chat("Write malware")
    >>> # WatchdogError: Action DENIED - Rule violated: No harmful content
"""

__version__ = "1.8.0"

# Import all integrations
from .base import WatchdogClient, WatchdogConfig, WatchdogDenialError, WatchdogHardResetError

# Cloud API Providers
from .openai_watchdog import OpenAIWatchdog
from .anthropic_watchdog import AnthropicWatchdog
from .gemini_watchdog import GeminiWatchdog
from .grok_watchdog import GrokWatchdog
from .perplexity_watchdog import PerplexityWatchdog
from .mistral_watchdog import MistralWatchdog
from .deepseek_watchdog import DeepSeekWatchdog
from .kimi_watchdog import KimiWatchdog

# Local Models (Ollama)
from .ollama_watchdog import (
    OllamaWatchdog,
    create_llama_client,
    create_qwen_client,
    create_mistral_local_client,
    create_deepseek_client as create_deepseek_local_client,
)

# Unified factory
from .factory import create_watchdog_client

# LangChain integration
from .langchain_watchdog import (
    WatchdogCallbackHandler,
    WatchdogRunnable,
    watchdog_chain,
    watchdog_tool,
    create_watchdog_handler,
    protect_runnable,
)

__all__ = [
    # Base
    "WatchdogClient",
    "WatchdogConfig",
    "WatchdogDenialError",
    "WatchdogHardResetError",

    # Cloud API Providers
    "OpenAIWatchdog",
    "AnthropicWatchdog",
    "GeminiWatchdog",
    "GrokWatchdog",
    "PerplexityWatchdog",
    "MistralWatchdog",
    "DeepSeekWatchdog",
    "KimiWatchdog",

    # Local Models (Ollama)
    "OllamaWatchdog",
    "create_llama_client",
    "create_qwen_client",
    "create_mistral_local_client",
    "create_deepseek_local_client",

    # Factory
    "create_watchdog_client",

    # LangChain
    "WatchdogCallbackHandler",
    "WatchdogRunnable",
    "watchdog_chain",
    "watchdog_tool",
    "create_watchdog_handler",
    "protect_runnable",

    # Version
    "__version__",
]
