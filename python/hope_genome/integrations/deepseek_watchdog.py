# ============================================================
# HOPE GENOME - DEEPSEEK WATCHDOG INTEGRATION
# ============================================================
#
# Watchdog-protected DeepSeek API client.
# Every request is verified, every response is checked!
#
# Usage:
#   from hope_genome.integrations import DeepSeekWatchdog
#
#   client = DeepSeekWatchdog(
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
DeepSeek Watchdog Integration
=============================

Production-ready DeepSeek API wrapper with Watchdog protection.

Features:
- DeepSeek-V3, DeepSeek-R1 (reasoning)
- DeepSeek-Coder for code
- Automatic request filtering
- OpenAI-compatible API

Example:
    >>> from hope_genome.integrations import DeepSeekWatchdog
    >>>
    >>> client = DeepSeekWatchdog(
    ...     api_key="sk-...",
    ...     model="deepseek-chat",
    ...     rules=["No harmful content"]
    ... )
    >>>
    >>> response = client.chat("Solve this math problem")
"""

from typing import Optional, List, Dict, Any, Generator
from .base import WatchdogClient, WatchdogConfig, WatchdogDenialError, APIError

# DeepSeek uses OpenAI-compatible API
try:
    from openai import OpenAI
    DEEPSEEK_AVAILABLE = True
except ImportError:
    OpenAI = None
    DEEPSEEK_AVAILABLE = False


class DeepSeekWatchdog(WatchdogClient):
    """
    Watchdog-protected DeepSeek API client.

    Args:
        api_key: DeepSeek API key
        model: Model to use (default: deepseek-chat)
        config: WatchdogConfig object
        rules: Shortcut to set rules

    Example:
        >>> client = DeepSeekWatchdog(
        ...     api_key="sk-...",
        ...     rules=["No harmful content"]
        ... )
        >>> response = client.chat("Hello!")
    """

    BASE_URL = "https://api.deepseek.com"

    DEFAULT_MODEL = "deepseek-chat"

    MODELS = {
        # Chat models
        "deepseek-chat": "deepseek-chat",
        "deepseek-v3": "deepseek-chat",

        # Reasoning model
        "deepseek-reasoner": "deepseek-reasoner",
        "deepseek-r1": "deepseek-reasoner",

        # Coder
        "deepseek-coder": "deepseek-coder",
    }

    def __init__(
        self,
        api_key: str,
        model: str = DEFAULT_MODEL,
        config: Optional[WatchdogConfig] = None,
        rules: Optional[List[str]] = None,
        timeout: float = 120.0,  # Reasoning can be slow
        **kwargs
    ):
        self.model = self.MODELS.get(model, model)
        self.timeout = timeout

        super().__init__(api_key, config, rules, **kwargs)

    def _init_client(self, **kwargs):
        """Initialize DeepSeek client."""
        if not DEEPSEEK_AVAILABLE:
            raise ImportError(
                "OpenAI package not installed (required for DeepSeek). "
                "Install with: pip install openai"
            )

        self._client = OpenAI(
            api_key=self.api_key,
            base_url=self.BASE_URL,
            timeout=self.timeout,
        )

    @property
    def provider_name(self) -> str:
        return "deepseek"

    def _make_request(self, prompt: str, **kwargs) -> Any:
        """Make DeepSeek API request."""
        system = kwargs.pop("system", "You are a helpful AI assistant.")
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

    def reason(self, prompt: str, **kwargs) -> str:
        """
        Use DeepSeek R1 for reasoning tasks.

        Args:
            prompt: Reasoning prompt
            **kwargs: Additional options

        Returns:
            Reasoning response
        """
        original_model = self.model
        self.model = "deepseek-reasoner"

        try:
            return self.chat(prompt, **kwargs)
        finally:
            self.model = original_model

    def code(self, prompt: str, **kwargs) -> str:
        """
        Use DeepSeek Coder for coding tasks.

        Args:
            prompt: Coding prompt
            **kwargs: Additional options

        Returns:
            Generated code
        """
        original_model = self.model
        self.model = "deepseek-coder"

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

        system = kwargs.pop("system", "You are a helpful AI assistant.")
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


def create_deepseek_client(
    api_key: str,
    rules: Optional[List[str]] = None,
    **kwargs
) -> DeepSeekWatchdog:
    """Create DeepSeek Watchdog client."""
    return DeepSeekWatchdog(api_key=api_key, rules=rules, **kwargs)
