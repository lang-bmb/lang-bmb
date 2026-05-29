# BMB Session Handoff — 2026-05-29 (Cycles 3281-3284)

> **HEAD**: `0de467ba` (feat(ai-native): M12 Phase 6 + M13 Phase 5 + M14 Phase 4)
> **실무 앵커**: `claudedocs/ROADMAP.md` (§ 6 AI-Native Pivot + M12-M15 진척)
> **전략 계획서**: `claudedocs/plans/ai-native-plan-2026.md`

---

## 현재 상태 스냅샷

| 항목 | 상태 |
|------|------|
| cargo test --release | ✅ 3800+2390 tests, 0 FAILED |
| 3-Stage Fixed Point | ✅ S3c == S4c (Cycle 3284) |
| bmb lint warnings | ✅ 178 non-recursive (pre-existing) |
| Z3 verify | ✅ 141/141 |
| P-track 7/7 | ✅ ALL ≤1.010× |
| B-axis Claude | ✅ 98.0% (stale: 2026-08-13) |
| B-axis GPUStack | ✅ 100.0% (2026-05-21) |

---

## 이번 세션 완료 (Cycles 3281-3284)

| 마일스톤 | 완료 사이클 | 내용 |
|---------|-----------|------|
| M12 Phase 6 | 3281-3282 | `effect-verify` Z3 formal verification |
| M13 Phase 5 | 3283 | `.bmb-contracts` + `contracts-check` |
| M14 Phase 4 | 3284 | `semantic-duplicate` call set 비교 |
| Fixed Point | 3282/3284 | S3==S4 2회 확인 ✅ |

---

## 새로 추가된 기능 요약

### M12 Phase 6: effect-verify (Z3 formal verification)

```bash
$ compiler.exe effect-verify foo.bmb
# violation case:
{"type":"effect_verify","file":"foo.bmb","status":"violation","z3":"unsat",
 "violations":[{"caller":"bad_caller","callee":"safe_net","caller_effect":"IO","callee_effect":"Net"}]}
# safe case:
{"type":"effect_verify","file":"foo.bmb","status":"safe","z3":"sat"}
```

- SMT-LIB2 생성: 함수 effect Boolean 변수 + call edge implication
- `exec_with_stdin("z3", "-smt2 -in", smt)` → Z3 응답 파싱
- UNSAT = 위반 (제약 불일치), SAT = 안전 (모든 제약 충족)
- PLAT: prefix 확인으로 platform 함수 false positive 제거

### M13 Phase 5: .bmb-contracts (Session-persistent contracts)

```bash
# .bmb-contracts 파일:
require_postcondition = true
forbid_effect = Net

$ compiler.exe contracts-check src/module.bmb
{"type":"contracts_check","file":"src/module.bmb","status":"violation","violations":[
  {"rule":"require_postcondition","function":"foo","message":"function foo has no postcondition"},
  {"rule":"forbid_effect","function":"bar","message":"function bar uses <Net> (forbidden by project contracts)"}
]}
```

- `bc_parse_contracts` + `bc_get`: key-value 파서
- `bc_check_post_scan`: post clause 검사 (vr_after_params_pos 활용)
- `bc_check_forbid_eff`: 금지 effect 전이 검사

### M14 Phase 4: semantic-duplicate (Call set duplicate detection)

```bash
$ compiler.exe semantic-duplicate src/big_module.bmb
{"type":"semantic_duplicate","file":"src/big_module.bmb","pairs":[
  {"fn_a":"parse_for_body","fn_b":"parse_for_body_inclusive","shared_calls":11,"total_a":11,"total_b":11},
  {"fn_a":"process_v1","fn_b":"process_v2","shared_calls":3,"total_a":3,"total_b":3}
]}
```

- 기준: `semdp_count_shared == max(cnt_a, cnt_b)` AND ≥3 calls
- `semdp_count_shared` 직접 구현 (`sim_count_shared` 버그 우회)
- compiler.bmb에서 실제 중복 쌍 발견 (parse_for_body 계열 등)

---

## 즉시 실행 가능한 다음 태스크

### M12 Z3 더 깊은 통합

현재 M12 Phase 6는 선언된 effect만 비교. 다음:
- `pure fn` 위반도 Z3로 확인 (Phase 1 위반의 Z3 cert)
- Effect annotation 추론 결과를 Z3 assertion으로 변환

### M15 Phase 5 — Platform capability 강화

`platform stdlib { fn ... }` 블록에서 platform-declared 함수들의
capability를 `module X requires [...]` 와 연계하는 강화.

### sim_count_shared 버그 수정

기존 `similar` 명령에서 같은 call set을 가진 함수들이 N-1 shared로 보고되는 버그.
원인 분석: `sim_count_shared`가 마지막 item을 count에 포함시키지 않는 것으로 추정.

---

## 보류/HUMAN-blocked 항목

| 항목 | 이유 |
|------|------|
| B-axis 재측정 (Claude) | ANTHROPIC_API_KEY 필요 (stale: 2026-08-13) |
| v1.0 선언 | 외부 신호 대기 |

---

## 주의사항

- **Rule 6**: 모든 새 기능은 bootstrap/compiler.bmb에서만.
- **Python write 금지**: bootstrap/compiler.bmb 수정 시 Python write 금지. Edit 도구 사용 필수.
- **fixed point**: S2 IR vs S3 IR 비교 (binary hash 아님).
- **Z3 경로**: `z3`는 C:/msys64/ucrt64/bin/z3.exe 에 있어 PATH에서 접근 가능.
- **Platform 블록 한계**: `callers_collect_source`가 platform 블록 내 선언을 처리 시
  이후 함수를 swallow할 수 있음 (contracts-check 파일에 platform 블록 없어야 정확).
- **semdp_count_shared**: `sim_count_shared` 버그 우회로 자체 구현. 두 함수는 별개.

---

## 주요 파일 위치

| 파일 | 역할 |
|------|------|
| `bootstrap/compiler.bmb` | 부트스트랩 컴파일러 (46K+ LOC) |
| `tests/golden/test_golden_effect_verify.bmb` | M12 Phase 6 골든 테스트 |
| `tests/golden/test_golden_contracts_check.bmb` | M13 Phase 5 골든 테스트 |
| `tests/golden/test_golden_semantic_duplicate.bmb` | M14 Phase 4 골든 테스트 |
| `claudedocs/ROADMAP.md` | 실무 앵커 (§ 6 AI-Native Pivot + 진척 표) |
