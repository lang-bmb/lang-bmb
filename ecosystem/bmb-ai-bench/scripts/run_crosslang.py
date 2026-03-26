#!/usr/bin/env python3
"""Cross-language comparison experiment — BMB vs C vs Python.

Same LLM, same problems, different target languages.
This is the key experiment for objectivity: if BMB scores 90% and C scores 95%,
BMB is NOT more AI-friendly. If BMB scores 90% and C scores 70%, it IS.

Usage:
    python scripts/run_crosslang.py --pilot --runs 1     # 3 problems, quick test
    python scripts/run_crosslang.py --all --runs 3        # Full experiment
    python scripts/run_crosslang.py --category algorithm  # Single category
"""
from __future__ import annotations

import argparse
import datetime
import json
import os
import subprocess
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
    if _ENV_FILE.exists():
        for line in _ENV_FILE.read_text().splitlines():
            line = line.strip()
            if line and not line.startswith("#") and "=" in line:
                k, v = line.split("=", 1)
                os.environ.setdefault(k.strip(), v.strip())


@dataclass
class RunResult:
    loop_count: int
    final_correct: bool
    loop_types: dict = field(default_factory=dict)
    error_sample: str = ""


def _build_prompt(desc: str, tests: list[dict], lang: str, reference: str = "") -> str:
    test_preview = "\n".join(
        f"stdin: {t['stdin']}  → stdout: {t['expected_stdout']}"
        for t in tests[:5]
    )
    parts = []
    if reference and lang == "bmb":
        parts.append(f"## BMB Reference\n{reference}\n---")
    parts.append(desc)
    parts.append(f"## Examples\n{test_preview}")
    parts.append(f"\nWrite a complete {lang} program that reads from stdin and writes to stdout.")
    parts.append(f"Output ONLY code in a ```{lang} block.")
    return "\n".join(parts)


def _sys_instruction(lang: str) -> str:
    return (
        f"You are a {lang} programmer. Output ONLY the complete source code "
        f"inside a ```{lang} code block. No explanation."
    )


def _run_bmb(code: str, tests: list[dict], tmpdir: Path) -> tuple[bool, bool, str]:
    """Returns (compiled, all_tests_pass, error_msg)."""
    src = tmpdir / "solution.bmb"
    src.write_text(code, encoding="utf-8")
    out = tmpdir / "solution.exe"

    try:
        r = subprocess.run([str(_BMB_EXE), "check", str(src)],
                          capture_output=True, text=True, timeout=60, cwd=str(tmpdir))
    except subprocess.TimeoutExpired:
        return False, False, "check timeout"
    if r.returncode != 0:
        return False, False, (r.stdout + r.stderr).strip()

    try:
        r = subprocess.run([str(_BMB_EXE), "build", str(src), "-o", str(out), "--release"],
                          capture_output=True, text=True, timeout=60, cwd=str(tmpdir))
    except subprocess.TimeoutExpired:
        return False, False, "build timeout"
    if r.returncode != 0:
        return False, False, (r.stderr + r.stdout).strip()

    return _run_tests(str(out), tests, tmpdir)


def _run_c(code: str, tests: list[dict], tmpdir: Path) -> tuple[bool, bool, str]:
    src = tmpdir / "solution.c"
    src.write_text(code, encoding="utf-8")
    out = tmpdir / "solution.exe"

    try:
        r = subprocess.run(["gcc", "-O2", "-o", str(out), str(src), "-lm"],
                          capture_output=True, text=True, timeout=60, cwd=str(tmpdir))
    except (subprocess.TimeoutExpired, FileNotFoundError) as e:
        return False, False, str(e)
    if r.returncode != 0:
        return False, False, (r.stderr + r.stdout).strip()

    return _run_tests(str(out), tests, tmpdir)


def _run_python(code: str, tests: list[dict], tmpdir: Path) -> tuple[bool, bool, str]:
    src = tmpdir / "solution.py"
    src.write_text(code, encoding="utf-8")

    for i, tc in enumerate(tests):
        try:
            r = subprocess.run([sys.executable, str(src)],
                              input=tc.get("stdin", ""), capture_output=True,
                              text=True, timeout=10, cwd=str(tmpdir))
        except subprocess.TimeoutExpired:
            return True, False, f"Test {i}: timeout"
        if r.returncode != 0:
            return False, False, f"Test {i} error: {(r.stderr+r.stdout).strip()[:200]}"
        if r.stdout != tc.get("expected_stdout", ""):
            return True, False, f"Test {i}: expected {tc['expected_stdout']!r}, got {r.stdout!r}"

    return True, True, ""


def _run_tests(binary: str, tests: list[dict], tmpdir: Path) -> tuple[bool, bool, str]:
    for i, tc in enumerate(tests):
        try:
            r = subprocess.run([binary], input=tc.get("stdin", ""),
                              capture_output=True, text=True, timeout=10, cwd=str(tmpdir))
        except subprocess.TimeoutExpired:
            return True, False, f"Test {i}: timeout"
        if r.stdout != tc.get("expected_stdout", ""):
            return True, False, f"Test {i}: expected {tc['expected_stdout']!r}, got {r.stdout!r}"
    return True, True, ""


RUNNERS = {"bmb": _run_bmb, "c": _run_c, "python": _run_python}


def run_problem_lang(problem_dir: Path, lang: str, llm, reference: str) -> RunResult:
    """Run one problem in one language."""
    from bmb_ai_bench.runner.llm_client import LlmClient

    desc = (problem_dir / "problem.md").read_text(encoding="utf-8", errors="replace")
    tests = json.loads((problem_dir / "tests.json").read_text())
    runner = RUNNERS[lang]

    prompt = _build_prompt(desc, tests, lang, reference)
    messages = [{"role": "user", "content": prompt}]
    sys_inst = _sys_instruction(lang)
    loop_types = {"compile": 0, "test": 0}

    for attempt in range(1, MAX_LOOPS + 1):
        response = llm.generate(sys_inst, messages)
        code = LlmClient.extract_code(response, lang)

        with tempfile.TemporaryDirectory() as tmp:
            compiled, passed, err = runner(code, tests, Path(tmp))

        if passed:
            return RunResult(loop_count=attempt, final_correct=True, loop_types=loop_types)

        if not compiled:
            loop_types["compile"] += 1
        else:
            loop_types["test"] += 1

        feedback = f"{'compile_error' if not compiled else 'test_failure'}: {err[:500]}\n"
        feedback += f"Fix the error. Output ONLY the complete corrected {lang} code in a ```{lang} block."
        messages.append({"role": "assistant", "content": response})
        messages.append({"role": "user", "content": feedback})

    return RunResult(loop_count=MAX_LOOPS + 1, final_correct=False,
                     loop_types=loop_types, error_sample=err[:200] if err else "")


def main() -> int:
    _load_env()

    parser = argparse.ArgumentParser(description="Cross-language comparison")
    parser.add_argument("--pilot", action="store_true")
    parser.add_argument("--category", default=None)
    parser.add_argument("--all", action="store_true")
    parser.add_argument("--problems", default=None, help="Comma-separated numbers")
    parser.add_argument("--langs", default="bmb,c,python", help="Languages to test")
    parser.add_argument("--runs", type=int, default=1)
    parser.add_argument("--model", default=None)
    parser.add_argument("--api-base", default=None)
    parser.add_argument("--api-key", default=None)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--output", default=None)
    args = parser.parse_args()

    model = args.model or os.environ.get("OPENAI_COMPATIBLE_MODEL", "claude-text")
    api_base = args.api_base or os.environ.get("OPENAI_COMPATIBLE_ENDPOINT", "http://172.30.1.62:6190") + "/v1"
    api_key = args.api_key or os.environ.get("OPENAI_COMPATIBLE_API_KEY", "no-key")
    langs = [l.strip() for l in args.langs.split(",")]

    problems_dir = _BASE / "problems"
    ref_path = _BASE.parent / "ai-proof" / "protocol" / "bmb_reference.md"
    reference = ref_path.read_text(encoding="utf-8") if ref_path.exists() else ""

    dirs = sorted(d for d in problems_dir.iterdir() if d.is_dir() and d.name[0].isdigit())

    if args.pilot:
        dirs = [d for d in dirs if int(d.name.split("_")[0]) in (1, 4, 22)]
    elif args.problems:
        nums = set(int(x) for x in args.problems.split(","))
        dirs = [d for d in dirs if int(d.name.split("_")[0]) in nums]
    elif args.category:
        filtered = []
        for d in dirs:
            mf = d / "metadata.json"
            if mf.exists():
                m = json.loads(mf.read_text())
                if m.get("category") == args.category:
                    filtered.append(d)
        dirs = filtered
    elif not args.all:
        print("Specify --pilot, --category, --all, or --problems")
        return 1

    if args.dry_run:
        print(f"DRY RUN — {len(dirs)} problems × {len(langs)} langs × {args.runs} runs = {len(dirs)*len(langs)*args.runs} total")
        print(f"Languages: {langs}")
        print(f"Model: {model}")
        return 0

    from bmb_ai_bench.runner.llm_client import LlmClient
    llm = LlmClient(model=model, base_url=api_base, api_key=api_key)

    out_dir = Path(args.output) if args.output else _BASE / "results" / f"crosslang-{datetime.date.today().isoformat()}"
    out_dir.mkdir(parents=True, exist_ok=True)

    all_results = []
    for run_id in range(1, args.runs + 1):
        for lang in langs:
            print(f"\n=== Run {run_id}/{args.runs} — {lang.upper()} ===")
            for d in dirs:
                pid = d.name
                rf = out_dir / f"{pid}_{lang}_run{run_id}.json"
                if rf.exists():
                    print(f"  SKIP {pid}/{lang}")
                    all_results.append(json.loads(rf.read_text()))
                    continue

                print(f"  {pid} [{lang}]...", end=" ", flush=True)
                try:
                    result = run_problem_lang(d, lang, llm, reference)
                    status = f"PASS({result.loop_count})" if result.final_correct else f"FAIL({result.loop_count})"
                    print(status)
                    data = {
                        "problem_id": pid, "lang": lang, "run_id": run_id,
                        **asdict(result),
                    }
                    rf.write_text(json.dumps(data, indent=2))
                    all_results.append(data)
                except Exception as e:
                    print(f"ERROR: {e}")
                    data = {"problem_id": pid, "lang": lang, "run_id": run_id,
                            "final_correct": False, "loop_count": -1, "error": str(e)}
                    all_results.append(data)

    # === Analysis ===
    from collections import defaultdict
    stats = defaultdict(lambda: {"total": 0, "passed": 0, "loops": []})
    for r in all_results:
        lang = r.get("lang", "?")
        stats[lang]["total"] += 1
        if r.get("final_correct"):
            stats[lang]["passed"] += 1
            stats[lang]["loops"].append(r["loop_count"])

    print(f"\n{'='*70}")
    print(f"CROSS-LANGUAGE COMPARISON — {model}")
    print(f"Problems: {len(dirs)} × {args.runs} runs")
    print(f"{'='*70}")
    print(f"{'Language':<10} {'Pass':>6} {'Total':>6} {'Rate':>8} {'Median':>8} {'Avg':>8}")
    print(f"{'-'*10} {'-'*6} {'-'*6} {'-'*8} {'-'*8} {'-'*8}")
    for lang in langs:
        s = stats[lang]
        rate = s["passed"] / s["total"] * 100 if s["total"] else 0
        loops = s["loops"]
        med = sorted(loops)[len(loops)//2] if loops else 0
        avg = sum(loops)/len(loops) if loops else 0
        print(f"{lang:<10} {s['passed']:>6} {s['total']:>6} {rate:>7.1f}% {med:>8} {avg:>8.2f}")
    print(f"{'='*70}")

    # Save summary
    summary = {
        "date": datetime.date.today().isoformat(),
        "model": model, "problems": len(dirs), "runs": args.runs,
        "languages": langs,
        "results": {lang: {
            "total": stats[lang]["total"],
            "passed": stats[lang]["passed"],
            "success_rate": stats[lang]["passed"] / stats[lang]["total"] if stats[lang]["total"] else 0,
            "median_loops": sorted(stats[lang]["loops"])[len(stats[lang]["loops"])//2] if stats[lang]["loops"] else 0,
            "avg_loops": sum(stats[lang]["loops"])/len(stats[lang]["loops"]) if stats[lang]["loops"] else 0,
        } for lang in langs},
    }
    (out_dir / "summary.json").write_text(json.dumps(summary, indent=2))
    print(f"\nResults saved to: {out_dir}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
