# Cycle 2817: C/Python Reference 문서 추가 (ISSUE-crosslang-reference-asymmetry)
Date: 2026-05-13

## Re-plan
Plan valid, inherited scope — ISSUE-20260326-crosslang-reference-asymmetry 해결.
BMB는 `bmb_reference.md` (~200줄)를 프롬프트에 포함하지만 C/Python은 언어 레퍼런스 없이 실행됨 → 비대칭.

## Scope & Implementation
- `protocol/c_reference.md` 신규 (~120줄):
  - Basics, I/O (scanf/printf), Types, Control Flow
  - Dynamic arrays (malloc), Strings
  - Patterns: n items, multiple test cases, 2D array, stack, sort, tokenize
  - Common Pitfalls (long long overflow, format specifiers 등)

- `protocol/python_reference.md` 신규 (~120줄):
  - Basics, I/O (input/print/sys.stdin), Types, Control Flow
  - Lists, Dicts, Strings
  - Patterns: n items, test cases, stack, deque, defaultdict, sort
  - Common Pitfalls (integer division, reference vs copy 등)

- `scripts/run_crosslang.py` 수정:
  - 3개 reference 파일을 `references` dict로 로드
  - `_build_prompt()` — `lang == "bmb"` 조건 제거 → 모든 언어에 reference 포함
  - `run_problem_lang()` 호출 시 `references.get(lang, "")` 전달
  - 레거시 bmb_reference 경로 backward-compat 유지

## Verification & Defect Resolution
- `py scripts/run_crosslang.py --pilot --runs 1 --dry-run` → 정상 (9 total)
- `py -m pytest tests/ -x -q` → 30/30 PASS

## Reflection
**Scope fit**: ISSUE 완전 해소.
**의미**: 다음 crosslang 실험부터는 세 언어가 동등한 컨텍스트를 받는다.
**단, 기존 crosslang-2026-03-26 결과는 비대칭 조건에서 측정됨** — 이 사실을 HANDOFF에 명시해야 함.
**Philosophy drift**: 없음.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals: None
- Pending Human Decisions:
  - 공정한 비교를 위해 crosslang 실험 재실행 필요 (C/Python reference 포함 조건으로) — HUMAN
- Roadmap Revisions: ISSUE-20260326-crosslang-reference-asymmetry → 해소됨
- Next Recommendation: ISSUE-20260326-first-shot-rate-low — BMB reference 확장 (현재 200줄 → 더 많은 패턴)
