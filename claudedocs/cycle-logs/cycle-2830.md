# Cycle 2830: to_string<T> Generic Builtin 구현

Date: 2026-05-14

## Re-plan

Plan valid. Cycle 2829 carry-forward: string interpolation / `for x in vec` / while-let 조사.

**설계 조사 결과 (STEP 1)**:
- **string interpolation**: lexer에 `f"..."` 파싱 추가 필요 — 다수 컴포넌트 변경, 이번 사이클 범위 초과
- **`for x in vec`**: BMB vec은 `Value::Int` (raw pointer) — 인터프리터가 vec 핸들을 iterable로 처리 불가. `for i in 0..n` + `vec_get` 패턴이 기존 해결책 (문서화됨)
- **println generic화**: `llvm_text.rs` line 798에 `declare void @println(i64)` 하드코딩 — generic 등록 시 mono_requests `println<String>` 생성 → codegen 불가. bootstrap Stage 1 파괴 위험
- **채택**: `to_string<T>(x: T) -> String` — 순수 additive 새 함수, 기존 codegen 건드리지 않음. LLM이 자주 쓰는 패턴 지원

## Scope & Implementation

**`bmb/src/types/mod.rs`**:
- `generic_functions`에 `to_string` 등록: `to_string<T>(x: T) -> String`

**`bmb/src/interp/eval.rs`**:
- `builtin_to_string` 구현: `Value::Str` → 그대로 (따옴표 없음), `Value::StringRope` → materialize, 기타 → `format!("{other}")`
- builtins HashMap에 등록

**`bmb/tests/integration.rs`**:
- `test_interp_to_string` 추가 (5개 케이스: i64, 0, String identity, f64, concat)

**`ecosystem/bmb-ai-bench/protocol/bmb_reference.md`**:
- String Operations 섹션에 `to_string` 4개 예시 추가

## Verification & Defect Resolution

| 항목 | 결과 |
|------|------|
| 초기 빌드 | ❌ scope 오류 — `generic_functions` 로컬 변수가 line 550에 정의되는데 line 422에 삽입 시도 |
| 위치 조정 후 재빌드 | ✅ |
| `test_interp_to_string` | ✅ 1 passed |
| `cargo test --release -p bmb` (전체) | ✅ 2359 passed |

## Reflection

**Scope fit**: 완전히 충족 — generic builtin 구현 + 문서화.

**Latent defects**: 
- `to_string(f64)` 출력 형식은 Rust의 `Display` for f64에 의존 (`3.14` 출력 시 `3.14` 그대로 나옴). 단, Rust f64 Display는 trailing zero 생략 방식이 플랫폼 의존적일 수 있음 — 알고리즘 문제 ASCII 출력에는 무관.
- `to_string(bool)` → `"true"/"false"` — BMB bool 출력이 `true`/`false` 이므로 자연스러움.

**Philosophy drift**: 없음. interpreter runtime builtin 추가 (Cycle 2828과 동일 패턴). Rule 6 허용 범위.

**Roadmap impact**: LLM이 `to_string(x)` 패턴 사용 시 타입 에러 없이 동작. B축 재측정 시 도움될 것.

## Carry-Forward

- **Actionable**: Cycle 2831 — `for x in vec` 대안 패턴 문서화 + `println_str` + `print_str` 직관적 사용 패턴 추가 (또는 추가 언어 갭 조사)
- **Structural Improvement Proposals**: `println` generic화는 codegen 재설계 없이는 불가 — P4로 기록 (bootstrap이 BMB 자체로 완전히 전환 후 고려)
- **Pending Human Decisions**: B축 재측정
- **Roadmap Revisions**: None
- **Next Recommendation**: Cycle 2831 — 문서 패턴 보강 또는 `f64_to_string` 추가 (to_string<f64> 결과 포맷 제어 필요 시)
