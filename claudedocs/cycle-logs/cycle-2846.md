# Cycle 2846: str_hashmap (String → i64 HashMap)
Date: 2026-05-14

## Re-plan
Carry-Forward (2845): Structural proposals — HashMap<String, Value>, `set x.field += e`, `{expr}` interpolation.
Discovery: `str_hashmap_*` 스텁이 이미 type checker + LLVM codegen에 선언되어 있으나 **인터프리터 구현 없음** + type signature 오류 (`String` 대신 `i64` key). 이를 완성하는 것이 우선.

## Scope & Implementation

**접근법**: `Box<HashMap<String, i64>>`를 힙에 할당하고 raw ptr을 `i64` handle로 반환하는 interpreter-only builtin 6종 구현.

변경 파일:
- `bmb/src/types/mod.rs`:
  - `str_hashmap_insert`: `(i64, i64, i64) → i64` → `(i64, String, i64) → Unit` 수정
  - `str_hashmap_get`: `(i64, i64) → i64` → `(i64, String) → i64` 수정
  - `str_hashmap_contains`: 신규 (i64, String) → i64
  - `str_hashmap_len`: 신규 (i64) → i64
- `bmb/src/interp/eval.rs`:
  - `builtin_str_hashmap_new/insert/get/contains/len/free` 구현 (6개 함수)
  - `str_key()` helper 함수 (String 값 추출, 기존 `extract_string` 이름 충돌 회피)
  - 빌트인 등록 6개
- `bmb/tests/integration.rs`:
  - `test_interp_str_hashmap` 6개 케이스: 기본 insert/get, contains, len, overwrite, absent key sentinel, word frequency pattern
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`:
  - HashMap 섹션 제목을 "i64 key" 명시
  - String HashMap 섹션 신규 (6개 builtin + get-with-default 패턴)
  - Pattern: String word frequency 신규
  - Pitfalls: str_hashmap 항목 추가 + hashmap_get 주의사항 통합

**결함 수정**: 
1. `i64::MIN` 리터럴 `-9223372036854775808` BMB 렉서 오버플로 → `v < 0` 조건으로 변경
2. `RuntimeError::runtime_error` 없음 → `RuntimeError::io_error` 사용
3. `extract_string` 함수명 충돌 → `str_key` 로 명명

## Verification & Defect Resolution
- test_interp_str_hashmap: 6/6 케이스 통과 ✅
- cargo test --release 전체: **2370 passed; 0 failed** ✅ (EXIT:0)

## Reflection
- ✅ str_hashmap 6종 builtin 완성 — 기존 스텁(v0.90.83)을 interpreter-only 완전 구현으로 업그레이드
- ✅ 기존 type signature 오류(String key가 i64로 선언) 수정 — 타입 안정성 복원
- **인사이트**: 기존 `str_hashmap_*` 스텁은 v0.90.83에서 선언만 되고 구현은 미뤄진 상태였음. 이번 Cycle에서 완성.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  * `set x.field += e` 필드 복합 할당 (현재 `set` + 단순 할당만 지원)
  * `{expr}` 복잡 표현식 지원 (현재 ident-only)
  * str_hashmap keys iterator (현재 key 나열 방법 없음)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: `{expr}` 복잡 표현식 보간 or `set x.field += e` 필드 복합 할당
