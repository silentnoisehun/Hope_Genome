# ============================================================
# HOPE GENOME - PERPLEXITY WATCHDOG INTEGRATION
# ============================================================
#
# Watchdog-protected Perplexity API client.
# Every request is verified, every response is checked!
#
# Usage:
#   from hope_genome.integrations import PerplexityWatchdog
#
#   client = PerplexityWatchdog(
#       api_key="pplx-...",
#       rules=["No harmful content"]
#   )
#
#   response = client.chat("What is 2+2?")
#
# Mate Robert + Claude
# VAS SZIGORA - 2026.01.01.
# ============================================================

"""
Perplexity Watchdog Integration
===============================

Production-ready Perplexity API wrapper with Watchdog protection.

Features:
- Sonar, Sonar Pro models
- Online search integration
- Automatic request filtering
- Response content analysis

Example:
    >>> from hope_genome.integrations import PerplexityWatchdog
    >>>
    >>> client = PerplexityWatchdog(
    ...     api_key="pplx-...",
    ...     model="sonar",
    ...     rules=["No harmful content"]
    ... )
    >>>
    >>> response = client.chat("What's the latest news?")
"""

from typing import Optional, List, Dict, Any, Generator
from .base import WatchdogClient, WatchdogConfig, WatchdogDenialError, APIError

# Perplexity uses OpenAI-compatible API
try:
    from openai import OpenAI
    PERPLEXITY_AVAILABLE = True
except ImportError:
    OpenAI = None
    PERPLEXITY_AVAILABLE = False


class PerplexityWatchdog(WatchdogClient):
    """
    Watchdog-protected Perplexity API client.

    Args:
        api_key: Perplexity API key (starts with pplx-)
        model: Model to use (default: sonar)
        config: WatchdogConfig object
        rules: Shortcut to set rules

    Example:
        >>> client = PerplexityWatchdog(
        ...     api_key="pplx-...",
        ...     rules=["No harmful content"]
        ... )
        >>> response = client.chat("What's trending today?")
    """

    BASE_URL = "https://api.perplexity.ai"

    DEFAULT_MODEL = "sonar"

    MODELS = {
        "sonar": "sonar",
        "sonar-pro": "sonar-pro",
        "sonar-reasoning": "sonar-reasoning",
    }

    def __init__(
        self,
        api_key: str,
        model: str = DEFAULT_MODEL,
        config: Optional[WatchdogConfig] = None,
        rules: Optional[List[str]] = None,
        timeout: float = 60.0,
        **kwargs
    ):
        self.model = self.MODELS.get(model, model)
        self.timeout = timeout

        super().__init__(api_key, config, rules, **kwargs)

    def _init_client(self, **kwargs):
        """Initialize Perplexity client."""
        if not PERPLEXITY_AVAILABLE:
            raise ImportError(
                "OpenAI package not installed (required for Perplexity). "
                "Install with: pip install openai"
            )

        self._client = OpenAI(
            api_key=self.api_key,
            base_url=self.BASE_URL,
            timeout=self.timeout,
        )

    @property
    def provider_name(self) -> str:
        return "perplexity"

    def _make_request(self, prompt: str, **kwargs) -> Any:
        """Make Perplexity API request."""
        system = kwargs.pop("system", "You are a helpful AI assistant with access to real-time information.")
        temperature = kwargs.pop("temperature", 0.7)
        max_tokens = kwargs.pop("max_tokens", 1024)

        messages = [
            {"role": "system", "content": system},
            {"role": "user", "content": prompt}
        ]

        if "messages" in kwargs:
            messages = kwargs.pop("messages")

        response = self._client.chat.completions.create(
            model=self.model,
            messages=messages,
            temperature=temperature,
            max_tokens=max_tokens,
            **kwargs
        )

        return response

    def _parse_response(self, response: Any) -> str:
        """Extract text from response."""
        if hasattr(response, "choices") and response.choices:
            return response.choices[0].message.content or ""
        return ""

    def search(self, query: str, **kwargs) -> str:
        """
        Search with Perplexity (uses online model).

        Args:
            query: Search query
            **kwargs: Additional options

        Returns:
            Search response with citations
        """
        return self.chat(query, **kwargs)


def create_perplexity_client(
    api_key: str,
    rules: Optional[List[str]] = None,
    **kwargs
) -> PerplexityWatchdog:
    """Create Perplexity Watchdog client."""
    return PerplexityWatchdog(api_key=api_key, rules=rules, **kwargs)
