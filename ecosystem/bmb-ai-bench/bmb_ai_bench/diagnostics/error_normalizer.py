"""Normalizes compiler/test error output into a standard format."""

import re

LOCATION_PATTERNS = {
    "bmb": r"-->\s*(\S+:\d+:\d+)",
    "rust": r"-->\s*(\S+:\d+:\d+)",
    "python": r'File "([^"]+)", line (\d+)',
}

ERROR_MSG_PATTERNS = {
    "bmb": r"error(?:\[E\d+\])?: (.+)",
    "rust": r"error(?:\[E\d+\])?: (.+)",
    "python": r"(\w+Error: .+)",
}


def normalize_error(raw: str, lang: str = "bmb", is_test_failure: bool = False) -> dict:
    """Return a normalised error dict."""
    error_type = "test_failure" if is_test_failure else "compile_error"

    location = ""
    loc_re = LOCATION_PATTERNS.get(lang)
    if loc_re:
        m = re.search(loc_re, raw)
        if m:
            location = f"{m.group(1)}:{m.group(2)}" if lang == "python" else m.group(1)

    normalized = raw.strip()
    if not is_test_failure:
        msg_re = ERROR_MSG_PATTERNS.get(lang)
        if msg_re:
            m = re.search(msg_re, raw)
            if m:
                normalized = m.group(1).strip()

    # Extract suggestion if present in JSONL output
    suggestion = ""
    for line in raw.split("\n"):
        line = line.strip()
        if line.startswith("{"):
            try:
                import json
                obj = json.loads(line)
                if "suggestion" in obj:
                    suggestion = obj["suggestion"]
                    break
            except Exception:
                pass

    return {
        "type": error_type,
        "normalized": normalized,
        "location": location,
        "suggestion": suggestion,
        "raw": raw,
    }
