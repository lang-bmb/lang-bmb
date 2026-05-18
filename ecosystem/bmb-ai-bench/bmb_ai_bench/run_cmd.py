"""Run LLM experiment — generate→build→test loop per problem."""

from __future__ import annotations

import datetime
import json
import os
import sys
import tempfile
import subprocess
from dataclasses import asdict, dataclass, field
from pathlib import Path

_BASE = Path(__file__).resolve().parents[1]
_REPO_ROOT = _BASE.parents[1]  # lang-bmb/
_DEFAULT_BMB_EXE = _REPO_ROOT / "target" / "release" / (
    "bmb.exe" if sys.platform == "win32" else "bmb"
)

_MAX_LOOPS = 10
_BUILD_TIMEOUT = 60
_TEST_TIMEOUT = 10


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


def _load_env_file(env_file: Path | None = None) -> None:
    """Load .env.local into os.environ (no-op if missing)."""
    candidates = [env_file, _REPO_ROOT / ".env.local", Path(".env.local")]
    for candidate in candidates:
        if candidate and candidate.exists():
            for line in candidate.read_text(encoding="utf-8").splitlines():
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    k, _, v = line.partition("=")
                    os.environ.setdefault(k.strip(), v.strip())
            return


def _run_one_problem(
    problem_dir: Path,
    llm,
    bmb_exe: Path,
    reference: str,
    max_loops: int = _MAX_LOOPS,
    progress: bool = True,
) -> ProblemResult:
    """Generate→build→test loop for one problem. Returns ProblemResult."""
    from bmb_ai_bench.diagnostics.error_normalizer import normalize_error
    from bmb_ai_bench.diagnostics.loop_classifier import classify_loop

    pid = problem_dir.name
    meta_file = problem_dir / "metadata.json"
    meta = json.loads(meta_file.read_text(encoding="utf-8")) if meta_file.exists() else {}
    desc = (problem_dir / "problem.md").read_text(encoding="utf-8", errors="replace") \
        if (problem_dir / "problem.md").exists() else ""
    tests = json.loads((problem_dir / "tests.json").read_text(encoding="utf-8"))

    test_preview = "\n".join(
        f"stdin: {t.get('stdin','')}  → stdout: {t.get('expected_stdout','')}"
        for t in tests[:5]
    )
    initial_prompt = (
        f"## BMB Reference\n{reference}\n---\n"
        f"{desc}\n"
        f"## Examples\n{test_preview}\n\n"
        "Write a complete BMB program. Output ONLY code in a ```bmb block."
    )
    sys_instruction = (
        "You are a BMB programmer. Output ONLY the complete source code "
        "inside a ```bmb code block. No explanation, no comments outside code."
    )

    messages = [{"role": "user", "content": initial_prompt}]
    loop_types: dict[str, int] = {"A": 0, "B": 0, "C": 0, "D": 0}
    attempts: list[AttemptRecord] = []

    for attempt_num in range(1, max_loops + 1):
        # Context truncation (HTTP 413 prevention): keep initial prompt + last 2 assistant/user pairs
        if len(messages) > 5:
            messages = [messages[0]] + messages[-4:]
        response = llm.generate(sys_instruction, messages)
        code = llm.extract_code(response, "bmb")

        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = Path(tmpdir)
            src = tmp / "solution.bmb"
            src.write_text(code, encoding="utf-8")
            out = tmp / ("solution.exe" if sys.platform == "win32" else "solution")

            # Check
            try:
                check = subprocess.run(
                    [str(bmb_exe), "check", str(src)],
                    capture_output=True, text=True, timeout=_BUILD_TIMEOUT,
                )
            except subprocess.TimeoutExpired:
                check = _fake_proc(1, "check timeout")

            if check.returncode != 0:
                err_raw = (check.stdout + check.stderr).strip()
                error = normalize_error(err_raw, "bmb")
                lt = classify_loop(error, "bmb")
                loop_types[lt.value] += 1
                attempts.append(AttemptRecord(attempt_num, code, False, False, err_raw, lt.value))
                feedback = (
                    f"compile_error: {error['normalized']}\n"
                    + (f"Suggestion: {error['suggestion']}\n" if error.get("suggestion") else "")
                    + f"\n{err_raw[:500]}\n"
                    + "\nFix the error. Output ONLY the complete corrected code in a ```bmb block."
                )
                messages.append({"role": "assistant", "content": response})
                messages.append({"role": "user", "content": feedback})
                continue

            # Build
            try:
                build = subprocess.run(
                    [str(bmb_exe), "build", str(src), "-o", str(out), "--release"],
                    capture_output=True, text=True, timeout=_BUILD_TIMEOUT,
                )
            except subprocess.TimeoutExpired:
                build = _fake_proc(1, "build timeout")

            if build.returncode != 0:
                err_raw = (build.stderr + build.stdout).strip()
                error = normalize_error(err_raw, "bmb")
                lt = classify_loop(error, "bmb")
                loop_types[lt.value] += 1
                attempts.append(AttemptRecord(attempt_num, code, False, False, err_raw, lt.value))
                feedback = (
                    f"build_error: {err_raw[:500]}\nFix. Output ONLY corrected code in a ```bmb block."
                )
                messages.append({"role": "assistant", "content": response})
                messages.append({"role": "user", "content": feedback})
                continue

            # Test — collect up to 3 failures for richer feedback
            failures: list[str] = []
            for i, tc in enumerate(tests):
                if len(failures) >= 3:
                    break
                try:
                    result = subprocess.run(
                        [str(out)], input=tc.get("stdin", ""),
                        capture_output=True, text=True, timeout=_TEST_TIMEOUT,
                    )
                    if result.stdout != tc.get("expected_stdout", ""):
                        stdin_str = tc.get("stdin", "")
                        failures.append(
                            f"Test {i}:\n"
                            f"  stdin: {stdin_str!r}\n"
                            f"  expected: {tc['expected_stdout']!r}\n"
                            f"  got: {result.stdout!r}"
                        )
                except subprocess.TimeoutExpired:
                    failures.append(f"Test {i}: timeout (stdin: {tc.get('stdin', '')!r})")
            all_pass = len(failures) == 0
            fail_msg = "\n".join(failures) if failures else ""

            if not all_pass:
                error = normalize_error(failures[0], "bmb", is_test_failure=True)
                lt = classify_loop(error, "bmb")
                loop_types[lt.value] += 1
                attempts.append(AttemptRecord(attempt_num, code, True, False, fail_msg, lt.value))
                n_fail = len(failures)
                n_total = len(tests)
                feedback = (
                    f"test_failure ({n_fail} of {n_total} tests failed):\n"
                    f"{fail_msg}\n"
                    "Fix the logic error. Output ONLY corrected code in a ```bmb block."
                )
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
        loop_count=max_loops + 1, final_correct=False,
        loop_types=loop_types, attempts=attempts,
    )


class _fake_proc:
    def __init__(self, returncode: int, msg: str):
        self.returncode = returncode
        self.stdout = msg
        self.stderr = ""


def _select_problems(
    problems_dir: Path,
    pilot: bool = False,
    category: str = "all",
    problem_nums: str | None = None,
) -> list[Path]:
    """Return sorted list of problem directories per filter."""
    dirs = sorted(
        d for d in problems_dir.iterdir()
        if d.is_dir() and d.name[0].isdigit()
    )
    if pilot:
        pilot_nums = {1, 21, 50}
        return [d for d in dirs if int(d.name.split("_")[0]) in pilot_nums]
    if problem_nums:
        nums = {int(x) for x in problem_nums.split(",")}
        return [d for d in dirs if int(d.name.split("_")[0]) in nums]
    if category != "all":
        filtered = []
        for d in dirs:
            meta_file = d / "metadata.json"
            if meta_file.exists():
                meta = json.loads(meta_file.read_text(encoding="utf-8"))
                if meta.get("category") == category:
                    filtered.append(d)
        return filtered
    return dirs


def run_run(
    model: str,
    api_base: str,
    api_key: str,
    category: str = "all",
    runs: int = 1,
    output_dir: str = "results/",
    pilot: bool = False,
    problem_nums: str | None = None,
    max_loops: int = _MAX_LOOPS,
    dry_run: bool = False,
    json_output: bool = False,
    bmb_exe: Path | None = None,
    env_file: Path | None = None,
) -> int:
    """Entry point for the `run` subcommand."""
    _load_env_file(env_file)

    # Env fallbacks — BMB_BENCH_* → GPUSTACK_* → Anthropic defaults
    if not model:
        model = os.environ.get("BMB_BENCH_MODEL",
                os.environ.get("GPUSTACK_MODEL", ""))
    if not api_base:
        gpustack_ep = os.environ.get("GPUSTACK_ENDPOINT", "")
        gpustack_base = (gpustack_ep.rstrip("/") + "/v1") if gpustack_ep else ""
        api_base = os.environ.get("BMB_BENCH_API_BASE",
                   gpustack_base or "https://api.anthropic.com/v1")
    if not api_key:
        api_key = os.environ.get("BMB_BENCH_API_KEY",
                  os.environ.get("GPUSTACK_API_KEY",
                  os.environ.get("ANTHROPIC_API_KEY", "")))

    if not model:
        msg = "ERROR: --model is required (or set BMB_BENCH_MODEL)"
        if json_output:
            print(json.dumps({"error": msg}))
        else:
            print(msg, file=sys.stderr)
        return 1

    bmb_exe = bmb_exe or _DEFAULT_BMB_EXE

    problems_dir = _BASE / "problems"
    ref_path = _BASE / "protocol" / "bmb_reference.md"
    reference = ref_path.read_text(encoding="utf-8") if ref_path.exists() else ""

    dirs = _select_problems(problems_dir, pilot=pilot, category=category, problem_nums=problem_nums)
    if not dirs:
        msg = "ERROR: no problems selected (check --category or --pilot)"
        if json_output:
            print(json.dumps({"error": msg}))
        else:
            print(msg, file=sys.stderr)
        return 1

    if dry_run:
        plan = {
            "dry_run": True,
            "model": model,
            "api_base": api_base,
            "problems": len(dirs),
            "runs": runs,
            "bmb_exe": str(bmb_exe),
            "bmb_exe_exists": bmb_exe.exists(),
            "reference_chars": len(reference),
            "output_dir": output_dir,
            "problem_list": [d.name for d in dirs[:20]],
        }
        if json_output:
            print(json.dumps(plan, indent=2))
        else:
            print(f"DRY RUN — {len(dirs)} problems, model={model}")
            print(f"API: {api_base}")
            print(f"BMB: {bmb_exe} (exists={bmb_exe.exists()})")
            print(f"Reference: {len(reference)} chars")
            for d in dirs[:10]:
                print(f"  {d.name}")
            if len(dirs) > 10:
                print(f"  ... and {len(dirs) - 10} more")
        return 0

    from bmb_ai_bench.runner.llm_client import LlmClient
    # GPUStack (local inference): larger token budget + disable Qwen3 thinking mode
    gpustack_ep = os.environ.get("GPUSTACK_ENDPOINT", "")
    using_gpustack = bool(gpustack_ep) and gpustack_ep.split("/")[2] in api_base
    llm_max_tokens = 16384 if using_gpustack else 4096
    llm_extra = {"chat_template_kwargs": {"enable_thinking": False}} if using_gpustack else {}
    llm = LlmClient(model=model, base_url=api_base, api_key=api_key,
                    max_tokens=llm_max_tokens, extra_body=llm_extra)

    out_dir = Path(output_dir)
    if not out_dir.is_absolute():
        out_dir = _BASE / "results" / datetime.date.today().isoformat()
    out_dir.mkdir(parents=True, exist_ok=True)

    all_results: list[dict] = []
    for run_id in range(1, runs + 1):
        if not json_output:
            print(f"\n=== Run {run_id}/{runs} ===", file=sys.stderr)
        for d in dirs:
            pid = d.name
            result_file = out_dir / f"{pid}_run{run_id}.json"
            if result_file.exists():
                if not json_output:
                    print(f"  SKIP {pid} run{run_id}", file=sys.stderr)
                prev = json.loads(result_file.read_text(encoding="utf-8"))
                all_results.append(prev)
                continue

            if not json_output:
                print(f"  {pid}...", end=" ", flush=True, file=sys.stderr)
            try:
                result = _run_one_problem(d, llm, bmb_exe, reference, max_loops=max_loops)
                status = (
                    f"PASS (loop={result.loop_count})"
                    if result.final_correct
                    else f"FAIL (loop={result.loop_count})"
                )
                if not json_output:
                    print(status, file=sys.stderr)
                data = asdict(result)
                data["run_id"] = run_id
                result_file.write_text(json.dumps(data, indent=2), encoding="utf-8")
                all_results.append(data)
            except Exception as e:
                if not json_output:
                    print(f"ERROR: {e}", file=sys.stderr)
                all_results.append({
                    "problem_id": pid, "run_id": run_id,
                    "final_correct": False, "loop_count": -1, "error": str(e),
                })

    total = len(all_results)
    passed = sum(1 for r in all_results if r.get("final_correct"))
    loops = [r["loop_count"] for r in all_results if r.get("final_correct") and r["loop_count"] > 0]
    median_loops = sorted(loops)[len(loops) // 2] if loops else 0

    summary = {
        "date": datetime.date.today().isoformat(),
        "model": model,
        "total_problems": len(dirs),
        "runs": runs,
        "total_runs": total,
        "passed": passed,
        "success_rate": round(passed / total * 100, 1) if total else 0.0,
        "median_loops": median_loops,
        "output_dir": str(out_dir),
    }
    (out_dir / "summary.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")

    # Write results.json in report.py-compatible format
    problems_agg: dict = {}
    for r in all_results:
        pid = r.get("problem_id", "")
        if not pid:
            continue
        if pid not in problems_agg:
            problems_agg[pid] = {
                "category": r.get("category", ""),
                "difficulty": r.get("difficulty", ""),
                "runs": [],
            }
        # last attempt determines compiled status
        attempts = r.get("attempts", [])
        compiled = any(a.get("compiled", False) for a in attempts) if attempts else False
        problems_agg[pid]["runs"].append({
            "run_id": r.get("run_id", 1),
            "loop_count": r.get("loop_count", 999),
            "final_correct": r.get("final_correct", False),
            "compiled": compiled,
            "perf_ratio": r.get("perf_ratio"),
        })
    results_data = {"model": model, "date": summary["date"], "problems": problems_agg}
    (out_dir / "results.json").write_text(json.dumps(results_data, indent=2), encoding="utf-8")

    if json_output:
        print(json.dumps(summary))
    else:
        sep = "=" * 60
        print(f"\n{sep}", file=sys.stderr)
        print(f"Results: {out_dir}", file=sys.stderr)
        print(
            f"Total: {total}, Passed: {passed}, "
            f"Success: {summary['success_rate']}%",
            file=sys.stderr,
        )
        print(f"Median Loops: {median_loops}", file=sys.stderr)
        print(sep, file=sys.stderr)
        print(json.dumps(summary))

    return 0
