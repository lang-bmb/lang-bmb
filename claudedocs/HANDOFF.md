# BMB Session Handoff — 2026-05-30 (Cycles 3315-3323)

> **HEAD**: `ff9c0f8b` (chore(docs): cycle-3323 세션 종료 로그 추가)
> **실무 앵커**: `claudedocs/ROADMAP.md` (§ 6 AI-Native Pivot)
> **전략 계획서**: `claudedocs/plans/ai-native-plan-2026.md`

---

## 현재 상태 스냅샷

| 항목 | 상태 |
|------|------|
| cargo test --release | ✅ 3800+47+22+2390+23 = 6282, 0 FAILED |
| Within-gen Fixed Point | ✅ fp3321a.ll == fp3321b.ll (Cycle 3321) |
| Cross-gen Fixed Point | ✅ S2 IR == S3 IR (Cycle 3322) |
| bmb lint warnings | ✅ 178 non-recursive (pre-existing) |
| Z3 verify | ✅ 144/144 |
| P-track 7/7 | ✅ ALL ≤1.010× |
| B-axis Claude | ✅ 98.0% (stale: 2026-08-13) |
| B-axis GPUStack | ✅ 100.0% (2026-05-21) |
| diagnose compiler.bmb | 352 semantic_duplicate (pre-existing trivial), 0 others |

---

## 이번 세션 완료 (Cycles 3315-3323)

| 마일스톤 | 완료 사이클 | 내용 |
|---------|-----------|------|
| diagnose summary 섹션 | 3315 | `json_extract_int_field` + `{"total_issues":N,...}` |
| contracts-check forbid_function | 3316 | `bc_check_forbid_fn` + `.bmb-contracts forbid_function = fn_name` |
| P1 Phase 1 (effect_verify) | 3317 | `count_caller_entries` `{"type":` + eff_emit_viol_pair/pure_calls/missing_annot 형식 통일 |
| P1 Phase 2 (contracts+lint) | 3318 | `count_rule_entries` `{"type":` + 8종 `"rule":` → `"type":` |
| P1 Phase 3 (semantic_dup) | 3319 | `count_fn_a_entries` `{"type":` + fn_a+fn_b → type+function+similar_to |
| 커밋 | 3320 | `03f6ec80` — 8 files, 256 ins |
| M15 Phase 6a (enforce_module_caps) | 3321 | `bc_check_module_cap_fn` + `bc_check_module_caps_scan` + cc_build_json 통합 |
| Cross-gen FP S2==S3 | 3322 | 검증 완료 + `ee17c8e4` |
| 세션 종료 정리 | 3323 | HANDOFF/ROADMAP 갱신 + `ff9c0f8b` |

---

## diagnose 통합 현황 (4섹션 + summary)

```bash
$ compiler.exe diagnose test.bmb
{
  "type": "diagnose",
  "file": "test.bmb",
  "effect_verify": {
    "type":"effect_verify","status":"safe","z3":"sat","violations_count":0
  },
  "contracts_check": {
    "type":"contracts_check","status":"safe","violations_count":0
  },
  "lint_effects": {
    "type":"lint_effects","count":0,"warnings":[]
  },
  "semantic_duplicate": {
    "type":"semantic_duplicate","pairs_count":0,"pairs":[]
  },
  "summary": {
    "total_issues":0,
    "effect_issues":0,
    "contract_issues":0,
    "lint_issues":0,
    "duplicate_pairs":0
  }
}
```

### violations 통일 형식 (이번 세션 완성)

모든 위반/경고 항목: `{"type":"규칙명", "function":"함수명", ...}`

| 출처 | type 값 | 추가 필드 |
|------|---------|---------|
| effect_verify | `effect_mismatch` | callee, caller_effect, callee_effect, message |
| effect_verify | `pure_violation` | callee, callee_effect, message |
| effect_verify | `missing_annotation` | missing_effect, message |
| contracts_check | `max_params` | params, max, message |
| contracts_check | `require_postcondition` | message |
| contracts_check | `forbid_effect` | message |
| contracts_check | `forbid_function` | forbidden, message |
| contracts_check | `require_effect_annotation` | message |
| contracts_check | `module_capability` | effect, declared, message |
| lint_effects | `effect_pure_violation` | message |
| lint_effects | `effect_propagation` | callee, caller_effect, missing_cap |
| lint_effects | `missing_effect_annotation` | inferred_effect |
| semantic_duplicate | `semantic_duplicate` | similar_to, shared_calls, total_a, total_b |

### .bmb-contracts 지원 규칙 (현재)

```
require_postcondition = true
require_effect_annotation = true
forbid_effect = IO
forbid_function = println_str
max_params = 5
enforce_module_caps = true
```

---

## 즉시 실행 가능한 다음 태스크

### [P1] declared 필드 JSON 배열 형식 개선 (1 사이클)

**배경**: `module_capability` violation의 `declared` 필드가 공백구분 문자열 (`[IO]` — 중괄호도 없음).
**방향**: `"declared":["IO","Net"]` JSON 배열 형식으로 개선.
**파일**: `bootstrap/compiler.bmb` → `bc_check_module_cap_fn` 함수의 JSON 생성 부분.
**복잡도**: 1 사이클.

### [P2] M15 Phase 6b — diagnose에 module_capability 전용 섹션 추가 (1-2 사이클)

**배경**: `enforce_module_caps` 결과가 `contracts_check.violations[]`에 섞임.
**방향**: `"module_capability": {"total_violations":N,"declared":["IO"],"violations":[...]}` 별도 섹션.
**복잡도**: 2 사이클.

### [P3] count_viol_entries 통합 리팩토링 (1 사이클)

**배경**: `count_caller_entries`, `count_rule_entries`, `count_fn_a_entries` 세 함수가 동일 패턴 `{"type":` 사용.
**방향**: 하나의 `count_viol_entries(s, pos)` 함수로 통합하고 기존 3개는 alias 처리.
**복잡도**: 1 사이클. 순수 리팩토링, 동작 변화 없음.

### [P4] M15 Phase 6c — 런타임 sandbox (5-7 사이클)

**배경**: 현재는 compile-time 체크만. 실제 런타임에서 capability 강제 필요.
**방향**: platform 선언 기반 런타임 샌드박싱 — AI 생성 코드 자동 제한.
**복잡도**: 5-7 사이클. 대규모 변경.

### [P5] bmb-mcp diagnose 도구 violations 형식 반영 (1 사이클)

**배경**: MCP `bmb_diagnose` 도구의 스키마/설명이 구 형식 (`"rule":`, `"fn_a":`) 기반일 수 있음.
**방향**: 새 통일 형식 `{"type":"...","function":"..."}` 기준으로 MCP 스키마 문서 업데이트.
**파일**: `ecosystem/bmb-mcp/mcp_server.bmb`.
**복잡도**: 1 사이클.

---

## 미비/결함/개선 도출

| 유형 | 내용 | 심각도 |
|------|------|--------|
| 미비 | `declared` 필드 JSON 배열 미완성 (`"declared":[IO]` — 문자열) | P2 |
| 미비 | module_capability가 contracts_check에 섞임 — 전용 섹션 없음 | P2 |
| 개선 | count_caller/rule/fn_a_entries 3중복 함수 통합 가능 | P3 |
| 미비 | MCP bmb_diagnose 스키마가 구 형식 기준일 수 있음 | P3 |
| 개선 | M15 Phase 6c 런타임 sandbox — 대규모, 장기 | P4 |

---

## 보류/HUMAN-blocked 항목

| 항목 | 이유 |
|------|------|
| B-axis 재측정 (Claude) | ANTHROPIC_API_KEY 필요 (stale: 2026-08-13) |
| v1.0 선언 | 외부 신호 대기 |

---

## 주의사항

- **Rule 6**: 모든 새 기능은 `bootstrap/compiler.bmb`에서만.
- **Python write 금지**: `bootstrap/compiler.bmb` 수정 시 `'wb'` 모드 필수.
- **Fixed Point 검증**: `.\bootstrap\compiler_s1.exe emit-ir bootstrap\compiler.bmb out1.ll` 2회 → 동일성 확인.
- **Stage 1 재빌드**: `bootstrap\compiler.exe build bootstrap\compiler.bmb -o bootstrap\compiler_s1.exe` (환경변수: `$env:BMB_ARENA_MAX_SIZE="32G"`).
- **violations 형식 변경**: 이전 `{"rule":...}`, `{"caller":...}`, `{"fn_a":...}` 완전 제거됨.
- **카운터 함수**: `count_caller_entries`, `count_rule_entries`, `count_fn_a_entries` 모두 `{"type":` 패턴 사용 (이름은 레거시).

---

## 주요 파일

| 파일 | 역할 |
|------|------|
| `bootstrap/compiler.bmb` | 부트스트랩 컴파일러 (~47K+ LOC) |
| `ecosystem/bmb-mcp/mcp_server.bmb` | MCP 서버 (10 tools) |
| `claudedocs/ROADMAP.md` | 실무 앵커 (§ 6 AI-Native Pivot) |
