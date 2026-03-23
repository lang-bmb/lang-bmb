# AI-Native Proof Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a reproducible experiment framework that proves BMB's contract system reduces AI code generation feedback loops, with C-equivalent performance.

**Architecture:** Python orchestrator calls Claude API to generate code in BMB/Rust/Python, builds each language via subprocess, runs tests, counts loops, measures performance. Two experiments: H1 (BMB+contract vs BMB-contract, same language) and H2 (BMB vs Rust vs Python, cross-language). Results stored as JSON, analyzed with statistical tests.

**Tech Stack:** Python 3.10+, Anthropic SDK (claude API), subprocess (compilers), json, scipy (statistics), matplotlib (charts)

**Spec:** `docs/superpowers/specs/2026-03-24-ai-native-proof-design.md`

---

## File Structure

```
ecosystem/ai-proof/
├── protocol/
│   ├── PROTOCOL.md              # Frozen experiment design
│   ├── bmb_reference.md         # Frozen BMB language reference excerpt
│   └── prompt_templates.py      # Prompt generation (Python module)
├── problems/
│   ├── __init__.py
│   ├── registry.py              # Problem registry + loader
│   ├── 01_binary_search/
│   │   ├── problem.md
│   │   ├── tests.json
│   │   ├── baseline.c
│   │   ├── solution.bmb         # Pre-verified BMB solution
│   │   └── solution.rs          # Pre-verified Rust solution
│   └── ... (30 problems)
├── runners/
│   ├── __init__.py
│   ├── base.py                  # Abstract runner interface
│   ├── bmb_runner.py            # BMB build + test + perf
│   ├── rust_runner.py           # Rust build + test + perf
│   ├── python_runner.py         # Python test (no perf)
│   └── perf.py                  # Performance measurement
├── orchestrator/
│   ├── __init__.py
│   ├── experiment.py            # Main experiment loop
│   ├── llm_client.py            # Claude API wrapper
│   ├── error_normalizer.py      # Error message normalization
│   └── loop_classifier.py       # Loop type classification (A/B/C/D)
├── analysis/
│   ├── __init__.py
│   ├── stats.py                 # Statistical tests (Wilcoxon, Friedman)
│   ├── visualize.py             # Charts + tables
│   └── report_generator.py      # Markdown report generation
├── results/                     # Generated (not committed until experiment)
│   ├── raw/
│   └── summary.json
├── report/
│   └── AI_NATIVE_PROOF.md       # Generated report
├── scripts/
│   ├── run_experiment.py        # CLI entry point
│   ├── validate_problems.py     # Pre-verify all solutions compile
│   └── strip_contracts.py       # Remove pre/post from BMB source (for H1)
├── requirements.txt
├── conftest.py                  # pytest fixtures
├── tests/
│   ├── test_runners.py
│   ├── test_orchestrator.py
│   ├── test_error_normalizer.py
│   └── test_loop_classifier.py
└── README.md
```

---

## Task 1: Project Scaffold + Dependencies

**Files:**
- Create: `ecosystem/ai-proof/requirements.txt`
- Create: `ecosystem/ai-proof/README.md`
- Create: `ecosystem/ai-proof/__init__.py` (empty)
- Create: `ecosystem/ai-proof/conftest.py`
- Create: `ecosystem/ai-proof/pyproject.toml`

- [ ] **Step 1: Create directory structure**

```bash
cd D:/data/lang-bmb
mkdir -p ecosystem/ai-proof/{protocol,problems,runners,orchestrator,analysis,results/raw,report,scripts,tests}
```

- [ ] **Step 2: Write requirements.txt**

```
anthropic>=0.40.0
scipy>=1.11.0
matplotlib>=3.8.0
pytest>=8.0.0
```

- [ ] **Step 3: Write pyproject.toml (package config for imports)**

```toml
[project]
name = "ai-proof"
version = "0.1.0"

[tool.pytest.ini_options]
testpaths = ["tests"]
pythonpath = ["."]
```

- [ ] **Step 4: Write conftest.py**

```python
# conftest.py
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
```

- [ ] **Step 5: Write README.md**

README with: project purpose, quick start (`pip install -r requirements.txt && python scripts/run_experiment.py`), directory structure, link to spec.

- [ ] **Step 6: Create __init__.py files**

Empty `__init__.py` in: `ecosystem/ai-proof/`, `problems/`, `runners/`, `orchestrator/`, `analysis/`, `tests/`, `scripts/`.

- [ ] **Step 7: Install dependencies**

```bash
cd ecosystem/ai-proof && pip install -r requirements.txt
```

- [ ] **Step 8: Commit**

```bash
git add ecosystem/ai-proof/
git commit -m "feat(ai-proof): scaffold project structure and dependencies"
```

---

## Task 2: Language Runners (BMB, Rust, Python)

**Files:**
- Create: `ecosystem/ai-proof/runners/base.py`
- Create: `ecosystem/ai-proof/runners/bmb_runner.py`
- Create: `ecosystem/ai-proof/runners/rust_runner.py`
- Create: `ecosystem/ai-proof/runners/python_runner.py`
- Create: `ecosystem/ai-proof/runners/perf.py`
- Create: `ecosystem/ai-proof/tests/test_runners.py`

### Step-by-step

- [ ] **Step 1: Write base runner interface test**

```python
# tests/test_runners.py
import pytest
from runners.base import RunResult, RunnerBase

def test_run_result_fields():
    r = RunResult(compiled=True, test_passed=True, error_msg="", perf_ns=1000)
    assert r.compiled is True
    assert r.test_passed is True
    assert r.perf_ns == 1000

def test_runner_base_is_abstract():
    with pytest.raises(TypeError):
        RunnerBase()
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd ecosystem/ai-proof && python -m pytest tests/test_runners.py -v
```
Expected: FAIL (no module `runners.base`)

- [ ] **Step 3: Implement base.py**

```python
# runners/base.py
from dataclasses import dataclass
from abc import ABC, abstractmethod
from pathlib import Path

@dataclass
class RunResult:
    compiled: bool
    test_passed: bool
    error_msg: str
    perf_ns: int | None = None  # None if not measured or build failed
    raw_output: str = ""

class RunnerBase(ABC):
    @abstractmethod
    def build(self, source_code: str, work_dir: Path) -> RunResult:
        """Compile source code. Return result with compiled=True/False."""
        ...

    @abstractmethod
    def test(self, work_dir: Path, tests: list[dict]) -> RunResult:
        """Run test cases against built binary. Return result."""
        ...

    @abstractmethod
    def measure_perf(self, work_dir: Path, iterations: int = 10) -> int:
        """Return median execution time in nanoseconds."""
        ...
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cd ecosystem/ai-proof && python -m pytest tests/test_runners.py -v
```
Expected: PASS

- [ ] **Step 5: Write BMB runner test**

```python
# tests/test_runners.py (append)
from runners.bmb_runner import BmbRunner

def test_bmb_runner_build_valid(tmp_path):
    runner = BmbRunner(compiler_path="D:/data/lang-bmb/target/release/bmb.exe")
    source = 'fn main() -> i64 = 42;'
    result = runner.build(source, tmp_path)
    assert result.compiled is True

def test_bmb_runner_build_invalid(tmp_path):
    runner = BmbRunner(compiler_path="D:/data/lang-bmb/target/release/bmb.exe")
    source = 'fn main( -> i64 = 42;'  # syntax error
    result = runner.build(source, tmp_path)
    assert result.compiled is False
    assert len(result.error_msg) > 0
```

- [ ] **Step 6: Implement bmb_runner.py**

```python
# runners/bmb_runner.py
import subprocess
import time
from pathlib import Path
from .base import RunnerBase, RunResult

class BmbRunner(RunnerBase):
    def __init__(self, compiler_path: str):
        self.compiler = compiler_path

    def build(self, source_code: str, work_dir: Path) -> RunResult:
        src = work_dir / "solution.bmb"
        src.write_text(source_code, encoding="utf-8")
        out = work_dir / "solution"
        try:
            proc = subprocess.run(
                [self.compiler, "build", str(src), "-o", str(out), "--release"],
                capture_output=True, text=True, timeout=60, cwd=str(work_dir)
            )
            if proc.returncode != 0:
                return RunResult(compiled=False, test_passed=False,
                                 error_msg=proc.stderr, raw_output=proc.stdout + proc.stderr)
            return RunResult(compiled=True, test_passed=False, error_msg="",
                             raw_output=proc.stdout)
        except subprocess.TimeoutExpired:
            return RunResult(compiled=False, test_passed=False,
                             error_msg="Build timed out after 60s")

    def test(self, work_dir: Path, tests: list[dict]) -> RunResult:
        binary = work_dir / "solution"
        if not binary.exists() and not (work_dir / "solution.exe").exists():
            return RunResult(compiled=True, test_passed=False,
                             error_msg="Binary not found after build")
        bin_path = binary if binary.exists() else work_dir / "solution.exe"
        failures = []
        for i, tc in enumerate(tests):
            try:
                proc = subprocess.run(
                    [str(bin_path)] + [str(a) for a in tc.get("args", [])],
                    input=tc.get("stdin", ""),
                    capture_output=True, text=True, timeout=10
                )
                actual = proc.stdout.strip()
                expected = str(tc["expected"]).strip()
                if actual != expected:
                    failures.append(f"Test {i}: expected '{expected}', got '{actual}'")
            except subprocess.TimeoutExpired:
                failures.append(f"Test {i}: timed out")
        if failures:
            return RunResult(compiled=True, test_passed=False,
                             error_msg="\n".join(failures))
        return RunResult(compiled=True, test_passed=True, error_msg="")

    def measure_perf(self, work_dir: Path, iterations: int = 10) -> int:
        bin_path = work_dir / "solution.exe" if (work_dir / "solution.exe").exists() else work_dir / "solution"
        times = []
        for _ in range(iterations):
            start = time.perf_counter_ns()
            subprocess.run([str(bin_path)], capture_output=True, timeout=30)
            end = time.perf_counter_ns()
            times.append(end - start)
        times.sort()
        return times[len(times) // 2]  # median
```

- [ ] **Step 7: Run BMB runner tests**

```bash
cd ecosystem/ai-proof && python -m pytest tests/test_runners.py -v
```
Expected: PASS (requires `bmb.exe` to exist)

- [ ] **Step 8: Implement rust_runner.py and python_runner.py**

Similar structure: `RustRunner` uses `rustc --edition 2021 -O`, `PythonRunner` uses `python -c`.

- [ ] **Step 9: Implement perf.py**

```python
# runners/perf.py
import subprocess
import time
from pathlib import Path

def measure_binary(binary_path: Path, args: list[str] = None,
                   stdin: str = "", iterations: int = 10) -> dict:
    """Measure execution time. Return {median_ns, times_ns, ci_95}."""
    times = []
    for _ in range(iterations):
        start = time.perf_counter_ns()
        subprocess.run(
            [str(binary_path)] + (args or []),
            input=stdin, capture_output=True, timeout=30
        )
        end = time.perf_counter_ns()
        times.append(end - start)
    times.sort()
    n = len(times)
    # IQR outlier removal
    q1, q3 = times[n // 4], times[3 * n // 4]
    iqr = q3 - q1
    filtered = [t for t in times if q1 - 1.5 * iqr <= t <= q3 + 1.5 * iqr]
    median = filtered[len(filtered) // 2]
    return {"median_ns": median, "times_ns": filtered, "n": len(filtered)}
```

- [ ] **Step 10: Commit**

```bash
git add ecosystem/ai-proof/runners/ ecosystem/ai-proof/tests/test_runners.py
git commit -m "feat(ai-proof): language runners for BMB, Rust, Python with perf measurement"
```

---

## Task 3: Error Normalizer + Loop Classifier

**Files:**
- Create: `ecosystem/ai-proof/orchestrator/error_normalizer.py`
- Create: `ecosystem/ai-proof/orchestrator/loop_classifier.py`
- Create: `ecosystem/ai-proof/tests/test_error_normalizer.py`
- Create: `ecosystem/ai-proof/tests/test_loop_classifier.py`

- [ ] **Step 1: Write error normalizer test**

```python
# tests/test_error_normalizer.py
from orchestrator.error_normalizer import normalize_error

def test_normalize_bmb_contract_error():
    raw = "error[E001]: pre-condition violated: idx < len\n  --> solution.bmb:5:3"
    result = normalize_error(raw, lang="bmb")
    assert result["type"] == "compile_error"
    assert "pre-condition" in result["normalized"]
    assert result["location"] == "solution.bmb:5:3"
    assert result["raw"] == raw

def test_normalize_rust_borrow_error():
    raw = "error[E0505]: cannot move out of `x` because it is borrowed\n  --> solution.rs:10:5"
    result = normalize_error(raw, lang="rust")
    assert result["type"] == "compile_error"
    assert result["location"] == "solution.rs:10:5"

def test_normalize_test_failure():
    raw = "Test 3: expected '42', got '0'"
    result = normalize_error(raw, lang="bmb", is_test_failure=True)
    assert result["type"] == "test_failure"
```

- [ ] **Step 2: Run test to verify fail**

```bash
cd ecosystem/ai-proof && python -m pytest tests/test_error_normalizer.py -v
```
Expected: FAIL (no module)

- [ ] **Step 3: Implement error_normalizer.py**

```python
# orchestrator/error_normalizer.py
import re

# Patterns for extracting location from compiler output
LOCATION_PATTERNS = {
    "bmb": r"-->\s*(\S+:\d+:\d+)",
    "rust": r"-->\s*(\S+:\d+:\d+)",
    "python": r'File "([^"]+)", line (\d+)',
}

def normalize_error(raw: str, lang: str = "bmb", is_test_failure: bool = False) -> dict:
    """Normalize compiler/test error into standard format."""
    error_type = "test_failure" if is_test_failure else "compile_error"

    # Extract location
    location = ""
    pattern = LOCATION_PATTERNS.get(lang, "")
    if pattern:
        match = re.search(pattern, raw)
        if match:
            location = match.group(1) if lang != "python" else f"{match.group(1)}:{match.group(2)}"

    # Normalize message: first meaningful line
    lines = [l.strip() for l in raw.strip().split("\n") if l.strip()]
    normalized = lines[0] if lines else raw.strip()

    # For Rust, extract the main error message after "error[EXXXX]: "
    if lang == "rust":
        m = re.search(r"error\[E\d+\]:\s*(.+)", raw)
        if m:
            normalized = m.group(1).strip()

    # For BMB, extract after "error[EXXXX]: " or "error: "
    if lang == "bmb":
        m = re.search(r"error(?:\[E\d+\])?:\s*(.+)", raw)
        if m:
            normalized = m.group(1).strip()

    return {
        "type": error_type,
        "normalized": normalized,
        "location": location,
        "raw": raw,
    }
```

- [ ] **Step 4: Run test to verify pass**

```bash
cd ecosystem/ai-proof && python -m pytest tests/test_error_normalizer.py -v
```
Expected: PASS

- [ ] **Step 5: Write loop classifier test**

```python
# tests/test_loop_classifier.py
from orchestrator.loop_classifier import classify_loop, LoopType

def test_classify_contract_violation():
    error = {"type": "compile_error", "normalized": "pre-condition violated: idx < len", "raw": "..."}
    assert classify_loop(error, lang="bmb") == LoopType.A_CONTRACT

def test_classify_syntax_error():
    error = {"type": "compile_error", "normalized": "unexpected token '}'", "raw": "..."}
    assert classify_loop(error, lang="bmb") == LoopType.B_SYNTAX

def test_classify_semantic_error():
    error = {"type": "compile_error", "normalized": "type mismatch: expected i64, found bool", "raw": "..."}
    assert classify_loop(error, lang="bmb") == LoopType.C_SEMANTIC

def test_classify_test_failure():
    error = {"type": "test_failure", "normalized": "expected '42', got '0'", "raw": "..."}
    assert classify_loop(error, lang="bmb") == LoopType.D_TEST
```

- [ ] **Step 4: Implement loop_classifier.py**

```python
# orchestrator/loop_classifier.py
from enum import Enum

class LoopType(str, Enum):
    A_CONTRACT = "A"  # Contract/type violation caught at compile time
    B_SYNTAX = "B"    # Parser-level syntax error
    C_SEMANTIC = "C"  # Semantic compile error (type mismatch, undefined)
    D_TEST = "D"      # Test failure (runtime correctness)

# Patterns per language for classification
CONTRACT_PATTERNS = ["pre-condition", "post-condition", "contract", "invariant"]
SYNTAX_PATTERNS = ["unexpected token", "expected", "parse error", "syntax error", "unrecognized"]
# Everything else compile-time → C_SEMANTIC

def classify_loop(error: dict, lang: str) -> LoopType:
    if error["type"] == "test_failure":
        return LoopType.D_TEST
    msg = error["normalized"].lower()
    if any(p in msg for p in CONTRACT_PATTERNS):
        return LoopType.A_CONTRACT
    if any(p in msg for p in SYNTAX_PATTERNS):
        return LoopType.B_SYNTAX
    return LoopType.C_SEMANTIC
```

- [ ] **Step 5: Run tests, verify pass**

- [ ] **Step 6: Commit**

```bash
git add ecosystem/ai-proof/orchestrator/ ecosystem/ai-proof/tests/test_error_normalizer.py ecosystem/ai-proof/tests/test_loop_classifier.py
git commit -m "feat(ai-proof): error normalizer and loop classifier (A/B/C/D types)"
```

---

## Task 4: LLM Client + Prompt Templates

**Files:**
- Create: `ecosystem/ai-proof/orchestrator/llm_client.py`
- Create: `ecosystem/ai-proof/protocol/prompt_templates.py`
- Create: `ecosystem/ai-proof/tests/test_orchestrator.py`

- [ ] **Step 1: Write prompt template test**

```python
# tests/test_orchestrator.py
from protocol.prompt_templates import build_initial_prompt, build_error_feedback_prompt

def test_initial_prompt_contains_problem():
    prompt = build_initial_prompt(
        problem_desc="Implement binary search",
        lang="bmb",
        test_preview="Input: [1,2,3], target=2 → Output: 1",
        reference=None
    )
    assert "binary search" in prompt.lower()
    assert "bmb" in prompt.lower()
    assert "Input:" in prompt

def test_initial_prompt_with_reference():
    prompt = build_initial_prompt(
        problem_desc="Sort an array",
        lang="bmb",
        test_preview="...",
        reference="fn foo() -> i64 = 42;"
    )
    assert "fn foo" in prompt

def test_error_feedback_prompt():
    prompt = build_error_feedback_prompt(
        error_type="compile_error",
        normalized_msg="pre-condition violated",
        location="solution.bmb:5:3",
        raw_output="full compiler output here"
    )
    assert "compile_error" in prompt.lower() or "빌드" in prompt
    assert "pre-condition" in prompt
```

- [ ] **Step 2: Implement prompt_templates.py**

```python
# protocol/prompt_templates.py

def build_initial_prompt(problem_desc: str, lang: str,
                         test_preview: str, reference: str | None = None) -> str:
    parts = [f"# Language: {lang}\n"]
    if reference:
        parts.append(f"## {lang} Language Reference\n\n{reference}\n\n---\n")
    parts.append(f"{problem_desc}\n")
    parts.append(f"## Test Cases\n{test_preview}\n")
    parts.append(f"\nWrite a complete, compilable {lang} program that satisfies the above requirements.")
    return "\n".join(parts)

def build_error_feedback_prompt(error_type: str, normalized_msg: str,
                                 location: str, raw_output: str) -> str:
    return f"""Build/test failed.

[Error type]: {error_type}
[Message]: {normalized_msg}
[Location]: {location}

Raw compiler output:
{raw_output}

Fix the error and provide the complete corrected code."""
```

- [ ] **Step 3: Implement llm_client.py**

```python
# orchestrator/llm_client.py
import anthropic
import re

class LlmClient:
    def __init__(self, model: str = "claude-opus-4-6", temperature: float = 0.0):
        self.client = anthropic.Anthropic()
        self.model = model
        self.temperature = temperature

    def generate(self, system: str, messages: list[dict]) -> str:
        """Send messages to Claude, return text response."""
        resp = self.client.messages.create(
            model=self.model,
            max_tokens=8192,
            temperature=self.temperature,
            system=system,
            messages=messages
        )
        return resp.content[0].text

    @staticmethod
    def extract_code(response: str, lang: str) -> str:
        """Extract code block from LLM response."""
        pattern = rf"```(?:{lang}|)\n(.*?)```"
        match = re.search(pattern, response, re.DOTALL)
        if match:
            return match.group(1).strip()
        # Fallback: entire response is code
        return response.strip()
```

- [ ] **Step 4: Run tests, verify pass**

- [ ] **Step 5: Commit**

```bash
git add ecosystem/ai-proof/orchestrator/llm_client.py ecosystem/ai-proof/protocol/prompt_templates.py ecosystem/ai-proof/tests/test_orchestrator.py
git commit -m "feat(ai-proof): LLM client and prompt templates"
```

---

## Task 5: Contract Stripper (for H1)

**Files:**
- Create: `ecosystem/ai-proof/scripts/strip_contracts.py`

- [ ] **Step 1: Write test**

```python
# tests/test_strip_contracts.py
from scripts.strip_contracts import strip_contracts

def test_strip_pre():
    source = """fn safe_get(arr: &[i64; 10], idx: i64) -> i64
    pre idx >= 0 and idx < 10
= arr[idx];"""
    result = strip_contracts(source)
    assert "pre" not in result
    assert "fn safe_get" in result
    assert "arr[idx]" in result

def test_strip_post():
    source = """fn sort(arr: &mut [i64]) -> &[i64]
    post is_sorted(ret)
{ /* body */ }"""
    result = strip_contracts(source)
    assert "post" not in result
    assert "fn sort" in result

def test_strip_preserves_non_contract():
    source = "fn add(a: i64, b: i64) -> i64 = a + b;"
    result = strip_contracts(source)
    assert result.strip() == source.strip()
```

- [ ] **Step 2: Implement strip_contracts.py**

```python
# scripts/strip_contracts.py
import re

def strip_contracts(source: str) -> str:
    """Remove pre/post contract clauses from BMB source code.

    Handles:
      fn name(...) -> Type
          pre condition
          post condition
      = body;
    """
    # Remove lines that are purely pre/post conditions
    # Pattern: optional whitespace + pre/post + condition (until next line starting with = { or another keyword)
    lines = source.split("\n")
    result = []
    i = 0
    while i < len(lines):
        stripped = lines[i].strip()
        if stripped.startswith("pre ") or stripped.startswith("post "):
            # Skip this contract line
            # Also handle multi-line contracts (indented continuation)
            i += 1
            while i < len(lines) and lines[i].strip().startswith("and "):
                i += 1
            continue
        result.append(lines[i])
        i += 1
    return "\n".join(result)
```

- [ ] **Step 3: Run tests, verify pass**

- [ ] **Step 4: Commit**

```bash
git add ecosystem/ai-proof/scripts/strip_contracts.py ecosystem/ai-proof/tests/test_strip_contracts.py
git commit -m "feat(ai-proof): contract stripper for H1 experiment (BMB+contract vs BMB-contract)"
```

---

## Task 6: Experiment Orchestrator

**Files:**
- Create: `ecosystem/ai-proof/orchestrator/experiment.py`
- Create: `ecosystem/ai-proof/problems/registry.py`

- [ ] **Step 1: Write problem registry**

```python
# problems/registry.py
import json
from pathlib import Path
from dataclasses import dataclass

@dataclass
class Problem:
    name: str
    number: int
    category: str  # "algorithm", "system", "contract"
    description: str
    tests: list[dict]
    baseline_c: str

def load_problem(problem_dir: Path) -> Problem:
    """Load a problem from its directory."""
    desc = (problem_dir / "problem.md").read_text(encoding="utf-8")
    tests = json.loads((problem_dir / "tests.json").read_text(encoding="utf-8"))
    baseline = (problem_dir / "baseline.c").read_text(encoding="utf-8") if (problem_dir / "baseline.c").exists() else ""
    # Extract number and name from dir name like "01_binary_search"
    parts = problem_dir.name.split("_", 1)
    number = int(parts[0])
    name = parts[1] if len(parts) > 1 else problem_dir.name
    # Category from problem number range (deterministic per spec)
    if number <= 10:
        category = "algorithm"
    elif number <= 20:
        category = "system"
    else:
        category = "contract"
    return Problem(name=name, number=number, category=category,
                   description=desc, tests=tests, baseline_c=baseline)

def load_all_problems(problems_dir: Path) -> list[Problem]:
    """Load all problems sorted by number."""
    dirs = sorted(d for d in problems_dir.iterdir() if d.is_dir() and d.name[0].isdigit())
    return [load_problem(d) for d in dirs]
```

- [ ] **Step 2: Write experiment orchestrator**

```python
# orchestrator/experiment.py
import json
import time
from pathlib import Path
from dataclasses import dataclass, field, asdict

from orchestrator.llm_client import LlmClient
from orchestrator.error_normalizer import normalize_error
from orchestrator.loop_classifier import classify_loop, LoopType
from runners.base import RunnerBase
from problems.registry import Problem
from scripts.strip_contracts import strip_contracts

MAX_LOOPS = 10

@dataclass
class AttemptRecord:
    attempt: int
    code: str
    compiled: bool
    test_passed: bool
    error: dict | None
    loop_type: str | None

@dataclass
class RunRecord:
    run_id: int
    condition: str  # "bmb_contract", "bmb_nocontract", "rust", "python"
    loop_count: int
    loop_types: dict  # {"A": 0, "B": 0, "C": 0, "D": 0}
    final_correct: bool
    perf_ratio: float | None
    attempts: list[AttemptRecord] = field(default_factory=list)

class ExperimentRunner:
    def __init__(self, llm: LlmClient, runners: dict[str, RunnerBase],
                 problems_dir: Path, results_dir: Path,
                 bmb_reference: str = ""):
        self.llm = llm
        self.runners = runners
        self.problems_dir = problems_dir
        self.results_dir = results_dir
        self.bmb_reference = bmb_reference

    def run_single(self, problem: Problem, condition: str,
                   run_id: int, work_dir: Path) -> RunRecord:
        """Run one problem in one condition for one trial."""
        lang = "bmb" if "bmb" in condition else condition
        runner = self.runners[lang if lang != "bmb" else "bmb"]
        reference = self.bmb_reference if lang == "bmb" else None

        # Build test preview
        test_preview = "\n".join(
            f"Input: {t.get('args', t.get('stdin', ''))} → Expected: {t['expected']}"
            for t in problem.tests[:5]
        )

        # Initial prompt
        from protocol.prompt_templates import build_initial_prompt, build_error_feedback_prompt
        prompt = build_initial_prompt(problem.description, lang, test_preview, reference)
        messages = [{"role": "user", "content": prompt}]

        loop_types = {"A": 0, "B": 0, "C": 0, "D": 0}
        attempts = []

        for attempt_num in range(1, MAX_LOOPS + 1):
            # Generate code
            response = self.llm.generate(
                system=f"You are a {lang} programmer. Write complete, compilable code.",
                messages=messages
            )
            code = LlmClient.extract_code(response, lang)

            # For H1 no-contract condition: strip contracts from generated code
            if condition == "bmb_nocontract":
                code = strip_contracts(code)

            # Build
            result = runner.build(code, work_dir)

            if not result.compiled:
                error = normalize_error(result.error_msg, lang=lang)
                lt = classify_loop(error, lang=lang)
                loop_types[lt.value] += 1
                attempts.append(AttemptRecord(
                    attempt=attempt_num, code=code, compiled=False,
                    test_passed=False, error=error, loop_type=lt.value
                ))
                # Feed error back
                feedback = build_error_feedback_prompt(
                    error["type"], error["normalized"],
                    error.get("location", ""), result.error_msg
                )
                messages.append({"role": "assistant", "content": response})
                messages.append({"role": "user", "content": feedback})
                continue

            # Test
            test_result = runner.test(work_dir, problem.tests)

            if not test_result.test_passed:
                error = normalize_error(test_result.error_msg, lang=lang, is_test_failure=True)
                lt = classify_loop(error, lang=lang)
                loop_types[lt.value] += 1
                attempts.append(AttemptRecord(
                    attempt=attempt_num, code=code, compiled=True,
                    test_passed=False, error=error, loop_type=lt.value
                ))
                feedback = build_error_feedback_prompt(
                    "test_failure", error["normalized"],
                    "", test_result.error_msg
                )
                messages.append({"role": "assistant", "content": response})
                messages.append({"role": "user", "content": feedback})
                continue

            # Success!
            perf = None
            if lang in ("bmb", "rust"):
                perf_ns = runner.measure_perf(work_dir)
                # Compute ratio against C baseline
                c_perf = self._measure_c_baseline(problem, work_dir)
                perf = perf_ns / c_perf if c_perf and c_perf > 0 else None

            attempts.append(AttemptRecord(
                attempt=attempt_num, code=code, compiled=True,
                test_passed=True, error=None, loop_type=None
            ))

            return RunRecord(
                run_id=run_id, condition=condition,
                loop_count=attempt_num, loop_types=loop_types,
                final_correct=True, perf_ratio=perf,
                attempts=attempts
            )

        # Max loops exceeded
        return RunRecord(
            run_id=run_id, condition=condition,
            loop_count=MAX_LOOPS + 1, loop_types=loop_types,
            final_correct=False, perf_ratio=None,
            attempts=attempts
        )

    def _measure_c_baseline(self, problem: Problem, work_dir: Path) -> int | None:
        """Build and measure C baseline for performance ratio."""
        if not problem.baseline_c:
            return None
        import subprocess
        c_src = work_dir / "baseline.c"
        c_bin = work_dir / "baseline"
        c_src.write_text(problem.baseline_c, encoding="utf-8")
        proc = subprocess.run(
            ["clang", "-O2", str(c_src), "-o", str(c_bin), "-lm"],
            capture_output=True, timeout=60
        )
        if proc.returncode != 0:
            return None
        from runners.perf import measure_binary
        result = measure_binary(c_bin)
        return result["median_ns"]

    def save_run(self, problem: Problem, record: RunRecord):
        """Save run results to disk."""
        prob_dir = self.results_dir / "raw" / f"{problem.number:02d}_{problem.name}"
        run_dir = prob_dir / f"run{record.run_id}"
        run_dir.mkdir(parents=True, exist_ok=True)
        (run_dir / f"{record.condition}_result.json").write_text(
            json.dumps(asdict(record), indent=2, ensure_ascii=False),
            encoding="utf-8"
        )
        # Save each attempt's code
        for att in record.attempts:
            ext = {"bmb_contract": "bmb", "bmb_nocontract": "bmb",
                   "rust": "rs", "python": "py"}.get(record.condition, "txt")
            (run_dir / f"{record.condition}_attempt_{att.attempt}.{ext}").write_text(
                att.code, encoding="utf-8"
            )
```

- [ ] **Step 3: Write orchestrator test**

```python
# tests/test_experiment.py
from unittest.mock import MagicMock, patch
from orchestrator.experiment import ExperimentRunner, MAX_LOOPS
from runners.base import RunResult
from problems.registry import Problem

def _make_problem():
    return Problem(name="test", number=0, category="algorithm",
                   description="Add two numbers", tests=[{"args": ["2", "3"], "expected": "5"}],
                   baseline_c="")

def test_first_attempt_success():
    """Loop count = 1 when first attempt compiles and passes tests."""
    llm = MagicMock()
    llm.generate.return_value = "```bmb\nfn main() -> i64 = 42;\n```"
    llm.extract_code = MagicMock(return_value="fn main() -> i64 = 42;")

    runner = MagicMock()
    runner.build.return_value = RunResult(compiled=True, test_passed=False, error_msg="")
    runner.test.return_value = RunResult(compiled=True, test_passed=True, error_msg="")

    exp = ExperimentRunner(llm, {"bmb": runner}, None, None)
    record = exp.run_single(_make_problem(), "bmb_contract", 1, MagicMock())
    assert record.loop_count == 1
    assert record.final_correct is True

def test_max_loops_exceeded():
    """Loop count = MAX_LOOPS+1 when all attempts fail."""
    llm = MagicMock()
    llm.generate.return_value = "```bmb\nbad code\n```"
    llm.extract_code = MagicMock(return_value="bad code")

    runner = MagicMock()
    runner.build.return_value = RunResult(compiled=False, test_passed=False, error_msg="syntax error")

    exp = ExperimentRunner(llm, {"bmb": runner}, None, None)
    record = exp.run_single(_make_problem(), "bmb_contract", 1, MagicMock())
    assert record.loop_count == MAX_LOOPS + 1
    assert record.final_correct is False
    assert len(record.attempts) == MAX_LOOPS
```

- [ ] **Step 4: Run tests, verify pass**

```bash
cd ecosystem/ai-proof && python -m pytest tests/test_experiment.py -v
```

- [ ] **Step 5: Commit**

```bash
git add ecosystem/ai-proof/orchestrator/experiment.py ecosystem/ai-proof/problems/registry.py ecosystem/ai-proof/tests/test_experiment.py
git commit -m "feat(ai-proof): experiment orchestrator with loop tracking and result persistence"
```

---

## Task 7: Pilot Problems (3 problems for Step 0)

**Files:**
- Create: `ecosystem/ai-proof/problems/01_binary_search/` (problem.md, tests.json, baseline.c, solution.bmb, solution.rs)
- Create: `ecosystem/ai-proof/problems/02_quicksort/` (same)
- Create: `ecosystem/ai-proof/problems/21_bounded_array/` (same — contract category)

- [ ] **Step 1: Create binary_search problem**

Write `problem.md` with clear description, `tests.json` with 15 test cases covering edge cases (empty array, single element, not found, first/last element), `baseline.c` with standard implementation, `solution.bmb` verified to compile, `solution.rs` verified to compile.

- [ ] **Step 2: Create quicksort problem**

Similar structure. Tests include: empty, single, sorted, reverse-sorted, duplicates, large array.

- [ ] **Step 3: Create bounded_array problem (contract category)**

Description emphasizes safe array access. BMB solution uses `pre idx < len`. Tests include out-of-bounds inputs that should be handled gracefully.

- [ ] **Step 4: Verify all solutions compile**

```bash
# BMB
D:/data/lang-bmb/target/release/bmb.exe build ecosystem/ai-proof/problems/01_binary_search/solution.bmb --release -o /tmp/test_bs
# Rust
rustc ecosystem/ai-proof/problems/01_binary_search/solution.rs -O -o /tmp/test_bs_rs
# C
clang ecosystem/ai-proof/problems/01_binary_search/baseline.c -O2 -o /tmp/test_bs_c
```

- [ ] **Step 5: Commit**

```bash
git add ecosystem/ai-proof/problems/
git commit -m "feat(ai-proof): 3 pilot problems (binary_search, quicksort, bounded_array)"
```

---

## Task 8: CLI Entry Point + Pilot Run

**Files:**
- Create: `ecosystem/ai-proof/scripts/run_experiment.py`
- Create: `ecosystem/ai-proof/scripts/validate_problems.py`

- [ ] **Step 1: Write validate_problems.py**

Script that iterates all problem dirs, compiles each solution.bmb and solution.rs, reports pass/fail.

- [ ] **Step 2: Write run_experiment.py**

```python
# scripts/run_experiment.py
"""BMB AI-Native Proof — Experiment Runner

Usage:
    python scripts/run_experiment.py --phase 1 --runs 3
    python scripts/run_experiment.py --pilot          # Run 3 pilot problems only
    python scripts/run_experiment.py --h1-only        # H1 experiment only
    python scripts/run_experiment.py --h2-only        # H2 experiment only
"""
import argparse
import json
from pathlib import Path

def main():
    parser = argparse.ArgumentParser(description="BMB AI-Native Proof Experiment")
    parser.add_argument("--phase", type=int, default=1)
    parser.add_argument("--runs", type=int, default=3)
    parser.add_argument("--pilot", action="store_true", help="Run 3 pilot problems only")
    parser.add_argument("--h1-only", action="store_true")
    parser.add_argument("--h2-only", action="store_true")
    parser.add_argument("--problems-dir", default="problems")
    parser.add_argument("--results-dir", default="results")
    args = parser.parse_args()

    base = Path(__file__).parent.parent
    problems_dir = base / args.problems_dir
    results_dir = base / args.results_dir
    results_dir.mkdir(parents=True, exist_ok=True)

    # Load BMB reference
    ref_path = base / "protocol" / "bmb_reference.md"
    bmb_ref = ref_path.read_text(encoding="utf-8") if ref_path.exists() else ""

    # Setup runners
    from runners.bmb_runner import BmbRunner
    from runners.rust_runner import RustRunner
    from runners.python_runner import PythonRunner
    from orchestrator.llm_client import LlmClient
    from orchestrator.experiment import ExperimentRunner
    from problems.registry import load_all_problems

    runners = {
        "bmb": BmbRunner(compiler_path="D:/data/lang-bmb/target/release/bmb.exe"),
        "rust": RustRunner(),
        "python": PythonRunner(),
    }
    llm = LlmClient(model="claude-opus-4-6", temperature=0.0)

    # Load problems
    problems = load_all_problems(problems_dir)
    if args.pilot:
        problems = [p for p in problems if p.number in (1, 2, 21)]
        print(f"Pilot mode: {len(problems)} problems")

    exp = ExperimentRunner(llm, runners, problems_dir, results_dir, bmb_ref)

    # Define conditions
    h1_conditions = ["bmb_contract", "bmb_nocontract"]
    h2_conditions = ["bmb_contract", "rust", "python"]
    conditions = []
    if not args.h2_only:
        conditions += h1_conditions
    if not args.h1_only:
        conditions += [c for c in h2_conditions if c not in conditions]

    # Run experiments
    import tempfile
    all_records = []
    for problem in problems:
        for condition in conditions:
            for run_id in range(1, args.runs + 1):
                # Skip if result already exists (resume support)
                prob_key = f"{problem.number:02d}_{problem.name}"
                result_file = results_dir / "raw" / prob_key / f"run{run_id}" / f"{condition}_result.json"
                if result_file.exists():
                    print(f"  SKIP {prob_key}/{condition}/run{run_id} (exists)")
                    continue

                print(f"  Running {prob_key} / {condition} / run {run_id}...")
                with tempfile.TemporaryDirectory() as tmp:
                    record = exp.run_single(problem, condition, run_id, Path(tmp))
                    exp.save_run(problem, record)
                    all_records.append({"problem": prob_key, **record.__dict__})
                    status = "PASS" if record.final_correct else f"FAIL (loops={record.loop_count})"
                    print(f"    → {status}")

    # Write summary.json
    summary = {"experiment": f"ai-native-proof-phase{args.phase}",
               "date": __import__("datetime").date.today().isoformat(),
               "llm": {"model": "claude-opus-4-6", "temperature": 0.0},
               "runs_per_condition": args.runs,
               "records": all_records}
    (results_dir / "summary.json").write_text(
        json.dumps(summary, indent=2, ensure_ascii=False, default=str), encoding="utf-8")
    print(f"\nDone. Results: {results_dir / 'summary.json'}")

if __name__ == "__main__":
    main()
```

- [ ] **Step 3: Run pilot (3 problems × 4 conditions × 1 run)**

```bash
cd ecosystem/ai-proof && python scripts/run_experiment.py --pilot --runs 1
```

Verify: results/raw/ populated, JSON valid, loop counts recorded.

- [ ] **Step 4: Fix any issues discovered in pilot**

- [ ] **Step 5: Commit**

```bash
git add ecosystem/ai-proof/scripts/
git commit -m "feat(ai-proof): CLI entry point + pilot run validated"
```

---

## Task 9: Statistical Analysis + Visualization

**Files:**
- Create: `ecosystem/ai-proof/analysis/stats.py`
- Create: `ecosystem/ai-proof/analysis/visualize.py`

- [ ] **Step 1: Write stats.py**

```python
# analysis/stats.py
from scipy import stats

def wilcoxon_test(contract_loops: list[float], nocontract_loops: list[float]) -> dict:
    """Paired Wilcoxon signed-rank test for H1."""
    stat, p = stats.wilcoxon(contract_loops, nocontract_loops, alternative="less")
    effect_size = stat / (len(contract_loops) * (len(contract_loops) + 1) / 2)
    return {"statistic": stat, "p_value": p, "effect_size": effect_size,
            "significant": p < 0.05}

def friedman_test(bmb_loops: list, rust_loops: list, python_loops: list) -> dict:
    """Friedman test for H2 (3 repeated measures)."""
    stat, p = stats.friedmanchisquare(bmb_loops, rust_loops, python_loops)
    return {"statistic": stat, "p_value": p,
            "significant": p < (0.05 / 3)}  # Bonferroni

def compute_aggregates(results: list[dict]) -> dict:
    """Compute per-condition aggregates from summary.json."""
    # median loops, correctness rate, median perf ratio per condition
    ...
```

- [ ] **Step 2: Write visualize.py**

Generate charts: loop count comparison bar chart, loop type stacked bar, performance ratio scatter, H1 paired difference plot.

- [ ] **Step 3: Test with pilot results**

- [ ] **Step 4: Commit**

```bash
git add ecosystem/ai-proof/analysis/
git commit -m "feat(ai-proof): statistical analysis (Wilcoxon, Friedman) + visualization"
```

---

## Task 10: Report Generator

**Files:**
- Create: `ecosystem/ai-proof/analysis/report_generator.py`

- [ ] **Step 1: Implement report_generator.py**

Reads `summary.json`, runs statistical tests, generates `AI_NATIVE_PROOF.md` with all sections from the spec (9.1-9.9).

- [ ] **Step 2: Generate report from pilot data**

- [ ] **Step 3: Commit**

```bash
git add ecosystem/ai-proof/analysis/report_generator.py
git commit -m "feat(ai-proof): markdown report generator"
```

---

## Task 11: Protocol Freeze + Pre-registration

**Files:**
- Create: `ecosystem/ai-proof/protocol/PROTOCOL.md`
- Create: `ecosystem/ai-proof/protocol/bmb_reference.md`

- [ ] **Step 1: Write PROTOCOL.md**

Copy the essential experimental parameters from the spec: hypotheses, conditions, problem list, statistical tests, success criteria, effect sizes.

- [ ] **Step 2: Create bmb_reference.md**

Extract relevant sections from `docs/LANGUAGE_REFERENCE.md`: syntax, types, contracts (pre/post), control flow, functions. Freeze version.

- [ ] **Step 3: Pre-registration commit**

```bash
git add ecosystem/ai-proof/protocol/
git commit -m "pre-register: ai-native-proof protocol v1 — frozen before experiment execution"
```

This commit hash becomes the pre-registration timestamp.

---

## Task 12: Full Problem Set (30 problems)

**Files:**
- Create: `ecosystem/ai-proof/problems/03_knapsack_dp/` through `30_clamped_lerp/`

- [ ] **Step 1: Create algorithm problems (03-10)**

For each: problem.md, tests.json (15-20 cases), baseline.c, solution.bmb, solution.rs.

- [ ] **Step 2: Create system problems (11-20)**

Same structure. Verify BMB solutions compile for each.

- [ ] **Step 3: Create contract problems (21-30)**

Same structure. BMB solutions use pre/post contracts.

- [ ] **Step 4: Run validate_problems.py**

```bash
cd ecosystem/ai-proof && python scripts/validate_problems.py
```

All 30 × 3 solutions must compile and pass tests.

- [ ] **Step 5: Commit**

```bash
git add ecosystem/ai-proof/problems/
git commit -m "feat(ai-proof): complete problem set — 30 problems with verified solutions"
```

---

## Task 13: Full Experiment Execution

- [ ] **Step 1: Run H1 experiment**

```bash
cd ecosystem/ai-proof && python scripts/run_experiment.py --h1-only --runs 3
```

30 problems × 2 conditions × 3 runs = 180 API calls.

- [ ] **Step 2: Review H1 results, fix runner issues if any**

- [ ] **Step 3: Run H2 experiment**

```bash
cd ecosystem/ai-proof && python scripts/run_experiment.py --h2-only --runs 3
```

30 problems × 3 languages × 3 runs = 270 API calls.

- [ ] **Step 4: Generate final report**

```bash
cd ecosystem/ai-proof && python -c "from analysis.report_generator import generate; generate()"
```

- [ ] **Step 5: Commit results**

```bash
git add ecosystem/ai-proof/results/ ecosystem/ai-proof/report/
git commit -m "results: ai-native-proof phase 1 — H1 + H2 experiment data + report"
```

---

## Dependency Graph

```
Task 1 (scaffold)
  ├→ Task 2 (runners)          ─┐
  ├→ Task 3 (normalizer)        ├→ Task 6 (orchestrator)
  ├→ Task 4 (LLM client)       ─┘       │
  └→ Task 5 (contract stripper)──────────┘
                                         │
Task 7 (pilot problems) ────────→ Task 8 (CLI + pilot run)
                                         │
                                Task 12 (full 30 problems)
                                         │
                                Task 11 (protocol freeze) ← problems must be frozen here
                                         │
                                Task 13 (full experiment)
                                         │
                                Task 9 (analysis) → Task 10 (report)
```

**Parallelizable**: Tasks 2, 3, 4, 5 are independent after Task 1.
**Critical path**: Task 1 → 2+3+4+5 → 6 → 7+8 (pilot) → 12 (full problems) → 11 (freeze) → 13 (experiment) → 9+10 (analysis).

**Note**: Pilot data (Task 8) is for calibration only and is NOT part of the pre-registered experiment. Task 11 freezes the protocol AFTER problems are finalized but BEFORE the real experiment runs.
