#!/usr/bin/env python3
"""Rules of scripts/backlog.py (INF-30): what moves to the archive and where."""
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import backlog  # noqa: E402

H = "\n".join(backlog.HEADER)
ACTIVE = f"""# 백로그

## 지금 할 일

### 실행 계획(셋째)

`1A-01` 다음.

### 실행 계획(둘째)

둘째 글.

### 실행 계획(첫째)

첫째 글.

## 1A 프론트엔드

{H}
| 1A-01 | 렉서 | 토큰 | — | ✅ `abc1234` |
| 1A-02 | 파서 | 트리 \\| 오류 | 1A-01 | 🔄 |
| ~~1A-03~~ | ~~버림~~ | 사유 | — | 취소 |

## 통과 기준 추적

| 단계 | 통과 기준 | 확인 태스크 |
| --- | --- | --- |
| 1A | 기준 | `1A-01` ✅ |

## 2단계 (M2)

### 2A 측정

{H}
| 2A-01 | 측정 | 결과 | 1A-02 | ⏳ |
| 2A-02 | 표 | 표 | — | ✅ 시험 `t` |
"""


def setup(t):
    d = Path(t) / "dev-docs"
    d.mkdir()
    (d / "BACKLOG.md").write_text(ACTIVE, encoding="utf-8")
    return Path(t)


def main():
    assert backlog.task("| 1A-02 | 파서 | 트리 \\| 오류 | 1A-01 | 🔄 |") == ("1A-02", False, "🔄")
    assert backlog.task("| ~~1A-03~~ | a | b | — | 취소 |")[1]
    assert backlog.task("| 1A | 기준 | `1A-01` ✅ |") is None
    assert backlog.group_of("통과 기준 추적") is None

    with tempfile.TemporaryDirectory() as t:
        root = setup(t)
        before = {l for l in backlog.text(root).splitlines() if backlog.task(l)}
        assert backlog.archive(root, write=False) == 4  # three rows, one plan
        assert backlog.archive(root) == 4
        active = (root / "dev-docs" / "BACKLOG.md").read_text(encoding="utf-8")
        # finished rows left; rows in progress, plain text and other tables stayed
        assert "1A-01 | 렉서" not in active and "1A-03" not in active and "2A-02" not in active
        assert "| 1A-02 |" in active and "| 2A-01 |" in active and "| 1A | 기준 |" in active
        assert "실행 계획(셋째)" in active and "실행 계획(둘째)" in active and "첫째" not in active
        # no row is lost or doubled, and the archive keeps the heading
        assert {l for l in backlog.text(root).splitlines() if backlog.task(l)} == before
        s1 = (root / "dev-docs" / "backlog" / "stage1.md").read_text(encoding="utf-8")
        assert "## 1A 프론트엔드" in s1 and s1.count("| 1A-01 |") == 1 and "~~1A-03~~" in s1
        s2 = (root / "dev-docs" / "backlog" / "stage2.md").read_text(encoding="utf-8")
        assert "## 2A 측정" in s2 and "| 2A-02 |" in s2
        plans = (root / "dev-docs" / "backlog" / "plans.md").read_text(encoding="utf-8")
        assert "## 실행 계획(첫째)" in plans and "첫째 글." in plans
        # a second run moves nothing
        assert backlog.archive(root, write=False) == 0
        # a row finished later joins its heading; a newer plan pushes the oldest kept one out
        p = root / "dev-docs" / "BACKLOG.md"
        text = p.read_text(encoding="utf-8").replace("| 1A-01 | 🔄 |", "| 1A-01 | ✅ `def5678` |")
        p.write_text(text.replace("### 실행 계획(셋째)", "### 실행 계획(넷째)\n\n넷째 글.\n\n### 실행 계획(셋째)"),
                     encoding="utf-8")
        assert backlog.archive(root) == 2
        s1 = (root / "dev-docs" / "backlog" / "stage1.md").read_text(encoding="utf-8")
        rows = [l for l in s1.splitlines() if backlog.task(l)]
        assert [backlog.task(l)[0] for l in rows] == ["1A-01", "1A-03", "1A-02"]
        assert s1.count("## 1A 프론트엔드") == 1
        plans = (root / "dev-docs" / "backlog" / "plans.md").read_text(encoding="utf-8")
        assert plans.index("실행 계획(둘째)") < plans.index("실행 계획(첫째)")
        assert backlog.size(root) == len(p.read_text(encoding="utf-8"))

    # the repository's backlog is split
    assert backlog.archive(write=False) == 0
    print("ok")


if __name__ == "__main__":
    main()
