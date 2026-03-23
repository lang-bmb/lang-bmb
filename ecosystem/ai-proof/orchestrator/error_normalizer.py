"""Normalizes compiler/test error output into a standard format."""

import re

LOCATION_PATTERNS = {
    "bmb": r"-->\s*(\S+:\d+:\d+)",
    "rust": r"-->\s*(\S+:\d+:\d+)",
    "python": r'File "([^"]+)", line (\d+)',
}

# Patterns to extract the main error message from raw compiler output.
ERROR_MSG_PATTERNS = {
    "bmb": r"error(?:\[E\d+\])?: (.+)",
    "rust": r"error(?:\[E\d+\])?: (.+)",
    "python": r"(\w+Error: .+)",
}


def normalize_error(
    raw: str, lang: str = "bmb", is_test_failure: bool = False
) -> dict:
    """Return a normalised error dict.

    Keys:
        type: "compile_error" or "test_failure"
        normalized: the main error message stripped of noise
        location: source location string (e.g. "file.bmb:5:3") or ""
        raw: the original unmodified string
    """
    error_type = "test_failure" if is_test_failure else "compile_error"

    # --- extract location ---
    location = ""
    loc_re = LOCATION_PATTERNS.get(lang)
    if loc_re:
        m = re.search(loc_re, raw)
        if m:
            if lang == "python":
                # groups: filename, line number
                location = f"{m.group(1)}:{m.group(2)}"
            else:
                location = m.group(1)

    # --- extract normalised message ---
    normalized = raw.strip()
    if is_test_failure:
        # For test failures keep the whole message as-is.
        pass
    else:
        msg_re = ERROR_MSG_PATTERNS.get(lang)
        if msg_re:
            m = re.search(msg_re, raw)
            if m:
                normalized = m.group(1).strip()

    return {
        "type": error_type,
        "normalized": normalized,
        "location": location,
        "raw": raw,
    }
