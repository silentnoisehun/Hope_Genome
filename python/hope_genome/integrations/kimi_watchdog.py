# ============================================================
# HOPE GENOME - KIMI (MOONSHOT AI) WATCHDOG INTEGRATION
# ============================================================
#
# Watchdog-protected Kimi/Moonshot AI API client.
# Every request is verified, every response is checked!
#
# Usage:
#   from hope_genome.integrations import KimiWatchdog
#
#   client = KimiWatchdog(
#       api_key="sk-...",
#       rules=["No harmful content"]
#   )
#
#   response = client.chat("What is 2+2?")
#
# Mate Robert + Claude
# VAS SZIGORA - 2026.01.01.
# ============================================================

"""
Kimi (Moonshot AI) Watchdog Integration
=======================================

Production-ready Kimi/Moonshot AI API wrapper with Watchdog protection.

Features:
- Moonshot-v1 models (8k, 32k, 128k context)
- Long context support
- OpenAI-compatible API
- Automatic request filtering

Example:
    >>> from hope_genome.integrations import KimiWatchdog
    >>>
    >>> client = KimiWatchdog(
    ...     api_key="sk-...",
    ...     model="moonshot-v1-8k",
    ...     rules=["No harmful content"]
    ... )
    >>>
    >>> response = client.chat("Summarize this document...")
"""

from typing import Optional, List, Dict, Any, Generator
from .base import WatchdogClient, WatchdogConfig, WatchdogDenialError, APIError

# Kimi uses OpenAI-compatible API
try:
    from openai import OpenAI
    KIMI_AVAILABLE = True
except ImportError:
    OpenAI = None
    KIMI_AVAILABLE = False


class KimiWatchdog(WatchdogClient):
    """
    Watchdog-protected Kimi/Moonshot AI API client.

    Args:
        api_key: Moonshot AI API key
        model: Model to use (default: moonshot-v1-8k)
        config: WatchdogConfig object
        rules: Shortcut to set rules

    Example:
        >>> client = KimiWatchdog(
        ...     api_key="sk-...",
        ...     rules=["No harmful content"]
        ... )
        >>> response = client.chat("Hello!")
    """

    BASE_URL = "https://api.moonshot.cn/v1"

    DEFAULT_MODEL = "moonshot-v1-8k"

    MODELS = {
        # Context length variants
        "moonshot-v1-8k": "moonshot-v1-8k",
        "moonshot-v1-32k": "moonshot-v1-32k",
        "moonshot-v1-128k": "moonshot-v1-128k",

        # Aliases
        "kimi": "moonshot-v1-8k",
        "kimi-8k": "moonshot-v1-8k",
        "kimi-32k": "moonshot-v1-32k",
        "kimi-128k": "moonshot-v1-128k",
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
        """Initialize Kimi client."""
        if not KIMI_AVAILABLE:
            raise ImportError(
                "OpenAI package not installed (required for Kimi). "
                "Install with: pip install openai"
            )

        self._client = OpenAI(
            api_key=self.api_key,
            base_url=self.BASE_URL,
            timeout=self.timeout,
        )

    @property
    def provider_name(self) -> str:
        return "kimi"

    def _make_request(self, prompt: str, **kwargs) -> Any:
        """Make Kimi API request."""
        system = kwargs.pop("system", "You are Kimi, a helpful AI assistant by Moonshot AI.")
        temperature = kwargs.pop("temperature", 0.7)
        max_tokens = kwargs.pop("max_tokens", 2048)

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

    # ========================================================
    # EXTENDED API
    # ========================================================

    def chat_long(self, prompt: str, **kwargs) -> str:
        """
        Chat with 128k context model.

        Args:
            prompt: User message (can be very long)
            **kwargs: Additional options

        Returns:
            Response text
        """
        original_model = self.model
        self.model = "moonshot-v1-128k"

        try:
            return self.chat(prompt, **kwargs)
        finally:
            self.model = original_model

    def stream(
        self,
        prompt: str,
        **kwargs
    ) -> Generator[str, None, None]:
        """Stream chat response."""
        allowed, reason = self.check_request(prompt)
        if not allowed:
            self._record_violation(prompt[:100], reason)
            raise WatchdogDenialError(f"Request DENIED: {reason}")

        system = kwargs.pop("system", "You are Kimi, a helpful AI assistant.")
        temperature = kwargs.pop("temperature", 0.7)
        max_tokens = kwargs.pop("max_tokens", 2048)

        messages = [
            {"role": "system", "content": system},
            {"role": "user", "content": prompt}
        ]

        response = self._client.chat.completions.create(
            model=self.model,
            messages=messages,
            temperature=temperature,
            max_tokens=max_tokens,
            stream=True,
            **kwargs
        )

        full_response = []
        for chunk in response:
            if chunk.choices and chunk.choices[0].delta.content:
                text = chunk.choices[0].delta.content
                full_response.append(text)
                yield text

        full_text = "".join(full_response)
        allowed, reason = self.check_response(full_text)
        if not allowed:
            raise WatchdogDenialError(f"Response BLOCKED: {reason}")

    def summarize(self, long_text: str, **kwargs) -> str:
        """
        Summarize long text using 128k context.

        Args:
            long_text: Text to summarize
            **kwargs: Additional options

        Returns:
            Summary
        """
        prompt = f"Please summarize the following text:\n\n{long_text}"
        return self.chat_long(prompt, **kwargs)


def create_kimi_client(
    api_key: str,
    rules: Optional[List[str]] = None,
    **kwargs
) -> KimiWatchdog:
    """Create Kimi Watchdog client."""
    return KimiWatchdog(api_key=api_key, rules=rules, **kwargs)
