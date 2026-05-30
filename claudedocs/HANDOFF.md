# BMB Session Handoff — 2026-05-30 (Cycles 3315-3323 + 재점검)

> **HEAD**: `c94458f6` (docs(roadmap): 핵심 목표 + 10-레이어 반복 사이클 구조 추가)
> **실무 앵커**: `claudedocs/ROADMAP.md` (§ 6 AI-Native Pivot)
> **전략 계획서**: `claudedocs/plans/ai-native-plan-2026.md`

---

## 현재 상태 스냅샷

| 항목 | 상태 |
|------|------|
| cargo test --release | ✅ 6282 PASS, 0 FAILED |
| Within-gen Fixed Point | ✅ fp3321a.ll == fp3321b.ll (Cycle 3321) |
| Cross-gen Fixed Point | ✅ S2 IR == S3 IR (Cycle 3322) |
| bmb lint warnings | ✅ 0 (178 non-recursive pre-existing 제외) |
| Z3 verify | ✅ 144/144 |
| P-track 7/7 (Rust 컴파일러) | ✅ ALL ≤1.010× BMB faster than C |
| Bootstrap P-track | ⚠️ csv 1.134×❌ / lexer 1.459×❌ (tuple calloc 오버헤드) |
| B-axis Claude | ✅ 98.0% (stale: 2026-08-13) |
| B-axis GPUStack | ✅ 100.0% (2026-05-21) |
| diagnose compiler.bmb | 352 semantic_duplicate (pre-existing trivial), 0 others |

---

## 이번 세션 완료 (Cycles 3315-3323 + 세션 종료)

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
| **docs/ROADMAP.md 핵심목표+10-레이어** | 세션종료 | 핵심 목표(Performance/AI-Native) + 개발 레이어 구조 영속화 + 전체 재점검 `c94458f6` |

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

> 재점검 (2026-05-30) 기준 도출. 10-레이어 구조 기준으로 분류.

### [P1 / L2 컴파일러] declared 필드 JSON 배열 형식 개선 (1 사이클)

**배경**: `module_capability` violation의 `declared` 필드가 공백구분 문자열 (`[IO]` — 중괄호도 없음).
**방향**: `"declared":["IO","Net"]` JSON 배열 형식으로 개선.
**파일**: `bootstrap/compiler.bmb` → `bc_check_module_cap_fn` 함수의 JSON 생성 부분.
**복잡도**: 1 사이클.

### [P2 / L5 AI-Native] M15 Phase 6b — diagnose module_capability 전용 섹션 (1-2 사이클)

**배경**: `enforce_module_caps` 결과가 `contracts_check.violations[]`에 섞임.
**방향**: `"module_capability": {"total_violations":N,"declared":["IO"],"violations":[...]}` 별도 섹션.
**복잡도**: 2 사이클.

### [P2 / L4 벤치마크 ← L1 언어사양] bootstrap P-track 회귀 분석 (2-3 사이클)

**배경**: bootstrap 컴파일러로 컴파일한 벤치마크에서 csv 1.134×❌, lexer 1.459×❌. Rust 컴파일러 기준은 7/7 ✅.
**원인 가설**: tuple 타입이 항상 heap calloc(2-word) 형태 → 고빈도 호출 시 오버헤드 누적.
**방향**: IR 비교(bootstrap vs Rust) → 근본 원인이 tuple 표현이면 **L1(언어사양) 재검토** (stack-allocated tuple 가능성).
**복잡도**: 분석 1 사이클 + 수정 1-2 사이클.

### [P3 / L2 컴파일러] count_viol_entries 통합 리팩토링 (1 사이클)

**배경**: `count_caller_entries`, `count_rule_entries`, `count_fn_a_entries` 세 함수가 동일 패턴.
**방향**: 하나의 `count_viol_entries(s, pos)` 로 통합, 기존 3개는 alias.
**복잡도**: 1 사이클. 순수 리팩토링.

### [P3 / L6 에코시스템] MCP bmb_diagnose 스키마 신 형식 업데이트 (1 사이클)

**배경**: MCP `bmb_diagnose` 도구의 스키마/설명이 구 형식 기반일 수 있음.
**방향**: 신 통일 형식 `{"type":"...","function":"..."}` 기준으로 MCP 스키마 업데이트.
**파일**: `ecosystem/bmb-mcp/mcp_server.bmb`.
**복잡도**: 1 사이클.

### [P4 / L5 AI-Native] M15 Phase 6c 런타임 sandbox (5-7 사이클)

**배경**: 현재는 compile-time 체크만. platform 선언 기반 런타임 capability 강제 필요.
**복잡도**: 5-7 사이클. 대규모. 장기 항목.

---

## 미비/결함/개선 도출

| 레이어 | 유형 | 내용 | 심각도 |
|--------|------|------|--------|
| L2 컴파일러 | 미비 | `declared` 필드 JSON 배열 미완성 (`"declared":[IO]` — 문자열) | P1 |
| L5 AI-Native | 미비 | module_capability가 contracts_check에 섞임 — 전용 섹션 없음 | P2 |
| L4 벤치마크 | 결함 | bootstrap P-track csv 1.134× / lexer 1.459× 미통과 | P2 |
| L1 언어사양 | 개선 | tuple 표현이 heap-only → stack-allocated tuple 가능성 검토 필요 | P2 |
| L2 컴파일러 | 개선 | count_caller/rule/fn_a_entries 3중복 함수 통합 가능 | P3 |
| L6 에코시스템 | 미비 | MCP bmb_diagnose 스키마가 구 형식 기준일 수 있음 | P3 |
| L5 AI-Native | 개선 | M15 Phase 6c 런타임 sandbox — 대규모, 장기 | P4 |

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
