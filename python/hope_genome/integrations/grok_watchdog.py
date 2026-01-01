# ============================================================
# HOPE GENOME - GROK (xAI) WATCHDOG INTEGRATION
# ============================================================
#
# Watchdog-protected xAI Grok API client.
# Every request is verified, every response is checked!
#
# Usage:
#   from hope_genome.integrations import GrokWatchdog
#
#   client = GrokWatchdog(
#       api_key="xai-...",
#       rules=["No harmful content"]
#   )
#
#   response = client.chat("What is 2+2?")
#
# Mate Robert + Claude
# VAS SZIGORA - 2026.01.01.
# ============================================================

"""
Grok (xAI) Watchdog Integration
===============================

Production-ready xAI Grok API wrapper with Watchdog protection.

Features:
- Grok-2, Grok-2-mini support
- Automatic request filtering
- Response content analysis
- Streaming support
- OpenAI-compatible API

Example:
    >>> from hope_genome.integrations import GrokWatchdog
    >>>
    >>> client = GrokWatchdog(
    ...     api_key="xai-...",
    ...     model="grok-2",
    ...     rules=["No harmful content", "Respect privacy"]
    ... )
    >>>
    >>> # Simple chat
    >>> response = client.chat("Explain quantum computing")
    >>> print(response)
"""

from typing import Optional, List, Dict, Any, Generator
from .base import WatchdogClient, WatchdogConfig, WatchdogDenialError, APIError

# Grok uses OpenAI-compatible API
try:
    from openai import OpenAI
    GROK_AVAILABLE = True
except ImportError:
    OpenAI = None
    GROK_AVAILABLE = False


class GrokWatchdog(WatchdogClient):
    """
    Watchdog-protected xAI Grok API client.

    Uses OpenAI-compatible API with xAI base URL.

    Args:
        api_key: xAI API key (starts with xai-)
        model: Model to use (default: grok-2)
        config: WatchdogConfig object
        rules: Shortcut to set rules
        timeout: Request timeout in seconds

    Example:
        >>> client = GrokWatchdog(
        ...     api_key="xai-...",
        ...     model="grok-2",
        ...     rules=["No harmful content"]
        ... )
        >>> response = client.chat("Hello!")
    """

    # xAI API base URL
    BASE_URL = "https://api.x.ai/v1"

    DEFAULT_MODEL = "grok-2"

    # Available models
    MODELS = {
        "grok-2": "grok-2",
        "grok-2-mini": "grok-2-mini",
        "grok-beta": "grok-beta",
    }

    def __init__(
        self,
        api_key: str,
        model: str = DEFAULT_MODEL,
        config: Optional[WatchdogConfig] = None,
        rules: Optional[List[str]] = None,
        timeout: float = 60.0,
        max_retries: int = 2,
        **kwargs
    ):
        # Resolve model alias
        self.model = self.MODELS.get(model, model)
        self.timeout = timeout
        self.max_retries = max_retries

        super().__init__(api_key, config, rules, **kwargs)

    def _init_client(self, **kwargs):
        """Initialize Grok client (OpenAI-compatible)."""
        if not GROK_AVAILABLE:
            raise ImportError(
                "OpenAI package not installed (required for Grok). "
                "Install with: pip install openai"
            )

        self._client = OpenAI(
            api_key=self.api_key,
            base_url=self.BASE_URL,
            timeout=self.timeout,
            max_retries=self.max_retries,
        )

    @property
    def provider_name(self) -> str:
        return "grok"

    def _make_request(self, prompt: str, **kwargs) -> Any:
        """Make Grok API request."""
        system = kwargs.pop("system", "You are Grok, a helpful AI assistant.")
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
        """Extract text from Grok response."""
        if hasattr(response, "choices") and response.choices:
            return response.choices[0].message.content or ""
        return ""

    # ========================================================
    # EXTENDED API
    # ========================================================

    def stream(
        self,
        prompt: str,
        **kwargs
    ) -> Generator[str, None, None]:
        """
        Stream chat response.

        Args:
            prompt: User message
            **kwargs: Additional options

        Yields:
            Response chunks
        """
        # Pre-flight check
        allowed, reason = self.check_request(prompt)
        if not allowed:
            self._record_violation(prompt[:100], reason)
            raise WatchdogDenialError(f"Request DENIED: {reason}")

        system = kwargs.pop("system", "You are Grok, a helpful AI assistant.")
        temperature = kwargs.pop("temperature", 0.7)
        max_tokens = kwargs.pop("max_tokens", 1024)

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

        # Post-flight check
        full_text = "".join(full_response)
        allowed, reason = self.check_response(full_text)
        if not allowed:
            self._record_violation(f"stream:{prompt[:50]}", reason)
            raise WatchdogDenialError(f"Response BLOCKED: {reason}")

    def chat_with_history(
        self,
        messages: List[Dict[str, str]],
        **kwargs
    ) -> str:
        """
        Chat with conversation history.

        Args:
            messages: List of {"role": "user/assistant", "content": "..."}
            **kwargs: Additional options
        """
        for msg in messages:
            if msg.get("role") == "user":
                allowed, reason = self.check_request(msg.get("content", ""))
                if not allowed:
                    self._record_violation(msg["content"][:100], reason)
                    raise WatchdogDenialError(f"Message DENIED: {reason}")

        return self.chat("", messages=messages, **kwargs)


# ============================================================
# CONVENIENCE FUNCTIONS
# ============================================================

def create_grok_client(
    api_key: str,
    rules: Optional[List[str]] = None,
    **kwargs
) -> GrokWatchdog:
    """
    Create Grok Watchdog client.

    Args:
        api_key: xAI API key
        rules: Ethical rules to enforce
        **kwargs: Additional options

    Returns:
        GrokWatchdog instance
    """
    return GrokWatchdog(api_key=api_key, rules=rules, **kwargs)
