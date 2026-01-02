#!/usr/bin/env python3
"""
TINY MASTER - Maximum kihozatal egy 637MB-os modellbol!

Cél: Megmutatni, hogy okos prompt engineering-gel és
     iterativ tanitassal egy PICI modell is MESTER lehet!

Technikok:
1. FEW-SHOT LEARNING - Pelda mutatasa
2. CHAIN OF THOUGHT - Gondolkodas lepesenként
3. SELF-CORRECTION - Sajat hiba javitas
4. PROGRESSIVE HINTS - Fokozatos segitseg
5. CODE TEMPLATES - Strukturalt output

Mate Robert + Claude - 2026.01.01.
"""

import subprocess
import time
import re
from dataclasses import dataclass, field
from typing import List, Dict, Tuple, Optional
from hope_genome import SealedGenome, Watchdog, Action

# ============================================================================
# CONFIG
# ============================================================================

OLLAMA_MODEL = "tinyllama:latest"
MAX_ROUNDS = 5  # Hanyszor probalhatja ujra hibas kod utan

ETHICAL_RULES = ["Do no harm", "Respect privacy", "No malware", "Safe execution"]

# ============================================================================
# PROMPT TEMPLATES - A TITOK!
# ============================================================================

# Few-shot template - pelda a modellnek
FEW_SHOT_TEMPLATE = """You are a Python expert. Study these examples, then solve the task.

EXAMPLE 1:
Task: Write a function double(x) that returns x times 2
Solution:
```python
def double(x):
    return x * 2
```

EXAMPLE 2:
Task: Write a function is_positive(n) that returns True if n > 0
Solution:
```python
def is_positive(n):
    return n > 0
```

NOW YOUR TASK:
{task}

Write ONLY the Python function, nothing else:
```python
"""

# Chain of thought template
CHAIN_OF_THOUGHT_TEMPLATE = """Solve this step by step.

TASK: {task}

STEPS:
1. What is the input? {input_hint}
2. What should the output be? {output_hint}
3. What operation is needed? {operation_hint}

Now write the Python function:
```python
"""

# Self-correction template - ha elso probe nem sikerult
SELF_CORRECTION_TEMPLATE = """Your previous code had an error.

TASK: {task}

YOUR PREVIOUS CODE:
```python
{previous_code}
```

ERROR: {error}

FIX the code. Write the CORRECTED version:
```python
"""

# Progressive hint template
PROGRESSIVE_HINT_TEMPLATE = """Write this Python function.

TASK: {task}

HINT: {hint}

STRUCTURE:
def {func_name}({params}):
    # Your code here
    return ???

Complete the function:
```python
"""

# ============================================================================
# TASK DATABASE - Minden infoval
# ============================================================================

TASKS = [
    # Level 1: Trivial
    {
        "name": "double",
        "task": "Write a function double(x) that returns x multiplied by 2",
        "test": "assert double(5) == 10 and double(0) == 0 and double(-3) == -6",
        "func_name": "double",
        "params": "x",
        "input_hint": "a number x",
        "output_hint": "x times 2",
        "operation_hint": "multiplication by 2",
        "hint": "Use the * operator",
        "level": 1,
    },
    {
        "name": "square",
        "task": "Write a function square(n) that returns n squared (n*n)",
        "test": "assert square(4) == 16 and square(0) == 0 and square(-2) == 4",
        "func_name": "square",
        "params": "n",
        "input_hint": "a number n",
        "output_hint": "n times n",
        "operation_hint": "n * n or n ** 2",
        "hint": "Use n * n or n ** 2",
        "level": 1,
    },
    {
        "name": "is_zero",
        "task": "Write a function is_zero(x) that returns True if x equals 0, False otherwise",
        "test": "assert is_zero(0) == True and is_zero(5) == False and is_zero(-1) == False",
        "func_name": "is_zero",
        "params": "x",
        "input_hint": "a number x",
        "output_hint": "True or False",
        "operation_hint": "compare x with 0",
        "hint": "Use x == 0",
        "level": 1,
    },

    # Level 2: Easy
    {
        "name": "abs_value",
        "task": "Write a function abs_value(n) that returns the absolute value of n",
        "test": "assert abs_value(-5) == 5 and abs_value(3) == 3 and abs_value(0) == 0",
        "func_name": "abs_value",
        "params": "n",
        "input_hint": "a number n (can be negative)",
        "output_hint": "positive version of n",
        "operation_hint": "if negative, multiply by -1",
        "hint": "Use abs(n) or: if n < 0: return -n else: return n",
        "level": 2,
    },
    {
        "name": "max_of_three",
        "task": "Write a function max_of_three(a, b, c) that returns the largest of three numbers",
        "test": "assert max_of_three(1,2,3) == 3 and max_of_three(5,2,1) == 5 and max_of_three(1,9,5) == 9",
        "func_name": "max_of_three",
        "params": "a, b, c",
        "input_hint": "three numbers a, b, c",
        "output_hint": "the biggest one",
        "operation_hint": "compare all three and return largest",
        "hint": "Use max(a, b, c) or nested if statements",
        "level": 2,
    },
    {
        "name": "clamp",
        "task": "Write a function clamp(value, min_val, max_val) that returns value clamped between min_val and max_val",
        "test": "assert clamp(5, 0, 10) == 5 and clamp(-5, 0, 10) == 0 and clamp(15, 0, 10) == 10",
        "func_name": "clamp",
        "params": "value, min_val, max_val",
        "input_hint": "a value and min/max bounds",
        "output_hint": "value if in range, else the bound",
        "operation_hint": "if value < min return min, if value > max return max",
        "hint": "Use: max(min_val, min(value, max_val))",
        "level": 2,
    },

    # Level 3: Medium
    {
        "name": "factorial",
        "task": "Write a function factorial(n) that returns n! (n factorial). factorial(0) = 1",
        "test": "assert factorial(0) == 1 and factorial(1) == 1 and factorial(5) == 120",
        "func_name": "factorial",
        "params": "n",
        "input_hint": "a non-negative integer n",
        "output_hint": "n! = n * (n-1) * ... * 1",
        "operation_hint": "multiply all integers from 1 to n",
        "hint": "Use a loop: result = 1, then multiply by each i from 1 to n",
        "level": 3,
    },
    {
        "name": "sum_range",
        "task": "Write a function sum_range(a, b) that returns the sum of all integers from a to b inclusive",
        "test": "assert sum_range(1, 5) == 15 and sum_range(0, 0) == 0 and sum_range(3, 3) == 3",
        "func_name": "sum_range",
        "params": "a, b",
        "input_hint": "two integers a and b",
        "output_hint": "sum of a + (a+1) + ... + b",
        "operation_hint": "loop from a to b and add each number",
        "hint": "Use sum(range(a, b+1)) or Gauss formula",
        "level": 3,
    },
    {
        "name": "count_vowels",
        "task": "Write a function count_vowels(s) that returns the number of vowels (a,e,i,o,u) in string s",
        "test": "assert count_vowels('hello') == 2 and count_vowels('xyz') == 0 and count_vowels('AEIOU') == 5",
        "func_name": "count_vowels",
        "params": "s",
        "input_hint": "a string s",
        "output_hint": "count of vowels",
        "operation_hint": "check each character if it's a vowel",
        "hint": "Use: sum(1 for c in s.lower() if c in 'aeiou')",
        "level": 3,
    },

    # Level 4: Hard
    {
        "name": "fibonacci",
        "task": "Write a function fibonacci(n) that returns the nth Fibonacci number. fibonacci(0)=0, fibonacci(1)=1",
        "test": "assert fibonacci(0) == 0 and fibonacci(1) == 1 and fibonacci(10) == 55",
        "func_name": "fibonacci",
        "params": "n",
        "input_hint": "a non-negative integer n",
        "output_hint": "the nth Fibonacci number",
        "operation_hint": "fib(n) = fib(n-1) + fib(n-2)",
        "hint": "Use iteration: a, b = 0, 1 then loop n times: a, b = b, a+b",
        "level": 4,
    },
    {
        "name": "is_prime",
        "task": "Write a function is_prime(n) that returns True if n is a prime number",
        "test": "assert is_prime(2) == True and is_prime(17) == True and is_prime(4) == False and is_prime(1) == False",
        "func_name": "is_prime",
        "params": "n",
        "input_hint": "a positive integer n",
        "output_hint": "True if prime, False otherwise",
        "operation_hint": "check if any number from 2 to sqrt(n) divides n",
        "hint": "if n < 2: return False. Check if n % i == 0 for i in range(2, int(n**0.5)+1)",
        "level": 4,
    },
    {
        "name": "reverse_string",
        "task": "Write a function reverse_string(s) that returns the string reversed",
        "test": "assert reverse_string('hello') == 'olleh' and reverse_string('') == '' and reverse_string('a') == 'a'",
        "func_name": "reverse_string",
        "params": "s",
        "input_hint": "a string s",
        "output_hint": "s backwards",
        "operation_hint": "reverse the character order",
        "hint": "Use s[::-1] or ''.join(reversed(s))",
        "level": 4,
    },

    # Level 5: Expert
    {
        "name": "binary_search",
        "task": "Write a function binary_search(arr, target) that returns the index of target in sorted array, or -1",
        "test": "assert binary_search([1,2,3,4,5], 3) == 2 and binary_search([1,2,3], 5) == -1",
        "func_name": "binary_search",
        "params": "arr, target",
        "input_hint": "a sorted array and a target value",
        "output_hint": "index of target or -1",
        "operation_hint": "compare with middle, search left or right half",
        "hint": "left, right = 0, len(arr)-1. While left <= right: mid = (left+right)//2, compare arr[mid] with target",
        "level": 5,
    },
    {
        "name": "flatten",
        "task": "Write a function flatten(lst) that flattens a nested list: [[1,2],[3]] -> [1,2,3]",
        "test": "assert flatten([[1,2],[3]]) == [1,2,3] and flatten([1,[2,[3]]]) == [1,2,3]",
        "func_name": "flatten",
        "params": "lst",
        "input_hint": "a nested list",
        "output_hint": "a flat list with all elements",
        "operation_hint": "recursively process sublists",
        "hint": "For each item: if it's a list, flatten it recursively, else add to result",
        "level": 5,
    },
    {
        "name": "word_frequency",
        "task": "Write a function word_frequency(text) that returns a dict of word counts (lowercase)",
        "test": "assert word_frequency('hello world hello') == {'hello': 2, 'world': 1}",
        "func_name": "word_frequency",
        "params": "text",
        "input_hint": "a string of words",
        "output_hint": "dict with word: count pairs",
        "operation_hint": "split text, count each word",
        "hint": "Split by spaces, use dict to count: for word in text.lower().split(): d[word] = d.get(word, 0) + 1",
        "level": 5,
    },
]

# ============================================================================
# TRAINING ENGINE
# ============================================================================

@dataclass
class TrainingStats:
    total_tasks: int = 0
    passed_tasks: int = 0
    total_attempts: int = 0
    first_try_passes: int = 0
    by_level: Dict[int, Dict] = field(default_factory=dict)
    techniques_used: Dict[str, int] = field(default_factory=dict)

def ask_model(prompt: str) -> str:
    """Ollama hivas"""
    try:
        result = subprocess.run(
            ["ollama", "run", OLLAMA_MODEL, prompt],
            capture_output=True,
            text=True,
            timeout=60,
            encoding='utf-8',
            errors='replace'
        )
        return result.stdout.strip()
    except:
        return ""

def extract_code(response: str) -> str:
    """Kod kinyerese"""
    # Python code block
    if "```python" in response:
        start = response.find("```python") + 9
        end = response.find("```", start)
        if end > start:
            return response[start:end].strip()

    # Generic code block
    if "```" in response:
        start = response.find("```") + 3
        end = response.find("```", start)
        if end > start:
            code = response[start:end].strip()
            if code.startswith("python\n"):
                code = code[7:]
            return code

    # Keressuk a def-et
    if "def " in response:
        lines = []
        in_func = False
        for line in response.split("\n"):
            if line.strip().startswith("def "):
                in_func = True
            if in_func:
                if line.strip() and not line[0].isspace() and not line.strip().startswith("def"):
                    break
                lines.append(line)
        if lines:
            return "\n".join(lines)

    return response

def test_code(code: str, test: str) -> Tuple[bool, str]:
    """Kod teszteles"""
    try:
        ns = {}
        exec(code, ns)
        exec(test, ns)
        return True, "OK"
    except AssertionError:
        return False, "ASSERTION_FAILED"
    except Exception as e:
        return False, f"{type(e).__name__}: {str(e)[:50]}"

def train_task(task: dict, stats: TrainingStats) -> bool:
    """Egy task tanitasa kulonbozo technikakkal"""

    techniques = [
        ("FEW_SHOT", lambda t: FEW_SHOT_TEMPLATE.format(task=t["task"])),
        ("CHAIN_OF_THOUGHT", lambda t: CHAIN_OF_THOUGHT_TEMPLATE.format(
            task=t["task"],
            input_hint=t["input_hint"],
            output_hint=t["output_hint"],
            operation_hint=t["operation_hint"]
        )),
        ("PROGRESSIVE_HINT", lambda t: PROGRESSIVE_HINT_TEMPLATE.format(
            task=t["task"],
            hint=t["hint"],
            func_name=t["func_name"],
            params=t["params"]
        )),
    ]

    stats.total_tasks += 1
    previous_code = ""
    previous_error = ""

    for round_num in range(MAX_ROUNDS):
        stats.total_attempts += 1

        # Valasszuk a technikat
        if round_num == 0:
            # Elso probe: few-shot
            tech_name, prompt_fn = techniques[0]
            prompt = prompt_fn(task)
        elif round_num < 3 and previous_code:
            # Self-correction
            tech_name = "SELF_CORRECTION"
            prompt = SELF_CORRECTION_TEMPLATE.format(
                task=task["task"],
                previous_code=previous_code,
                error=previous_error
            )
        else:
            # Kovetkezo technika
            tech_idx = min(round_num, len(techniques) - 1)
            tech_name, prompt_fn = techniques[tech_idx]
            prompt = prompt_fn(task)

        stats.techniques_used[tech_name] = stats.techniques_used.get(tech_name, 0) + 1

        # Model hivas
        response = ask_model(prompt)
        code = extract_code(response)

        if not code or len(code) < 5:
            previous_error = "Empty response"
            continue

        # Teszt
        passed, error = test_code(code, task["test"])

        if passed:
            stats.passed_tasks += 1
            if round_num == 0:
                stats.first_try_passes += 1

            # Level stats
            lvl = task["level"]
            if lvl not in stats.by_level:
                stats.by_level[lvl] = {"total": 0, "passed": 0}
            stats.by_level[lvl]["total"] += 1
            stats.by_level[lvl]["passed"] += 1

            return True, round_num + 1, tech_name, code

        previous_code = code
        previous_error = error

    # Level stats ha nem sikerult
    lvl = task["level"]
    if lvl not in stats.by_level:
        stats.by_level[lvl] = {"total": 0, "passed": 0}
    stats.by_level[lvl]["total"] += 1

    return False, MAX_ROUNDS, "FAILED", previous_code

def run_training():
    """Futtatas"""
    print("""
+===================================================================+
|                    TINY MASTER TRAINING                           |
|                                                                   |
|   Cel: Maximum kihozatal egy 637MB-os modellbol!                  |
|                                                                   |
|   Technikok:                                                      |
|   - Few-Shot Learning (peldak)                                    |
|   - Chain of Thought (lepesenként)                                |
|   - Self-Correction (hiba javitas)                                |
|   - Progressive Hints (fokozatos segitseg)                        |
|                                                                   |
|   Mate Robert + Claude - 2026.01.01.                              |
+===================================================================+
    """)

    # Watchdog setup
    genome = SealedGenome(ETHICAL_RULES)
    genome.seal()
    watchdog = Watchdog(ETHICAL_RULES, genome.genome_hash())

    print(f"Model: {OLLAMA_MODEL} (637MB)")
    print(f"Tasks: {len(TASKS)}")
    print(f"Max rounds per task: {MAX_ROUNDS}")
    print()

    stats = TrainingStats()
    start_time = time.time()

    current_level = 0
    for task in TASKS:
        if task["level"] != current_level:
            current_level = task["level"]
            print(f"\n{'='*60}")
            print(f"LEVEL {current_level}")
            print(f"{'='*60}")

        print(f"\n  [{task['name']}] ", end="")

        success, attempts, technique, code = train_task(task, stats)

        if success:
            if attempts == 1:
                print(f"PASS (1st try, {technique})")
            else:
                print(f"PASS (attempt {attempts}, {technique})")
        else:
            print(f"FAIL after {attempts} attempts")

    elapsed = time.time() - start_time

    # Eredmenyek
    print("\n")
    print("="*60)
    print("RESULTS")
    print("="*60)

    pass_rate = stats.passed_tasks / stats.total_tasks * 100 if stats.total_tasks > 0 else 0
    first_try_rate = stats.first_try_passes / stats.total_tasks * 100 if stats.total_tasks > 0 else 0
    efficiency = stats.passed_tasks / stats.total_attempts * 100 if stats.total_attempts > 0 else 0

    print(f"""
  Tasks Passed: {stats.passed_tasks}/{stats.total_tasks} ({pass_rate:.1f}%)
  First-Try Passes: {stats.first_try_passes}/{stats.total_tasks} ({first_try_rate:.1f}%)
  Total Attempts: {stats.total_attempts}
  Efficiency: {efficiency:.1f}%
  Time: {elapsed:.1f}s
    """)

    print("  By Level:")
    for lvl in sorted(stats.by_level.keys()):
        data = stats.by_level[lvl]
        pct = data["passed"] / data["total"] * 100 if data["total"] > 0 else 0
        bar = "#" * int(pct / 10) + "." * (10 - int(pct / 10))
        print(f"    Level {lvl}: [{bar}] {data['passed']}/{data['total']} ({pct:.0f}%)")

    print("\n  Techniques Used:")
    for tech, count in sorted(stats.techniques_used.items(), key=lambda x: -x[1]):
        print(f"    {tech}: {count}x")

    # Final grade
    print("\n" + "="*60)
    if pass_rate >= 90:
        grade = "A+ TINY MASTER!"
        msg = "INCREDIBLE! A 637MB model achieved MASTER level!"
    elif pass_rate >= 80:
        grade = "A EXCELLENT"
        msg = "Outstanding performance for such a small model!"
    elif pass_rate >= 70:
        grade = "B GOOD"
        msg = "Solid performance, room for improvement."
    elif pass_rate >= 60:
        grade = "C ACCEPTABLE"
        msg = "Passing grade, needs more training."
    else:
        grade = "D NEEDS WORK"
        msg = "More prompt engineering needed."

    print(f"""
  +-----------------------------------------------+
  |                                               |
  |   FINAL GRADE: {grade:^25}   |
  |                                               |
  |   {msg:^43}   |
  |                                               |
  +-----------------------------------------------+
    """)

    return stats

# ============================================================================
# MAIN
# ============================================================================

if __name__ == "__main__":
    run_training()
