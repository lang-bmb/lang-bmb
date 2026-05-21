# Cycle 2680: nested struct array 검증 — 무구현 통과
Date: 2026-05-11

## Re-plan
이전 Carry-Forward (Cycle 2679): nested struct array `p.inner.tags[0]` 검증이 자율 잔여 작업.
트리거 없음. 인수받은 스코프 그대로 진행.

## Scope & Implementation

### 검증한 시나리오
1. `o.inner.tags[0]` — 2-level nested struct field array indexing
2. `o.inner.tags[i]` for-loop iter — 변수 인덱스
3. `o.mid.innermost.tags[i]` — 3-level nested struct field array

### 결과
- 모두 **별도 구현 없이 통과**. M5-5d 인프라(`get_field_ptr` + `~a` suffix)가 nested 경로를 자연 처리.
- 골든 3개 추가:
  - `test_golden_arr_str_nested_struct.bmb` → exit 42
  - `test_golden_arr_str_nested_struct_loop.bmb` → exit 42
  - `test_golden_arr_str_triple_nested.bmb` → exit 42
- `golden_tests.txt` 2857 → 2860

### 구현 분석
M5-5d가 도입한 `~a` field type suffix는 `llvm_gen_field_access` 시점에 체크된다.
nested field access (`o.inner.tags[0]`)는 두 번의 field access:
1. `o.inner` → returns Inner struct ptr
2. `(o.inner).tags` → field is `~a` typed → push_str_ptr_marker 발행
이미 합성식 처리가 재귀이므로 nested 경로 별도 처리 불필요.

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| `cargo test --release` | ✅ 6210 passed (회귀 없음) |
| Stage 1 빌드 | ✅ OK (재빌드 불필요) |
| nested 골든 3개 | ✅ 모두 exit 42 |

결함: 없음.

## Reflection

**Scope fit**: 진단 목표 달성. 보너스로 무구현 통과를 골든으로 영구 가드.

**Latent defects**: 없음 — nested 경로가 동작함을 골든으로 회귀 방지.

**Structural improvement opportunities**:
- 4-level 이상 깊이는 미테스트 (실용 부족)
- nested struct field에 `Array<i64>` / `Array<f64>` 등 다른 타입 — 다음 사이클 범위

**Philosophy drift**: 없음. M5-5d 인프라의 깨끗한 재사용 = AI-native 언어 설계 원칙 검증.

**Roadmap impact**:
- Cycle 2681-2682 "nested 구현" 작업 불필요 → 로드맵 수정 필요
- 노출된 다음 갭: `Array<i64>` / `Array<f64>` 등 String 외 타입 → Cycle 2681-2682로 흡수 가능

**User-facing quality**: N/A (컴파일러 내부 검증)

## Carry-Forward
- Actionable: 없음 (자율 작업 잘 완료)
- Structural Improvement Proposals:
  - `Array<X>` 일반화 — i64/f64/struct ptr (현재 String만)
  - 4-level 이상 nested 깊이 골든 (낮은 우선순위)
- Pending Human Decisions: 없음
- Roadmap Revisions:
  - cycle-logs/ROADMAP.md Phase 1: Cycle 2681 ~ 2682를 "Array<X> 일반화 본격 진입"으로 재매핑
  - 원래 Cycle 2683 작업을 앞당김
- Next Recommendation: **Cycle 2681 — Array<i64> 일반화 첫 진입** (코드 경로 진단)
