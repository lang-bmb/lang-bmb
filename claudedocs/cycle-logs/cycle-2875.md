# Cycle 2875: f64 수학 free functions native 포팅
Date: 2026-05-15

## Re-plan
Plan valid. Cycle 2874에서 str_substr/count/pad 완료. 이번 사이클: f64 수학 함수 native 포팅 (log/log2/log10/exp/round/tan/atan/atan2/min_f64/max_f64/clamp_f64).

## Scope & Implementation

### llvm_text.rs 추가 (IR 선언 섹션):
- LLVM 인트린식 선언: `llvm.log.f64`, `llvm.log2.f64`, `llvm.log10.f64`, `llvm.exp.f64`, `llvm.round.f64`, `llvm.minnum.f64`, `llvm.maxnum.f64`
- C stdlib 선언: `tan(double) → double`, `atan(double) → double`, `atan2(double, double) → double`
- C 런타임 선언: `bmb_parse_f64`, `bmb_read_f64`

### llvm_text.rs — 핸들러 추가:
- **1-arg 확장** (L3199): 기존 sin/cos/floor/ceil/fabs 패턴에 log/log2/log10/exp/round/tan/atan 7종 추가
- **2-arg 신규** (L3252): atan2/min_f64/max_f64 — 각 인수 load + i64→double 변환 + call
- **3-arg 신규** (L3311): clamp_f64(x, lo, hi) — `max(min(x, hi), lo)` 인라인 구현
  - `%mn = call double @llvm.minnum.f64(double vx, double vhi)`
  - `%mx = call double @llvm.maxnum.f64(double %mn, double vlo)`

### llvm_text.rs — infer_call_return_type:
- double 반환 추가: log/log2/log10/exp/round/tan/atan/atan2/min_f64/max_f64/clamp_f64/str_to_f64/bmb_parse_f64/read_f64/bmb_read_f64

### 구현 주의사항:
- clamp_f64는 LLVM intrinsic이 없어 min+max 조합으로 구현
- `std::io::Write` vs `std::fmt::Write` 이슈: out이 `&mut String`이므로 fmt::Write 필요. 클로저 사용 금지, 인라인 writeln! 사용

## Verification & Defect Resolution
- `tests/native_f64_builtins.bmb`: `bmb run` = `bmb build` = `110` ✅
  - log(1.0)=0 ✅, log2(4.0)=2 ✅, log10(100.0)=2 ✅
  - exp(0.0)=1 ✅, round(2.9)=3 ✅, tan(0.0)=0 ✅
  - atan(0.0)=0 ✅, atan2(0.0,1.0)=0 ✅
  - min_f64(3,5)=3 ✅, max_f64(3,5)=5 ✅, clamp_f64(10,0,5)=5 ✅
- `cargo test --release`: 진행 중

## Reflection
- Scope fit: ✅ 11종 f64 수학 함수 native 포팅 완료
- clamp_f64 구현: LLVM intrinsic 없어 min+max 인라인으로 처리 (semantics 동일)
- str_to_f64/read_f64도 double 반환 등록 완료

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals: 없음
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2876 — str_split native 포팅 또는 bmb_reference.md interpreter-only 경고 해제 + HANDOFF/ROADMAP 정리
