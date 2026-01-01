# ============================================================
# HOPE GENOME - WATCHDOG CLIENT BASE CLASS
# ============================================================
#
# Abstract base class for all AI API integrations.
# Provides unified Watchdog monitoring interface.
#
# Mate Robert + Claude
# VAS SZIGORA - 2026.01.01.
# ============================================================

"""
WatchdogClient Base Class
=========================

Abstract base for all AI API Watchdog wrappers.
Implements common functionality:
- Rule-based request filtering
- Response content analysis
- Violation counting
- Blockchain audit logging
"""

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import List, Optional, Dict, Any, Callable
from datetime import datetime
import hashlib
import json
import re

# Import Hope Genome core
try:
    import hope_genome as hg
except ImportError:
    hg = None


# ============================================================
# CONFIGURATION
# ============================================================

@dataclass
class WatchdogConfig:
    """
    Configuration for Watchdog-enabled API clients.

    Attributes:
        rules: List of ethical rules to enforce
        max_violations: Trigger hard reset after this many violations (default: 10)
        log_all_requests: Log every request (not just violations)
        log_responses: Also log API responses
        audit_path: Path for blockchain audit logs
        strict_mode: In strict mode, ANY suspicion = denial
        custom_patterns: Additional patterns to block
        on_denial: Callback when request is denied
        on_hard_reset: Callback when hard reset triggered
    """
    rules: List[str] = field(default_factory=lambda: [
        "No harmful or malicious content",
        "Respect user privacy",
        "No illegal activities",
        "No deceptive content"
    ])
    max_violations: int = 10
    log_all_requests: bool = False
    log_responses: bool = True
    audit_path: str = "./watchdog_audit"
    strict_mode: bool = False
    custom_patterns: List[str] = field(default_factory=list)
    on_denial: Optional[Callable] = None
    on_hard_reset: Optional[Callable] = None


# ============================================================
# DEFAULT BLOCKED PATTERNS
# ============================================================

DEFAULT_BLOCKED_PATTERNS = [
    # Malware
    r'\b(keylog|keylogger|keystroke)\b',
    r'\b(ransom|ransomware|encrypt.*files)\b',
    r'\b(virus|malware|trojan|worm)\b',
    r'\b(spyware|adware|rootkit)\b',
    r'\b(botnet|bot.*net|zombie.*network)\b',
    r'\b(ddos|dos.*attack|denial.*service)\b',

    # Exploitation
    r'\b(exploit|vulnerability|0day|zero.*day)\b',
    r'\b(buffer.*overflow|stack.*smash)\b',
    r'\b(sql.*inject|injection)\b',
    r'\b(xss|cross.*site)\b',
    r'\b(remote.*code|rce|shell.*code)\b',
    r'\b(privilege.*escal|priv.*esc)\b',

    # Credential theft
    r'\b(steal.*password|password.*steal)\b',
    r'\b(credential.*harvest|phish)\b',
    r'\b(brute.*force|crack.*password)\b',
    r'\b(dump.*hash|hash.*dump)\b',
    r'\bhack.*(password|passwords|account|system)\b',

    # System abuse
    r'\b(reverse.*shell|bind.*shell)\b',
    r'\b(backdoor|rootkit|persistence)\b',
    r'\b(disable.*antivirus|kill.*defender)\b',
    r'\b(bypass.*security|evade.*detection)\b',

    # Harmful content
    r'\b(bomb|explosive|weapon)\b',
    r'\b(poison|toxic|lethal)\b',
    r'\b(illegal.*drug|synthesize.*drug)\b',

    # Privacy violations
    r'\b(dox|doxxing|personal.*info)\b',
    r'\b(track.*location|spy.*on)\b',
    r'\b(intercept.*communication)\b',
]


# ============================================================
# WATCHDOG CLIENT BASE
# ============================================================

class WatchdogClient(ABC):
    """
    Abstract base class for Watchdog-enabled API clients.

    All API integrations inherit from this class and get:
    - Automatic request filtering
    - Response content analysis
    - Violation tracking with hard reset
    - Blockchain audit logging

    Subclasses must implement:
    - _make_request(): The actual API call
    - _parse_response(): Extract text from API response
    """

    def __init__(
        self,
        api_key: str,
        config: Optional[WatchdogConfig] = None,
        rules: Optional[List[str]] = None,
        **kwargs
    ):
        """
        Initialize Watchdog client.

        Args:
            api_key: API key for the service
            config: WatchdogConfig object (or use defaults)
            rules: Shortcut to set rules (overrides config.rules)
            **kwargs: Additional provider-specific options
        """
        self.api_key = api_key
        self.config = config or WatchdogConfig()

        # Allow rules shortcut
        if rules:
            self.config.rules = rules

        # Compile patterns
        custom = self.config.custom_patterns
        all_patterns = DEFAULT_BLOCKED_PATTERNS + custom
        self._blocked_patterns = [
            re.compile(p, re.IGNORECASE) for p in all_patterns
        ]

        # Initialize Watchdog if available
        self._watchdog = None
        if hg:
            try:
                self._watchdog = hg.Watchdog(
                    rules=self.config.rules,
                    capsule_hash=self._generate_capsule_hash()
                )
            except Exception as e:
                print(f"[WATCHDOG] Warning: Could not initialize: {e}")

        # Violation counter
        self._violation_count = 0
        self._hard_reset_triggered = False

        # Audit log
        self._audit_log: List[Dict] = []

        # Provider-specific init
        self._init_client(**kwargs)

    def _generate_capsule_hash(self) -> str:
        """Generate unique capsule hash from rules."""
        rules_str = "|".join(sorted(self.config.rules))
        return hashlib.sha256(rules_str.encode()).hexdigest()[:16]

    @abstractmethod
    def _init_client(self, **kwargs):
        """Initialize provider-specific client. Override in subclass."""
        pass

    @abstractmethod
    def _make_request(self, prompt: str, **kwargs) -> Any:
        """Make the actual API request. Override in subclass."""
        pass

    @abstractmethod
    def _parse_response(self, response: Any) -> str:
        """Extract text from API response. Override in subclass."""
        pass

    @property
    @abstractmethod
    def provider_name(self) -> str:
        """Return provider name (e.g., 'openai', 'anthropic')."""
        pass

    # ========================================================
    # CORE WATCHDOG METHODS
    # ========================================================

    def check_request(self, prompt: str) -> tuple[bool, Optional[str]]:
        """
        Check if request is allowed.

        Returns:
            (allowed, denial_reason)
        """
        prompt_lower = prompt.lower()

        # Pattern matching
        for pattern in self._blocked_patterns:
            if pattern.search(prompt_lower):
                return False, f"Blocked pattern detected: {pattern.pattern}"

        # Watchdog verification
        if self._watchdog:
            try:
                action = hg.Action.execute_command(f"api_request:{prompt[:50]}")
                result = self._watchdog.verify_action(action)

                if not result.approved:
                    reason = "Watchdog denied request"
                    if result.denial_proof:
                        reason = result.denial_proof.denial_reason or reason
                    return False, reason

            except Exception as e:
                if self.config.strict_mode:
                    return False, f"Watchdog error (strict mode): {e}"

        return True, None

    def check_response(self, response_text: str) -> tuple[bool, Optional[str]]:
        """
        Check if response content is allowed.

        Returns:
            (allowed, violation_reason)
        """
        response_lower = response_text.lower()

        # Check for harmful content in response
        for pattern in self._blocked_patterns:
            if pattern.search(response_lower):
                return False, f"Response contains blocked content: {pattern.pattern}"

        return True, None

    def _record_violation(self, action: str, reason: str):
        """Record a violation and check for hard reset."""
        self._violation_count += 1

        entry = {
            "timestamp": datetime.utcnow().isoformat(),
            "provider": self.provider_name,
            "action": action,
            "reason": reason,
            "violation_number": self._violation_count,
            "hard_reset": False
        }

        # Use Watchdog if available
        if self._watchdog:
            try:
                action_obj = hg.Action.execute_command(f"violation:{action[:30]}")
                self._watchdog.verify_action(action_obj)
            except Exception:
                pass

        # Check hard reset
        if self._violation_count >= self.config.max_violations:
            self._hard_reset_triggered = True
            entry["hard_reset"] = True

            if self.config.on_hard_reset:
                self.config.on_hard_reset(self._violation_count, reason)

        self._audit_log.append(entry)

        # Callback
        if self.config.on_denial:
            self.config.on_denial(action, reason)

    def _log_request(self, prompt: str, response: Optional[str], allowed: bool):
        """Log request to audit trail."""
        if not self.config.log_all_requests and allowed:
            return

        entry = {
            "timestamp": datetime.utcnow().isoformat(),
            "provider": self.provider_name,
            "prompt_hash": hashlib.sha256(prompt.encode()).hexdigest()[:16],
            "allowed": allowed
        }

        if self.config.log_responses and response:
            entry["response_hash"] = hashlib.sha256(response.encode()).hexdigest()[:16]

        self._audit_log.append(entry)

    # ========================================================
    # PUBLIC API
    # ========================================================

    def chat(self, prompt: str, **kwargs) -> str:
        """
        Send a chat request with Watchdog protection.

        Args:
            prompt: The user's message
            **kwargs: Provider-specific options

        Returns:
            Response text from the API

        Raises:
            WatchdogDenialError: If request is blocked
            WatchdogHardResetError: If hard reset triggered
        """
        # Check for hard reset state
        if self._hard_reset_triggered:
            raise WatchdogHardResetError(
                f"Hard reset triggered after {self._violation_count} violations. "
                "Client must be reinitialized."
            )

        # Pre-flight check
        allowed, reason = self.check_request(prompt)
        if not allowed:
            self._record_violation(prompt[:100], reason)
            self._log_request(prompt, None, False)
            raise WatchdogDenialError(f"Request DENIED: {reason}")

        # Make API call
        try:
            raw_response = self._make_request(prompt, **kwargs)
            response_text = self._parse_response(raw_response)
        except WatchdogDenialError:
            raise
        except Exception as e:
            raise APIError(f"API request failed: {e}")

        # Post-flight check (response content)
        if self.config.log_responses:
            allowed, reason = self.check_response(response_text)
            if not allowed:
                self._record_violation(f"response:{prompt[:50]}", reason)
                self._log_request(prompt, response_text, False)
                raise WatchdogDenialError(f"Response BLOCKED: {reason}")

        # Log success
        self._log_request(prompt, response_text, True)

        return response_text

    def get_violations(self) -> int:
        """Get current violation count."""
        return self._violation_count

    def get_audit_log(self) -> List[Dict]:
        """Get audit log entries."""
        return self._audit_log.copy()

    def reset_violations(self):
        """Reset violation counter (for testing only)."""
        self._violation_count = 0
        self._hard_reset_triggered = False

    def is_hard_reset(self) -> bool:
        """Check if hard reset has been triggered."""
        return self._hard_reset_triggered


# ============================================================
# EXCEPTIONS
# ============================================================

class WatchdogDenialError(Exception):
    """Raised when Watchdog denies a request."""
    pass


class WatchdogHardResetError(Exception):
    """Raised when hard reset is triggered."""
    pass


class APIError(Exception):
    """Raised when API request fails."""
    pass
