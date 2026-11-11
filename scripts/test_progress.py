#!/usr/bin/env python3
"""scripts/progress.py (INF-24): every source still parses, and the comparison
with the previous report is right."""
import datetime
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import progress  # noqa: E402

TODAY = datetime.date(2026, 11, 4)


def test_every_source_parses():
    # A source whose format drifted raises SystemExit instead of an empty cell.
    secs = progress.sections(TODAY)
    assert [t for t, _ in secs] == ["1. 부트스트랩", "2. 언어 개발 수준", "3. 성능", "4. 생태계", "5. 미션 효용"]
    for title, rows in secs:
        assert rows, title
        for item, cur, src in rows:
            assert str(cur).strip(), (title, item)
            assert src, (title, item)
        items = [i for i, _, _ in rows]
        assert len(items) == len(set(items)), f"{title}: duplicate items"


def test_previous_values_round_trip():
    text = progress.block(None, TODAY)
    prev = progress.previous_values(text)
    expected = {(t, i): progress.esc(c) for t, rows in progress.sections(TODAY) for i, c, _ in rows}
    assert prev == expected
    again = progress.block(text, TODAY)
    assert "지난 보고: 이 파일의 직전 커밋 판" in again
    assert "| — | — |" not in again.split("\n", 4)[4]  # every row has a previous value


def test_change_column():
    assert progress.change(None, "5") == "—"
    assert progress.change("5", "5") == "같음"
    assert progress.change("1,000", "1,250") == "+250"
    assert progress.change("297", "277") == "-20"
    assert progress.change("0.8868", "0.9") == "+0.0132"
    assert progress.change("L0", "L1") == "바뀜"


def test_ladder_reads_the_first_unreached_rung():
    reached, nxt = progress.ladder()
    assert reached.startswith("L") and nxt.startswith("L")


def test_replace_needs_the_markers():
    try:
        progress.replace("no block here", "x")
    except SystemExit:
        pass
    else:
        raise AssertionError("replace accepted a file without markers")
    text = f"a\n{progress.BEGIN}\nold\n{progress.END}\nb"
    assert progress.replace(text, "NEW") == "a\nNEW\nb"


def test_roadmap_status_block():
    ts = {"1A-01": "done", "1A-02": "doing", "1A-03": "cancelled", "4D-01": "todo", "4D-02": "blocked"}
    block = progress.roadmap_block(ts)
    assert "| 1 최소 관통 경로 | 1A | 1/2 | `1A-02` | — | — |" in block
    assert "| 4 자립 | 4D | 0/2 | — | `4D-01` | `4D-02` |" in block
    try:
        progress.roadmap_block({"9Z-01": "todo"})
    except SystemExit:
        pass
    else:
        raise AssertionError("a phase outside STAGES was accepted")
    # the roadmap's block is the backlog's state
    text = progress.read(progress.ROADMAP)
    assert progress.replace_roadmap(text, progress.roadmap_block()) == text, \
        "docs/ROADMAP.md status block is stale: run scripts/progress.py --write-roadmap"


if __name__ == "__main__":
    for name, fn in list(globals().items()):
        if name.startswith("test_"):
            fn()
    print("progress: ok")
