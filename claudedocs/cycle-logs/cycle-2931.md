# Cycle 2931: http_parse flat 재작성 + str_data P0 버그 수정
Date: 2026-05-19

## Re-plan
Cycle 2930 Next Recommendation: http_parse flat 단일함수 재작성 (csv_parse v2 패턴).
추가 발견: str_data P0 버그 (literal 변수에서 crash) → P0이므로 Rule 6 예외 적용, Rust 수정.

## Scope & Implementation

### 1. http_parse flat 재작성

#### 설계
- `fn parse_http_flat(data: String) -> i64`: 단일 함수, `str_data(data)` 한 번 호출 후 compound-cond while 루프
- 요청줄(request line) 스킵 — C와 동일하게 (기존 BMB는 request line 파싱 포함했음)
- `@inline fn is_content_length(ptr, pos)`: 14자 case-insensitive 비교를 compound-and로 인라인
- `@inline fn tol(c)`: tolower 인라인
- `run_benchmark(r1..r5: String, iters, acc, i)`: 기존과 동일 구조 (8 params)

#### str_data P0 버그 발견 (중단)

첫 시도: `main()`에서 `str_data(r1)` 호출 → STATUS_ACCESS_VIOLATION (exit -1073741819).

**근본 원인 분석**:
- `let r1 = "hello world"` 직후 `str_data(r1)` → MIR에서 `r1` = `Constant::String("hello world")` (상수 전파)
- `format_operand_with_strings(Constant::String)` → `@.str.0` (raw bytes) 반환
- `str_data` 에미터: `GEP {ptr,i64,i64}, ptr @.str.0, 0, 0` → 문자열 바이트("hello wo") 를 포인터로 해석
- `load_u8("hello wo" as ptr)` → SEGV

**str_data on function parameter 동작 이유**:
- 함수 파라미터 `s: String`은 `Operand::Place(p)` + `local_names.contains(p.name)` → alloca로부터 load → 정상 BmbString ptr
- 로컬 literal 변수는 상수 전파되어 `Constant::String` → raw bytes

### 2. P0 버그 수정 (llvm_text.rs)

**변경 위치**: `bmb/src/codegen/llvm_text.rs:5693`

```rust
// Before: falls through to format_operand_with_strings → @.str.0 (bytes, WRONG)
_ => self.format_operand_with_strings(&args[0], string_table),

// After: handles Constant::String explicitly → @.str.0.bmb (struct, CORRECT)
Operand::Constant(Constant::String(s)) => {
    if let Some(global_name) = string_table.get(s) {
        format!("@{}.bmb", global_name)
    } else {
        self.format_operand_with_strings(&args[0], string_table)
    }
}
_ => self.format_operand_with_strings(&args[0], string_table),
```

P0 기준: `str_data`가 literal에서 crash = 잘못된 IR 생성 (P0 correctness bug).
Rule 6 예외: 최소 패치, 해당 버그만 수정.

### 3. http_parse flat v1 (수정 후)

설계 변경: `main()`에서 `str_data` 호출 제거 → `parse_http_flat(data: String)`으로 유지.
이유: 13-param `run_benchmark` 과부하 없이도 동작; `str_data`는 function param에서 이미 정상.

### 측정 결과 (11회 median)

| 버전 | BMB (µs) | C GCC (µs) | 비율 |
|------|----------|-----------|------|
| Cycle 2924 multi-function | ~2906 | ~2451 | ~1.186× |
| **flat v1 (compound-cond)** | **2542** | **2313** | **1.099×** |

- **BMB flat vs 이전 BMB: 12.5% 개선** (2542 vs 2906)
- **비율 개선: 1.186× → 1.099×** (7% 개선)

**주의**: 원래 BMB는 request line 파싱(method/path/version)을 포함했으나 C는 스킵. flat 버전은 C와 동등하게 스킵. 순수 알고리즘 동등화 후 비율임.

### 체크섬 검증
- BMB flat: 160002980000
- C GCC: 160002980000
- 체크섬 동일 ✓

## Verification & Defect Resolution

### cargo test
- 3778 + 2388 + 47 + 13 + 23 = **6249 passed, 0 FAILED** ✅

### str_data P0 수정 검증
- `let s = "hello,world"; let p = str_data(s);` → comma count = 1, len = 11 ✓

### Fixed Point 영향
- Rust text backend 수정 (llvm_text.rs) → bootstrap 컴파일러 불변 (bootstrap은 text backend 미사용)
- Stage 2/3 재검증 불필요 (bootstrap compiler.bmb 변경 없음)

## Reflection

### Scope fit
- ✅ http_parse flat 재작성 완료 (1.099×)
- ✅ str_data P0 버그 수정 — literal에서 crash 해소
- ⚠️ ≤1.05× 목표 미달성 — i64 구조적 한계 (~10% remaining)

### 핵심 발견
1. **str_data on literal bug**: `Constant::String` → raw bytes → SEGV. 모든 BMB 코드에서 `str_data(literal)` 패턴이 이전에는 crash했음
2. **compound-cond 효과**: http_parse에서도 csv_parse와 유사하게 ~7% 비율 개선
3. **request line 파싱**: 기존 BMB가 C보다 더 많은 작업을 수행함. 동등화 후 비율이 개선됨

### Philosophy 평가
- Principle 2 (Workaround 금지) 준수: P0 버그 근본 수정 (Rust backend)
- Rule 6 P0 예외 적용: 6줄 최소 패치, 관련 버그만 수정, 주변 정리 없음

## Carry-Forward
- Actionable: **다음 언어 갭 해소** — ROADMAP 권장 (HANDOFF 기준). 성능 ≤1.05× = i32 타입 추가 필요 (Pending Human Decision 유지)
- Structural Improvement Proposals:
  1. **str_data literal 테스트 추가**: BMB 골든 테스트에 literal str_data 케이스 없음 — Cycle 2932에 추가 권장
  2. **inttoptr + GEP → GEP from base**: Cycle 2929 제안 — ptr alias analysis 개선 가능
- Pending Human Decisions: i32 타입 추가 (≤1.05× 유일한 경로) — 자율 범위 초과
- Roadmap Revisions: http_parse 1.186× → 1.099× 갱신 필요 (ROADMAP.md)
- Next Recommendation: Cycle 2932 — str_data literal 테스트 추가 + HANDOFF/ROADMAP 갱신 + 언어 갭 작업 시작
