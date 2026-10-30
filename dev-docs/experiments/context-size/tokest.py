"""Offline token estimate (backlog 2D-25, 2D-23). Standard library only.

No model tokenizer is available offline, and the experiments must not call a
model. The estimate is the number of pre-tokenizer pieces (the GPT-style
split into letter runs, digit groups of up to three, punctuation runs and
whitespace), with letter runs longer than 6 and punctuation runs longer than
3 counted as several pieces, times one scale factor. The factor is fitted on
the only exact token counts this repository has offline: the Claude input
tokens of T1-05 (`dev-docs/experiments/view-size/results.md`) for the texts
whose bytes `bmb` still reproduces (stored, normal form, minified, L0 of the
m1, m1b and BVB seeds). `calibration()` reports the fit point by point.
"""
import math
import os
import re
import subprocess

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))

LETTERS = r"[^\W\d_]"
PIECE = re.compile(r"'(?i:[sdmt]|ll|ve|re)|[^\r\n\w]?" + LETTERS + r"+|\d{1,3}| ?[^\s\w]+[\r\n]*|\s*[\r\n]+"
                   r"|\s+(?!\S)|\s+|_+")
WORD = re.compile(r"[^\r\n\w]?" + LETTERS + r"+")
PUNCT = re.compile(r" ?[^\s\w]+[\r\n]*")


def pieces(text):
    n = 0
    for m in PIECE.finditer(text):
        p = m.group(0)
        if WORD.fullmatch(p) and len(p.strip()) > 6:
            n += math.ceil(len(p.strip()) / 6)
        elif PUNCT.fullmatch(p) and len(p) > 3:
            n += math.ceil(len(p) / 3)
        else:
            n += 1
    return n


# Calibration points: (workspace label in view-size/results.md, view).
POINTS = [(label, view) for label in ("m1 seed", "m1b seed", "bvb seed")
          for view in ("stored", "normal", "minified", "L0")]
SEEDS = {"m1 seed": "bench/m1/seed/src", "m1b seed": "bench/m1b/seed/src", "bvb seed": "bench/bvb/seed/src"}


def _bmb(bmb, *args):
    return subprocess.run([bmb, *args], capture_output=True, text=True, cwd=ROOT).stdout


def _texts(bmb, label):
    d = os.path.join(ROOT, SEEDS[label])
    fs = sorted(os.path.join(d, f) for f in os.listdir(d) if f.endswith(".bmb"))
    stored = "".join(open(f).read() for f in fs)
    normal = _bmb(bmb, "fmt", *fs)
    minified = re.sub(r"\s+", " ", normal).strip() + "\n"
    return {"stored": stored, "normal": normal, "minified": minified, "L0": _bmb(bmb, "map", "L0", *fs)}


def _measured():
    """(label, view) -> (bytes, Claude tokens) from the T1-05 table."""
    out = {}
    path = os.path.join(ROOT, "dev-docs", "experiments", "view-size", "results.md")
    for line in open(path):
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) == 7 and cells[3].isdigit():
            out[(cells[0], cells[2])] = (int(cells[3]), int(cells[4]))
    return out


_scale = [None]


def calibration(bmb):
    """Rows (label, view, bytes, Claude tokens, pieces) of the points whose
    bytes are reproduced, and the fitted scale (sum of tokens / sum of pieces)."""
    measured = _measured()
    rows = []
    for label in SEEDS:
        texts = _texts(bmb, label)
        for view, text in texts.items():
            b, t = measured[(label, view)]
            if len(text.encode()) == b:
                rows.append((label, view, b, t, pieces(text)))
    scale = sum(r[3] for r in rows) / sum(r[4] for r in rows)
    _scale[0] = scale
    return rows, scale


def tokens(text):
    """Estimated tokens of `text` (call `calibration` first)."""
    return round(pieces(text) * _scale[0])
