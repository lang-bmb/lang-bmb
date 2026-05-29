# BMB Session Handoff — 2026-05-29 (Cycles 3271-3280)

> **HEAD**: (커밋 예정)
> **실무 앵커**: `claudedocs/ROADMAP.md` (§ 6 AI-Native Pivot + M12-M15 진척)
> **전략 계획서**: `claudedocs/plans/ai-native-plan-2026.md`

---

## 현재 상태 스냅샷

| 항목 | 상태 |
|------|------|
| cargo test --release | ✅ 2390 tests, 0 FAILED |
| 3-Stage Fixed Point | ✅ S2 == S3 (Cycle 3276) |
| bmb lint warnings | ✅ 178 non-recursive (pre-existing) |
| Z3 verify | ✅ 141/141 |
| P-track 7/7 | ✅ ALL ≤1.010× |
| B-axis Claude | ✅ 98.0% (stale: 2026-08-13) |
| B-axis GPUStack | ✅ 100.0% (2026-05-21) |

---

## 이번 세션 완료 (Cycles 3271-3276)

| 마일스톤 | 완료 사이클 | 내용 |
|---------|-----------|------|
| M12 Phase 4 | 3271-3272 | 전이 효과 전파 (A→B→C): `build_transitive_effect_map` |
| M12 Phase 5 | 3273 | `[missing_effect_annotation]` lint: 미선언 함수 transitive 효과 경고 |
| M13 Phase 4 | 3274 | `verify-repair` 통합 명령: all functions + status (contracted/partial/intent_only/uncontracted) |
| M15 Phase 3 | 3275 | `module X requires [IO, Net]` 파싱 + `[module_capability]` lint |
| Fixed Point | 3272/3274/3276 | S2 == S3 3회 확인 ✅ |

---

## 새로 추가된 기능 요약

### M12 Phase 4: Transitive Effect Propagation

```bmb
fn fetch(): <Net> -> String = "data";
fn helper() -> String = fetch();
fn handler(): <IO> -> i64 = { let _s = helper(); 0 };
// [effect_propagation] handler: declares <IO> but calls helper which uses <Net>
```

- `build_transitive_effect_map(entries, direct_map, 5)` — 5회 반복 수렴
- 명시적 선언 함수: direct effect 유지 (callee check에서 정확한 비교)
- 미선언 함수: callee transitive effect 확장

### M12 Phase 5: Missing Effect Annotation

```
[missing_effect_annotation] b_helper: inferred effects <Net> but no explicit annotation
```

- `lint_check_missing_effect_annotations(entries, eff_map, transitive_map, 0, 0)` — w9

### M13 Phase 4: verify-repair 통합 명령

```
$ compiler.exe verify-repair foo.bmb
{"type":"verify_repair","file":"foo.bmb","functions":[
  {"name":"safe_div","status":"contracted","pre":["a >= 0","b > 0"],"post":["it >= 0"]},
  {"name":"main","status":"uncontracted"}
]}
```

- `vr_contract_status(has_pre, has_post, has_intent)` → 4-way status
- `verify_repair_scan` (ALL 함수, 20 calls ≤ max 20)
- `verify_repair_file(input)` — 진입점

### M15 Phase 3: Module Capability Declaration

```bmb
module MyApp requires [IO]

fn fetch_data(): <Net> -> String = "data";
// [module_capability] fetch_data: uses <Net> not declared in module requires [IO]
```

- `scan_module_requires(src, 0)` → "IO Net" 형식 capability 문자열
- `check_fn_vs_module_caps` → `[module_capability]` 경고
- `lint_check_module_capabilities` — w10

---

## 즉시 실행 가능한 다음 태스크

### M15 Phase 4 — Full Transitive Module Capability Check

현재 한계: 명시적 선언 함수(e.g., `process: <IO>`)의 transitive Net 효과가 module_capability 체크에서 미검출.

수정: `build_full_transitive_effect_map` (모든 함수 확장, direct 유지 없음) 추가 → module cap check에서 사용.

### M12 Z3 통합

M12 Phase 3의 효과 제약을 SMT predicate로 변환 → Z3 자동 증명.

### M14 Phase 4 — SemanticDuplicate

함수 AST 정규화 → 구조 해시 → 중복 경고.

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
- **BMB 예약어**: `pre`, `post`, `ret`, `type`, `for`, `in`, `let`, `fn`, `if`, `else`, `and`, `or`, `not`, `mut`, `set`, `as`, `bor` 등은 변수명/파라미터명으로 사용 불가.
- **M12 lint**: effect propagation lint는 bootstrap의 `lint_file` 경로. `bootstrap/compiler.exe lint` 로 실행.
- **fixed point**: 항상 S2 IR vs S3 IR 비교 (binary hash 아님).
- **module requires**: `module X requires [IO]` — `module` 키워드 다음 이름, 그 다음 `requires`, 그 다음 `[...]`.
- **transitive map 설계**: build_transitive_effect_map은 명시 선언 함수의 direct effect 유지. module cap check는 이 제한으로 명시 선언 함수의 transitive 효과 미검출 (알려진 한계).

---

## 주요 파일 위치

| 파일 | 역할 |
|------|------|
| `bootstrap/compiler.bmb` | 부트스트랩 컴파일러 (32K+ LOC) |
| `tests/golden/test_golden_effect_transitive.bmb` | M12 Phase 4+5 골든 테스트 |
| `tests/golden/test_golden_module_capability.bmb` | M15 Phase 3 골든 테스트 |
| `claudedocs/ROADMAP.md` | 실무 앵커 (§ 6 AI-Native Pivot + 진척 표) |
