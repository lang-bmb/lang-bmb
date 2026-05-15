# Cycle 2856: pow_i64 / clamp_i64 / gcd_i64
Date: 2026-05-15

## Re-plan
Carry-Forward (2855): None. Math builtins 확장 — pow/clamp/gcd 기존에 method call로만 있었음.

## Scope & Implementation

기존 현황:
- `abs`, `min`, `max` — top-level 함수 존재
- `pow`, `clamp`, `sign`, `gcd` — method call 전용 (`n.pow(e)`, `x.clamp(lo, hi)` 등)
- 참조 문서가 `pow(2, 10)` 등 free function 형식으로 잘못 기재 → 수정 필요

**pow_i64(base, exp)** (Cycle 2856, interpreter-only):
- `pow_i64(base: i64, exp: i64) -> i64`: `base.wrapping_pow(exp as u32)`, 음수 exp → error

**clamp_i64(val, lo, hi)** (Cycle 2856, interpreter-only):
- `clamp_i64(val: i64, lo: i64, hi: i64) -> i64`: Rust `clamp()` — val을 [lo, hi]로 제한

**gcd_i64(a, b)** (Cycle 2856, interpreter-only):
- `gcd_i64(a: i64, b: i64) -> i64`: Euclidean algorithm, 음수 입력 처리

변경 파일:
- `bmb/src/interp/eval.rs`: 3종 함수 구현 + 등록
- `bmb/src/types/mod.rs`: 3종 타입 서명 추가
- `bmb/tests/integration.rs`: `test_interp_math_builtins` (3케이스)
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`: Math Builtins 섹션 수정 + 3종 신규 문서화

## Verification & Defect Resolution
- test_interp_math_builtins: 3/3 통과 ✅
  - pow_i64(2, 10) = 1024 ✅
  - clamp_i64(-5, 0, 100) + clamp_i64(50, 0, 100) + clamp_i64(200, 0, 100) = 150 ✅
  - gcd_i64(48, 18) = 6 ✅
- cargo test --release 전체: **2381 passed; 0 failed** ✅ (EXIT:0)
- 기존 bmb_reference.md `pow(2, 10)` 오표기 수정 → `pow_i64(2, 10)` (사실 정정)

## Reflection
- ✅ AI 코드에서 자주 쓰이는 integer math 3종 추가
- ✅ 문서 오류 수정 (pow/clamp/sign이 free function인 것처럼 기재 → method 표기 추가)

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  * `for x in svec {}` — `Value::SvecHandle(usize)` 별도 값 타입 필요
  * 필드 복합 할당 native 지원 (codegen)
  * InterpMini `consume()` dead_code 경고 — minor
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: `str_count(s, sub)` / `str_split_at(s, idx)` 추가 (Cycle 2857) 또는 HANDOFF/ROADMAP 정리 (Cycle 2857/2858/2859/2860)
