# Cycle 2646: 중첩 struct + mut struct String 필드 검증
Date: 2026-05-11

## Re-plan
Cycle 2645 Carry-Forward: 중첩 struct String 필드 검증.
스코프 확장 — mut struct field assignment(String)도 함께 검증.

## Scope & Implementation

**검증 대상**:
1. **중첩 struct (`o.inner.label`)**: 포인터 traverse → String 필드 접근
2. **mut struct field assignment**: `set b.label = "updated"` 후 출력

**중첩 테스트** (구두 검증, golden 미등록):
```bmb
let i = Inner { label: "deep", value: 7 };
let o = Outer { title: "top", inner: &i };
println(o.title);          // "top"
println(o.inner.label);    // "deep"  
println(o.inner.value);    // 7
```
결과: 모두 정상 출력 ✅ — Cycle 2645의 `~s` registry가 다단 traversal에서도 작동.

**mut struct 골든 테스트** (`test_golden_struct_str_mut.bmb`):
```bmb
let mut b = Box { label: "initial", count: 0 };
println(b.label);                  // "initial"
set b.label = "updated";
set b.count = 5;
println(b.label);                  // "updated"
println(b.count);                  // 5
```
결과: 정상 출력 ✅ — `set_field` 경로도 영향 없음 (마커는 load 시점에 push되므로 store는 무관).

**파서 주의사항**: BMB는 `b.label = x` 직접 syntax 미지원, `set b.label = x` 명시 필요. 함수 body가 다중 statement면 `{...}` 블록 필수.

## Verification & Defect Resolution

**중첩 테스트**: top + deep + 7 출력 ✅

**mut 골든 테스트**: initial + updated + 5 출력, exit 42 ✅

**cargo test --release**: ✅ 6210 passed (변경 없음)

## Reflection

**Scope fit**: Cycle 2645의 fix가 중첩 + mut 시나리오에서도 작동함을 검증.

**아키텍처 통찰**:
- String marker push는 **load 시점**에서 발생 (struct 자체가 아닌 그 결과 temp에 부여)
- 따라서 store 경로는 별도 처리 불필요
- 중첩 traversal도 각 단계에서 `~s` registry 조회로 자동 동작

**Latent defects**: 없음.

**Philosophy drift**: 없음.

**Roadmap impact**: M5 struct String 필드 사용성 완전 검증.

## Carry-Forward
- Actionable: 없음
- Structural Improvement Proposals: 없음
- Pending Human Decisions: PyPI push (로컬 커밋 완료, push 미실행)
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2647 — HANDOFF/ROADMAP 종합 갱신 (M5 dispatch + struct String 모두 반영)
