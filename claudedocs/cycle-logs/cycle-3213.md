# Cycle 3213: M11-A Phase 5d — contract/lr2l/cx_most/repl 체인 (8개)
Date: 2026-05-27

## Re-plan

**Inherited scope**: Cycle 3212 Carry-Forward — M11-A Phase 5d 탐색.
**전략**: with-pre + no-pre 함수들 중 always-non-empty 패턴 식별.

## Scope & Implementation

### 적용 함수 (8개)

| 함수 | pre | 근거 |
|------|-----|------|
| `contract_ast_to_assumes` | `pre counter >= 0` | 모든 분기 `pack_assume_result(...)` 반환 → `"N:"` → `>= 1` |
| `lr2l_transform` | 없음 | 20개 `sco_push_line` push 후 `sb_build` → 항상 non-empty |
| `cx_most_params` | `pre pos >= 0` | base case: `best_name + " (" + int_to_string(N) + " params)"` → 최소 `" (0 params)"` 10자 |
| `cx_most_calls` | `pre pos >= 0` | base case: `best_name + " (" + int_to_string(N) + " calls)"` → 최소 `" (0 calls)"` 9자 |
| `repl_try_fallback` | 없음 | 항상 `compile_program(...)` 반환 → `compile_program >= 1` |
| `repl_try_int_first` | 없음 | `compile_program` or `repl_try_fallback` → 모두 `>= 1` |
| `repl_try_str_first` | 없음 | `compile_program` or `repl_try_fallback` → 모두 `>= 1` |
| `repl_try_compile` | 없음 | `repl_try_str_first` or `repl_try_int_first` 위임 → `>= 1` |

### 분석 패턴

**`contract_ast_to_assumes`**: `pack_assume_result(counter, ir)` 체인
```
모든 exit path:
  pack_assume_result(right_counter, "") → "N:"
  pack_assume_result(left_counter, left_ir) → "N:..." 
  pack_assume_result(right_counter, left_ir + SEP() + right_ir)
  pack_assume_result(counter, "") → "N:"
  pack_assume_result(counter+1, icmp + SEP() + assume)
  pack_assume_result(counter, "") → "N:"
→ 항상 >= 1 ✅
```

**`lr2l_transform`**: 20개 고정 IR 라인 push
```
sb_push(sb, def_line)  // "" 가능
sco_push_line(sb, "entry:")  // 항상 push
sco_push_line(sb, "  %_lr_cmp = ...")  // 항상 push
... (18개 더)
→ 항상 >= 1 ✅
```

**`repl_try_*`**: `compile_program` 체인
```
repl_try_compile → repl_try_str/int_first → compile_program (>= 1) or repl_try_fallback
repl_try_fallback → compile_program (>= 1) 항상
→ 전체 chain >= 1 ✅
```

### 건너뛴 패턴들

| 함수 | 이유 |
|------|------|
| `pick_best_name` | `name` or `best_name` — 빈 문자열 가능 |
| `hot_precompute` | base: `scored` (빈 초기값 가능) |
| `chain_search` | `depth > max_depth` 시 `""` 반환 |
| `lr2l_process_fn` | `fn_ir` 반환 — 빈 문자열 가능 |
| `callers_get_field` | not-found 시 `""` 반환 |

## Verification & Defect Resolution

```json
{"type":"lint","file":"bootstrap/compiler.bmb","warnings":0}
{"type":"verify_result","total":141,"verified":140,"failed":1}
✗ ifs_flex_check_goto (pre-existing)
```

2390 tests passed ✅

### trivials 추적

| 종류 | Cycle 3212 후 | Cycle 3213 후 | 변화 |
|------|------|------|------|
| bool `post it or not it` | 27 | 27 | 0 |
| i64 `post it == it` | 7 | 7 | 0 |
| String `post it.len() >= 0` | 252 | **244** | **-8** |
| **합계** | **286** | **278** | **-8** |
| String `post it.len() >= 1` | 134 | **142** | +8 |

**누적 진척**: 358 → 278 (-80, 22.3%)

## Reflection

**Scope fit**: 8개 업그레이드. 새 패턴 2가지: pack_assume_result 체인, repl compile 체인.

**Latent defects**: 없음.

**Structural improvement opportunities**:
- 남은 244개 중 추가 탐색 가능
- `llvm_gen_string_ref` (no-pre) 확인 필요 — 단일 식 `"  " + ... + ";"` → 항상 non-empty

**Philosophy drift**: 없음.

**Roadmap impact**: M11-A 22.3% 달성.

## Carry-Forward

- **Actionable**: M11-A Phase 5e — 남은 244개 추가 탐색
  - `llvm_gen_string_ref` (no-pre) — 직접 `"  " + ...` 형태
  - 기타 남은 LLVM codegen 함수 분석
- **Structural Improvement Proposals**: 없음
- **Pending Human Decisions**: 없음
- **Roadmap Revisions**: M11-A 358→278 (-80, 22.3%)
- **Next Recommendation**: M11-A Phase 5e — 남은 244개 중 직접 IR 생성 함수 탐색
