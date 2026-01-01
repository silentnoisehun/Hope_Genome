# ============================================================
# HOPE GENOME - OLLAMA WATCHDOG INTEGRATION
# ============================================================
#
# Watchdog-protected Ollama client for LOCAL models!
# Supports: Llama, Qwen, Mistral, Gemma, Phi, and more!
#
# Usage:
#   from hope_genome.integrations import OllamaWatchdog
#
#   client = OllamaWatchdog(
#       model="llama3.2",
#       rules=["No harmful content"]
#   )
#
#   response = client.chat("What is 2+2?")
#
# Mate Robert + Claude
# VAS SZIGORA - 2026.01.01.
# ============================================================

"""
Ollama Watchdog Integration
===========================

Production-ready Ollama wrapper with Watchdog protection.
Run ANY model locally with ethical enforcement!

Supported Models:
- Llama 3.2, Llama 3.1, Llama 2
- Qwen 2.5, Qwen 2
- Mistral, Mixtral
- Gemma 2
- Phi-3, Phi-2
- DeepSeek
- And many more!

Example:
    >>> from hope_genome.integrations import OllamaWatchdog
    >>>
    >>> # Use any Ollama model
    >>> client = OllamaWatchdog(
    ...     model="llama3.2",
    ...     rules=["No harmful content"]
    ... )
    >>>
    >>> response = client.chat("Explain Python decorators")
    >>> print(response)
"""

from typing import Optional, List, Dict, Any, Generator
import json
from .base import WatchdogClient, WatchdogConfig, WatchdogDenialError, APIError

# Try to import requests for Ollama API
try:
    import requests
    REQUESTS_AVAILABLE = True
except ImportError:
    requests = None
    REQUESTS_AVAILABLE = False


class OllamaWatchdog(WatchdogClient):
    """
    Watchdog-protected Ollama client for local models.

    Supports all Ollama models: Llama, Qwen, Mistral, Gemma, Phi, etc.

    Args:
        model: Ollama model name (e.g., "llama3.2", "qwen2.5", "mistral")
        host: Ollama server URL (default: http://localhost:11434)
        config: WatchdogConfig object
        rules: Shortcut to set rules
        timeout: Request timeout in seconds

    Example:
        >>> client = OllamaWatchdog(
        ...     model="llama3.2",
        ...     rules=["No harmful content"]
        ... )
        >>> response = client.chat("Hello!")
    """

    DEFAULT_HOST = "http://localhost:11434"
    DEFAULT_MODEL = "llama3.2"

    # Popular models
    MODELS = {
        # Llama family
        "llama3.2": "llama3.2",
        "llama3.1": "llama3.1",
        "llama3": "llama3",
        "llama2": "llama2",

        # Qwen family (Alibaba)
        "qwen2.5": "qwen2.5",
        "qwen2": "qwen2",
        "qwen": "qwen",

        # Mistral family
        "mistral": "mistral",
        "mixtral": "mixtral",

        # Google Gemma
        "gemma2": "gemma2",
        "gemma": "gemma",

        # Microsoft Phi
        "phi3": "phi3",
        "phi": "phi",

        # DeepSeek
        "deepseek": "deepseek-r1",
        "deepseek-coder": "deepseek-coder",

        # Others
        "tinyllama": "tinyllama",
        "codellama": "codellama",
        "vicuna": "vicuna",
    }

    def __init__(
        self,
        model: str = DEFAULT_MODEL,
        host: str = DEFAULT_HOST,
        api_key: str = "",  # Ollama doesn't need API key
        config: Optional[WatchdogConfig] = None,
        rules: Optional[List[str]] = None,
        timeout: float = 120.0,
        **kwargs
    ):
        self.model = self.MODELS.get(model, model)
        self.host = host.rstrip("/")
        self.timeout = timeout

        super().__init__(api_key or "ollama-local", config, rules, **kwargs)

    def _init_client(self, **kwargs):
        """Initialize Ollama client."""
        if not REQUESTS_AVAILABLE:
            raise ImportError(
                "requests package not installed. "
                "Install with: pip install requests"
            )

        # Test connection
        try:
            resp = requests.get(f"{self.host}/api/tags", timeout=5)
            if resp.status_code != 200:
                raise APIError(f"Ollama not responding at {self.host}")
        except requests.exceptions.ConnectionError:
            raise APIError(
                f"Cannot connect to Ollama at {self.host}. "
                "Make sure Ollama is running: ollama serve"
            )

    @property
    def provider_name(self) -> str:
        return "ollama"

    def _make_request(self, prompt: str, **kwargs) -> Any:
        """Make Ollama API request."""
        system = kwargs.pop("system", "You are a helpful AI assistant.")
        temperature = kwargs.pop("temperature", 0.7)

        messages = [
            {"role": "system", "content": system},
            {"role": "user", "content": prompt}
        ]

        if "messages" in kwargs:
            messages = kwargs.pop("messages")

        payload = {
            "model": self.model,
            "messages": messages,
            "stream": False,
            "options": {
                "temperature": temperature,
            }
        }

        response = requests.post(
            f"{self.host}/api/chat",
            json=payload,
            timeout=self.timeout
        )

        if response.status_code != 200:
            raise APIError(f"Ollama error: {response.text}")

        return response.json()

    def _parse_response(self, response: Any) -> str:
        """Extract text from Ollama response."""
        if isinstance(response, dict):
            message = response.get("message", {})
            return message.get("content", "")
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

        system = kwargs.pop("system", "You are a helpful AI assistant.")
        temperature = kwargs.pop("temperature", 0.7)

        messages = [
            {"role": "system", "content": system},
            {"role": "user", "content": prompt}
        ]

        payload = {
            "model": self.model,
            "messages": messages,
            "stream": True,
            "options": {
                "temperature": temperature,
            }
        }

        response = requests.post(
            f"{self.host}/api/chat",
            json=payload,
            stream=True,
            timeout=self.timeout
        )

        full_response = []
        for line in response.iter_lines():
            if line:
                try:
                    data = json.loads(line)
                    if "message" in data and "content" in data["message"]:
                        text = data["message"]["content"]
                        full_response.append(text)
                        yield text
                except json.JSONDecodeError:
                    continue

        # Post-flight check
        full_text = "".join(full_response)
        allowed, reason = self.check_response(full_text)
        if not allowed:
            self._record_violation(f"stream:{prompt[:50]}", reason)
            raise WatchdogDenialError(f"Response BLOCKED: {reason}")

    def list_models(self) -> List[str]:
        """List available models on Ollama server."""
        response = requests.get(f"{self.host}/api/tags", timeout=10)
        if response.status_code == 200:
            data = response.json()
            return [m["name"] for m in data.get("models", [])]
        return []

    def pull_model(self, model_name: str) -> bool:
        """Pull a model from Ollama library."""
        response = requests.post(
            f"{self.host}/api/pull",
            json={"name": model_name},
            timeout=600  # Models can be large
        )
        return response.status_code == 200

    def generate(self, prompt: str, **kwargs) -> str:
        """
        Simple generation (no chat format).

        Args:
            prompt: Raw prompt
            **kwargs: Additional options

        Returns:
            Generated text
        """
        # Pre-flight check
        allowed, reason = self.check_request(prompt)
        if not allowed:
            self._record_violation(prompt[:100], reason)
            raise WatchdogDenialError(f"Request DENIED: {reason}")

        payload = {
            "model": self.model,
            "prompt": prompt,
            "stream": False,
        }

        response = requests.post(
            f"{self.host}/api/generate",
            json=payload,
            timeout=self.timeout
        )

        if response.status_code != 200:
            raise APIError(f"Ollama error: {response.text}")

        result = response.json().get("response", "")

        # Post-flight check
        allowed, reason = self.check_response(result)
        if not allowed:
            self._record_violation(f"generate:{prompt[:50]}", reason)
            raise WatchdogDenialError(f"Response BLOCKED: {reason}")

        return result


# ============================================================
# CONVENIENCE FUNCTIONS
# ============================================================

def create_ollama_client(
    model: str = "llama3.2",
    rules: Optional[List[str]] = None,
    **kwargs
) -> OllamaWatchdog:
    """
    Create Ollama Watchdog client.

    Args:
        model: Model name (llama3.2, qwen2.5, mistral, etc.)
        rules: Ethical rules to enforce
        **kwargs: Additional options

    Returns:
        OllamaWatchdog instance
    """
    return OllamaWatchdog(model=model, rules=rules, **kwargs)


# Aliases for popular models
def create_llama_client(rules: Optional[List[str]] = None, **kwargs) -> OllamaWatchdog:
    """Create Llama 3.2 client via Ollama."""
    return OllamaWatchdog(model="llama3.2", rules=rules, **kwargs)


def create_qwen_client(rules: Optional[List[str]] = None, **kwargs) -> OllamaWatchdog:
    """Create Qwen 2.5 client via Ollama."""
    return OllamaWatchdog(model="qwen2.5", rules=rules, **kwargs)


def create_mistral_local_client(rules: Optional[List[str]] = None, **kwargs) -> OllamaWatchdog:
    """Create Mistral client via Ollama."""
    return OllamaWatchdog(model="mistral", rules=rules, **kwargs)


def create_deepseek_client(rules: Optional[List[str]] = None, **kwargs) -> OllamaWatchdog:
    """Create DeepSeek client via Ollama."""
    return OllamaWatchdog(model="deepseek-r1", rules=rules, **kwargs)
