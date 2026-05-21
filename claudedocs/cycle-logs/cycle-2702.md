# Cycle 2702: hardcoded String-fn 리스트에서 `tokenize` 제거 (컴파일러 fix)
Date: 2026-05-11

## Re-plan
🟠 RE-PLAN: 작업 제목 "builtin @bit_* arity 정정" → "hardcoded String-fn 리스트 dynamic 우선화" (Cycle 2697 + 2700 통합 부류).
@bit_* arity는 별도 issue로 deferral (보다 광범위한 builtin 명세 정리 필요).

## Scope & Implementation

### 옵션 비교 (sketch)
| 옵션 | 변경 범위 | 리스크 | 효과 |
|------|---------|------|------|
| A: `tokenize`만 제거 | 1줄 | 매우 낮음 | 즉시 검증된 회귀만 fix |
| B: 모든 일반 명사 제거 | ~5-10 entries | 낮음-medium | 광범위 회귀 예방 (chr/slice 등 메서드 dispatch 영향 가능) |
| C: collect_string_fns에 `U:` prefix로 user fn 추적 + dynamic 우선 정책 | ~30 LOC | medium | 근본 해결 |

### 선택: Option A (안전 최소 fix)
**Rationale**: 
- `tokenize`는 compiler.bmb에 정의 없고 (0 callsites) 호출도 없음 → 100% safe to remove
- chr/slice/concat 등은 compiler.bmb 내부 메서드 dispatch에 영향 가능 (별도 사이클 위험 분리)
- Option C 근본 해결은 Cycle 2705-2706에서 검토

### 변경
`bootstrap/compiler.bmb:15560` `is_string_fn_group3`:
```bmb
// before
fn is_string_fn_group3(name: String) -> bool =
    name == "parse_source" or name == "gen_function" or name == "gen_program" or
    name == "tokenize" or name == "read_file" or name == "make_error" or
    ...

// after
fn is_string_fn_group3(name: String) -> bool =
    name == "parse_source" or name == "gen_function" or name == "gen_program" or
    name == "read_file" or name == "make_error" or
    ...
```

### 추가 작업
- `tests/bootstrap/test_golden_token_scan.bmb` + `test_golden_tokenizer.bmb` source rename 되돌림 (`user_tokenize` → `tokenize`)
- 이로써 사용자가 `tokenize` 이름을 자유롭게 사용 가능

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| Stage 1 빌드 | ✅ |
| token_scan 원본 이름으로 PASS | ✅ rc=0 actual='10' |
| tokenizer 원본 이름으로 PASS | ✅ rc=0 actual='5' |
| IR `println_str` 발행 | ✅ 0개 (println_i64로 정상 dispatch) |
| Sample golden (8개) | ✅ 6/6 (2개는 manifest 미등록) |
| `cargo test --release` | ✅ 6210/6210 PASS |
| Stage 2 self-compile | ❌ pre-existing arena OOM (M5-1 알려진 이슈, 본 변경과 무관) |

## Reflection

**핵심 통찰**:
- 1줄 변경으로 sourceless 회귀 fix — 사용자가 `tokenize` 이름 자유롭게 사용 가능
- Option B/C는 risk-vs-reward 측면에서 별도 사이클 (chr/slice 영향 검증 필요, broader refactor)
- `is_string_fn_group*` 5개 그룹의 약 50개 항목 중 대다수는 BMB-prefix (extern) 또는 LLVM helper로, 일반 명사 충돌 위험은 적음. 그러나 systematic audit 가치 있음 (Cycle 2705 후보)

**도그푸딩 가치**:
- Cycle 2697 `bit_or` + Cycle 2700 `tokenize` = 동일 패턴 인식 → 단일 사이클 fix
- 컴파일러 자체의 hardcoded list가 사용자 코드를 silent IR corruption 시키는 것은 BMB 1차 사용자 (LLM) 경험 손상 — 우선 fix 정당화

**Roadmap impact**:
- M5-5 series 안정 → M4-9 (clang knapsack outlier) 분석 진행 가능
- builtin/hardcoded 정합성 audit는 별도 Cycle 2705 후보

## Carry-Forward
- Actionable:
  - Cycle 2703: Track Q lint 규칙 (user fn 이름 = builtin/hardcoded 충돌 감지 — 2 케이스 누적)
  - Cycle 2704: M4-9 clang knapsack outlier IR diff
- Structural Improvement Proposals:
  - **컴파일러 (Option C)**: `collect_string_fns_acc`에 user fn ret_type 추적 (`U:` prefix) + dispatch에서 dynamic 우선 정책. broader refactor, 여러 callsite 영향. Cycle 2705-2706 후보.
  - **컴파일러**: `is_string_fn_group*` 5개 그룹 systematic audit — 일반 명사 (e.g., `concat`, `chr`, `slice`, `make_error`) 사용자 충돌 가능성 vs BMB 자체 의존도 가중치 분석
  - **builtin arity 체크**: `compiler.bmb:7142` `@bit_or` 분기에 arity 체크 (Cycle 2697 carry)
- Pending Human Decisions: 없음
- Roadmap Revisions: M4-7 (set obj.field[idx]) ✅ 완료 (Cycle 2690-2692). 이전 작업 항목.
- Next Recommendation: Cycle 2703 lint 규칙 추가
