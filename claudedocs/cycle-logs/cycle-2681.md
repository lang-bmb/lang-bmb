# Cycle 2681: Array<f64> literal dispatch — 1-line fix
Date: 2026-05-11

## Re-plan
이전 Carry-Forward (Cycle 2680): `Array<X>` 일반화 진단 + 구현.
드러난 갭:
- `Array<i64>` literal/fn return/struct field: ✅ 이미 동작
- `Array<f64>` literal: ❌ raw bits 출력 (1.5 → 4609434218613702656)
- `Array<f64>` fn return: ❌ raw bits 출력 (M5-5c 동형 갭)

트리거 없음. fix 진행.

## Scope & Implementation

### 핵심 fix (1-line)
`bootstrap/compiler.bmb` line 5303 `lower_array_literal_sb`:

Before:
```bmb
let w_mark = if first_type == "string" { sb_push_mir(sb, "  mark_str_ptr " + result_tmp) } else { 0 };
```

After:
```bmb
let w_mark = if first_type == "string" { sb_push_mir(sb, "  mark_str_ptr " + result_tmp) }
    else if first_type == "float" { sb_push_mir(sb, "  mark_f64_ptr " + result_tmp) }
    else { 0 };
```

### 동작 원리
1. AST `(array (float 1.5) ...)` → first element node type = `"float"`
2. `mark_f64_ptr` MIR 명령어 발행 → `push_f64_ptr_marker(str_sb, result)` 호출
3. GEP propagation (line 6711-6714) — `base_is_f64` 검출 시 result에 f64_ptr marker 자동 propagate
4. load 시점에 src_is_f64_ptr 인식 → `push_f64_marker` → `println(arr[0])` 가 `@println_f64` 로 dispatch

### 시나리오별 상태

| 시나리오 | Before | After |
|---------|--------|-------|
| `Array<i64>` literal | ✅ | ✅ |
| `Array<i64>` fn return | ✅ | ✅ |
| `Array<i64>` struct field | ✅ | ✅ |
| `Array<f64>` literal | ❌ raw bits | ✅ "1.500..." |
| `Array<f64>` fn return | ❌ raw bits | ❌ (다음 cycle) |
| `Array<f64>` struct field | 미검증 | 미검증 |

### 골든 추가
- `test_golden_arr_f64_literal.bmb` → exit 42, prints "1.500..." × 3
- `test_golden_arr_i64_baseline.bmb` → exit 42, prints "10\n20\n30\n" (회귀 가드)
- `golden_tests.txt` 2860 → 2862

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| `cargo test --release` | ✅ 6210 passed (회귀 없음) |
| Stage 1 빌드 | ✅ OK (15.3s) |
| `Array<f64>` literal | ✅ fix |
| `Array<f64>` fn return | ⏳ Cycle 2682 (4-point fix 필요) |

결함: 없음.

## Reflection

**Scope fit**: Array<f64> literal 케이스가 1-line fix로 해결. 정확한 분석 후 정확한 위치 수정.

**Latent defects**:
- `Array<f64>` fn return — `parse_return_type` Array<f64> 인식 + collect_f64_fns + dispatch 필요 (M5-5c 동형, 다음 사이클)
- `Array<f64>` struct field — `check_field_type` Array<f64> 인식 + `~af` suffix 필요 (M5-5d 동형, 다음 사이클)
- Both bool, char 등 array type element 일반화 — 더 거시적

**Structural improvement opportunities**:
- `lower_array_literal_sb` 의 type detection이 `"string"` / `"float"` hardcoded — element type → marker mapping 표 추출 가능 (현재 2개라 미경제)
- `mark_*_ptr` MIR 명령어 family가 확장될 가능성 — `mark_ptr <type>` 일반화? (지금은 i64로 충분)

**Philosophy drift**: 없음.
- 1-line fix는 workaround가 아니다 — 동형 인프라가 이미 깔려있어 정확한 채널 1개만 연결.
- `mark_f64_ptr` MIR 명령어 인프라는 다른 경로 (e.g., cast_ptr_f64, field_access f64)에서 이미 사용. 새로 추가하지 않음.

**Roadmap impact**:
- Cycle 2682를 `Array<f64>` fn return + struct field로 정확화
- Cycle 2683-2684 분량은 fn return + struct field 동형 4-point fix로 채워짐

**User-facing quality**: LLM 자연 패턴 `[1.5, 2.5]` 가 의도대로 동작 → AI-native 언어성 강화.

## Carry-Forward
- Actionable: Cycle 2682에서 `Array<f64>` fn return + struct field fix (M5-5c/d 동형 4-point fix)
- Structural Improvement Proposals:
  - 4-level 이상 nested 깊이 골든 (낮은 우선순위)
  - element type → marker mapping 추상화 (현재 2개라 미경제)
- Pending Human Decisions: 없음
- Roadmap Revisions:
  - cycle-logs/ROADMAP.md Phase 1: Cycle 2682를 "Array<f64> fn return + struct field 4-point fix"로 매핑
- Next Recommendation: **Cycle 2682 — `Array<f64>` fn return + struct field 4-point fix (M5-5c/d 동형)**
