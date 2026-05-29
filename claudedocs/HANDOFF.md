# BMB Session Handoff — 2026-05-29 (Cycles 3286-3295)

> **HEAD**: `2eb5658c` (chore(docs): 세션 종료 정리 — Cycles 3286-3295)
> **실무 앵커**: `claudedocs/ROADMAP.md` (§ 6 AI-Native Pivot + M12-M15 진척)
> **전략 계획서**: `claudedocs/plans/ai-native-plan-2026.md`

---

## 현재 상태 스냅샷

| 항목 | 상태 |
|------|------|
| cargo test --release | ✅ 3800+2390+23 tests, 0 FAILED |
| Within-gen Fixed Point | ✅ S3f==S4f (Cycle 3292) |
| bmb lint warnings | ✅ 178 non-recursive (pre-existing) |
| Z3 verify | ✅ 144/144 |
| P-track 7/7 | ✅ ALL ≤1.010× |
| B-axis Claude | ✅ 98.0% (stale: 2026-08-13) |
| B-axis GPUStack | ✅ 100.0% (2026-05-21) |

---

## 이번 세션 완료 (Cycles 3286-3295)

| 마일스톤 | 완료 사이클 | 내용 |
|---------|-----------|------|
| M14 Phase 4b | 3286 | sim_count_shared 버그수정 + semdp 통합 |
| M12 Phase 6b | 3287 | @pure fn violation → effect-verify 탐지 |
| M12 Phase 6c | 3288 | missing_effect_annotation → effect-verify 통합 |
| M15 Phase 5 | 3289 | module-suggest + callers_collect_source platform fix |
| ROADMAP | 3290 | M12/M14/M15 Phase 완료 마킹 |
| M12 Phase 6d | 3291 | @pure fn → Z3 UNSAT 공식 검증 |
| 검증 | 3292 | Fixed Point + 전체 테스트 ✅ |
| Commit | 3293 | adc0f0a1 + b5d89b5e |
| contracts-check | 3294 | require_effect_annotation JSON violations |
| 최종 | 3295 | commit + HANDOFF + 메모리 갱신 |

---

## 신규 기능 요약

### M14 Phase 4b: sim_count_shared 버그 수정

**원인**: `sim_find_start_rev`의 `if pos < 0`이 LLVM pre-condition(`pre pos >= 0`) 최적화로 dead code 제거 → `byte_at(-1)` UB.
**수정**: `if pos < 0` → `if pos <= 0` (pos==0에서 즉시 0 반환).
**결과**: similar 명령: 1-call segfault 수정, N-1→N shared 정확 보고. semdp_count_shared 제거(sim_count_shared 재사용).

### M12 Phase 6b/6c/6d: effect-verify 3-way violations

```bash
$ compiler.exe effect-verify test.bmb
# 위반 유형 1: Z3 UNSAT (declared effect 불일치)
{"caller":"bad_caller","caller_effect":"IO","callee_effect":"Net"}
# 위반 유형 2: @pure fn effectful 호출 → Z3 UNSAT (NEW Phase 6d)
{"caller":"bad_fn","caller_effect":"pure","callee_effect":"IO"}
# 위반 유형 3: missing annotation (transitive effect 있으나 선언 없음)
{"caller":"wrapper","caller_effect":"missing","callee_effect":"IO"}
```

**Phase 6d**: @pure fn → Z3 SMT에 `all-effects=false` 선언 + `(=> callee_X caller_X)` assertion → UNSAT.

### M15 Phase 5: module-suggest + platform fix

```bash
$ compiler.exe module-suggest test.bmb
{"type":"module_suggest","module":"myapp","declared":["IO"],"suggested":["File"],"status":"mismatch"}
```

**P4 버그 수정**: `callers_collect_source`의 platform 블록 swallow 버그 수정.
- `skip_nested_brace` / `skip_platform_block` 신규.
- TK_IDENT "platform" 감지 → 블록 전체 스킵.

### contracts-check require_effect_annotation JSON (Cycle 3294)

```bash
# .bmb-contracts: require_effect_annotation = true
$ compiler.exe contracts-check test.bmb
{"violations":[{"rule":"require_effect_annotation","function":"missing_fn",
  "message":"function missing_fn has inferred effects <IO> but no explicit annotation"}]}
```

---

## 즉시 실행 가능한 다음 태스크

### [P1] effect-verify + lint 통합 CLI

**배경**: 현재 effect-verify와 lint는 별개 명령. AI 워크플로우에서는 단일 명령으로 모든 effect 진단이 필요.
**구현 방향**:
```bash
compiler.exe diagnose src.bmb   # effect-verify + lint effect rules + contracts 통합
```
- `effect-verify` 결과 + `lint --effect-only` 결과를 하나의 JSON으로 통합
- AI agent가 한 번에 모든 effect 관련 위반을 파악 가능

### [P2] index 명령 platform 버그 수정

**배경**: `callers_collect_source`는 수정됨(Cycle 3289). `index_file`의 `index_collect_source` 함수(별도 코드 경로)는 platform 블록 swallow 버그 미수정.
**증상**: `compiler.exe index file_with_platform.bmb` → platform 내부 fn이 잘못된 callee 목록 포함.
**수정 방향**: `index_collect_source`에도 `skip_platform_block` 적용.

### [P3] module-suggest set-equality 비교

**배경**: 현재 `declared_caps == used_caps` (string 직접 비교, 순서 의존). "IO File" ≠ "File IO" → 오분류.
**수정**: `eff_set_equals(a, b)` 함수 추가 — 각 cap이 상대방에 포함되는지 양방향 체크.

### [P4] effect lattice 더 깊은 Z3 모델 (장기)

**배경**: 현재 missing_effect_annotation은 heuristic scan. Z3 formal certification이 없음.
**구현 방향**: `transitive_map` 기반으로 미선언 함수들도 SMT 모델에 추가 → Z3가 더 넓은 범위 검증.
**복잡도**: 높음 (2-3 사이클), effect 집합 크기에 따라 SMT 폭발 가능.

### [P5] bootstrap Fixed Point 완전 검증 (선택)

**배경**: Within-gen Fixed Point(emit-ir 두 번)는 ✅. 하지만 cross-gen(S2 IR vs S3 IR)은 18446744073709551615 vs -1 표현 차이로 diff 존재. 이는 기능 동일하나 LLVM IR 표현 차이.
**검증 방향**: `sed 's/18446744073709551615/-1/g'` 정규화 후 비교하면 완전 Fixed Point 확인 가능.
**필요 조건**: llc + gcc 링크 (~40분), 큰 시간 투자.

---

## 보류/HUMAN-blocked 항목

| 항목 | 이유 |
|------|------|
| B-axis 재측정 (Claude) | ANTHROPIC_API_KEY 필요 (stale: 2026-08-13) |
| v1.0 선언 | 외부 신호 대기 |

---

## 주의사항

- **Rule 6**: 모든 새 기능은 bootstrap/compiler.bmb에서만.
- **Python write 금지**: bootstrap/compiler.bmb 수정 시 Python write 금지.
- **fixed point**: `compiler.exe emit-ir src out1.ll` 두 번 실행하여 동일성 확인.
- **Z3 경로**: `z3`는 C:/msys64/ucrt64/bin/z3.exe, PATH 접근 가능.
- **sim_count_shared**: 수정 완료 (`if pos <= 0`). semdp 제거됨.
- **callers_collect_source**: platform 블록 스킵 수정 완료. index 명령은 미수정.

---

## 주요 파일

| 파일 | 역할 |
|------|------|
| `bootstrap/compiler.bmb` | 부트스트랩 컴파일러 (~46K+ LOC) |
| `tests/golden/test_golden_effect_verify_pure.bmb` | M12 Phase 6b 골든 테스트 |
| `tests/golden/test_golden_effect_verify_comprehensive.bmb` | M12 3-way 통합 골든 테스트 |
| `claudedocs/ROADMAP.md` | 실무 앵커 (§ 6 AI-Native Pivot + 진척 표) |
