# Cycle 2869: bmb_reference 비트 연산 + 알고리즘 패턴 문서화
Date: 2026-05-15

## Re-plan
Cycle 2868 Next Recommendation 수행: 비트 연산 문서 + 알고리즘 패턴 추가.
이전 사이클들에서 `band`/`bor`/`bxor` 키워드가 문서화되지 않았음 발견.

## Scope & Implementation
코드 변경 없음 — bmb_reference 문서 전용.

- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`:
  - Math Builtins 섹션: Bitwise operators (band/bor/bxor/<</>>) 문서 추가 + 주의: &/|/^ 기호 불가
  - 신규 패턴 섹션 2개:
    - "Palindrome check" — `str_reverse(s)` 활용
    - "Whitespace-tokenized input" — `str_split_whitespace` + `str_to_int` + vec 조합
  - Common Pitfalls: 비트 연산자 키워드 경고, str_reverse/popcount 추가

## Verification & Defect Resolution
- `cargo test --release -p bmb`: **2388 PASS** ✅ (코드 변경 없음)
- defect 없음

## Reflection
- Scope fit: ✅ `&` 연산자 사용 시도가 AI의 자주 실수하는 패턴 — 문서화로 예방
- "Whitespace-tokenized input" 패턴은 경쟁 프로그래밍에서 가장 자주 쓰이는 입력 형태
- 10 사이클 기준으로 M4 ① 언어 갭 해소 작업의 큰 묶음이 완성되어 가고 있음

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals: 없음
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2870 — HANDOFF/ROADMAP 정리 + 전체 10 사이클 커밋
