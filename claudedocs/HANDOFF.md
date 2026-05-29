# BMB Session Handoff — 2026-05-29 (Cycles 3286-3293)

> **HEAD**: `adc0f0a1` (feat(ai-native): M12 Phase 6b/6c/6d + M15 Phase 5 + sim_count_shared)
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

## 이번 세션 완료 (Cycles 3286-3293)

| 마일스톤 | 완료 사이클 | 내용 |
|---------|-----------|------|
| M14 Phase 4b | 3286 | sim_count_shared 버그수정 + semdp 통합 |
| M12 Phase 6b | 3287 | @pure fn violation → effect-verify 탐지 |
| M12 Phase 6c | 3288 | missing_effect_annotation → effect-verify 통합 |
| M15 Phase 5 | 3289 | module-suggest + callers_collect_source platform fix |
| M12 Phase 6d | 3291 | @pure fn → Z3 UNSAT 공식 검증 |
| Fixed Point | 3292 | S3f==S4f Within-gen ✅ |
| Commit | 3293 | adc0f0a1 |

---

## 새로 추가된 기능 요약

### M14 Phase 4b: sim_count_shared 버그 수정

```
# 수정 전 (버그)
./compiler similar test.bmb foo  →  [2 shared] (3-call 중 2만 매치)

# 수정 후
./compiler similar test.bmb foo  →  [3 shared] ✅

# 원인: sim_find_start_rev의 `if pos < 0`이 LLVM pre-condition 최적화로 dead code 제거
# 수정: `if pos < 0` → `if pos <= 0` (pos==0 안전 처리)
```

- `semdp_count_shared` / `semdp_name_start` 제거 (sim_count_shared 재사용)

### M12 Phase 6b: @pure fn violation → effect-verify

```bash
$ compiler.exe effect-verify test.bmb
{"type":"effect_verify","status":"violation","z3":"sat","violations":[
  {"caller":"bad_fn","callee":"io_fn","caller_effect":"pure","callee_effect":"IO"}
]}
```

### M12 Phase 6c: missing_effect_annotation → effect-verify

```bash
{"violations":[
  {"caller":"wrapper","callee":"","caller_effect":"missing","callee_effect":"IO"}
]}
```

### M12 Phase 6d: @pure fn → Z3 UNSAT

```bash
# 기존: z3:"sat" (heuristic만)
# 신규: z3:"unsat" ✅ (Z3 formal verification)
{"status":"violation","z3":"unsat","violations":[...]}
```

- `eff_z3_gen_pure_decls`: @pure fn을 SMT에 all-effect=false로 선언
- `eff_z3_gen_pure_edges`: 직접 implication 생성

### M15 Phase 5: module-suggest + platform 버그 수정

```bash
$ compiler.exe module-suggest test.bmb
{"type":"module_suggest","module":"myapp","declared":["IO"],"suggested":["File"],"status":"mismatch"}

# status: "ok" | "mismatch" | "needs_module"
```

**P4 버그 수정**: `callers_collect_source`가 platform 블록을 swallow하던 버그 수정.
- `skip_nested_brace` / `skip_platform_block` 추가
- `callers_collect_source`: "platform" 식별 시 블록 전체 스킵

---

## effect-verify 3가지 위반 유형 (통합 완료)

| 유형 | 탐지 방법 | Z3 상태 | 예시 |
|------|---------|---------|------|
| declared-effect 불일치 | Z3 SMT | `"unsat"` | `<IO>` caller가 `<Net>` callee 호출 |
| @pure fn effectful 호출 | Z3 SMT | `"unsat"` ← NEW | `@pure fn` → IO 함수 호출 |
| missing_effect_annotation | heuristic scan | `"sat"` | transitive IO 있으나 선언 없음 |

---

## 즉시 실행 가능한 다음 태스크

### [P1] effect lattice 더 깊은 모델링 (optional)

- missing_effect_annotation도 Z3 SMT에 추가 (inferred → assert constraints)
- effect lattice partial order: `IO ≤ IO+Net` formally modeled
- 현재: @pure Z3 UNSAT ✅, missing heuristic only

### [P2] index 명령 platform 버그 수정

- `callers_collect_source`는 수정됨 → `index` 명령의 별도 코드 경로 수정 필요
- platform 블록 내 fn이 잘못된 callee 목록으로 표시됨 (낮은 우선순위)

### [P3] contracts-check `require_effect_annotation` 통합

- 현재: `lint_check_missing_effect_annotations`를 별도 호출
- 개선: contracts-check JSON에 missing_annotation 위반도 포함

### [P4] module-suggest `declared`==`suggested` 비교 정교화

- 현재: 문자열 직접 비교 (순서 의존)
- 개선: set-equality 비교 (순서 무관)

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
- **fixed point**: emit-ir 두 번 실행하여 동일성 확인 (binary hash 아님).
- **Z3 경로**: `z3`는 C:/msys64/ucrt64/bin/z3.exe, PATH에서 접근 가능.
- **sim_count_shared**: 수정 완료 (`if pos <= 0`). semdp_count_shared는 제거됨.
- **callers_collect_source**: platform 블록 스킵 수정 완료. index 명령은 미수정.

---

## 주요 파일 위치

| 파일 | 역할 |
|------|------|
| `bootstrap/compiler.bmb` | 부트스트랩 컴파일러 (~46K LOC) |
| `tests/golden/test_golden_effect_verify_pure.bmb` | M12 Phase 6b 골든 테스트 |
| `tests/golden/test_golden_effect_verify_comprehensive.bmb` | M12 3-way 통합 테스트 |
| `claudedocs/ROADMAP.md` | 실무 앵커 (§ 6 AI-Native Pivot + 진척 표) |
