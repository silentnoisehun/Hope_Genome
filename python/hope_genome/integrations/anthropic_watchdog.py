# ============================================================
# HOPE GENOME - ANTHROPIC WATCHDOG INTEGRATION
# ============================================================
#
# Watchdog-protected Anthropic Claude API client.
# Every request is verified, every response is checked!
#
# Usage:
#   from hope_genome.integrations import AnthropicWatchdog
#
#   client = AnthropicWatchdog(
#       api_key="sk-ant-...",
#       rules=["No harmful content"]
#   )
#
#   response = client.chat("What is 2+2?")
#
# Mate Robert + Claude
# VAS SZIGORA - 2026.01.01.
# ============================================================

"""
Anthropic Watchdog Integration
==============================

Production-ready Anthropic Claude API wrapper with Watchdog protection.

Features:
- Claude 3.5 Sonnet, Claude 3 Opus, Haiku support
- Automatic request filtering
- Response content analysis
- Streaming support
- Vision support (Claude 3)
- Tool use support

Example:
    >>> from hope_genome.integrations import AnthropicWatchdog
    >>>
    >>> client = AnthropicWatchdog(
    ...     api_key="sk-ant-...",
    ...     model="claude-3-5-sonnet-20241022",
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
"""

from typing import Optional, List, Dict, Any, Generator
from .base import WatchdogClient, WatchdogConfig, WatchdogDenialError, APIError

# Try to import Anthropic
try:
    import anthropic
    ANTHROPIC_AVAILABLE = True
except ImportError:
    anthropic = None
    ANTHROPIC_AVAILABLE = False


class AnthropicWatchdog(WatchdogClient):
    """
    Watchdog-protected Anthropic Claude API client.

    Args:
        api_key: Anthropic API key
        model: Model to use (default: claude-3-5-sonnet-20241022)
        base_url: Custom API base URL (optional)
        config: WatchdogConfig object
        rules: Shortcut to set rules
        timeout: Request timeout in seconds
        max_retries: Max retry attempts

    Example:
        >>> client = AnthropicWatchdog(
        ...     api_key="sk-ant-...",
        ...     model="claude-3-5-sonnet-20241022",
        ...     rules=["No harmful content"]
        ... )
        >>> response = client.chat("Hello!")
    """

    DEFAULT_MODEL = "claude-3-5-sonnet-20241022"

    # Available models
    MODELS = {
        "claude-3-5-sonnet": "claude-3-5-sonnet-20241022",
        "claude-3-opus": "claude-3-opus-20240229",
        "claude-3-sonnet": "claude-3-sonnet-20240229",
        "claude-3-haiku": "claude-3-haiku-20240307",
        "claude-opus-4": "claude-opus-4-20250514",
        "claude-sonnet-4": "claude-sonnet-4-20250514",
    }

    def __init__(
        self,
        api_key: str,
        model: str = DEFAULT_MODEL,
        base_url: Optional[str] = None,
        config: Optional[WatchdogConfig] = None,
        rules: Optional[List[str]] = None,
        timeout: float = 60.0,
        max_retries: int = 2,
        **kwargs
    ):
        # Resolve model alias
        self.model = self.MODELS.get(model, model)
        self.base_url = base_url
        self.timeout = timeout
        self.max_retries = max_retries

        super().__init__(api_key, config, rules, **kwargs)

    def _init_client(self, **kwargs):
        """Initialize Anthropic client."""
        if not ANTHROPIC_AVAILABLE:
            raise ImportError(
                "Anthropic package not installed. "
                "Install with: pip install anthropic"
            )

        self._client = anthropic.Anthropic(
            api_key=self.api_key,
            base_url=self.base_url,
            timeout=self.timeout,
            max_retries=self.max_retries,
        )

    @property
    def provider_name(self) -> str:
        return "anthropic"

    def _make_request(self, prompt: str, **kwargs) -> Any:
        """Make Anthropic API request."""
        system = kwargs.pop("system", "You are a helpful assistant.")
        temperature = kwargs.pop("temperature", 0.7)
        max_tokens = kwargs.pop("max_tokens", 1024)

        # Handle messages format
        if "messages" in kwargs:
            messages = kwargs.pop("messages")
        else:
            messages = [{"role": "user", "content": prompt}]

        response = self._client.messages.create(
            model=self.model,
            system=system,
            messages=messages,
            temperature=temperature,
            max_tokens=max_tokens,
            **kwargs
        )

        return response

    def _parse_response(self, response: Any) -> str:
        """Extract text from Anthropic response."""
        if hasattr(response, "content") and response.content:
            # Claude returns a list of content blocks
            text_blocks = [
                block.text for block in response.content
                if hasattr(block, "text")
            ]
            return "".join(text_blocks)
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
        """
        # Check all user messages
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

        messages = [{"role": "user", "content": prompt}]

        full_response = []

        with self._client.messages.stream(
            model=self.model,
            system=system,
            messages=messages,
            temperature=temperature,
            max_tokens=max_tokens,
            **kwargs
        ) as stream:
            for text in stream.text_stream:
                full_response.append(text)
                yield text

        # Post-flight check
        full_text = "".join(full_response)
        allowed, reason = self.check_response(full_text)
        if not allowed:
            self._record_violation(f"stream:{prompt[:50]}", reason)
            raise WatchdogDenialError(f"Response BLOCKED: {reason}")

    def vision(
        self,
        prompt: str,
        image_data: str,
        media_type: str = "image/png",
        **kwargs
    ) -> str:
        """
        Analyze image with Claude Vision.

        Args:
            prompt: Question about the image
            image_data: Base64-encoded image data
            media_type: Image MIME type (image/png, image/jpeg, etc.)
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
                    {
                        "type": "image",
                        "source": {
                            "type": "base64",
                            "media_type": media_type,
                            "data": image_data
                        }
                    },
                    {
                        "type": "text",
                        "text": prompt
                    }
                ]
            }
        ]

        response = self._client.messages.create(
            model=self.model,
            messages=messages,
            max_tokens=kwargs.get("max_tokens", 1024),
            **kwargs
        )

        response_text = self._parse_response(response)

        # Post-flight check
        allowed, reason = self.check_response(response_text)
        if not allowed:
            self._record_violation(f"vision:{prompt[:50]}", reason)
            raise WatchdogDenialError(f"Response BLOCKED: {reason}")

        return response_text

    def tool_use(
        self,
        prompt: str,
        tools: List[Dict],
        **kwargs
    ) -> Dict[str, Any]:
        """
        Make tool use request.

        Args:
            prompt: User message
            tools: List of tool definitions
            **kwargs: Additional options

        Returns:
            Tool use result

        Example:
            >>> tools = [{
            ...     "name": "get_weather",
            ...     "description": "Get weather for a location",
            ...     "input_schema": {
            ...         "type": "object",
            ...         "properties": {
            ...             "location": {"type": "string"}
            ...         }
            ...     }
            ... }]
            >>> result = client.tool_use("What's the weather in Paris?", tools)
        """
        # Pre-flight check
        allowed, reason = self.check_request(prompt)
        if not allowed:
            self._record_violation(prompt[:100], reason)
            raise WatchdogDenialError(f"Request DENIED: {reason}")

        # Check tool definitions
        tools_str = str(tools).lower()
        for pattern in self._blocked_patterns:
            if pattern.search(tools_str):
                self._record_violation("tool_def", f"Blocked tool: {pattern.pattern}")
                raise WatchdogDenialError("Tool definition BLOCKED: suspicious pattern")

        messages = [{"role": "user", "content": prompt}]

        response = self._client.messages.create(
            model=self.model,
            messages=messages,
            tools=tools,
            max_tokens=kwargs.get("max_tokens", 1024),
            **kwargs
        )

        # Check for tool use
        for block in response.content:
            if block.type == "tool_use":
                return {
                    "tool": block.name,
                    "tool_id": block.id,
                    "input": block.input
                }

        # No tool use, return text
        return {
            "tool": None,
            "content": self._parse_response(response)
        }

    def count_tokens(self, text: str) -> int:
        """
        Count tokens in text.

        Args:
            text: Text to count

        Returns:
            Token count
        """
        return self._client.count_tokens(text)


# ============================================================
# CONVENIENCE FUNCTIONS
# ============================================================

def create_anthropic_client(
    api_key: str,
    rules: Optional[List[str]] = None,
    **kwargs
) -> AnthropicWatchdog:
    """
    Create Anthropic Watchdog client.

    Args:
        api_key: Anthropic API key
        rules: Ethical rules to enforce
        **kwargs: Additional options

    Returns:
        AnthropicWatchdog instance
    """
    return AnthropicWatchdog(api_key=api_key, rules=rules, **kwargs)
