# Cycle 3206: M11-A Phase 1 — bool + llvm_gen_* semantic postconditions
Date: 2026-05-27

## Re-plan

**Inherited scope**: M11-A — trivial postcondition 교체 (~1,114개 → 현재 ~358개 남은 trivials).
HANDOFF에서 M11-A를 다음 우선순위 작업으로 지정. RE-PLAN: ROADMAP이 HUMAN-blocked 작업들만
남겨놨으므로, M11-A를 자율 사이클 작업으로 채택 (run-cycle RE-PLAN 규칙 적용).

**연속 주의사항**:
- skip 확정 (6 bool + 7 i64 + 77 String no-pre) 절대 변경 금지
- `ifs_flex_check_goto: post it >= 0` 기존 Z3 실패 — 우리 작업과 무관한 pre-existing issue

## Scope & Implementation

### 변경 대상 선정

1. **cf_is_pow2** (smoke test): `post it or not it` → `post not it or n > 1`
   - 함수 body: `n > 1 and (n band (n - 1)) == 0`
   - `it = true` → `n > 1` 필수 증명 → 의미있는 계약

2. **cp_is_var_char**: `post it or not it` → `post not it or (c >= 48 and c <= 122)`
   - 함수 body: `[48,57] ∪ [65,90] ∪ [97,122] ∪ {95}` — 모두 [48,122] 범위 내
   - `it = true` → c ∈ [48,122] 증명 가능

3. **14개 llvm_gen_* 함수**: `post it.len() >= 0` → `post it.len() >= 1`
   - `llvm_gen_binop`, `llvm_gen_sat_binop`, `llvm_gen_sat_mul`, `llvm_gen_wrap_binop`
   - `llvm_gen_cmp`, `llvm_gen_not`, `llvm_gen_bnot`, `llvm_gen_gep`
   - `llvm_gen_store_ptr_sb`, `llvm_gen_load_ptr_sb`, `llvm_gen_gep_sb`
   - `llvm_gen_phi`, `llvm_gen_phi_typed`, `llvm_gen_fn_header`
   - 모두 `"  " + ...` 패턴으로 시작 → 항상 len >= 2 보장
   - `same_mapping(x)` = x (post it == llvm_line) → 전달 인수가 non-empty면 결과도 non-empty

### 변경 방법

- bool 2개: Edit tool 직접 수정
- llvm_gen_* 14개: Python regex batch 교체 (함수명+pre 패턴으로 unique match)

## Verification & Defect Resolution

### lint

```json
{"type":"lint","file":"bootstrap/compiler.bmb","warnings":0}
```

0 warnings ✅

### Z3 verify

```
{"type":"verify_result","total":141,"verified":140,"failed":1}
✗ ifs_flex_check_goto: post verification failed
```

- 실패 1개: pre-existing (우리 변경 전부터 존재 — git stash로 확인)
- 신규 실패 없음 ✅

### Stage 1 bootstrap

```
186,293 lines IR 생성 ✅
```

### cargo test

```
3800 passed; 0 failed ✅
```

### 삭제된 trivials

| 종류 | 이전 | 이후 | 변화 |
|------|------|------|------|
| bool tautology `post it or not it` | 49 | 47 | -2 |
| i64 tautology `post it == it` | 7 | 7 | 0 |
| String `post it.len() >= 0` | 302 | 288 | -14 |
| **합계 trivials** | **358** | **342** | **-16** |
| String `post it.len() >= 1` (semantic) | ~84 | 98 | +14 |

## Reflection

**Scope fit**: 16개 trivial postcondition 교체 — smoke test 통과 후 배치 확장. 의도한 범위 내.

**Latent defects**: 없음. `ifs_flex_check_goto: post it >= 0`는 pre-existing — `next_p: i64`에
`pre next_p >= 0` 없음이 원인. 범위 외 (M11-A가 아닌 새 계약 설계 필요 — carry-forward).

**Structural improvement opportunities**:
- `ifs_flex_check_goto`에 `pre next_p >= 0` 추가 + Z3 verify fix → M11-A 후속 배치에서 처리 가능
- bool trivials 47개 중 skip 확정 6개 제외 → ~41개 교체 대상 잔여
- String >= 0 trivials 288개 중 skip 확정 77개 제외 → ~211개 교체 대상 잔여

**Philosophy drift**: 없음. 계약의 의미적 강화는 BMB 핵심 목표.

**Roadmap impact**: M11-A 1차 배치 완료. 342개 trivials 잔여 (358 → 342, -16).

## Carry-Forward

- **Actionable**: M11-A Phase 2 — 나머지 bool 41개 + String 211개 semantic 교체 (배치별 처리)
- **Structural Improvement Proposals**:
  - `ifs_flex_check_goto`: `pre next_p >= 0` 추가 시 Z3 verify 통과 가능 — HUMAN 결정 불필요, 실행 가능
- **Pending Human Decisions**: 없음
- **Roadmap Revisions**: M11-A 진행 중 (16/358+ 완료)
- **Next Recommendation**: M11-A Phase 2 — bool trivials 배치 처리 (skip 확정 6개 제외 후 ~41개)
  `post it or not it` → 각 함수 의미 분석 후 적절한 semantic post 부여
