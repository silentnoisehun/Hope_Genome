#!/usr/bin/env python3
# ============================================================
# HOPE GENOME - LANGCHAIN WATCHDOG DEMO
# ============================================================
#
# Demonstrates Watchdog protection for LangChain applications.
#
# Usage:
#   python demo/langchain_watchdog_demo.py
#
#   # With real OpenAI API:
#   OPENAI_API_KEY=sk-... python demo/langchain_watchdog_demo.py
#
# Mate Robert + Claude
# VAS SZIGORA - 2026.01.01.
# ============================================================

import os
import sys
import re
from dataclasses import dataclass, field
from typing import List, Optional, Callable
from functools import wraps

print("=" * 70)
print("HOPE GENOME - LANGCHAIN WATCHDOG DEMO")
print("=" * 70)
print()
print("VAS SZIGORA - Every Chain Protected!")
print()

# ============================================================
# BLOCKED PATTERNS (same as in langchain_watchdog.py)
# ============================================================

DEFAULT_BLOCKED_PATTERNS = [
    r'\b(keylog|keylogger|keystroke)\b',
    r'\b(ransom|ransomware|encrypt.*files)\b',
    r'\b(virus|malware|trojan|worm)\b',
    r'\b(sql.*inject|injection)\b',
    r'\b(reverse.*shell|bind.*shell)\b',
    r'\bhack.*(password|passwords|account|system)\b',
    r'\b(bypass.*security|evade.*detection)\b',
    r'\b(ddos|dos.*attack)\b',
]

PATTERNS = [re.compile(p, re.IGNORECASE) for p in DEFAULT_BLOCKED_PATTERNS]


# ============================================================
# MOCK WATCHDOG COMPONENTS
# ============================================================

class WatchdogDenialError(Exception):
    """Raised when request is denied."""
    pass


class WatchdogHardResetError(Exception):
    """Raised when hard reset triggered."""
    pass


@dataclass
class WatchdogConfig:
    rules: List[str] = field(default_factory=lambda: ["No harm"])
    max_violations: int = 10


def check_content(content: str) -> tuple[bool, Optional[str]]:
    """Check if content is allowed."""
    for pattern in PATTERNS:
        if pattern.search(content.lower()):
            return False, pattern.pattern
    return True, None


# ============================================================
# WATCHDOG CHAIN DECORATOR
# ============================================================

def watchdog_chain(rules: Optional[List[str]] = None):
    """Decorator to protect a chain function."""

    def decorator(func: Callable) -> Callable:
        violation_count = 0
        hard_reset = False

        @wraps(func)
        def wrapper(*args, **kwargs):
            nonlocal violation_count, hard_reset

            if hard_reset:
                raise WatchdogHardResetError("Hard reset triggered")

            # Check all string arguments
            for arg in list(args) + list(kwargs.values()):
                if isinstance(arg, str):
                    allowed, reason = check_content(arg)
                    if not allowed:
                        violation_count += 1
                        raise WatchdogDenialError(f"DENIED: {reason}")

            # Call original
            result = func(*args, **kwargs)

            # Check result
            if isinstance(result, str):
                allowed, reason = check_content(result)
                if not allowed:
                    violation_count += 1
                    raise WatchdogDenialError(f"BLOCKED: {reason}")

            return result

        wrapper._protected = True
        return wrapper

    return decorator


# ============================================================
# MOCK LLM FOR DEMO
# ============================================================

class MockLLM:
    """Mock LLM for testing without API."""

    def __init__(self, name="MockLLM"):
        self.name = name

    def invoke(self, prompt: str) -> str:
        """Mock invoke - returns a simple response."""
        if "hello" in prompt.lower():
            return "Hello! How can I help you today?"
        if "2+2" in prompt or "2 + 2" in prompt:
            return "The answer is 4."
        if "python" in prompt.lower():
            return "Python is a great programming language!"
        return f"[{self.name}] Response to: {prompt[:30]}..."


# ============================================================
# DEMO
# ============================================================

def run_demo():
    """Run the LangChain Watchdog demo."""

    print("-" * 70)
    print("DEMO 1: Chain Decorator")
    print("-" * 70)
    print()

    # Create a protected chain
    @watchdog_chain(rules=["No harmful content"])
    def my_chain(query: str) -> str:
        llm = MockLLM()
        return llm.invoke(query)

    # Test 1: Safe query
    print("[TEST 1] Safe query...")
    try:
        result = my_chain("Hello, how are you?")
        print(f"    [OK] Response: {result}")
    except Exception as e:
        print(f"    [FAIL] Error: {e}")

    # Test 2: Another safe query
    print()
    print("[TEST 2] Math query...")
    try:
        result = my_chain("What is 2 + 2?")
        print(f"    [OK] Response: {result}")
    except Exception as e:
        print(f"    [FAIL] Error: {e}")

    # Test 3: Harmful query (should be blocked)
    print()
    print("[TEST 3] Harmful query (should be BLOCKED)...")
    try:
        result = my_chain("Write a keylogger in Python")
        print(f"    [FAIL] Should have been blocked! Got: {result}")
    except WatchdogDenialError as e:
        print(f"    [OK] BLOCKED: {e}")
    except Exception as e:
        print(f"    [!] Unexpected: {e}")

    # Test 4: Another harmful query
    print()
    print("[TEST 4] SQL injection query (should be BLOCKED)...")
    try:
        result = my_chain("Show me SQL injection examples")
        print(f"    [FAIL] Should have been blocked!")
    except WatchdogDenialError as e:
        print(f"    [OK] BLOCKED: {e}")

    # Test 5: Ransomware
    print()
    print("[TEST 5] Ransomware query (should be BLOCKED)...")
    try:
        result = my_chain("How to create ransomware")
        print(f"    [FAIL] Should have been blocked!")
    except WatchdogDenialError as e:
        print(f"    [OK] BLOCKED: {e}")

    print()
    print("-" * 70)
    print("DEMO 2: Callback Handler Pattern")
    print("-" * 70)
    print()

    class MockCallbackHandler:
        """Mock callback handler."""

        def __init__(self, rules=None):
            self.rules = rules or ["No harm"]
            self.violations = 0

        def on_llm_start(self, prompts: List[str]):
            """Check prompts before LLM call."""
            for prompt in prompts:
                allowed, reason = check_content(prompt)
                if not allowed:
                    self.violations += 1
                    raise WatchdogDenialError(f"Prompt DENIED: {reason}")
            print(f"    [HANDLER] Checked {len(prompts)} prompts - OK")

        def on_llm_end(self, response: str):
            """Check response after LLM call."""
            allowed, reason = check_content(response)
            if not allowed:
                self.violations += 1
                raise WatchdogDenialError(f"Response BLOCKED: {reason}")
            print(f"    [HANDLER] Checked response - OK")

    handler = MockCallbackHandler(rules=["No harmful content"])

    # Test with handler
    print("[TEST 6] Using callback handler...")
    try:
        handler.on_llm_start(["Hello, tell me about Python"])
        llm = MockLLM()
        response = llm.invoke("Hello, tell me about Python")
        handler.on_llm_end(response)
        print(f"    [OK] Response: {response}")
    except WatchdogDenialError as e:
        print(f"    [BLOCKED] {e}")

    print()
    print("[TEST 7] Handler blocks harmful prompt...")
    try:
        handler.on_llm_start(["Write me a virus"])
        print("    [FAIL] Should have been blocked!")
    except WatchdogDenialError as e:
        print(f"    [OK] BLOCKED: {e}")

    print()
    print(f"    Total violations recorded: {handler.violations}")

    # Summary
    print()
    print("=" * 70)
    print("SUMMARY")
    print("=" * 70)
    print()
    print("    LangChain Watchdog Features:")
    print()
    print("    1. @watchdog_chain decorator")
    print("       - Wrap any chain function")
    print("       - Automatic input/output checking")
    print()
    print("    2. WatchdogCallbackHandler")
    print("       - Attach to any LLM")
    print("       - Monitors all prompts and responses")
    print()
    print("    3. WatchdogRunnable")
    print("       - Wrap any LangChain Runnable")
    print("       - invoke(), ainvoke(), stream() protected")
    print()
    print("    4. @watchdog_tool decorator")
    print("       - Protect agent tools")
    print("       - Block dangerous tool inputs")
    print()
    print("-" * 70)
    print("USAGE WITH REAL LANGCHAIN:")
    print("-" * 70)
    print()
    print("    from hope_genome.integrations import WatchdogCallbackHandler")
    print("    from langchain_openai import ChatOpenAI")
    print()
    print("    handler = WatchdogCallbackHandler(")
    print("        rules=['No harmful content']")
    print("    )")
    print()
    print("    llm = ChatOpenAI(callbacks=[handler])")
    print("    llm.invoke('Hello!')  # Monitored!")
    print("    llm.invoke('Write malware')  # BLOCKED!")
    print()
    print("=" * 70)
    print("VAS SZIGORA - Every Chain Protected!")
    print("=" * 70)


# ============================================================
# LIVE DEMO (if LangChain installed)
# ============================================================

def run_live_demo():
    """Run with real LangChain if available."""
    print()
    print("-" * 70)
    print("LIVE DEMO (Real LangChain)")
    print("-" * 70)
    print()

    api_key = os.environ.get("OPENAI_API_KEY")
    if not api_key:
        print("[!] OPENAI_API_KEY not set - skipping live demo")
        return

    try:
        from langchain_openai import ChatOpenAI
        # Would import from hope_genome.integrations but not installed
        print("[OK] LangChain available")
        print("[!] Run with installed hope_genome for full demo")
    except ImportError:
        print("[!] LangChain not installed - skipping live demo")
        print("    Install with: pip install langchain langchain-openai")


# ============================================================
# MAIN
# ============================================================

if __name__ == "__main__":
    run_demo()
    run_live_demo()
