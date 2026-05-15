# Cycle 2858: dead_code 경고 제거 + ROADMAP 갱신
Date: 2026-05-15

## Re-plan
Carry-Forward (2857): `InterpMini::consume()` dead_code 경고 정리. 동시에 ROADMAP 업데이트.

## Scope & Implementation

**InterpMini::consume() 제거** (Cycle 2858):
- Cycle 2848에서 InterpMini 구현 시 `consume()` 메서드 추가됐으나 코드 어디에서도 호출 안 됨
- `fn consume(&mut self) -> Option<char>` 6줄 완전 제거
- `expect()` 메서드로 충분 (pos를 consume의 skip_ws + advance 역할 대신 처리)
- 빌드 후 dead_code 경고 0건 확인 ✅

**ROADMAP.md 갱신**:
- 최신 갱신 헤더: Cycles 2851-2858 요약
- M4 준비 태스크에 M4-12 (API 완성 2851-2858) 추가
- 2382 tests ✅ 기록

변경 파일:
- `bmb/src/ast/expr.rs`: `InterpMini::consume()` 제거
- `claudedocs/ROADMAP.md`: 헤더 + M4-12 추가

## Verification & Defect Resolution
- cargo build --release: **경고 없음** ✅ (dead_code 경고 제거 확인)
- cargo test --release 전체: **2382 passed; 0 failed** ✅ (EXIT:0)

## Reflection
- ✅ Cycles 2851-2858 세션에서 총 7 사이클 (API+보간+수학+문자열) + 1 정리 사이클 완성
- ✅ 2375 → 2382 (+7 integration tests), 0 failures
- 구현 총계:
  - str_hashmap: delete, update (11종 완성)
  - str: to_upper, to_lower, char_at, count, pad_left, pad_right (6종 신규)
  - vec: remove, reverse, fill (3종 신규 → 19종 완성)
  - svec: sort, contains, remove, clear (4종 신규 → 10종 완성)
  - interp: {fn_call(args)} 보간 (InterpMini 확장)
  - math: pow_i64, clamp_i64, gcd_i64 (3종 신규)

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  * `for x in svec {}` — `Value::SvecHandle(usize)` 별도 값 타입 필요 (별도 사이클)
  * 필드 복합 할당 native 지원 (codegen)
- Pending Human Decisions: None
- Roadmap Revisions: ROADMAP.md M4-12 추가 완료
- Next Recommendation: HANDOFF.md 갱신 + git commit (Cycle 2859/2860)
