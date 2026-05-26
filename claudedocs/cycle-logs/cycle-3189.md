# Cycle 3189: M10 착수 — unused_binding 781→64 (−717, 91.8% 감소)
Date: 2026-05-26

## Re-plan
Plan valid. M9 COMPLETE 상태에서 M10 방향 결정: "Warning Zero" — `bootstrap/compiler.bmb`의 2,839개 경고를 단계적으로 0으로 감소. 
첫 번째 타겟: `unused_binding` 781개 (전체의 27.5%).

## Scope & Implementation

### M10 정의
- **목표**: `bmb check bootstrap/compiler.bmb` warnings 2,839 → 0
- **Phase 1 (이번 Cycle)**: unused_binding 781개 처리
- **Phase 2**: chained_comparison (758), non_snake_case (108), unused_return_value (36) 등

### 알고리즘 개발
`scripts/fix_unused_bindings.py` 작성 (반복 개선):

1. **1차 알고리즘** (실패): warning `start` byte 근처 ±5000 bytes rfind → 잘못된 위치 rename (t4 오례)
2. **2차 알고리즘** (채택): warning `start` byte를 함수 경계로 삼아 `content[:start_byte].rfind('let var ')` + 함수 바디 내 사용 여부 word-boundary 검증
3. **핵심 발견**: warning `start` byte ≈ 해당 함수의 끝 (다음 함수 시작 위치)

### 적용 결과 (반복 패스)
| 패스 | renames | 잔여 unused_binding |
|------|---------|---------------------|
| 1차 | 475 | 306 |
| 2차 | 137 | 169 |
| 3차 | 45 | 124 |
| 4-8차 | 15+9+3+3+1 | 97 → 수렴 |
| 파라미터 처리 추가 | 12+5 | 80 |
| 추가 패스 | 5+3+3+3+1 | 64 → 수렴 |

### 남은 64개 분석
| 변수 | 개수 | 원인 |
|------|------|------|
| `sb` | 22 | BMB lint semantic: sb_push(sb,...)는 사용이지만 "builder not consumed"로 판단 |
| `cur_exit_label` | 18 | 함수 내에서 실제 사용됨 (do_step/step_expr에 전달) |
| `item` | 10 | sb와 유사한 semantic lint |
| `loop_exit`, 기타 | 14 | 함수 내 사용되는 변수 |

### 함수 파라미터 처리
5개 함수 (step_bool, step_binop_final, step_unary_final, step_array_index_final, step_set_index_final)의
`cur_exit_label` → `_cur_exit_label` 파라미터 직접 rename.
`mapping` (2), `cleanup_file` (1) 파라미터도 처리.

## Verification & Defect Resolution

### 검증 결과
- `bmb check` warnings: 2,839 → **2,121** (−718, −25.3%)
- `bmb check` unused_binding: 781 → **64** (−717, **−91.8%**) ✅
- `bmb check` errors: **0** ✅
- `cargo test --release`: **6278 tests, 0 failed** ✅
- Stage 1 bootstrap: **✅** (`./target/release/bmb build bootstrap/compiler.bmb -o bootstrap/compiler.exe`)

### Stage 2/3 Fixed Point
Stage 2 (`./bootstrap/compiler.exe build compiler.bmb --emit-ir`) 실행 시 `fn SEP() -> String` at line 12 파싱 오류 발생.
**원인 확인**: 원본 git HEAD (M10 이전)에서도 동일 오류 재현 → **M10과 무관한 기존 이슈**.
(bootstrap binary가 postcondition 있는 함수 구문을 처리 못 하는 선재 문제. M10 작업 대상 아님.)

## Reflection
- **Scope fit**: 781개 unused_binding 경고의 91.8%를 자동화 스크립트로 처리. 나머지 8.2%는 BMB lint의 semantic 특성 (builder handle, 전달 전용 파라미터)으로 단순 prefix만으로 해결 불가.
- **알고리즘 품질**: warning `start` byte의 의미 (≈ 함수 끝 경계)를 발견하여 정확한 위치 탐색 구현. 함수 경계 내 word-boundary 검증으로 false positive 방지.
- **Latent defects**: Stage 2 bootstrap 미동작은 pre-existing issue (postcondition 문법 지원 부재). M10 작업 독립.
- **Philosophy drift**: None — warning 제거는 코드 품질 향상, IR/semantic 변경 없음.
- **Roadmap impact**: M10 Phase 1 진행 중. chained_comparison (758) 및 non_snake_case (108) 가 다음 타겟.

## Carry-Forward
- **Actionable**: 
  - 잔여 `unused_binding` 64개 중 `sb`/`item` 케이스: BMB lint false positive 여부 확인 필요 (lint semantics 이해)
  - `cur_exit_label` 18개: 함수 내 실제 사용이지만 lint가 unused로 판단하는 이유 분석
- **Structural Improvement Proposals**: 
  - `scripts/fix_unused_bindings.py`는 재실행 가능한 멱등 스크립트로 향후 재사용 가능
  - Stage 2 bootstrap 복구: postcondition 구문 지원을 bootstrap parser에 추가 (별도 이슈)
- **Pending Human Decisions**: Stage 2 bootstrap 복구 우선순위
- **Roadmap Revisions**: M10 Phase 1 진행 → ROADMAP에 M10 "Warning Zero" 추가 필요
- **Next Recommendation**: M10 Phase 2 — `chained_comparison` 758개 처리 (각 체인을 `match` 표현으로 변환 또는 suppress)
