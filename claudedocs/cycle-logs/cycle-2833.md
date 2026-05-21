# Cycle 2833: `str_split` + `svec_*` Builtins 구현

Date: 2026-05-14

## Re-plan

Plan valid. Cycle 2832 carry-forward: `split(s, delim)` builtin (P3) 구현.
기존 Cycle 2828 string builtins 패턴 따름 (interpreter-only + type registration).

## Scope & Implementation

**접근법 결정**: `str_split(s, delim) -> i64` 반환 opaque handle.
Thread-local `SVEC_REGISTRY: Vec<Option<Vec<String>>>` 레지스트리 사용 — 안전한 Rust, unsafe 없음.

**추가 builtins**:
- `str_split(s, delim) -> i64`: 구분자로 분리, 레지스트리에 저장, 핸들 반환. 빈 구분자 = 문자 단위 분리.
- `svec_len(handle) -> i64`: 분리된 문자열 개수
- `svec_get(handle, idx) -> String`: 인덱스별 문자열 반환
- `svec_free(handle)`: 메모리 해제 (레지스트리 슬롯을 None으로)

**변경 파일**:
- `bmb/src/interp/eval.rs`: `SVEC_REGISTRY` thread_local + 4개 builtin 함수 구현 + `register_builtins` 등록
- `bmb/src/types/mod.rs`: 4개 builtin 타입 서명 등록
- `bmb/tests/integration.rs`: `test_interp_str_split` (4케이스, 2360th test)
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`: str_split 섹션 추가 + Pitfalls 갱신

**발견한 BMB 문법 주의사항 (Cycle 2833 확인)**:
1. 함수 블록 문법: `fn f() -> T = { STMT; FINAL_EXPR };` — `=`와 `};` 필수
2. Unit 반환 함수 호출: `svec_free(parts);` ← 파서 오류. `let _f = svec_free(parts);` 사용
3. `str_contains()` 반환은 `i64` (bool 아님) — `if str_contains(s, sub) == 1 {}` 형태 필요

## Verification & Defect Resolution

| 항목 | 결과 |
|------|------|
| `cargo test --release -p bmb` | ✅ 2360 passed |
| `cargo clippy -p bmb -- -D warnings` | ✅ |
| `test_interp_str_split` 4케이스 | ✅ |

**수정된 결함**:
- 테스트에서 잘못된 블록 문법 (`{ }` → `= { };`) 수정
- Unit 반환 standalone 표현식 (`svec_free(parts);`) → `let _f = ...` 패턴 수정
- `if str_contains(...)` → `if str_contains(...) == 1` 수정

## Reflection

**Scope fit**: 완전 충족. `str_split` + `svec_*` 4종 구현 + 테스트 + 문서.

**Latent defects**: 없음. BMB 문법 오류들은 이 사이클에서 모두 발견·수정.

**Structural improvement**: `svec_free` 없이 사용해도 레지스트리가 thread_local이므로 메모리 누수 없음(프로세스 종료 시 자동 정리). interpreter-only 특성상 허용 범위.

**Roadmap impact**: M4 언어 갭 ① 목록 진척. 다음: `while let Some(x) = ...` 또는 string interpolation(고복잡도).

**발견 사항 (CLAUDE.md 보강용)**:
- BMB 블록 함수 문법 명시: `fn f() -> T = { STMTS; FINAL };` — 기존 CLAUDE.md 예시에 없음
- `str_contains` 반환이 `i64`이므로 if 조건에는 `== 1` 비교 필요 — bmb_reference 패턴 보강 권장

## Carry-Forward

- **Actionable**: Cycle 2834 — `while let Some(x) = opt {}` 패턴 또는 추가 언어 갭 조사
- **Structural Improvement Proposals**: bmb_reference에 `if i64_returning_fn() == 1 {}` 패턴 명시 (현재 `str_contains`가 bool 아닌 i64임이 사용자에게 혼란 야기 가능)
- **Pending Human Decisions**: B축 재측정 (API key 필요)
- **Roadmap Revisions**: ROADMAP.md ① 항목에 `str_split + svec_*` 추가 필요
- **Next Recommendation**: Cycle 2834 — ROADMAP 갱신 후 다음 언어 갭 선택
