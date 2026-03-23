"""Remove pre/post contract clauses from BMB source code.

Used by the H1 experiment to compare AI performance on BMB+contract vs BMB-contract.
"""


def strip_contracts(source: str) -> str:
    """Remove pre/post contract clauses from BMB source code."""
    lines = source.split("\n")
    result = []
    i = 0
    while i < len(lines):
        stripped = lines[i].strip()
        if stripped.startswith("pre ") or stripped.startswith("post "):
            i += 1
            while i < len(lines) and lines[i].strip().startswith("and "):
                i += 1
            continue
        result.append(lines[i])
        i += 1
    return "\n".join(result)
