# Cycle 2863: str_to_f64 / read_f64 / str_lines 신규 빌틴
Date: 2026-05-15

## Re-plan
Cycle 2862 Carry-Forward: 없음. Roadmap M4 ① 언어 갭 해소 계속.
새 사이클 방향: float I/O + 라인 분할 갭 해소 (str_to_f64 / read_f64 / str_lines).

## Scope & Implementation
- `bmb/src/interp/eval.rs`:
  - `builtin_str_to_f64(s: String) -> Value::Float` — `parse::<f64>()` 실패 시 0.0 반환
  - `builtin_read_f64() -> Value::Float` — stdin readline → parse::<f64>()
  - `builtin_str_lines(s: String) -> Value::SvecHandle` — `\n` split + strip `\r`, SVEC_REGISTRY push
  - 3종 builtin 등록: `read_f64`, `str_to_f64`, `str_lines`
- `bmb/src/types/mod.rs`:
  - `str_to_f64` → `(String) -> F64`
  - `read_f64` → `() -> F64`
  - `str_lines` → `(String) -> SvecHandle`
- `bmb/tests/integration.rs`: `test_interp_str_to_f64_str_lines` 신규 (4개 assert)
  - str_to_f64("3.14") → 3.14
  - str_to_f64("42") → 42.0
  - str_lines("a\nb\nc") 길이 → 3
  - str_lines 첫 element str_len → 5

## Verification & Defect Resolution
- `cargo test --release -p bmb`: **2384 PASS** ✅
- defect 없음

## Reflection
- Scope fit: ✅ float 파싱 + 라인 분할 모두 통과
- str_lines는 SvecHandle 반환으로 for-in 자동 지원 (Cycle 2861-2862 인프라 활용)
- read_f64는 stdin readline 기반 — 인터프리터 I/O 테스트 불가하므로 통합 테스트 생략
- Roadmap: M4 ① 언어 갭 계속 해소 중

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals: 없음
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2864 — 추가 언어 갭 발굴 (str_starts_with/ends_with, read_line, i64_to_str 등 미지원 패턴)
