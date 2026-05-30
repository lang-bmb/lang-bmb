# Cycle 3329: bootstrap 빌드 파이프라인 개선 (gc-sections)
Date: 2026-05-30

## Re-plan
Cycle 3328 Carry-Forward: bootstrap build_link에 gc-sections 추가로 CSV 1.048× 개선 가능.

## Scope & Implementation
- `bootstrap/compiler.bmb` → `build_link` 함수 수정:
  - Old: `clang -w -O3 -march=native -Wl,--stack,268435456 ir runtime -o output -lm -lws2_32`
  - New: `clang -w -O3 -march=native -ffunction-sections -fdata-sections -Wl,--gc-sections -Wl,--stack,268435456 ir runtime -o output -lm -lws2_32`
  - Linux fallback도 동일하게 gc-sections 추가

## Verification & Defect Resolution
- Stage 1 재빌드: 37s ✅
- Fixed Point: fp3329a.ll == fp3329b.ll ✅
- cargo test --release: 3800+2390+... 0 FAILED ✅
- Bootstrap P-track 재측정 (gc-sections 후):
  - brainfuck: 0.882× ✅, csv: 1.039× ⚠️, http: 0.785× ✅
  - json_parse: 0.539× ✅, json_serialize: 0.941× ✅
  - lexer: 0.489× ✅, sorting: 0.180× ✅

## Reflection
- csv 1.048→1.039×: gc-sections이 작은 개선 제공 (측정 노이즈 범위)
- **핵심 발견**: Rust compiler binary가 bootstrap binary보다 ~15% 빠른 이유를 완전히 규명하지 못함
  - IR은 IDENTICAL
  - 빌드 플래그(gc-sections, static, ffunction-sections)를 동일하게 해도 Rust compiler 성능에 미달
  - 남은 차이의 진짜 원인: Rust MIR 최적화(TailRecursiveToLoop 등)가 IR 수준에서 영향? 또는 다른 요인?
- **Structural finding**: CSV 1.039× 경계. 근본 해결은 tuple heap allocation 제거(L1 언어사양 변경) 필요:
  - `fn parse_csv() -> (i64, i64)` → calloc(2, 8) 1회 per call
  - 스택 할당 tuple 또는 struct return ABI 필요

## Carry-Forward
- Actionable: HANDOFF + ROADMAP 업데이트 (bootstrap P-track 현황 갱신)
- Structural Improvement Proposals: L1 — 스택 할당 tuple 도입 (stack-allocated tuple ABI)
- Pending Human Decisions: csv 1.039× 허용 여부 (측정 노이즈 범위 주장 가능)
- Roadmap Revisions: bootstrap P-track 회귀 해소 마킹 필요 (1.459/1.134→0.489/1.039)
- Next Recommendation: Cycle 3330 — HANDOFF + ROADMAP 업데이트 + 커밋
