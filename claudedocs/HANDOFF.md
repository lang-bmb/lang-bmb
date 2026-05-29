# BMB Session Handoff — 2026-05-29 (Cycles 3271-3280)

> **HEAD**: (커밋 예정)
> **실무 앵커**: `claudedocs/ROADMAP.md` (§ 6 AI-Native Pivot + M12-M15 진척)
> **전략 계획서**: `claudedocs/plans/ai-native-plan-2026.md`

---

## 현재 상태 스냅샷

| 항목 | 상태 |
|------|------|
| cargo test --release | ✅ 2390 tests, 0 FAILED |
| 3-Stage Fixed Point | ✅ S2 == S3 (Cycle 3278) |
| bmb lint warnings | ✅ 178 non-recursive (pre-existing) |
| Z3 verify | ✅ 141/141 |
| P-track 7/7 | ✅ ALL ≤1.010× |
| B-axis Claude | ✅ 98.0% (stale: 2026-08-13) |
| B-axis GPUStack | ✅ 100.0% (2026-05-21) |

---

## 이번 세션 완료 (Cycles 3271-3280)

| 마일스톤 | 완료 사이클 | 내용 |
|---------|-----------|------|
| M12 Phase 4 | 3271-3272 | 전이 효과 전파 (A→B→C): `build_transitive_effect_map` |
| M12 Phase 5 | 3273 | `[missing_effect_annotation]` lint: 미선언 함수 transitive 효과 경고 |
| M13 Phase 4 | 3274 | `verify-repair` 통합 명령: all functions + status (contracted/partial/intent_only/uncontracted) |
| M15 Phase 3 | 3275 | `module X requires [IO, Net]` 파싱 + `[module_capability]` lint |
| M15 Phase 4 | 3277 | `build_full_transitive_effect_map` — 명시 선언 함수도 전이 확장 (module cap check 전용) |
| Fixed Point | 3272/3274/3276/3278 | S2 == S3 4회 확인 ✅ |

---

## 새로 추가된 기능 요약

### M12 Phase 4: Transitive Effect Propagation

```bmb
fn fetch(): <Net> -> String = "data";
fn helper() -> String = fetch();   // no explicit effect
fn handler(): <IO> -> i64 = { let _s = helper(); 0 };
// [effect_propagation] handler: declares <IO> but calls helper which uses <Net>
// [missing_effect_annotation] helper: inferred effects <Net> but no explicit annotation
```

주요 함수: `build_transitive_effect_map(entries, eff_map, 5)`
- 명시적 선언 함수: direct effect 유지 (effect_propagation check용)
- 미선언 함수: callee transitive effect 확장

### M12 Phase 5: Missing Effect Annotation

```
[missing_effect_annotation] b_helper: inferred effects <Net> but no explicit annotation
```

`lint_check_missing_effect_annotations(entries, eff_map, transitive_map, 0, 0)` → w9

### M13 Phase 4: verify-repair 통합 명령

```
$ compiler.exe verify-repair foo.bmb
{"type":"verify_repair","file":"foo.bmb","functions":[
  {"name":"safe_div","status":"contracted","pre":["a >= 0","b > 0"],"post":["it >= 0"]},
  {"name":"nonneg","status":"partial","post":["it >= 0"]},
  {"name":"main","status":"uncontracted"}
]}
```

4-way status: contracted|partial|intent_only|uncontracted

### M15 Phase 3: Module Capability Declaration

```bmb
module MyApp requires [IO]

fn fetch_data(): <Net> -> String = "data";
fn helper() -> String = fetch_data();
fn process(): <IO> -> i64 = { let _d = helper(); 0 };
// [module_capability] fetch_data: uses <Net> not declared in module requires [IO]
// [module_capability] helper: uses <Net> not declared in module requires [IO]
```

`scan_module_requires(src, 0)` → "IO Net" 형식

### M15 Phase 4: Full Transitive Module Capability Check

```
// [module_capability] process: uses <Net> not declared in module requires [IO]
// [module_capability] main: uses <Net> not declared in module requires [IO]
```

`build_full_transitive_effect_map(entries, eff_map, 5)` — 모든 함수 전이 확장 (명시 선언도 포함)
- module_caps != "" 조건부로만 빌드 (성능 안전)

---

## 즉시 실행 가능한 다음 태스크

### M12 Z3 통합 (M12 Phase 6)

M12의 효과 제약을 SMT predicate로 변환 → Z3 자동 증명.
- `fn foo(): <pure>` → SMT: "foo doesn't use IO/Net/File"
- `fn bar(): <IO>` calling `fn baz(): <Net>` → Z3 counterexample

### M14 Phase 4 — SemanticDuplicate (재검토)

함수 AST 정규화 → 구조 해시 → 중복 경고.
- 현재 구현 어려운 이유: call-set 동일성 비교는 false positives 많음
- 대안: signature 동일 + call count 동일 + 소스 위치 근접 → "potential duplicate"

### M13 Phase 5 — .bmb-contracts 세션 영속 계약

ROADMAP M13 Phase 3: 프로젝트 레벨 불변식 파일 (.bmb-contracts)

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
- **BMB 예약어**: `pre`, `post`, `ret`, `type`, `for`, `in`, `let`, `fn`, `if`, `else`, `and`, `or`, `not`, `mut`, `set`, `as`, `bor` 등.
- **M12 lint**: `bootstrap/compiler.exe lint file.bmb` 로 실행.
- **fixed point**: S2 IR vs S3 IR 비교 (binary hash 아님).
- **transitive map 이원화**:
  - `transitive_map` = effect_propagation + missing_effect_annotation (명시 선언 유지)
  - `full_trans_map` = module_capability check (모든 함수 확장, module_caps != "" 조건부 빌드)
- **module requires**: `module X requires [IO]` — `module` 키워드 + 이름 + `requires` + `[...]`.

---

## 주요 파일 위치

| 파일 | 역할 |
|------|------|
| `bootstrap/compiler.bmb` | 부트스트랩 컴파일러 (32K+ LOC) |
| `tests/golden/test_golden_effect_transitive.bmb` | M12 Phase 4+5 골든 테스트 |
| `tests/golden/test_golden_module_capability.bmb` | M15 Phase 3+4 골든 테스트 |
| `claudedocs/ROADMAP.md` | 실무 앵커 (§ 6 AI-Native Pivot + 진척 표) |
