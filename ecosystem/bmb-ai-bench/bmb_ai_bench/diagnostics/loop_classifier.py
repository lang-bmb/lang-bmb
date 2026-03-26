"""Classifies each loop iteration by error type (A/B/C/D)."""

from enum import Enum


class LoopType(str, Enum):
    A_CONTRACT = "A"  # Contract/type violation (compile time)
    B_SYNTAX = "B"    # Parser-level syntax error
    C_SEMANTIC = "C"  # Semantic compile error (type mismatch, undefined, etc.)
    D_TEST = "D"      # Test failure (runtime correctness)


CONTRACT_PATTERNS = [
    "pre-condition", "post-condition", "contract", "invariant",
]

SYNTAX_PATTERNS = [
    "unexpected token", "expected ';'", "expected '}'", "expected '{'",
    "expected ')'", "expected '('", "expected token", "expected expression",
    "expected statement", "expected identifier", "expected type",
    "parse error", "syntax error", "unrecognized", "unrecognized token",
]


def classify_loop(error: dict, lang: str = "bmb") -> LoopType:
    """Classify an error dict into loop type A/B/C/D."""
    if error["type"] == "test_failure":
        return LoopType.D_TEST

    msg = error["normalized"].lower()

    if any(p in msg for p in CONTRACT_PATTERNS):
        return LoopType.A_CONTRACT

    if any(p in msg for p in SYNTAX_PATTERNS):
        return LoopType.B_SYNTAX

    return LoopType.C_SEMANTIC
