#!/usr/bin/env python3
"""Tests of scripts/surface.py: what counts as a change of
the language surface."""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import surface  # noqa: E402


def state(codes, progs):
    return {"codes": codes, "programs": {n: {"src": s, "out": o} for n, (s, o) in progs.items()}}


def test_compare():
    old = state(["E0101", "E0102"], {"a": ("s1", "o1"), "b": ("s2", "o2"), "c": ("s3", "o3")})
    new = state(
        ["E0101", "E0103"],
        {
            "a": ("s1", "o1"),  # unchanged
            "b": ("s2", "oX"),  # same source, another result: a change
            "c": ("sY", "oY"),  # the source was edited: not a change
            "d": ("s4", "o4"),  # a new program: not a change
        },
    )
    assert surface.compare(old, new) == [
        ("code removed", "E0102"),
        ("code added", "E0103"),
        ("result changed", "b"),
    ], surface.compare(old, new)
    assert surface.compare(new, new) == []


def test_a_record_behind_the_sources_is_stale():
    old = state(["E0001"], {"a": ("s1", "o1"), "b": ("s2", "o2"), "gone": ("s3", "o3")})
    new = state(["E0001"], {"a": ("s1", "o1"), "b": ("s2x", "o9"), "added": ("s4", "o4")})
    # An edited entry is not a change of the surface, and not comparable.
    assert surface.compare(old, new) == []
    assert surface.stale(old, new) == ["added", "b", "gone"], surface.stale(old, new)
    assert surface.stale(new, new) == []


def test_an_exception_excuses_one_change():
    old = state(["E0001"], {"a": ("s1", "o1"), "b": ("s2", "o2")})
    new = state(["E0001"], {"a": ("s1", "o9"), "b": ("s2", "o8")})
    assert surface.unexcused(old, new) == [("result changed", "a"), ("result changed", "b")]
    old["exceptions"] = [{"kind": "result changed", "name": "a", "reason": "X-1"}]
    assert surface.unexcused(old, new) == [("result changed", "b")]


def test_programs_are_in_a_fixed_order():
    names = [n for n, _ in surface.programs(surface.ROOT)]
    assert names == [n for n, _ in surface.programs(surface.ROOT)]
    assert names[-1] == surface.LIBRARY, names[-3:]
    for c in surface.CORPORA:
        group = [n for n in names if n.startswith(c + "/")]
        assert group == sorted(group), c
    assert any(n.startswith("tests/run/") for n in names)


if __name__ == "__main__":
    test_compare()
    test_a_record_behind_the_sources_is_stale()
    test_an_exception_excuses_one_change()
    test_programs_are_in_a_fixed_order()
    print("surface tests: ok")
