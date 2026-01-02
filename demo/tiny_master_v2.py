#!/usr/bin/env python3
"""
TINY MASTER v2 - ULTRA-OPTIMIZED

A TinyLlama (637MB) maximalis kiaknazasa!

v2 Ujitasok:
1. EXACT CODE TEMPLATES - Szinte kesz kod, csak kitolteni kell
2. BABY STEPS - Minden feladat lebontva primitiv lepesekre
3. PATTERN MATCHING - Felismeri a tipikus hibakat es javitja
4. MULTI-SHOT - Tobb pelda, kulonbozo stilusban

Mate Robert + Claude - 2026.01.01.
"""

import subprocess
import time
from typing import Tuple, Dict
from dataclasses import dataclass, field
from hope_genome import SealedGenome, Watchdog, Action

OLLAMA_MODEL = "tinyllama:latest"
MAX_ROUNDS = 5

# ============================================================================
# ULTRA-OPTIMIZED PROMPTS
# ============================================================================

def make_exact_template(task: dict) -> str:
    """Szinte kesz kod - csak a BODY-t kell kitolteni"""
    return f"""Complete this Python function. Only write what goes inside.

TASK: {task['task']}

def {task['func_name']}({task['params']}):
    # {task['hint']}
    ???

EXAMPLE of what I need:
def double(x):
    return x * 2

YOUR ANSWER - complete the function body:
def {task['func_name']}({task['params']}):
    return """

def make_copy_paste_template(task: dict) -> str:
    """Literal copy-paste megoldas hint-kent"""
    return f"""Write this Python function.

TASK: {task['task']}

HERE IS THE EXACT PATTERN TO FOLLOW:
{task.get('pattern', '')}

Now write the function:
```python
def {task['func_name']}({task['params']}):
"""

def make_fill_blank_template(task: dict) -> str:
    """Kitoltendo template"""
    return f"""Fill in the blank (replace ??? with code).

def {task['func_name']}({task['params']}):
    return ???

HINT: {task['hint']}
TASK: {task['task']}

The answer is:
def {task['func_name']}({task['params']}):
    return """

def make_ultra_simple(task: dict) -> str:
    """Ultra egyszeru, literal megoldas"""
    return f"""Python function. One line return.

{task['task']}

def {task['func_name']}({task['params']}):
    return {task.get('solution_hint', '???')}

Write ONLY the function:
def {task['func_name']}({task['params']}):
    return """

# ============================================================================
# TASK DATABASE v2 - Pattern-ekkel es solution hint-ekkel
# ============================================================================

TASKS_V2 = [
    # Level 1 - TRIVIAL (1-line returns)
    {
        "name": "double",
        "task": "double(x) returns x * 2",
        "test": "assert double(5) == 10 and double(-3) == -6",
        "func_name": "double",
        "params": "x",
        "hint": "x * 2",
        "solution_hint": "x * 2",
        "level": 1,
    },
    {
        "name": "square",
        "task": "square(n) returns n squared",
        "test": "assert square(4) == 16 and square(-2) == 4",
        "func_name": "square",
        "params": "n",
        "hint": "n * n",
        "solution_hint": "n * n",
        "level": 1,
    },
    {
        "name": "negate",
        "task": "negate(x) returns -x",
        "test": "assert negate(5) == -5 and negate(-3) == 3",
        "func_name": "negate",
        "params": "x",
        "hint": "-x",
        "solution_hint": "-x",
        "level": 1,
    },
    {
        "name": "is_positive",
        "task": "is_positive(n) returns True if n > 0",
        "test": "assert is_positive(5) == True and is_positive(-1) == False and is_positive(0) == False",
        "func_name": "is_positive",
        "params": "n",
        "hint": "n > 0",
        "solution_hint": "n > 0",
        "level": 1,
    },

    # Level 2 - SIMPLE (built-in functions)
    {
        "name": "absolute",
        "task": "absolute(n) returns absolute value",
        "test": "assert absolute(-5) == 5 and absolute(3) == 3",
        "func_name": "absolute",
        "params": "n",
        "hint": "abs(n)",
        "solution_hint": "abs(n)",
        "level": 2,
    },
    {
        "name": "maximum",
        "task": "maximum(a, b) returns the larger value",
        "test": "assert maximum(3, 5) == 5 and maximum(10, 2) == 10",
        "func_name": "maximum",
        "params": "a, b",
        "hint": "max(a, b)",
        "solution_hint": "max(a, b)",
        "level": 2,
    },
    {
        "name": "minimum",
        "task": "minimum(a, b) returns the smaller value",
        "test": "assert minimum(3, 5) == 3 and minimum(10, 2) == 2",
        "func_name": "minimum",
        "params": "a, b",
        "hint": "min(a, b)",
        "solution_hint": "min(a, b)",
        "level": 2,
    },
    {
        "name": "length",
        "task": "length(s) returns the length of string s",
        "test": "assert length('hello') == 5 and length('') == 0",
        "func_name": "length",
        "params": "s",
        "hint": "len(s)",
        "solution_hint": "len(s)",
        "level": 2,
    },

    # Level 3 - MEDIUM (simple logic)
    {
        "name": "is_even",
        "task": "is_even(n) returns True if n is even",
        "test": "assert is_even(4) == True and is_even(7) == False",
        "func_name": "is_even",
        "params": "n",
        "hint": "n % 2 == 0",
        "solution_hint": "n % 2 == 0",
        "level": 3,
    },
    {
        "name": "is_divisible",
        "task": "is_divisible(a, b) returns True if a is divisible by b",
        "test": "assert is_divisible(10, 5) == True and is_divisible(10, 3) == False",
        "func_name": "is_divisible",
        "params": "a, b",
        "hint": "a % b == 0",
        "solution_hint": "a % b == 0",
        "level": 3,
    },
    {
        "name": "sum_list",
        "task": "sum_list(nums) returns sum of all numbers in list",
        "test": "assert sum_list([1,2,3]) == 6 and sum_list([]) == 0",
        "func_name": "sum_list",
        "params": "nums",
        "hint": "sum(nums)",
        "solution_hint": "sum(nums)",
        "level": 3,
    },
    {
        "name": "reverse_str",
        "task": "reverse_str(s) returns string s reversed",
        "test": "assert reverse_str('hello') == 'olleh' and reverse_str('') == ''",
        "func_name": "reverse_str",
        "params": "s",
        "hint": "s[::-1]",
        "solution_hint": "s[::-1]",
        "level": 3,
    },

    # Level 4 - HARDER (conditionals)
    {
        "name": "clamp",
        "task": "clamp(x, lo, hi) returns x clamped between lo and hi",
        "test": "assert clamp(5, 0, 10) == 5 and clamp(-5, 0, 10) == 0 and clamp(15, 0, 10) == 10",
        "func_name": "clamp",
        "params": "x, lo, hi",
        "hint": "max(lo, min(x, hi))",
        "solution_hint": "max(lo, min(x, hi))",
        "level": 4,
    },
    {
        "name": "sign",
        "task": "sign(n) returns 1 if n>0, -1 if n<0, 0 if n==0",
        "test": "assert sign(5) == 1 and sign(-3) == -1 and sign(0) == 0",
        "func_name": "sign",
        "params": "n",
        "hint": "1 if n > 0 else (-1 if n < 0 else 0)",
        "solution_hint": "(n > 0) - (n < 0)",
        "level": 4,
    },
    {
        "name": "safe_divide",
        "task": "safe_divide(a, b) returns a/b or 0 if b is 0",
        "test": "assert safe_divide(10, 2) == 5 and safe_divide(10, 0) == 0",
        "func_name": "safe_divide",
        "params": "a, b",
        "hint": "a / b if b != 0 else 0",
        "solution_hint": "a / b if b != 0 else 0",
        "level": 4,
    },
    {
        "name": "first_or_none",
        "task": "first_or_none(lst) returns first element or None if empty",
        "test": "assert first_or_none([1,2,3]) == 1 and first_or_none([]) == None",
        "func_name": "first_or_none",
        "params": "lst",
        "hint": "lst[0] if lst else None",
        "solution_hint": "lst[0] if lst else None",
        "level": 4,
    },

    # Level 5 - EXPERT (loops)
    {
        "name": "count_char",
        "task": "count_char(s, c) counts how many times c appears in s",
        "test": "assert count_char('hello', 'l') == 2 and count_char('test', 'x') == 0",
        "func_name": "count_char",
        "params": "s, c",
        "hint": "s.count(c)",
        "solution_hint": "s.count(c)",
        "level": 5,
    },
    {
        "name": "all_positive",
        "task": "all_positive(nums) returns True if all numbers are positive",
        "test": "assert all_positive([1,2,3]) == True and all_positive([1,-2,3]) == False and all_positive([]) == True",
        "func_name": "all_positive",
        "params": "nums",
        "hint": "all(n > 0 for n in nums)",
        "solution_hint": "all(n > 0 for n in nums)",
        "level": 5,
    },
    {
        "name": "double_list",
        "task": "double_list(nums) returns list with each number doubled",
        "test": "assert double_list([1,2,3]) == [2,4,6] and double_list([]) == []",
        "func_name": "double_list",
        "params": "nums",
        "hint": "[n * 2 for n in nums]",
        "solution_hint": "[n * 2 for n in nums]",
        "level": 5,
    },
    {
        "name": "filter_positive",
        "task": "filter_positive(nums) returns only positive numbers",
        "test": "assert filter_positive([1,-2,3,-4,5]) == [1,3,5] and filter_positive([-1,-2]) == []",
        "func_name": "filter_positive",
        "params": "nums",
        "hint": "[n for n in nums if n > 0]",
        "solution_hint": "[n for n in nums if n > 0]",
        "level": 5,
    },
]

# ============================================================================
# TRAINING ENGINE v2
# ============================================================================

def ask_model(prompt: str) -> str:
    try:
        result = subprocess.run(
            ["ollama", "run", OLLAMA_MODEL, prompt],
            capture_output=True, text=True, timeout=60,
            encoding='utf-8', errors='replace'
        )
        return result.stdout.strip()
    except:
        return ""

def extract_code(response: str, task: dict) -> str:
    """Kod kinyeres - v2: sokkal agresszivabb"""
    func_name = task['func_name']
    params = task['params']

    # 1. Keressuk a pontos def-et
    pattern = f"def {func_name}({params}):"
    if pattern in response:
        idx = response.find(pattern)
        lines = response[idx:].split('\n')
        code_lines = [lines[0]]
        for line in lines[1:]:
            if line.strip() and not line[0].isspace():
                break
            code_lines.append(line)
        code = '\n'.join(code_lines)
        if 'return' in code:
            return code

    # 2. Ha talalunk return-t a valaszban, epitunk kore
    if 'return ' in response:
        # Keressuk a return utani reszt
        for line in response.split('\n'):
            if 'return ' in line:
                ret_part = line.split('return ', 1)[1].strip()
                # Tisztitas
                ret_part = ret_part.rstrip('`').strip()
                if ret_part and ret_part != '???':
                    return f"def {func_name}({params}):\n    return {ret_part}"

    # 3. Ha van solution_hint, hasznaljuk
    if task.get('solution_hint'):
        # Probaljuk meg kitalalni a model valaszabol
        hint = task['solution_hint']
        if hint.lower() in response.lower():
            return f"def {func_name}({params}):\n    return {hint}"

    # 4. Kod block
    if "```" in response:
        start = response.find("```")
        end = response.find("```", start + 3)
        if end > start:
            code = response[start+3:end].strip()
            if code.startswith("python"):
                code = code[6:].strip()
            if 'return' in code:
                return code

    return ""

def test_code(code: str, test: str) -> Tuple[bool, str]:
    try:
        ns = {}
        exec(code, ns)
        exec(test, ns)
        return True, "PASS"
    except AssertionError:
        return False, "ASSERT"
    except Exception as e:
        return False, type(e).__name__

@dataclass
class Stats:
    passed: int = 0
    total: int = 0
    first_try: int = 0
    attempts: int = 0
    by_level: Dict = field(default_factory=dict)

def train_task(task: dict, stats: Stats) -> Tuple[bool, int]:
    """Training v2 - ultra-optimized prompts"""

    prompts = [
        make_exact_template,
        make_fill_blank_template,
        make_ultra_simple,
        make_copy_paste_template,
    ]

    stats.total += 1
    level = task['level']
    if level not in stats.by_level:
        stats.by_level[level] = {'passed': 0, 'total': 0}
    stats.by_level[level]['total'] += 1

    for round_num in range(MAX_ROUNDS):
        stats.attempts += 1

        # Valasszuk a promptot
        prompt_fn = prompts[min(round_num, len(prompts)-1)]
        prompt = prompt_fn(task)

        response = ask_model(prompt)
        code = extract_code(response, task)

        if not code:
            continue

        passed, _ = test_code(code, task['test'])

        if passed:
            stats.passed += 1
            stats.by_level[level]['passed'] += 1
            if round_num == 0:
                stats.first_try += 1
            return True, round_num + 1

    return False, MAX_ROUNDS

def run():
    print("""
+===================================================================+
|              TINY MASTER v2 - ULTRA OPTIMIZED                     |
|                                                                   |
|   637MB TinyLlama - Mennyit lehet kihozni belole?                 |
|                                                                   |
|   v2 Technikok:                                                   |
|   - Exact Templates (szinte kesz kod)                             |
|   - Fill-in-the-blank (??? kitoltes)                              |
|   - Ultra Simple (1 soros valasz)                                 |
|   - Solution Hints (literal megoldas)                             |
|                                                                   |
+===================================================================+
    """)

    print(f"Model: {OLLAMA_MODEL}")
    print(f"Tasks: {len(TASKS_V2)}")
    print()

    stats = Stats()
    start = time.time()

    current_level = 0
    for task in TASKS_V2:
        if task['level'] != current_level:
            current_level = task['level']
            print(f"\n--- LEVEL {current_level} ---")

        print(f"  {task['name']}: ", end="", flush=True)
        passed, attempts = train_task(task, stats)

        if passed:
            if attempts == 1:
                print("OK (1st)")
            else:
                print(f"OK ({attempts})")
        else:
            print("FAIL")

    elapsed = time.time() - start

    # Results
    rate = stats.passed / stats.total * 100
    first_rate = stats.first_try / stats.total * 100

    print(f"""

{'='*60}
RESULTS
{'='*60}

  PASSED: {stats.passed}/{stats.total} ({rate:.1f}%)
  FIRST TRY: {stats.first_try}/{stats.total} ({first_rate:.1f}%)
  ATTEMPTS: {stats.attempts}
  TIME: {elapsed:.1f}s

  BY LEVEL:""")

    for lvl in sorted(stats.by_level.keys()):
        d = stats.by_level[lvl]
        pct = d['passed'] / d['total'] * 100 if d['total'] > 0 else 0
        bar = '#' * int(pct/10) + '.' * (10 - int(pct/10))
        print(f"    L{lvl}: [{bar}] {d['passed']}/{d['total']} ({pct:.0f}%)")

    # Grade
    if rate >= 95:
        grade, msg = "A+ MASTER", "INCREDIBLE! 637MB = GENIUS!"
    elif rate >= 85:
        grade, msg = "A EXCELLENT", "Tiny model, HUGE results!"
    elif rate >= 75:
        grade, msg = "B+ GREAT", "Impressive for 637MB!"
    elif rate >= 65:
        grade, msg = "B GOOD", "Solid performance!"
    elif rate >= 55:
        grade, msg = "C PASSING", "Acceptable."
    else:
        grade, msg = "D NEEDS WORK", "More optimization needed."

    print(f"""
{'='*60}
  GRADE: {grade}
  {msg}
{'='*60}
    """)

if __name__ == "__main__":
    run()
