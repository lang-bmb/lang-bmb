"""Environment prerequisite checker."""

from __future__ import annotations

import json
import shutil
import subprocess
import sys


def _check_tool(name: str, version_flag: str = "--version") -> dict:
    path = shutil.which(name)
    if not path:
        return {"name": name, "ok": False, "version": None, "path": None}
    try:
        proc = subprocess.run(
            [path, version_flag],
            capture_output=True, text=True, timeout=10,
        )
        ver = (proc.stdout.strip() or proc.stderr.strip()).split("\n")[0]
    except Exception:
        ver = "unknown"
    return {"name": name, "ok": True, "version": ver, "path": path}


def _check_bmb() -> dict:
    """Check for BMB compiler."""
    for name in ["bmb", "bmb.exe"]:
        path = shutil.which(name)
        if path:
            try:
                proc = subprocess.run(
                    [path, "--version"], capture_output=True, text=True, timeout=10,
                )
                ver = (proc.stdout.strip() or proc.stderr.strip()).split("\n")[0]
            except Exception:
                ver = "unknown"
            return {"name": "bmb", "ok": True, "version": ver, "path": path}

    # Try repo-local build
    import pathlib
    repo = pathlib.Path(__file__).resolve().parents[3]  # bmb_ai_bench/ -> bmb-ai-bench/ -> ecosystem/ -> lang-bmb/
    local = repo / "target" / "release" / ("bmb.exe" if sys.platform == "win32" else "bmb")
    if local.exists():
        try:
            proc = subprocess.run(
                [str(local), "--version"], capture_output=True, text=True, timeout=10,
            )
            ver = (proc.stdout.strip() or proc.stderr.strip()).split("\n")[0]
        except Exception:
            ver = "unknown"
        return {"name": "bmb", "ok": True, "version": ver, "path": str(local)}

    return {"name": "bmb", "ok": False, "version": None, "path": None}


def run_doctor(json_output: bool = False) -> int:
    checks = [
        _check_bmb(),
        _check_tool("opt"),
        _check_tool("gcc"),
        _check_tool("rustc"),
        _check_tool("python3" if sys.platform != "win32" else "python"),
    ]

    all_ok = all(c["ok"] for c in checks)

    if json_output:
        print(json.dumps({"ok": all_ok, "checks": checks}, indent=2))
    else:
        print("bmb-ai-bench doctor")
        print("=" * 50)
        for c in checks:
            status = "OK" if c["ok"] else "MISSING"
            ver = c["version"] or ""
            print(f"  [{status:>7}] {c['name']:<10} {ver}")
        print("=" * 50)
        print(f"  Result: {'ALL OK' if all_ok else 'MISSING TOOLS — install before running'}")

    return 0 if all_ok else 1
