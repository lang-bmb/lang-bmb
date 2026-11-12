"""The judgement of a review (frozen with the sets that list this file in
their MANIFEST). Standard library only, no compiler, no model.

The reviewer answers with one JSON object and nothing else:

    {"verdict": "defect" | "ok", "reason": "<one line>"}

`parse` accepts that object alone, or alone inside one Markdown code fence.
Anything else is malformed: other text around it, a JSON value that is not an
object, other or missing keys, a `verdict` that is not one of the two words, a
`reason` that is not a string. A malformed answer is never read for a hint of
what the reviewer meant.

Outcome of one review, from the change's hidden label and the answer:

    label    verdict    outcome
    defect   defect     detected
    defect   ok         missed
    clean    defect     false_alarm
    clean    ok         passed
    any      malformed  malformed    (a defect is not detected; a clean change
                                      is not a false alarm; counted on its own)

Detection rate = detected / defect changes. False-alarm rate = false alarms /
clean changes. Malformed answers stay in both denominators.

`compare` is the pre-registered rule of the set's hypothesis (ADR-0078):
candidate arm against baseline arm, read with the minimum difference D of
ADR-0077 §4 in percentage points.
"""
import json
import re

VERDICTS = ("defect", "ok")
LABELS = ("defect", "clean")
FENCE = re.compile(r"\A```(?:json)?[ \t]*\n(.*)\n```\Z", re.S)


def parse(text):
    """(verdict, reason, None) or (None, None, why the answer is malformed)."""
    s = (text or "").strip()
    if m := FENCE.match(s):
        s = m.group(1).strip()
    try:
        v = json.loads(s)
    except json.JSONDecodeError:
        return None, None, "not one JSON object"
    if not isinstance(v, dict):
        return None, None, "not a JSON object"
    if set(v) != {"verdict", "reason"}:
        return None, None, "keys are not exactly `verdict` and `reason`"
    if v["verdict"] not in VERDICTS or not isinstance(v["verdict"], str):
        return None, None, "`verdict` is not \"defect\" or \"ok\""
    if not isinstance(v["reason"], str):
        return None, None, "`reason` is not a string"
    return v["verdict"], v["reason"], None


def outcome(label, verdict):
    if label not in LABELS:
        raise ValueError(f"unknown label {label}")
    if verdict is None:
        return "malformed"
    if label == "defect":
        return "detected" if verdict == "defect" else "missed"
    return "false_alarm" if verdict == "defect" else "passed"


def summary(rows):
    """Counts and rates of one arm's review rows (`label`, `outcome`,
    `view_tokens`, `message_tokens`)."""
    def count(label, out=None):
        return sum(1 for r in rows if r["label"] == label and (out is None or r["outcome"] == out))

    def mean(key):
        return sum(r[key] for r in rows) / len(rows) if rows else None

    defects, cleans = count("defect"), count("clean")
    detected, alarms = count("defect", "detected"), count("clean", "false_alarm")
    return {"reviews": len(rows), "defects": defects, "detected": detected,
            "detection_rate": detected / defects if defects else None,
            "cleans": cleans, "false_alarms": alarms,
            "false_alarm_rate": alarms / cleans if cleans else None,
            "malformed": sum(1 for r in rows if r["outcome"] == "malformed"),
            "mean_view_tokens": mean("view_tokens"), "mean_message_tokens": mean("message_tokens")}


def compare(candidate, baseline, d, ratio):
    """The hypothesis verdict from two arm summaries. `d`: minimum difference
    in percentage points. `ratio`: (p, q), the candidate may show at most p/q
    of the baseline's view tokens.

    confirmed   detection not lower by more than D, and tokens within the ratio
    refuted     detection lower by more than D
    neither     detection not lower by more than D, tokens above the ratio
    None when a rate or a token mean is missing (no defect changes, no rows).
    """
    a, b = candidate["detection_rate"], baseline["detection_rate"]
    ta, tb = candidate["mean_view_tokens"], baseline["mean_view_tokens"]
    if None in (a, b, ta, tb):
        return None
    diff = round((a - b) * 100, 9)  # rates are k/n: a difference of exactly D is within D
    out = {"detection_diff_pp": diff, "d_pp": d, "token_ratio": ta / tb if tb else None,
           "max_token_ratio": ratio[0] / ratio[1],
           "message_token_ratio": candidate["mean_message_tokens"] / baseline["mean_message_tokens"]}
    if diff < -d:
        out["verdict"] = "refuted"
    elif ta * ratio[1] <= tb * ratio[0]:
        out["verdict"] = "confirmed"
    else:
        out["verdict"] = "neither"
    return out
