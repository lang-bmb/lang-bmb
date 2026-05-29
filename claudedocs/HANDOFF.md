# BMB Session Handoff — 2026-05-29 (Cycles 3261-3270)

> **HEAD**: `2ddbb93e` (M12 Phase 3 + M13 Phase 3 Full + M14 Phase 3 + M15 Phase 2)
> **실무 앵커**: `claudedocs/ROADMAP.md` (§ 6 AI-Native Pivot + M12-M15 진척)
> **전략 계획서**: `claudedocs/plans/ai-native-plan-2026.md`

---

## 현재 상태 스냅샷

| 항목 | 상태 |
|------|------|
| cargo test --release | ✅ 2390 tests, 0 FAILED |
| 3-Stage Fixed Point | ✅ S2 == S3 (Cycle 3269) |
| bmb lint warnings | ✅ 178 non-recursive (177 pre-existing + 1 [complex] repair_hint_scan) |
| Z3 verify | ✅ 141/141 |
| P-track 7/7 | ✅ ALL ≤1.010× |
| B-axis Claude | ✅ 98.0% (stale: 2026-08-13) |
| B-axis GPUStack | ✅ 100.0% (2026-05-21) |

---

## 이번 세션 완료 (Cycles 3261-3270)

| 마일스톤 | 완료 사이클 | 내용 |
|---------|-----------|------|
| M14 Phase 3 | 3261 | `gotgan add <name> <path>` 명령 + lock 업데이트 |
| M12 Phase 3 | 3262 | `[effect_propagation]` lint: callee effect ⊄ caller effect → warn |
| M15 Phase 2 | 3263 | platform { fn ... }: <X> } → effect map 등록 (`PLAT:` prefix) |
| M13 Phase 3 | 3264 | `repair-hint` 명령 스텁 (JSON `{type:repair_hints, file:..., functions:[]}`) |
| M13 Phase 3 Full | 3267 | `repair-hint` full: pre/post/intent JSON 출력 |
| Fixed Point | 3265+3269 | 3-Stage S2 == S3 ✅ |

---

## 새로 추가된 기능 요약

### M14 Phase 3: gotgan add

```
$ gotgan add my-lib ../my-lib
Added: my-lib = { path = "../my-lib" }
```

- `[dependencies]` 섹션 없으면 자동 생성
- `gotgan.lock` 존재 시 SHA-256 자동 업데이트
- 중복 dep 방지

### M12 Phase 3: Effect Callee Propagation

```bmb
fn fetch(): <Net> -> String = "data";
fn handler(): <IO> -> i64 = { let _d = fetch(); 0 };
// [effect_propagation] handler: declares <IO> but calls fetch which uses <Net>
```

- `build_fn_effect_map`: 소스 스캔으로 효과 맵 구성
- `lint_check_effect_propagation`: 호출자 선언 effect ⊆ 피호출자 effect 검증
- `eff_contains_name`, `check_callee_missing_effects` 등 헬퍼

### M15 Phase 2: Platform Capabilities

```bmb
platform stdlib {
    fn io_print(s: String): <IO> -> i64;
    fn net_fetch(url: String): <Net> -> String;
}
// io_print, net_fetch가 effect propagation 검사의 callee로 활용됨
```

- `scan_platform_effects`: platform { } 블록 내 fn 효과 추출
- `PLAT:` prefix: platform 선언 함수는 callee 조회에만 사용, 본인 검사 제외
- `eff_map_get_raw`: raw 조회 (PLAT: prefix 포함)

### M13 Phase 3: Repair-hint Full

```
$ compiler.exe repair-hint foo.bmb
{"type":"repair_hints","file":"foo.bmb","functions":[
  {"name":"add","intent":"Adds two numbers","pre":["x >= 0","y >= 0"],"post":["it >= 0"]}
]}
```

- `extract_pre_texts` / `extract_post_texts` / `extract_intent_tok` — contract 소스 텍스트 추출
- `emit_fn_hint` (6 params) — JSON emitter
- `repair_hint_scan` — 전체 소스 스캔
- **학습**: `let x = e1; e2; e3;` 체인에서 `e2; e3`는 `{ }` 블록 없이 불가 → `let _z = e2; e3;` 필수

---

## 즉시 실행 가능한 다음 태스크

### M13 Phase 4 — Repair Loop 자동화

**목표**: `bmb verify` 실패 → `repair-hint` → LLM 재시도 루프 자동화
**연관**: M13 Phase 3 완료됨 (repair-hint 명령 ✅). 다음은 자동 재시도 인프라.

```
bmb verify foo.bmb  →  실패 시 repair-hint JSON 출력
→ MCP suggest_contracts 연동 (M7-4 인프라 재활용)
→ LLM이 hint 기반으로 k회 재시도하는 테스트 루프
```

### M12 Phase 4 — Effect 전이 전파 (Transitive)

**목표**: A calls B calls C → A must declare C's effects (현재는 direct-callee only)

**구현 방법**:
1. `build_transitive_effect_map(entries, eff_map)` — 모든 호출자의 전이 효과 계산
2. `lint_check_effect_propagation`에서 direct + transitive 모두 검사

### M12 Phase 5 — Effect Inference

**목표**: 어노테이션 없는 함수의 effect를 body 분석으로 자동 추론
- `infer_fn_effect(src, fn_name, entries)` → "IO Net" 등 자동 제안
- `[missing_effect_annotation]` lint: 추론된 effect가 있으나 어노테이션 없음

---

## 보류/HUMAN-blocked 항목

| 항목 | 이유 |
|------|------|
| B-axis 재측정 (Claude) | ANTHROPIC_API_KEY 필요 (stale: 2026-08-13) |
| v1.0 선언 | 외부 신호 대기 |

---

## 주의사항

- **Rule 6**: 모든 새 기능은 bootstrap/compiler.bmb에서만.
- **Python write 금지**: bootstrap/compiler.bmb 수정 시 Python으로 직접 파일 write 금지. Python text mode `'w'`가 `\n`→`\r\n` 변환. Edit 도구 사용 필수.
- **BMB 예약어**: `pre`, `post`, `ret`, `type`, `for`, `in`, `let`, `fn`, `if`, `else`, `and`, `or`, `not`, `mut`, `set`, `as`, `bor` 등은 변수명/파라미터명으로 사용 불가.
- **M12 lint**: effect propagation lint는 bootstrap의 `lint_file` 경로 (Rust-side `bmb lint`와 별개). `bootstrap/compiler.exe lint` 로 실행.
- **fixed point**: 항상 S2 IR vs S3 IR 비교 (binary hash 아님).

---

## 주요 파일 위치

| 파일 | 역할 |
|------|------|
| `bootstrap/compiler.bmb` | 부트스트랩 컴파일러 (32K+ LOC) |
| `ecosystem/gotgan-bmb/gotgan.bmb` | 패키지 매니저 (add 추가됨) |
| `tests/golden/test_golden_effect_propagation.bmb` | M12 Phase 3 골든 테스트 |
| `tests/golden/test_golden_repair_hint.bmb` | M13 Phase 3 골든 테스트 |
| `claudedocs/ROADMAP.md` | 실무 앵커 (§ 6 AI-Native Pivot + 진척 표) |
