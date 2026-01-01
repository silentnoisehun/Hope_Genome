# ============================================================
# HOPE GENOME - GOOGLE GEMINI WATCHDOG INTEGRATION
# ============================================================
#
# Watchdog-protected Google Gemini API client.
# Every request is verified, every response is checked!
#
# Usage:
#   from hope_genome.integrations import GeminiWatchdog
#
#   client = GeminiWatchdog(
#       api_key="AIza...",
#       rules=["No harmful content"]
#   )
#
#   response = client.chat("What is 2+2?")
#
# Mate Robert + Claude
# VAS SZIGORA - 2026.01.01.
# ============================================================

"""
Google Gemini Watchdog Integration
==================================

Production-ready Google Gemini API wrapper with Watchdog protection.

Features:
- Gemini 2.0, 1.5 Pro, 1.5 Flash support
- Automatic request filtering
- Response content analysis
- Streaming support
- Vision/multimodal support
- Function calling support

Example:
    >>> from hope_genome.integrations import GeminiWatchdog
    >>>
    >>> client = GeminiWatchdog(
    ...     api_key="AIza...",
    ...     model="gemini-2.0-flash-exp",
    ...     rules=["No harmful content", "Respect privacy"]
    ... )
    >>>
    >>> # Simple chat
    >>> response = client.chat("Explain quantum computing")
    >>> print(response)
"""

from typing import Optional, List, Dict, Any, Generator
from .base import WatchdogClient, WatchdogConfig, WatchdogDenialError, APIError

# Try to import Google Generative AI
try:
    import google.generativeai as genai
    GEMINI_AVAILABLE = True
except ImportError:
    genai = None
    GEMINI_AVAILABLE = False


class GeminiWatchdog(WatchdogClient):
    """
    Watchdog-protected Google Gemini API client.

    Args:
        api_key: Google AI API key
        model: Model to use (default: gemini-2.0-flash-exp)
        config: WatchdogConfig object
        rules: Shortcut to set rules
        safety_settings: Gemini safety settings (optional)

    Example:
        >>> client = GeminiWatchdog(
        ...     api_key="AIza...",
        ...     model="gemini-2.0-flash-exp",
        ...     rules=["No harmful content"]
        ... )
        >>> response = client.chat("Hello!")
    """

    DEFAULT_MODEL = "gemini-2.0-flash-exp"

    # Available models
    MODELS = {
        "gemini-2.0": "gemini-2.0-flash-exp",
        "gemini-1.5-pro": "gemini-1.5-pro",
        "gemini-1.5-flash": "gemini-1.5-flash",
        "gemini-pro": "gemini-pro",
    }

    def __init__(
        self,
        api_key: str,
        model: str = DEFAULT_MODEL,
        config: Optional[WatchdogConfig] = None,
        rules: Optional[List[str]] = None,
        safety_settings: Optional[List[Dict]] = None,
        **kwargs
    ):
        # Resolve model alias
        self.model = self.MODELS.get(model, model)
        self.safety_settings = safety_settings

        super().__init__(api_key, config, rules, **kwargs)

    def _init_client(self, **kwargs):
        """Initialize Gemini client."""
        if not GEMINI_AVAILABLE:
            raise ImportError(
                "Google Generative AI package not installed. "
                "Install with: pip install google-generativeai"
            )

        genai.configure(api_key=self.api_key)

        generation_config = genai.GenerationConfig(
            temperature=0.7,
            max_output_tokens=1024,
        )

        self._model = genai.GenerativeModel(
            model_name=self.model,
            generation_config=generation_config,
            safety_settings=self.safety_settings
        )

        # Chat session for history
        self._chat_session = None

    @property
    def provider_name(self) -> str:
        return "gemini"

    def _make_request(self, prompt: str, **kwargs) -> Any:
        """Make Gemini API request."""
        # Check for system instruction
        system = kwargs.pop("system", None)

        if system:
            # Create new model with system instruction
            model = genai.GenerativeModel(
                model_name=self.model,
                system_instruction=system,
                generation_config=genai.GenerationConfig(
                    temperature=kwargs.get("temperature", 0.7),
                    max_output_tokens=kwargs.get("max_tokens", 1024),
                )
            )
            response = model.generate_content(prompt)
        else:
            response = self._model.generate_content(prompt)

        return response

    def _parse_response(self, response: Any) -> str:
        """Extract text from Gemini response."""
        try:
            if hasattr(response, "text"):
                return response.text
            elif hasattr(response, "parts"):
                return "".join(part.text for part in response.parts if hasattr(part, "text"))
            return ""
        except Exception:
            # Handle blocked responses
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
            messages: List of {"role": "user/model", "content": "..."}
            **kwargs: Additional options

        Example:
            >>> history = [
            ...     {"role": "user", "content": "My name is Alice"},
            ...     {"role": "model", "content": "Hello Alice!"},
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

        # Convert to Gemini format
        history = []
        for msg in messages[:-1]:  # All but last
            role = "user" if msg["role"] == "user" else "model"
            history.append({"role": role, "parts": [msg["content"]]})

        # Start chat with history
        chat = self._model.start_chat(history=history)

        # Send last message
        last_msg = messages[-1]["content"]
        response = chat.send_message(last_msg)

        response_text = self._parse_response(response)

        # Post-flight check
        allowed, reason = self.check_response(response_text)
        if not allowed:
            self._record_violation(f"history:{last_msg[:50]}", reason)
            raise WatchdogDenialError(f"Response BLOCKED: {reason}")

        return response_text

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

        response = self._model.generate_content(prompt, stream=True)

        full_response = []
        for chunk in response:
            if hasattr(chunk, "text"):
                full_response.append(chunk.text)
                yield chunk.text

        # Post-flight check
        full_text = "".join(full_response)
        allowed, reason = self.check_response(full_text)
        if not allowed:
            self._record_violation(f"stream:{prompt[:50]}", reason)
            raise WatchdogDenialError(f"Response BLOCKED: {reason}")

    def vision(
        self,
        prompt: str,
        image_path: str,
        **kwargs
    ) -> str:
        """
        Analyze image with Gemini Vision.

        Args:
            prompt: Question about the image
            image_path: Path to image file
            **kwargs: Additional options

        Returns:
            Vision analysis response
        """
        import PIL.Image

        # Pre-flight check
        allowed, reason = self.check_request(prompt)
        if not allowed:
            self._record_violation(prompt[:100], reason)
            raise WatchdogDenialError(f"Request DENIED: {reason}")

        # Load image
        image = PIL.Image.open(image_path)

        # Generate with image
        response = self._model.generate_content([prompt, image])

        response_text = self._parse_response(response)

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

        Example:
            >>> functions = [{
            ...     "name": "get_weather",
            ...     "description": "Get weather for a location",
            ...     "parameters": {
            ...         "type": "object",
            ...         "properties": {
            ...             "location": {"type": "string"}
            ...         }
            ...     }
            ... }]
            >>> result = client.function_call("Weather in Paris?", functions)
        """
        # Pre-flight check
        allowed, reason = self.check_request(prompt)
        if not allowed:
            self._record_violation(prompt[:100], reason)
            raise WatchdogDenialError(f"Request DENIED: {reason}")

        # Check function definitions
        func_str = str(functions).lower()
        for pattern in self._blocked_patterns:
            if pattern.search(func_str):
                self._record_violation("function_def", f"Blocked function: {pattern.pattern}")
                raise WatchdogDenialError("Function definition BLOCKED: suspicious pattern")

        # Convert to Gemini tools format
        tools = []
        for func in functions:
            tool = genai.protos.Tool(
                function_declarations=[
                    genai.protos.FunctionDeclaration(
                        name=func["name"],
                        description=func.get("description", ""),
                        parameters=genai.protos.Schema(
                            type=genai.protos.Type.OBJECT,
                            properties={
                                k: genai.protos.Schema(type=genai.protos.Type.STRING)
                                for k in func.get("parameters", {}).get("properties", {}).keys()
                            }
                        )
                    )
                ]
            )
            tools.append(tool)

        # Create model with tools
        model = genai.GenerativeModel(
            model_name=self.model,
            tools=tools
        )

        response = model.generate_content(prompt)

        # Check for function call
        if hasattr(response, "candidates") and response.candidates:
            candidate = response.candidates[0]
            if hasattr(candidate, "content") and candidate.content.parts:
                for part in candidate.content.parts:
                    if hasattr(part, "function_call"):
                        fc = part.function_call
                        return {
                            "function": fc.name,
                            "arguments": dict(fc.args)
                        }

        # No function call, return text
        return {
            "function": None,
            "content": self._parse_response(response)
        }

    def start_chat(self):
        """
        Start a chat session for multi-turn conversation.

        Returns:
            Self for chaining

        Example:
            >>> client.start_chat()
            >>> response1 = client.send_message("Hi, I'm Alice")
            >>> response2 = client.send_message("What's my name?")
        """
        self._chat_session = self._model.start_chat(history=[])
        return self

    def send_message(self, message: str) -> str:
        """
        Send message in active chat session.

        Args:
            message: User message

        Returns:
            Response text
        """
        if not self._chat_session:
            self.start_chat()

        # Pre-flight check
        allowed, reason = self.check_request(message)
        if not allowed:
            self._record_violation(message[:100], reason)
            raise WatchdogDenialError(f"Request DENIED: {reason}")

        response = self._chat_session.send_message(message)
        response_text = self._parse_response(response)

        # Post-flight check
        allowed, reason = self.check_response(response_text)
        if not allowed:
            self._record_violation(f"chat:{message[:50]}", reason)
            raise WatchdogDenialError(f"Response BLOCKED: {reason}")

        return response_text

    def count_tokens(self, text: str) -> int:
        """
        Count tokens in text.

        Args:
            text: Text to count

        Returns:
            Token count
        """
        return self._model.count_tokens(text).total_tokens


# ============================================================
# CONVENIENCE FUNCTIONS
# ============================================================

def create_gemini_client(
    api_key: str,
    rules: Optional[List[str]] = None,
    **kwargs
) -> GeminiWatchdog:
    """
    Create Gemini Watchdog client.

    Args:
        api_key: Google AI API key
        rules: Ethical rules to enforce
        **kwargs: Additional options

    Returns:
        GeminiWatchdog instance
    """
    return GeminiWatchdog(api_key=api_key, rules=rules, **kwargs)
