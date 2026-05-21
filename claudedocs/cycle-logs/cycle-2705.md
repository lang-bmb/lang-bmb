# Cycle 2705: hardcoded String-fn dead entries 추가 제거 (Cycle 2702 패턴 확장)
Date: 2026-05-11

## Re-plan
🟠 RE-PLAN: 인계받은 "Option C 또는 다른 갭" → advisor 권고 채택. Option C는 Stage 2 검증 경로 부재로 보류. 대신 Cycle 2702 패턴 확장 — dead entries (callsites=0 AND defined=0) 일괄 제거.

## Scope & Implementation

### Audit
| 이름 | callsites | defined | 처리 |
|------|----------|---------|------|
| `concat`, `concat3`, `concat5`, `concat7` | 0 | 0 | 제거 (group1) |
| `make_error` | 0 | 0 | 제거 (group3) |
| `gen_program_sb_with_strings` | 0 | 0 | 제거 (group3) |
| `compile_function`, `compile_source` | 0 | 0 | 제거 (group2) |
| `sb_build` | 88 | 0 | **유지** (extern, hardcoded 필요) |
| `chr`, `slice` | 69, 1065 | 0 | **유지** (extern/method) |
| `get_field`, `trim_end`, `i2s`, `int_to_string`, `digit_char` | 다수 | 1 | **유지** (compiler.bmb 정의, 사용자 충돌 시 hardcoded 보호 가치) |

### 변경
`bootstrap/compiler.bmb`:
- group1: 4 entries 제거 (concat/3/5/7)
- group2: 2 entries 제거 (compile_function, compile_source)
- group3: 2 entries 제거 (make_error, gen_program_sb_with_strings)
- 총 8 dead entries 제거

`bootstrap/lint/lint.bmb`:
- `is_reserved_builtin_name`: 5 entries 제거 (concat/3/5/7, make_error)
- group3에서 제거된 항목들 lint도 동기화 (gen_program_sb_with_strings는 lint에 원래 없었음, compile_function/source 동일)

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| Stage 1 rebuild | ✅ |
| Sample golden 5/5 PASS | ✅ aho_corasick, fibonacci, set_cover, token_scan, tokenizer |
| Lint 회귀 (테스트 케이스) | ✅ 2 warnings (bit_or + read_file 정확) |
| `cargo test --release` | ✅ 6 suites all OK |

## Reflection

**핵심 통찰**:
- "callsites=0 AND defined=0" 규칙은 안전 검증된 dead-entry 제거 전략 — Cycle 2702 (`tokenize`)에서 검증됨
- 8 entries 추가 제거로 사용자 충돌 가능성 감소 (특히 `concat`, `make_error`는 일반 명사)
- Option C (dynamic 우선화)는 Stage 2 검증 경로 부재 → 보류 (Stage 2 self-compile 복원 후 재검토)

**도그푸딩 가치**:
- 컴파일러 hardcoded 리스트의 systematic audit이 데드 코드 8개 발견 — 코드 cleanup 부수 효과
- "사용자 친화 이름 사용" 정책이 점진적으로 강화됨

**Roadmap impact**:
- Cycle 2702-2705 누적: hardcoded list 9 entries 제거 (`tokenize` + 8 dead)
- 전체 ~50개 hardcoded entries 중 ~20% cleanup, 나머지는 BMB-specific (bmb_*) 또는 LLVM helper로 충돌 위험 낮음

**Stage 2 진단 가설 정정 (advisor 지적)**:
- HANDOFF.md M5-1 ❌ "arena OOM" 분류는 32G+ 메모리 케이스
- Cycle 2702에서 본 parse error (`expected '}' after else body, got integer literal at line 19946:78`)는 **다른 케이스** — AST 어떤 상태에서 파서가 헛갈리는 경우 가설 (메모리 부족 ❌)
- 향후 Stage 2 진단 시 두 가설 분리

## Carry-Forward
- Actionable:
  - Cycle 2706: README/HANDOFF 라벨 정정 (clang vs gcc) 또는 vacuum 슬롯
- Structural Improvement Proposals:
  - **Option C** (dynamic 우선화): Stage 2 self-compile 복원 후 재검토
  - **Stage 2 진단**: parse error vs arena OOM 두 가설 분리 (별도 cycle)
- Pending Human Decisions:
  - **M3-5 [HUMAN]** README "knapsack 6.7x faster than C" → "vs Clang -O3 outlier" 라벨 정정
- Roadmap Revisions: Track Q lint reserved set 5 entries 감소 (concat/3/5/7, make_error 제거)
- Next Recommendation: Cycle 2706 — HANDOFF/ROADMAP 갱신 (라벨 정정 + 본 세션 변화 반영)
