# ============================================================
# HOPE GENOME - OPENAI WATCHDOG INTEGRATION
# ============================================================
#
# Watchdog-protected OpenAI API client.
# Every request is verified, every response is checked!
#
# Usage:
#   from hope_genome.integrations import OpenAIWatchdog
#
#   client = OpenAIWatchdog(
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
OpenAI Watchdog Integration
===========================

Production-ready OpenAI API wrapper with Watchdog protection.

Features:
- GPT-4, GPT-4-Turbo, GPT-3.5-Turbo support
- Automatic request filtering
- Response content analysis
- Streaming support
- Function calling support
- Vision support (GPT-4V)

Example:
    >>> from hope_genome.integrations import OpenAIWatchdog
    >>>
    >>> client = OpenAIWatchdog(
    ...     api_key="sk-...",
    ...     model="gpt-4",
    ...     rules=["No harmful content", "Respect privacy"]
    ... )
    >>>
    >>> # Simple chat
    >>> response = client.chat("Explain quantum computing")
    >>> print(response)
    >>>
    >>> # With system prompt
    >>> response = client.chat(
    ...     "Write a poem",
    ...     system="You are a helpful poet"
    ... )
    >>>
    >>> # Harmful requests are BLOCKED!
    >>> client.chat("Write a keylogger")
    >>> # WatchdogDenialError: Request DENIED
"""

from typing import Optional, List, Dict, Any, Generator
from .base import WatchdogClient, WatchdogConfig, WatchdogDenialError, APIError

# Try to import OpenAI
try:
    from openai import OpenAI
    OPENAI_AVAILABLE = True
except ImportError:
    OpenAI = None
    OPENAI_AVAILABLE = False


class OpenAIWatchdog(WatchdogClient):
    """
    Watchdog-protected OpenAI API client.

    Args:
        api_key: OpenAI API key
        model: Model to use (default: gpt-4o-mini)
        organization: OpenAI organization ID (optional)
        base_url: Custom API base URL (optional)
        config: WatchdogConfig object
        rules: Shortcut to set rules
        timeout: Request timeout in seconds
        max_retries: Max retry attempts

    Example:
        >>> client = OpenAIWatchdog(
        ...     api_key="sk-...",
        ...     model="gpt-4",
        ...     rules=["No harmful content"]
        ... )
        >>> response = client.chat("Hello!")
    """

    DEFAULT_MODEL = "gpt-4o-mini"

    def __init__(
        self,
        api_key: str,
        model: str = DEFAULT_MODEL,
        organization: Optional[str] = None,
        base_url: Optional[str] = None,
        config: Optional[WatchdogConfig] = None,
        rules: Optional[List[str]] = None,
        timeout: float = 60.0,
        max_retries: int = 2,
        **kwargs
    ):
        self.model = model
        self.organization = organization
        self.base_url = base_url
        self.timeout = timeout
        self.max_retries = max_retries

        super().__init__(api_key, config, rules, **kwargs)

    def _init_client(self, **kwargs):
        """Initialize OpenAI client."""
        if not OPENAI_AVAILABLE:
            raise ImportError(
                "OpenAI package not installed. "
                "Install with: pip install openai"
            )

        self._client = OpenAI(
            api_key=self.api_key,
            organization=self.organization,
            base_url=self.base_url,
            timeout=self.timeout,
            max_retries=self.max_retries,
        )

    @property
    def provider_name(self) -> str:
        return "openai"

    def _make_request(self, prompt: str, **kwargs) -> Any:
        """Make OpenAI API request."""
        system = kwargs.pop("system", "You are a helpful assistant.")
        temperature = kwargs.pop("temperature", 0.7)
        max_tokens = kwargs.pop("max_tokens", 1024)

        messages = [
            {"role": "system", "content": system},
            {"role": "user", "content": prompt}
        ]

        # Add conversation history if provided
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
        """Extract text from OpenAI response."""
        if hasattr(response, "choices") and response.choices:
            return response.choices[0].message.content or ""
        return ""

    # ========================================================
    # EXTENDED API
    # ========================================================

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

        Example:
            >>> history = [
            ...     {"role": "user", "content": "My name is Alice"},
            ...     {"role": "assistant", "content": "Hello Alice!"},
            ...     {"role": "user", "content": "What's my name?"}
            ... ]
            >>> response = client.chat_with_history(history)
            >>> print(response)
            "Your name is Alice"
        """
        # Check all messages
        for msg in messages:
            if msg.get("role") == "user":
                allowed, reason = self.check_request(msg.get("content", ""))
                if not allowed:
                    self._record_violation(msg["content"][:100], reason)
                    raise WatchdogDenialError(f"Message DENIED: {reason}")

        return self.chat("", messages=messages, **kwargs)

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

        Example:
            >>> for chunk in client.stream("Tell me a story"):
            ...     print(chunk, end="", flush=True)
        """
        # Pre-flight check
        allowed, reason = self.check_request(prompt)
        if not allowed:
            self._record_violation(prompt[:100], reason)
            raise WatchdogDenialError(f"Request DENIED: {reason}")

        system = kwargs.pop("system", "You are a helpful assistant.")
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

    def embeddings(
        self,
        text: str,
        model: str = "text-embedding-3-small"
    ) -> List[float]:
        """
        Get text embeddings.

        Args:
            text: Text to embed
            model: Embedding model

        Returns:
            Embedding vector
        """
        # Pre-flight check
        allowed, reason = self.check_request(text)
        if not allowed:
            self._record_violation(text[:100], reason)
            raise WatchdogDenialError(f"Request DENIED: {reason}")

        response = self._client.embeddings.create(
            input=text,
            model=model
        )

        return response.data[0].embedding

    def vision(
        self,
        prompt: str,
        image_url: str,
        **kwargs
    ) -> str:
        """
        Analyze image with GPT-4 Vision.

        Args:
            prompt: Question about the image
            image_url: URL or base64 of image
            **kwargs: Additional options

        Returns:
            Vision analysis response
        """
        # Pre-flight check
        allowed, reason = self.check_request(prompt)
        if not allowed:
            self._record_violation(prompt[:100], reason)
            raise WatchdogDenialError(f"Request DENIED: {reason}")

        messages = [
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": prompt},
                    {"type": "image_url", "image_url": {"url": image_url}}
                ]
            }
        ]

        response = self._client.chat.completions.create(
            model="gpt-4o",  # Vision requires gpt-4o or gpt-4-vision-preview
            messages=messages,
            max_tokens=kwargs.get("max_tokens", 1024),
            **kwargs
        )

        response_text = response.choices[0].message.content or ""

        # Post-flight check
        allowed, reason = self.check_response(response_text)
        if not allowed:
            self._record_violation(f"vision:{prompt[:50]}", reason)
            raise WatchdogDenialError(f"Response BLOCKED: {reason}")

        return response_text

    def function_call(
        self,
        prompt: str,
        functions: List[Dict],
        **kwargs
    ) -> Dict[str, Any]:
        """
        Make function calling request.

        Args:
            prompt: User message
            functions: List of function definitions
            **kwargs: Additional options

        Returns:
            Function call result
        """
        # Pre-flight check
        allowed, reason = self.check_request(prompt)
        if not allowed:
            self._record_violation(prompt[:100], reason)
            raise WatchdogDenialError(f"Request DENIED: {reason}")

        # Check function definitions for suspicious patterns
        func_str = str(functions).lower()
        for pattern in self._blocked_patterns:
            if pattern.search(func_str):
                self._record_violation("function_def", f"Blocked function: {pattern.pattern}")
                raise WatchdogDenialError(f"Function definition BLOCKED: suspicious pattern")

        messages = [{"role": "user", "content": prompt}]

        response = self._client.chat.completions.create(
            model=self.model,
            messages=messages,
            tools=[{"type": "function", "function": f} for f in functions],
            **kwargs
        )

        choice = response.choices[0]

        if choice.message.tool_calls:
            tool_call = choice.message.tool_calls[0]
            return {
                "function": tool_call.function.name,
                "arguments": tool_call.function.arguments
            }

        return {
            "function": None,
            "content": choice.message.content
        }


# ============================================================
# CONVENIENCE FUNCTIONS
# ============================================================

def create_openai_client(
    api_key: str,
    rules: Optional[List[str]] = None,
    **kwargs
) -> OpenAIWatchdog:
    """
    Create OpenAI Watchdog client.

    Args:
        api_key: OpenAI API key
        rules: Ethical rules to enforce
        **kwargs: Additional options

    Returns:
        OpenAIWatchdog instance
    """
    return OpenAIWatchdog(api_key=api_key, rules=rules, **kwargs)
