"""Validate problem pool — ensure all solutions build and pass tests."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

# Category mapping: problem number ranges -> category name
CATEGORY_RANGES = {
    "algorithm": (1, 10),
    "system": (11, 20),
    "contract": (21, 30),
    "performance": (31, 45),
    "practical": (46, 60),
    "edge": (61, 75),
    "integration": (76, 85),
}


def _default_problems_dir() -> Path:
    return Path(__file__).resolve().parent.parent / "problems"


def _get_category(number: int) -> str:
    for cat, (lo, hi) in CATEGORY_RANGES.items():
        if lo <= number <= hi:
            return cat
    return "unknown"


def _find_bmb() -> str:
    import shutil
    for name in ["bmb", "bmb.exe"]:
        path = shutil.which(name)
        if path:
            return path
    repo = Path(__file__).resolve().parents[3]
    local = repo / "target" / "release" / ("bmb.exe" if sys.platform == "win32" else "bmb")
    if local.exists():
        return str(local)
    raise FileNotFoundError("BMB compiler not found")


def _validate_problem(problem_dir: Path, bmb_exe: str) -> dict:
    """Validate a single problem: build solution.bmb + run tests.json."""
    name = problem_dir.name
    solution = problem_dir / "solution.bmb"
    tests_file = problem_dir / "tests.json"

    if not solution.exists():
        return {"name": name, "ok": False, "error": "solution.bmb missing"}
    if not tests_file.exists():
        return {"name": name, "ok": False, "error": "tests.json missing"}

    tests = json.loads(tests_file.read_text(encoding="utf-8"))

    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)
        src = tmp / "solution.bmb"
        src.write_text(solution.read_text(encoding="utf-8"), encoding="utf-8")
        out_name = "solution.exe" if sys.platform == "win32" else "solution"
        out_path = tmp / out_name

        # Build
        try:
            proc = subprocess.run(
                [bmb_exe, "build", str(src), "-o", str(out_path), "--release"],
                capture_output=True, text=True, timeout=60, cwd=str(tmp),
            )
        except subprocess.TimeoutExpired:
            return {"name": name, "ok": False, "error": "build timeout"}

        if proc.returncode != 0:
            err = (proc.stderr + proc.stdout).strip()[:200]
            return {"name": name, "ok": False, "error": f"build failed: {err}"}

        if not out_path.exists():
            return {"name": name, "ok": False, "error": "binary not produced"}

        # Test
        for i, tc in enumerate(tests):
            stdin_data = tc.get("stdin", "")
            expected = tc.get("expected_stdout", "")
            try:
                result = subprocess.run(
                    [str(out_path)],
                    input=stdin_data, capture_output=True, text=True,
                    timeout=10, cwd=str(tmp),
                )
            except subprocess.TimeoutExpired:
                return {"name": name, "ok": False, "error": f"test {i} timeout"}

            if result.stdout != expected:
                return {
                    "name": name, "ok": False,
                    "error": f"test {i}: expected {expected!r}, got {result.stdout!r}",
                }

    return {"name": name, "ok": True, "tests_passed": len(tests), "error": None}


def run_validate(
    category: str = "all",
    problems_dir: str | None = None,
    json_output: bool = False,
) -> int:
    pdir = Path(problems_dir) if problems_dir else _default_problems_dir()
    if not pdir.exists():
        print(f"ERROR: problems directory not found: {pdir}")
        return 1

    try:
        bmb_exe = _find_bmb()
    except FileNotFoundError as e:
        print(f"ERROR: {e}")
        return 1

    # Collect problem directories
    dirs = sorted(
        d for d in pdir.iterdir()
        if d.is_dir() and d.name[0].isdigit()
    )

    # Filter by category
    if category != "all":
        filtered = []
        for d in dirs:
            num = int(d.name.split("_")[0])
            if _get_category(num) == category:
                filtered.append(d)
        dirs = filtered

    results = []
    passed = 0
    failed = 0

    for d in dirs:
        r = _validate_problem(d, bmb_exe)
        results.append(r)
        if r["ok"]:
            passed += 1
            if not json_output:
                total = r.get("tests_passed", 0)
                print(f"  [  OK  ] {r['name']} ({total} tests)")
        else:
            failed += 1
            if not json_output:
                print(f"  [ FAIL ] {r['name']}: {r['error']}")

    if json_output:
        print(json.dumps({
            "total": len(results), "passed": passed, "failed": failed,
            "results": results,
        }, indent=2))
    else:
        print(f"\n  Total: {len(results)}, Passed: {passed}, Failed: {failed}")

    return 0 if failed == 0 else 1
