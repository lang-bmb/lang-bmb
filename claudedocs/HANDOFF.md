# BMB Session Handoff — 2026-05-29 (Cycles 3251-3260)

> **HEAD**: `[pending commit]` (M12+M13+M14+M15 Phase 1 완료)
> **실무 앵커**: `claudedocs/ROADMAP.md` (§ 6 AI-Native Pivot + M12-M15 진척)
> **전략 계획서**: `claudedocs/plans/ai-native-plan-2026.md`

---

## 현재 상태 스냅샷

| 항목 | 상태 |
|------|------|
| cargo test --release | ✅ 2390 tests, 0 FAILED |
| 3-Stage Fixed Point | ✅ S2 == S3 (Cycle 3259) |
| bmb lint 0-warning | ✅ 177 non-recursive (pre-existing), 0 새 경고 |
| Z3 verify | ✅ 141/141 |
| P-track 7/7 | ✅ ALL ≤1.010× |
| B-axis Claude | ✅ 98.0% (stale: 2026-08-13) |
| B-axis GPUStack | ✅ 100.0% (2026-05-21) |

---

## 이번 세션 완료 마일스톤 (Cycles 3251-3260)

| 마일스톤 | 완료 사이클 | 내용 |
|---------|-----------|------|
| M12 Phase 1 | 3251 | `fn foo(): <IO, Net> -> T` 파싱 (parse_fn_after_params + parse_fn_effect_tail) |
| M13 Phase 1+2 | 3252 | `intent: "..."` 어노테이션 + `[intent_no_contract]` lint |
| M14 Phase 1 | 3253 | `gotgan lock` SHA-256 lockfile |
| M12 Phase 2a | 3254 | effect row → MIR `@effect:IO Net` |
| M12 Phase 2b | 3255 | effect → LLVM `"bmb-effect"="IO"` 속성 |
| M12 Phase 2c | 3257 | `[effect_pure_violation]` lint: pure fn calling IO → warn |
| M14 Phase 2 | 3258 | `gotgan verify` 해시 검증 명령 |
| M15 Phase 1 | 3259 | `platform Name { ... }` 파싱 (skip) |

---

## 새로 추가된 언어 기능 요약

### M12: Effect Row 문법

```bmb
fn double(x: i64): <pure> -> i64 = x * 2;     // fn-pure → memory(none) nofree
fn read_io(path: String): <IO> -> String = ...;  // "bmb-effect"="IO" LLVM attr
fn transfer(): <IO, Net> -> i64 = ...;           // "bmb-effect"="IO Net"
fn free(n: i64): <*> -> i64 = n;                // 제한 없음
```

**Lint rule**: `[effect_pure_violation]` — `@pure fn` calling println/system/etc → warn

### M13: Intent 어노테이션

```bmb
fn add(x: i64, y: i64) -> i64
    intent: "Adds two numbers"
    pre x >= 0 and y >= 0
    post it >= 0
= x + y;
```

**Lint rule**: `[intent_no_contract]` — intent 있고 contracts 없으면 warn

### M14: gotgan SHA-256 Lockfile

```
$ gotgan lock    → 생성: gotgan.lock (SHA-256 hashes of deps)
$ gotgan verify  → 검증: locked vs current hash 비교
```

### M15: platform 선언

```bmb
platform stdlib {
    fn io_print(s: String): <IO> -> i64;
}
// → 파싱 되지만 현재는 무시 (Phase 2에서 effect capabilities 등록)
```

---

## 즉시 실행 가능한 다음 태스크

### M12 Phase 3 — Z3 effect constraint

**목표**: effect가 pre/post와 연동 - `fn(): <IO>` → Z3 검증에서 IO effect 제약 조건

### M15 Phase 2 — platform effect capabilities 등록

**목표**: `platform stdlib { fn io_print: <IO> }` → `io_print` 함수에 <IO> effect 자동 등록

### M13 Phase 3 — 구조화 Repair Signal

**목표**: `bmb verify` 실패 → LLM-consumable JSON repair signal

### M14 Phase 3 — gotgan add 명령

**목표**: `gotgan add <name> <path>` → gotgan.toml에 dep 추가 + SHA-256 계산

---

## 보류/HUMAN-blocked 항목

| 항목 | 이유 |
|------|------|
| B-axis 재측정 (Claude) | ANTHROPIC_API_KEY 필요 (stale: 2026-08-13) |
| v1.0 선언 | 외부 신호 대기 |

---

## 주의사항

- **Rule 6**: 모든 새 기능은 bootstrap/compiler.bmb에서만. HANDOFF의 Rust 파일 참조는 stale.
- **M12 주의**: `fn(): <pure>` → `fn-pure` AST (기존 @pure fn과 동등). `fn(): <IO>` → `fn` + `(eff IO)` 자식.
- **M13 주의**: `extract_pre_asts`/`extract_post_asts`가 `intent:` 스킵하도록 수정됨.
- **M14 주의**: `exec_with_stdin` LLVM declare 추가됨. `sha256sum` 먼저, certutil fallback.
- **M15 주의**: `skip_brace_block` ≠ `skip_nested_braces(..tok_end(t_brace), 1)` — `{` 이후 위치에서는 후자를 사용.
- **fixed point**: 항상 S2 IR vs S3 IR 비교 (binary hash 아님).

---

## 주요 파일 위치

| 파일 | 역할 |
|------|------|
| `bootstrap/compiler.bmb` | 부트스트랩 컴파일러 (32K+ LOC) |
| `ecosystem/gotgan-bmb/gotgan.bmb` | 패키지 매니저 (lock/verify 추가됨) |
| `tests/golden/test_golden_effect_row.bmb` | M12 effect row 골든 테스트 |
| `tests/golden/test_golden_intent_annotation.bmb` | M13 intent 골든 테스트 |
| `tests/golden/test_golden_platform.bmb` | M15 platform 골든 테스트 |
| `claudedocs/ROADMAP.md` | 실무 앵커 (§ 6 AI-Native Pivot + 진척 표) |
