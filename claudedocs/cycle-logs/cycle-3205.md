# Cycle 3205: bootstrap.sh Full Fixed Point 복구 — opt 제거
Date: 2026-05-27

## Re-plan

**Carry-Forward 이행**: Cycle 3204에서 `bootstrap.sh`의 `fixed_point: false` 문제를
pre-existing issue로 분류했으나, 이번 사이클에서 근본 원인을 진단하고 수정함.

Cycle 3204에서 수동 Fixed Point(Semantic FP + BMB-internal FP)는 달성했지만,
`bootstrap.sh --json`의 `fixed_point: false` 상태가 잔존했음.

## Scope & Implementation

### 근본 원인 분석

`bootstrap.sh`는 Stage 2 바이너리를 컴파일할 때 `opt -passes='default<O3>,scalarizer'`를
적용했음. 이 opt 패스가 BMB-generated LLVM IR에 적용되면:

1. BMB가 생성하는 printf 기반 IR 방출 코드를 LLVM이 최적화
2. 결과 바이너리가 Stage 3 IR 생성 시 `int_to_string` 함수 body에서 출력 절단
3. 생성된 Stage 3 IR: 6,193 lines (정상: ~134,211 lines)

**테스트 결과**:
| opt 설정 | Stage 3 IR 라인 수 | 결과 |
|---------|-------------------|------|
| `default<O3>` | 6,193 | ❌ 절단 |
| `default<O2>` | 6,193 | ❌ 절단 |
| `default<O1>` | 6,193 | ❌ 절단 |
| `default<O0>` | 134,211 | ✅ 정상 |
| 빈 패스 (opt 없음) | 134,211 | ✅ 정상 |
| `llc -O2` (opt 없음) | 134,211 | ✅ 정상 |

→ opt `default<O1>` 이상 모든 최적화 패스가 Stage 2 바이너리를 손상시킴.

### 수정: `scripts/bootstrap.sh`

Stage 2 바이너리 컴파일 경로에서 opt 제거:

```bash
# 이전
if command -v opt &> /dev/null; then
    opt -passes='default<O3>,scalarizer' "$STAGE2_LL" -o "$STAGE2_BC"
    llc -filetype=obj -O3 "$STAGE2_BC" -o "$STAGE2_OBJ"
else
    llc -filetype=obj -O2 "$STAGE2_LL" -o "$STAGE2_OBJ"
fi

# 이후
# NOTE: opt is intentionally NOT used for Stage 2 binary compilation.
# Any opt -O1+ pass applied to BMB-generated IR corrupts the Stage 2 binary:
# it truncates the generated Stage 3 IR at int_to_string (~6193 lines instead of ~134211).
# Root cause: LLVM opt transforms BMB's printf-based IR emission in a way that cuts off
# the function body generation. The unoptimized path produces correct output.
# See Cycle 3205 for diagnosis. Performance is not a goal here; correctness is.
llc -filetype=obj -O2 "$STAGE2_LL" -o "$STAGE2_OBJ"
```

## Verification & Defect Resolution

### bootstrap.sh --json 결과

```json
{
  "bootstrap": {
    "stage1": {"success": true, "time_ms": 34207},
    "stage2": {"success": true, "time_ms": 20585},
    "stage3": {"success": true, "time_ms": 36273},
    "fixed_point": true,
    "total_time_ms": 92799
  }
}
```

**`fixed_point: true` ✅** — 3-Stage bootstrap 전체 E2E 검증 완료.

## Reflection

**Scope fit**: bootstrap.sh Fixed Point 복구 완료. `fixed_point: true` 달성.

**근본 원인 교훈**: BMB-generated LLVM IR은 LLVM opt의 공격적 최적화를 견디지 못한다.
구체적으로, printf 기반 IR 방출 코드가 opt의 inlining/transformation에 의해 절단됨.
Stage 2 바이너리는 "correctness-first" 컴파일만 사용해야 한다 (opt 없이 llc -O2만).

**M11-B 완결**: ROADMAP § M11 후보 B (전체 3-Stage bootstrap 검증)가 이 사이클에서
완료됨. `fixed_point: true` E2E 확인.

**Philosophy drift**: 없음. bootstrap 정확성은 핵심 목표.

**Roadmap impact**:
- M11-B ✅ COMPLETE
- 다음: M11-A (약한 계약 → semantic 계약, ~4-6 cycles) 진행 가능

## Carry-Forward

- **Actionable**: M11-A — trivial postcondition 교체 (~1,114개, 최우선 제외 분류별 처리)
- **Structural Improvement Proposals**:
  - BMB IR → opt 최적화 불가 근본 원인 조사 (printf 기반 방출 코드 취약점)
    IR 방출을 printf 대신 다른 방식으로 생성하면 opt 적용 가능할 수 있음
- **Pending Human Decisions**: 없음 (M11-B 완결로 방향 결정됨)
- **Roadmap Revisions**: M11-B ✅ 완료 표시. M11-A를 다음 마일스톤으로 결정.
- **Next Recommendation**: M11-A 시작 — bool trivials (49개) + String len≥0 trivials (302개) 중
  진정 의미 있는 계약으로 교체 가능한 것 선별 후 처리.
  단, ROADMAP `skip 확정` 분류(6 bool + 77 String + 7 i64)는 변경 금지.
