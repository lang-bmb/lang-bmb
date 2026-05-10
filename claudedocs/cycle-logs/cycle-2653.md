# Cycle 2653: M5-5 매트릭스 확장 — mut set ✅ + struct field of array ❌
Date: 2026-05-11

## Re-plan
Cycle 2652 Carry-Forward: `set arr[i] = s` 검증 + 추가 dispatch 케이스 탐색. 계획 유효.

## Scope & Implementation

**검증 시나리오** (M5-5 매트릭스 확장):

| 케이스 | 코드 | 결과 |
|--------|------|------|
| `let mut arr` + `set arr[i] = "x"` | mut + literal RHS | ✅ R: marker propagates through alloca |
| `struct S { colors: i64 }` + `p.colors[i]` | struct field as array | ❌ struct field type=i64, R: marker 결손 |

**근본 원인 분석** (3개 미지원 케이스):
| 미지원 | 근본 원인 |
|--------|---------|
| `[s; N]` var-repeat | lowering 단계에서 var의 type 미상 (val_type="var") |
| `fn() -> Array<String>` | string_fns 등록 = ret_type "String" exact match only |
| `p.field[i]` (array in struct) | struct registry는 field type을 i64/f64/string만 추적, array-of-string은 i64 저장됨 |

→ 모두 **lowering 단계의 type info 부재**가 공통 원인. 해결 = lower-time type-tracking infra (대규모 변경).

**골든 테스트 추가** (1개): `test_golden_arr_str_mut_set.bmb`

**삭제**: `test_golden_struct_arr_str.bmb` (의도된 미지원, 회귀 안 됨)

## Verification & Defect Resolution

**`cargo test --release`**: ✅ 6210 passed (변경 없음)

**골든 테스트 카운트**: 2849 → 2850 (mut_set 추가)

**M5-5 최종 매트릭스**:
| `arr[i]` 변형 | 상태 | 메커니즘 / 한계 |
|--------------|------|---------------|
| `[s1, s2]` literal | ✅ | mark_str_ptr (Cycle 2651) |
| `let arr2 = arr` alias | ✅ | R: marker propagation |
| `while ... arr[i]` loop | ✅ | block-internal R: persist |
| `let mut arr; set arr[i] = ...` | ✅ | mut alloca R: 보존 |
| `[s; N]` var repeat | ❌ | val_type="var" 추적 부재 |
| `fn() -> Array<String>` | ❌ | string_fns ret_type "String" 한정 |
| `p.field[i]` struct array | ❌ | struct field type=i64 저장 (registry 부재) |

## Reflection

**Scope fit**: M5-5 사용 가능 매트릭스 확정. 4 ✅ / 3 ❌ — 핵심 케이스 (literal, alias, mut, iteration) 모두 동작.

**Latent defects**: 없음.

**Philosophy 점검**:
- 미지원 3개 모두 같은 근본 원인 → **하나의 큰 인프라 변경**으로 해결 가능 (lower-time type tracking)
- 임기응변적 patch 거부, 근본 인프라 도입을 별도 cycle로 분리 ✅
- 사용자 워크어라운드 명확화 (literal 직접 사용 → 가능)

**Roadmap impact**:
- M5-5 핵심 ✅ 완료. 잔여 = M5-5b/c/d (모두 동일 인프라 필요)
- M6 후보: lowering 단계 type registry — bootstrap 인프라 큰 변경

**Decision Framework 적용**:
- 1순위 언어 스펙: 필요 없음 (기존 문법 충분)
- 2순위 컴파일러 구조: ✅ 본 인프라 변경이 필요한 영역 (lower-time type info)
- 3순위 최적화: 영향 없음
→ 본 cycle은 컴파일러 구조 변경 없이 가능한 모든 케이스 매핑 완료. 미지원은 컴파일러 구조 변경 사이클로.

## Carry-Forward
- Actionable: Cycle 2654 — 다른 영역으로 전환. 후보:
  - M3-2 bmb-algo 공식 벤치마크 측정 (자율 가능)
  - M5-4-A tuple destructuring + String (별도 lowering 경로)
  - HANDOFF/ROADMAP M5-5 매트릭스 갱신
- Structural Improvement Proposals: lower-time type registry (M6 후보) — `[s;N]` / fn-return-Array / struct-field-array 모두 해결
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: M5-5 ~70% (4/7 매트릭스 케이스 ✅)
- Next Recommendation: Cycle 2654 — M3-2 시작 (bmb-algo 공식 벤치 측정) 또는 M5-4-A
