# Cycle 2862: 타입 체커 for-in svec String 추론
Date: 2026-05-15

## Re-plan
Cycle 2861 Carry-Forward: 타입 체커에서 for-in svec body의 `s : String` 추론 미지원.

## Scope & Implementation
- `bmb/src/types/mod.rs`:
  - `Type::Named("SvecHandle")` 도입 (svec_t 로컬 변수)
  - svec 관련 13개 함수 시그니처 업데이트:
    - svec_new() → `() -> SvecHandle`
    - str_split() → `(String, String) -> SvecHandle`
    - str_hashmap_keys/sorted_keys() → `(I64) -> SvecHandle`
    - svec_len/free/sort/clear → `(SvecHandle) -> ...`
    - svec_get() → `(SvecHandle, I64) -> String`
    - svec_join() → `(SvecHandle, String) -> String`
    - svec_push() → `(SvecHandle, String) -> Unit`
    - svec_contains() → `(SvecHandle, String) -> I64`
    - svec_remove() → `(SvecHandle, I64) -> Unit`
  - `Expr::For`: `Type::Named("SvecHandle") => Type::String` 추론 추가
- `bmb/tests/integration.rs`: `test_interp_for_in_svec`에 String 연산 테스트 2개 추가

## Verification & Defect Resolution
- `cargo test --release -p bmb`: **2383 PASS** ✅ (str_len(s) 타입 체크 + 런타임 실행 정상)
- defect 없음

## Reflection
- Scope fit: ✅ for s in sv { str_len(s) } 타입 체커+런타임 모두 통과
- `Type::Named("SvecHandle")`은 사용자가 `fn f() -> SvecHandle` 직접 선언 가능. 현재 interpreter-only이므로 허용.
- Roadmap: M4 ① svec 언어 갭 완전 해소.

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals: 없음
- Pending Human Decisions: 없음
- Roadmap Revisions: M4 ① SvecHandle 타입 통합 ✅
- Next Recommendation: Cycle 2863 — 새 언어 갭 발굴 (B축 재측정 실패 패턴 분석 or bmb_reference 패턴 추가)
