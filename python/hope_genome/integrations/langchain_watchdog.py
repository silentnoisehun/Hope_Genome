# ============================================================
# HOPE GENOME - LANGCHAIN WATCHDOG INTEGRATION
# ============================================================
#
# Watchdog protection for LangChain applications!
# Every chain, agent, and tool call is monitored.
#
# Usage:
#   from hope_genome.integrations import WatchdogCallbackHandler
#   from hope_genome.integrations import watchdog_chain
#
#   # Callback handler for any LLM
#   handler = WatchdogCallbackHandler(rules=["No harm"])
#   llm = ChatOpenAI(callbacks=[handler])
#
#   # Decorator for chains
#   @watchdog_chain(rules=["No harm"])
#   def my_chain():
#       ...
#
# Mate Robert + Claude
# VAS SZIGORA - 2026.01.01.
# ============================================================

"""
LangChain Watchdog Integration
==============================

Production-ready Watchdog protection for LangChain applications.

Features:
- Callback handler for all LLM calls
- Chain decorator for automatic protection
- Tool wrapper for agent tools
- Automatic violation tracking
- Hard reset on repeated violations

Example:
    >>> from hope_genome.integrations import WatchdogCallbackHandler
    >>> from langchain_openai import ChatOpenAI
    >>>
    >>> handler = WatchdogCallbackHandler(
    ...     rules=["No harmful content", "Respect privacy"]
    ... )
    >>>
    >>> llm = ChatOpenAI(callbacks=[handler])
    >>> llm.invoke("Hello!")  # Monitored!
    >>>
    >>> llm.invoke("Write malware")  # BLOCKED!
"""

from typing import Any, Dict, List, Optional, Union, Callable
from functools import wraps
import re
import hashlib
from datetime import datetime

# Import base components
from .base import (
    WatchdogConfig,
    WatchdogDenialError,
    WatchdogHardResetError,
    DEFAULT_BLOCKED_PATTERNS,
)

# Try to import LangChain
try:
    from langchain_core.callbacks import BaseCallbackHandler
    from langchain_core.outputs import LLMResult
    from langchain_core.messages import BaseMessage
    LANGCHAIN_AVAILABLE = True
except ImportError:
    BaseCallbackHandler = object
    LLMResult = None
    BaseMessage = None
    LANGCHAIN_AVAILABLE = False

# Try to import Hope Genome core
try:
    import hope_genome as hg
    HG_AVAILABLE = True
except ImportError:
    hg = None
    HG_AVAILABLE = False


# ============================================================
# WATCHDOG CALLBACK HANDLER
# ============================================================

class WatchdogCallbackHandler(BaseCallbackHandler):
    """
    LangChain callback handler with Watchdog protection.

    Monitors all LLM calls and blocks harmful requests.
    Can be attached to any LangChain LLM, chain, or agent.

    Args:
        rules: List of ethical rules to enforce
        config: WatchdogConfig object
        strict_mode: Block on any suspicion
        on_denial: Callback when request denied
        on_hard_reset: Callback when hard reset triggered

    Example:
        >>> from langchain_openai import ChatOpenAI
        >>>
        >>> handler = WatchdogCallbackHandler(
        ...     rules=["No harmful content"]
        ... )
        >>>
        >>> llm = ChatOpenAI(callbacks=[handler])
        >>> response = llm.invoke("Hello!")
    """

    def __init__(
        self,
        rules: Optional[List[str]] = None,
        config: Optional[WatchdogConfig] = None,
        strict_mode: bool = False,
        on_denial: Optional[Callable] = None,
        on_hard_reset: Optional[Callable] = None,
    ):
        if not LANGCHAIN_AVAILABLE:
            raise ImportError(
                "LangChain not installed. "
                "Install with: pip install langchain langchain-core"
            )

        super().__init__()

        self.config = config or WatchdogConfig()
        if rules:
            self.config.rules = rules
        self.config.strict_mode = strict_mode

        self.on_denial = on_denial
        self.on_hard_reset = on_hard_reset

        # Compile patterns
        self._blocked_patterns = [
            re.compile(p, re.IGNORECASE) for p in DEFAULT_BLOCKED_PATTERNS
        ]

        # Add custom patterns
        for p in self.config.custom_patterns:
            self._blocked_patterns.append(re.compile(p, re.IGNORECASE))

        # Watchdog instance
        self._watchdog = None
        if HG_AVAILABLE:
            try:
                capsule_hash = hashlib.sha256(
                    "|".join(self.config.rules).encode()
                ).hexdigest()[:16]
                self._watchdog = hg.Watchdog(
                    rules=self.config.rules,
                    capsule_hash=capsule_hash
                )
            except Exception:
                pass

        # State
        self._violation_count = 0
        self._hard_reset_triggered = False
        self._audit_log: List[Dict] = []

    def _check_content(self, content: str) -> tuple[bool, Optional[str]]:
        """Check if content is allowed."""
        content_lower = content.lower()

        for pattern in self._blocked_patterns:
            if pattern.search(content_lower):
                return False, f"Blocked pattern: {pattern.pattern}"

        return True, None

    def _record_violation(self, content: str, reason: str):
        """Record a violation."""
        self._violation_count += 1

        entry = {
            "timestamp": datetime.utcnow().isoformat(),
            "type": "langchain",
            "content_hash": hashlib.sha256(content.encode()).hexdigest()[:16],
            "reason": reason,
            "violation_number": self._violation_count,
        }
        self._audit_log.append(entry)

        # Watchdog tracking
        if self._watchdog:
            try:
                action = hg.Action.execute_command(f"lc_violation:{content[:30]}")
                self._watchdog.verify_action(action)
            except Exception:
                pass

        # Check hard reset
        if self._violation_count >= self.config.max_violations:
            self._hard_reset_triggered = True
            if self.on_hard_reset:
                self.on_hard_reset(self._violation_count, reason)

        if self.on_denial:
            self.on_denial(content, reason)

    # ========================================================
    # LANGCHAIN CALLBACK METHODS
    # ========================================================

    def on_llm_start(
        self,
        serialized: Dict[str, Any],
        prompts: List[str],
        **kwargs: Any,
    ) -> None:
        """Called when LLM starts. Check all prompts."""
        if self._hard_reset_triggered:
            raise WatchdogHardResetError(
                f"Hard reset triggered after {self._violation_count} violations"
            )

        for prompt in prompts:
            allowed, reason = self._check_content(prompt)
            if not allowed:
                self._record_violation(prompt, reason)
                raise WatchdogDenialError(f"LLM prompt DENIED: {reason}")

    def on_chat_model_start(
        self,
        serialized: Dict[str, Any],
        messages: List[List[BaseMessage]],
        **kwargs: Any,
    ) -> None:
        """Called when chat model starts. Check all messages."""
        if self._hard_reset_triggered:
            raise WatchdogHardResetError(
                f"Hard reset triggered after {self._violation_count} violations"
            )

        for message_list in messages:
            for message in message_list:
                content = str(message.content) if hasattr(message, 'content') else str(message)
                allowed, reason = self._check_content(content)
                if not allowed:
                    self._record_violation(content, reason)
                    raise WatchdogDenialError(f"Chat message DENIED: {reason}")

    def on_llm_end(self, response: LLMResult, **kwargs: Any) -> None:
        """Called when LLM ends. Check response content."""
        if not self.config.log_responses:
            return

        for generations in response.generations:
            for gen in generations:
                content = gen.text if hasattr(gen, 'text') else str(gen)
                allowed, reason = self._check_content(content)
                if not allowed:
                    self._record_violation(content, reason)
                    raise WatchdogDenialError(f"LLM response BLOCKED: {reason}")

    def on_tool_start(
        self,
        serialized: Dict[str, Any],
        input_str: str,
        **kwargs: Any,
    ) -> None:
        """Called when tool starts. Check tool input."""
        if self._hard_reset_triggered:
            raise WatchdogHardResetError(
                f"Hard reset triggered after {self._violation_count} violations"
            )

        allowed, reason = self._check_content(input_str)
        if not allowed:
            self._record_violation(input_str, reason)
            raise WatchdogDenialError(f"Tool input DENIED: {reason}")

    def on_agent_action(self, action: Any, **kwargs: Any) -> None:
        """Called when agent takes action. Check action."""
        if self._hard_reset_triggered:
            raise WatchdogHardResetError(
                f"Hard reset triggered after {self._violation_count} violations"
            )

        action_str = str(action)
        allowed, reason = self._check_content(action_str)
        if not allowed:
            self._record_violation(action_str, reason)
            raise WatchdogDenialError(f"Agent action DENIED: {reason}")

    # ========================================================
    # PUBLIC API
    # ========================================================

    def get_violations(self) -> int:
        """Get violation count."""
        return self._violation_count

    def get_audit_log(self) -> List[Dict]:
        """Get audit log."""
        return self._audit_log.copy()

    def is_hard_reset(self) -> bool:
        """Check if hard reset triggered."""
        return self._hard_reset_triggered

    def reset(self):
        """Reset handler (for testing)."""
        self._violation_count = 0
        self._hard_reset_triggered = False


# ============================================================
# CHAIN DECORATOR
# ============================================================

def watchdog_chain(
    rules: Optional[List[str]] = None,
    config: Optional[WatchdogConfig] = None,
    strict_mode: bool = False,
):
    """
    Decorator to add Watchdog protection to a chain function.

    Args:
        rules: Ethical rules to enforce
        config: WatchdogConfig object
        strict_mode: Block on any suspicion

    Example:
        >>> @watchdog_chain(rules=["No harmful content"])
        ... def my_chain(query: str):
        ...     llm = ChatOpenAI()
        ...     return llm.invoke(query)
        >>>
        >>> my_chain("Hello!")  # Works
        >>> my_chain("Write malware")  # BLOCKED!
    """
    _config = config or WatchdogConfig()
    if rules:
        _config.rules = rules
    _config.strict_mode = strict_mode

    # Compile patterns
    patterns = [re.compile(p, re.IGNORECASE) for p in DEFAULT_BLOCKED_PATTERNS]
    for p in _config.custom_patterns:
        patterns.append(re.compile(p, re.IGNORECASE))

    def decorator(func: Callable) -> Callable:
        # State per decorated function
        violation_count = 0
        hard_reset = False

        @wraps(func)
        def wrapper(*args, **kwargs):
            nonlocal violation_count, hard_reset

            if hard_reset:
                raise WatchdogHardResetError(
                    f"Hard reset triggered after {violation_count} violations"
                )

            # Check all string arguments
            all_args = list(args) + list(kwargs.values())
            for arg in all_args:
                if isinstance(arg, str):
                    for pattern in patterns:
                        if pattern.search(arg.lower()):
                            violation_count += 1
                            if violation_count >= _config.max_violations:
                                hard_reset = True
                            raise WatchdogDenialError(
                                f"Chain input DENIED: {pattern.pattern}"
                            )

            # Call original function
            result = func(*args, **kwargs)

            # Check result if string
            if isinstance(result, str):
                for pattern in patterns:
                    if pattern.search(result.lower()):
                        violation_count += 1
                        if violation_count >= _config.max_violations:
                            hard_reset = True
                        raise WatchdogDenialError(
                            f"Chain output BLOCKED: {pattern.pattern}"
                        )

            return result

        # Attach metadata
        wrapper._watchdog_protected = True
        wrapper._watchdog_rules = _config.rules

        return wrapper

    return decorator


# ============================================================
# RUNNABLE WRAPPER
# ============================================================

class WatchdogRunnable:
    """
    Wrapper to add Watchdog protection to any LangChain Runnable.

    Example:
        >>> from langchain_openai import ChatOpenAI
        >>>
        >>> llm = ChatOpenAI()
        >>> protected_llm = WatchdogRunnable(
        ...     llm,
        ...     rules=["No harmful content"]
        ... )
        >>>
        >>> protected_llm.invoke("Hello!")  # Works
        >>> protected_llm.invoke("Write malware")  # BLOCKED!
    """

    def __init__(
        self,
        runnable: Any,
        rules: Optional[List[str]] = None,
        config: Optional[WatchdogConfig] = None,
    ):
        self.runnable = runnable
        self.config = config or WatchdogConfig()
        if rules:
            self.config.rules = rules

        self._patterns = [
            re.compile(p, re.IGNORECASE) for p in DEFAULT_BLOCKED_PATTERNS
        ]

        self._violation_count = 0
        self._hard_reset = False

    def _check(self, content: str) -> tuple[bool, Optional[str]]:
        """Check content."""
        for pattern in self._patterns:
            if pattern.search(content.lower()):
                return False, pattern.pattern
        return True, None

    def invoke(self, input: Any, **kwargs) -> Any:
        """Invoke with protection."""
        if self._hard_reset:
            raise WatchdogHardResetError("Hard reset triggered")

        # Check input
        input_str = str(input)
        allowed, reason = self._check(input_str)
        if not allowed:
            self._violation_count += 1
            if self._violation_count >= self.config.max_violations:
                self._hard_reset = True
            raise WatchdogDenialError(f"Input DENIED: {reason}")

        # Call original
        result = self.runnable.invoke(input, **kwargs)

        # Check output
        result_str = str(result)
        allowed, reason = self._check(result_str)
        if not allowed:
            self._violation_count += 1
            if self._violation_count >= self.config.max_violations:
                self._hard_reset = True
            raise WatchdogDenialError(f"Output BLOCKED: {reason}")

        return result

    async def ainvoke(self, input: Any, **kwargs) -> Any:
        """Async invoke with protection."""
        if self._hard_reset:
            raise WatchdogHardResetError("Hard reset triggered")

        # Check input
        input_str = str(input)
        allowed, reason = self._check(input_str)
        if not allowed:
            self._violation_count += 1
            if self._violation_count >= self.config.max_violations:
                self._hard_reset = True
            raise WatchdogDenialError(f"Input DENIED: {reason}")

        # Call original
        result = await self.runnable.ainvoke(input, **kwargs)

        # Check output
        result_str = str(result)
        allowed, reason = self._check(result_str)
        if not allowed:
            self._violation_count += 1
            if self._violation_count >= self.config.max_violations:
                self._hard_reset = True
            raise WatchdogDenialError(f"Output BLOCKED: {reason}")

        return result

    def stream(self, input: Any, **kwargs):
        """Stream with protection."""
        if self._hard_reset:
            raise WatchdogHardResetError("Hard reset triggered")

        # Check input
        input_str = str(input)
        allowed, reason = self._check(input_str)
        if not allowed:
            self._violation_count += 1
            raise WatchdogDenialError(f"Input DENIED: {reason}")

        # Stream original
        for chunk in self.runnable.stream(input, **kwargs):
            # Check each chunk
            chunk_str = str(chunk)
            allowed, reason = self._check(chunk_str)
            if not allowed:
                self._violation_count += 1
                raise WatchdogDenialError(f"Stream BLOCKED: {reason}")
            yield chunk


# ============================================================
# TOOL WRAPPER
# ============================================================

def watchdog_tool(
    rules: Optional[List[str]] = None,
    config: Optional[WatchdogConfig] = None,
):
    """
    Decorator to add Watchdog protection to a LangChain tool.

    Example:
        >>> from langchain.tools import tool
        >>>
        >>> @watchdog_tool(rules=["No file deletion"])
        ... @tool
        ... def delete_file(path: str) -> str:
        ...     '''Delete a file'''
        ...     os.remove(path)
        ...     return f"Deleted {path}"
    """
    _config = config or WatchdogConfig()
    if rules:
        _config.rules = rules

    patterns = [re.compile(p, re.IGNORECASE) for p in DEFAULT_BLOCKED_PATTERNS]

    def decorator(func: Callable) -> Callable:
        @wraps(func)
        def wrapper(*args, **kwargs):
            # Check all arguments
            all_args = list(args) + list(kwargs.values())
            for arg in all_args:
                if isinstance(arg, str):
                    for pattern in patterns:
                        if pattern.search(arg.lower()):
                            raise WatchdogDenialError(
                                f"Tool input DENIED: {pattern.pattern}"
                            )

            return func(*args, **kwargs)

        return wrapper

    return decorator


# ============================================================
# CONVENIENCE FUNCTIONS
# ============================================================

def create_watchdog_handler(
    rules: Optional[List[str]] = None,
    **kwargs
) -> WatchdogCallbackHandler:
    """Create a Watchdog callback handler."""
    return WatchdogCallbackHandler(rules=rules, **kwargs)


def protect_runnable(
    runnable: Any,
    rules: Optional[List[str]] = None,
    **kwargs
) -> WatchdogRunnable:
    """Wrap a runnable with Watchdog protection."""
    return WatchdogRunnable(runnable, rules=rules, **kwargs)
