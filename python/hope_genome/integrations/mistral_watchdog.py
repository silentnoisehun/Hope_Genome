# ============================================================
# HOPE GENOME - MISTRAL AI WATCHDOG INTEGRATION
# ============================================================
#
# Watchdog-protected Mistral AI API client.
# Every request is verified, every response is checked!
#
# Usage:
#   from hope_genome.integrations import MistralWatchdog
#
#   client = MistralWatchdog(
#       api_key="...",
#       rules=["No harmful content"]
#   )
#
#   response = client.chat("What is 2+2?")
#
# Mate Robert + Claude
# VAS SZIGORA - 2026.01.01.
# ============================================================

"""
Mistral AI Watchdog Integration
===============================

Production-ready Mistral AI API wrapper with Watchdog protection.

Features:
- Mistral Large, Medium, Small
- Mixtral models
- Codestral for coding
- Automatic request filtering
- Response content analysis

Example:
    >>> from hope_genome.integrations import MistralWatchdog
    >>>
    >>> client = MistralWatchdog(
    ...     api_key="...",
    ...     model="mistral-large-latest",
    ...     rules=["No harmful content"]
    ... )
    >>>
    >>> response = client.chat("Explain neural networks")
"""

from typing import Optional, List, Dict, Any, Generator
from .base import WatchdogClient, WatchdogConfig, WatchdogDenialError, APIError

# Try to import Mistral SDK
try:
    from mistralai import Mistral
    MISTRAL_AVAILABLE = True
except ImportError:
    Mistral = None
    MISTRAL_AVAILABLE = False


class MistralWatchdog(WatchdogClient):
    """
    Watchdog-protected Mistral AI API client.

    Args:
        api_key: Mistral AI API key
        model: Model to use (default: mistral-large-latest)
        config: WatchdogConfig object
        rules: Shortcut to set rules

    Example:
        >>> client = MistralWatchdog(
        ...     api_key="...",
        ...     rules=["No harmful content"]
        ... )
        >>> response = client.chat("Hello!")
    """

    DEFAULT_MODEL = "mistral-large-latest"

    MODELS = {
        # Premier models
        "mistral-large": "mistral-large-latest",
        "mistral-large-latest": "mistral-large-latest",

        # Free tier
        "mistral-small": "mistral-small-latest",
        "mistral-small-latest": "mistral-small-latest",

        # Coding
        "codestral": "codestral-latest",
        "codestral-latest": "codestral-latest",

        # Open weights
        "open-mistral-7b": "open-mistral-7b",
        "open-mixtral-8x7b": "open-mixtral-8x7b",
        "open-mixtral-8x22b": "open-mixtral-8x22b",

        # Nemo
        "open-mistral-nemo": "open-mistral-nemo",
        "mistral-nemo": "open-mistral-nemo",
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
        """Initialize Mistral client."""
        if not MISTRAL_AVAILABLE:
            raise ImportError(
                "Mistral AI package not installed. "
                "Install with: pip install mistralai"
            )

        self._client = Mistral(api_key=self.api_key)

    @property
    def provider_name(self) -> str:
        return "mistral"

    def _make_request(self, prompt: str, **kwargs) -> Any:
        """Make Mistral API request."""
        system = kwargs.pop("system", "You are a helpful AI assistant.")
        temperature = kwargs.pop("temperature", 0.7)
        max_tokens = kwargs.pop("max_tokens", 1024)

        messages = [
            {"role": "system", "content": system},
            {"role": "user", "content": prompt}
        ]

        if "messages" in kwargs:
            messages = kwargs.pop("messages")

        response = self._client.chat.complete(
            model=self.model,
            messages=messages,
            temperature=temperature,
            max_tokens=max_tokens,
        )

        return response

    def _parse_response(self, response: Any) -> str:
        """Extract text from Mistral response."""
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
        """Stream chat response."""
        # Pre-flight check
        allowed, reason = self.check_request(prompt)
        if not allowed:
            self._record_violation(prompt[:100], reason)
            raise WatchdogDenialError(f"Request DENIED: {reason}")

        system = kwargs.pop("system", "You are a helpful AI assistant.")
        temperature = kwargs.pop("temperature", 0.7)
        max_tokens = kwargs.pop("max_tokens", 1024)

        messages = [
            {"role": "system", "content": system},
            {"role": "user", "content": prompt}
        ]

        response = self._client.chat.stream(
            model=self.model,
            messages=messages,
            temperature=temperature,
            max_tokens=max_tokens,
        )

        full_response = []
        for chunk in response:
            if chunk.data.choices and chunk.data.choices[0].delta.content:
                text = chunk.data.choices[0].delta.content
                full_response.append(text)
                yield text

        # Post-flight check
        full_text = "".join(full_response)
        allowed, reason = self.check_response(full_text)
        if not allowed:
            self._record_violation(f"stream:{prompt[:50]}", reason)
            raise WatchdogDenialError(f"Response BLOCKED: {reason}")

    def code(self, prompt: str, **kwargs) -> str:
        """
        Generate code using Codestral.

        Args:
            prompt: Coding prompt
            **kwargs: Additional options

        Returns:
            Generated code
        """
        # Use codestral for coding
        original_model = self.model
        self.model = "codestral-latest"

        try:
            return self.chat(prompt, **kwargs)
        finally:
            self.model = original_model

    def embeddings(self, texts: List[str], model: str = "mistral-embed") -> List[List[float]]:
        """
        Get text embeddings.

        Args:
            texts: List of texts to embed
            model: Embedding model

        Returns:
            List of embedding vectors
        """
        # Check all texts
        for text in texts:
            allowed, reason = self.check_request(text)
            if not allowed:
                raise WatchdogDenialError(f"Embedding DENIED: {reason}")

        response = self._client.embeddings.create(
            model=model,
            inputs=texts
        )

        return [item.embedding for item in response.data]


# ============================================================
# CONVENIENCE FUNCTIONS
# ============================================================

def create_mistral_client(
    api_key: str,
    rules: Optional[List[str]] = None,
    **kwargs
) -> MistralWatchdog:
    """Create Mistral AI Watchdog client."""
    return MistralWatchdog(api_key=api_key, rules=rules, **kwargs)
