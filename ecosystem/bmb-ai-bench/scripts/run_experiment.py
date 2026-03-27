#!/usr/bin/env python3
"""BMB AI-Bench Experiment Runner — real LLM evaluation.

Usage:
    python scripts/run_experiment.py --pilot --runs 1              # 3 problems, 1 run
    python scripts/run_experiment.py --category algorithm --runs 3  # Algorithm category
    python scripts/run_experiment.py --all --runs 1                 # All 100 problems
    python scripts/run_experiment.py --dry-run --pilot              # Validate setup
"""
from __future__ import annotations

import argparse
import datetime
import json
import os
import sys
import tempfile
from dataclasses import asdict, dataclass, field
from pathlib import Path

_BASE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(_BASE))

_BMB_EXE = Path("D:/data/lang-bmb/target/release/bmb.exe")
_ENV_FILE = Path("D:/data/lang-bmb/.env.local")
MAX_LOOPS = 10


def _load_env():
    """Load .env.local into os.environ."""
    if _ENV_FILE.exists():
        for line in _ENV_FILE.read_text().splitlines():
            line = line.strip()
            if line and not line.startswith("#") and "=" in line:
                k, v = line.split("=", 1)
                os.environ.setdefault(k.strip(), v.strip())


@dataclass
class AttemptRecord:
    attempt: int
    code: str
    compiled: bool
    test_passed: bool
    error_msg: str
    loop_type: str


@dataclass
class ProblemResult:
    problem_id: str
    category: str
    difficulty: str
    loop_count: int
    final_correct: bool
    loop_types: dict = field(default_factory=dict)
    attempts: list[AttemptRecord] = field(default_factory=list)


def run_problem(problem_dir: Path, llm, bmb_exe: str, reference: str) -> ProblemResult:
    """Run LLM generate→build→test loop for one problem."""
    from bmb_ai_bench.diagnostics.error_normalizer import normalize_error
    from bmb_ai_bench.diagnostics.loop_classifier import classify_loop

    pid = problem_dir.name
    meta = json.loads((problem_dir / "metadata.json").read_text())
    desc = (problem_dir / "problem.md").read_text(encoding="utf-8", errors="replace")
    tests = json.loads((problem_dir / "tests.json").read_text())

    # Build prompt
    test_preview = "\n".join(
        f"stdin: {t['stdin']}  → stdout: {t['expected_stdout']}"
        for t in tests[:5]
    )
    initial_prompt = (
        f"## BMB Reference\n{reference}\n---\n"
        f"{desc}\n"
        f"## Examples\n{test_preview}\n\n"
        f"Write a complete BMB program. Output ONLY code in a ```bmb block."
    )

    sys_instruction = (
        "You are a BMB programmer. Output ONLY the complete source code "
        "inside a ```bmb code block. No explanation, no comments outside code."
    )

    messages = [{"role": "user", "content": initial_prompt}]
    loop_types = {"A": 0, "B": 0, "C": 0, "D": 0}
    attempts = []

    for attempt_num in range(1, MAX_LOOPS + 1):
        # Generate
        response = llm.generate(sys_instruction, messages)
        code = llm.extract_code(response, "bmb")

        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = Path(tmpdir)
            src = tmp / "solution.bmb"
            src.write_text(code, encoding="utf-8")
            out = tmp / "solution.exe"

            # Check
            import subprocess
            try:
                check = subprocess.run(
                    [bmb_exe, "check", str(src)],
                    capture_output=True, text=True, timeout=60, cwd=str(tmp),
                )
            except subprocess.TimeoutExpired:
                check = type("R", (), {"returncode": 1, "stdout": "timeout", "stderr": ""})()

            if check.returncode != 0:
                err_raw = (check.stdout + check.stderr).strip()
                error = normalize_error(err_raw, "bmb")
                lt = classify_loop(error, "bmb")
                loop_types[lt.value] += 1
                attempts.append(AttemptRecord(attempt_num, code, False, False, err_raw, lt.value))

                feedback = (
                    f"compile_error: {error['normalized']}\n"
                    + (f"Suggestion: {error['suggestion']}\n" if error.get('suggestion') else "")
                    + f"\n{err_raw[:500]}\n"
                    + "\nFix the error. Output ONLY the complete corrected code in a ```bmb block."
                )
                messages.append({"role": "assistant", "content": response})
                messages.append({"role": "user", "content": feedback})
                continue

            # Build
            try:
                build = subprocess.run(
                    [bmb_exe, "build", str(src), "-o", str(out), "--release"],
                    capture_output=True, text=True, timeout=60, cwd=str(tmp),
                )
            except subprocess.TimeoutExpired:
                build = type("R", (), {"returncode": 1, "stdout": "", "stderr": "build timeout"})()

            if build.returncode != 0:
                err_raw = (build.stderr + build.stdout).strip()
                error = normalize_error(err_raw, "bmb")
                lt = classify_loop(error, "bmb")
                loop_types[lt.value] += 1
                attempts.append(AttemptRecord(attempt_num, code, False, False, err_raw, lt.value))
                feedback = f"build_error: {err_raw[:500]}\nFix. Output ONLY corrected code in a ```bmb block."
                messages.append({"role": "assistant", "content": response})
                messages.append({"role": "user", "content": feedback})
                continue

            # Test
            all_pass = True
            fail_msg = ""
            for i, tc in enumerate(tests):
                try:
                    result = subprocess.run(
                        [str(out)], input=tc.get("stdin", ""),
                        capture_output=True, text=True, timeout=10, cwd=str(tmp),
                    )
                    if result.stdout != tc.get("expected_stdout", ""):
                        all_pass = False
                        fail_msg = f"Test {i}: expected {tc['expected_stdout']!r}, got {result.stdout!r}"
                        break
                except subprocess.TimeoutExpired:
                    all_pass = False
                    fail_msg = f"Test {i}: timeout"
                    break

            if not all_pass:
                error = normalize_error(fail_msg, "bmb", is_test_failure=True)
                lt = classify_loop(error, "bmb")
                loop_types[lt.value] += 1
                attempts.append(AttemptRecord(attempt_num, code, True, False, fail_msg, lt.value))
                feedback = f"test_failure: {fail_msg}\nFix. Output ONLY corrected code in a ```bmb block."
                messages.append({"role": "assistant", "content": response})
                messages.append({"role": "user", "content": feedback})
                continue

            # Success
            attempts.append(AttemptRecord(attempt_num, code, True, True, "", ""))
            return ProblemResult(
                problem_id=pid, category=meta.get("category", ""),
                difficulty=meta.get("difficulty", ""),
                loop_count=attempt_num, final_correct=True,
                loop_types=loop_types, attempts=attempts,
            )

    return ProblemResult(
        problem_id=pid, category=meta.get("category", ""),
        difficulty=meta.get("difficulty", ""),
        loop_count=MAX_LOOPS + 1, final_correct=False,
        loop_types=loop_types, attempts=attempts,
    )


def main() -> int:
    _load_env()

    parser = argparse.ArgumentParser(description="BMB AI-Bench Experiment")
    parser.add_argument("--pilot", action="store_true", help="3 pilot problems only")
    parser.add_argument("--category", default=None, help="Filter by category")
    parser.add_argument("--all", action="store_true", help="All 100 problems")
    parser.add_argument("--runs", type=int, default=1, help="Runs per problem")
    parser.add_argument("--model", default=None, help="Model name")
    parser.add_argument("--api-base", default=None, help="API base URL")
    parser.add_argument("--api-key", default=None, help="API key")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--output", default=None, help="Output directory")
    parser.add_argument("--problems", default=None, help="Comma-separated problem numbers")
    args = parser.parse_args()

    # Defaults from env
    model = args.model or os.environ.get("OPENAI_COMPATIBLE_MODEL", "claude-text")
    api_base = args.api_base or os.environ.get("OPENAI_COMPATIBLE_ENDPOINT", "http://172.30.1.62:6190") + "/v1"
    api_key = args.api_key or os.environ.get("OPENAI_COMPATIBLE_API_KEY", "no-key")

    problems_dir = _BASE / "problems"
    ref_path = _BASE / "protocol" / "bmb_reference.md"
    if not ref_path.exists():
        ref_path = _BASE.parent / "ai-proof" / "protocol" / "bmb_reference.md"
    reference = ref_path.read_text(encoding="utf-8") if ref_path.exists() else ""

    # Load problems
    dirs = sorted(d for d in problems_dir.iterdir() if d.is_dir() and d.name[0].isdigit())

    if args.pilot:
        dirs = [d for d in dirs if int(d.name.split("_")[0]) in (1, 21, 50)]
    elif args.problems:
        nums = set(int(x) for x in args.problems.split(","))
        dirs = [d for d in dirs if int(d.name.split("_")[0]) in nums]
    elif args.category:
        filtered = []
        for d in dirs:
            meta_file = d / "metadata.json"
            if meta_file.exists():
                meta = json.loads(meta_file.read_text())
                if meta.get("category") == args.category:
                    filtered.append(d)
        dirs = filtered
    elif not args.all:
        print("Specify --pilot, --category, --all, or --problems")
        return 1

    bmb_exe = str(_BMB_EXE)

    if args.dry_run:
        print(f"DRY RUN — {len(dirs)} problems, model={model}")
        print(f"API: {api_base}")
        print(f"BMB: {bmb_exe} (exists={_BMB_EXE.exists()})")
        print(f"Reference: {ref_path} ({len(reference)} chars)")
        for d in dirs[:10]:
            print(f"  {d.name}")
        if len(dirs) > 10:
            print(f"  ... and {len(dirs) - 10} more")
        return 0

    from bmb_ai_bench.runner.llm_client import LlmClient
    llm = LlmClient(model=model, base_url=api_base, api_key=api_key)

    # Output dir
    out_dir = Path(args.output) if args.output else _BASE / "results" / datetime.date.today().isoformat()
    out_dir.mkdir(parents=True, exist_ok=True)

    all_results = []
    for run_id in range(1, args.runs + 1):
        print(f"\n=== Run {run_id}/{args.runs} ===")
        for d in dirs:
            pid = d.name
            result_file = out_dir / f"{pid}_run{run_id}.json"
            if result_file.exists():
                print(f"  SKIP {pid} run{run_id}")
                prev = json.loads(result_file.read_text())
                all_results.append(prev)
                continue

            print(f"  {pid}...", end=" ", flush=True)
            try:
                result = run_problem(d, llm, bmb_exe, reference)
                status = f"PASS (loop={result.loop_count})" if result.final_correct else f"FAIL (loop={result.loop_count})"
                print(status)
                data = asdict(result)
                data["run_id"] = run_id
                result_file.write_text(json.dumps(data, indent=2), encoding="utf-8")
                all_results.append(data)
            except Exception as e:
                print(f"ERROR: {e}")
                all_results.append({
                    "problem_id": pid, "run_id": run_id,
                    "final_correct": False, "loop_count": -1, "error": str(e),
                })

    # Summary
    total = len(all_results)
    passed = sum(1 for r in all_results if r.get("final_correct"))
    loops = [r["loop_count"] for r in all_results if r.get("final_correct")]
    median_loops = sorted(loops)[len(loops) // 2] if loops else 0

    summary = {
        "date": datetime.date.today().isoformat(),
        "model": model, "api_base": api_base,
        "total_problems": len(dirs), "runs": args.runs,
        "total_runs": total, "passed": passed,
        "success_rate": f"{passed / total * 100:.1f}%" if total else "N/A",
        "median_loops": median_loops,
    }
    (out_dir / "summary.json").write_text(json.dumps(summary, indent=2))

    print(f"\n{'='*60}")
    print(f"Results: {out_dir}")
    print(f"Total: {total}, Passed: {passed}, Success: {summary['success_rate']}")
    print(f"Median Loops: {median_loops}")
    print(f"{'='*60}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
