# BMB Session Handoff — 2026-05-30 (Cycles 3315-3323)

> **HEAD**: `ee17c8e4` (feat(diagnose/m15): enforce_module_caps + Cross-gen FP S2==S3)
> **실무 앵커**: `claudedocs/ROADMAP.md` (§ 6 AI-Native Pivot)
> **전략 계획서**: `claudedocs/plans/ai-native-plan-2026.md`

---

## 현재 상태 스냅샷

| 항목 | 상태 |
|------|------|
| cargo test --release | ✅ 3800+2390+23 tests, 0 FAILED |
| Within-gen Fixed Point | ✅ fp3321a.ll == fp3321b.ll (Cycle 3321) |
| Cross-gen Fixed Point | ✅ S2 IR == S3 IR (Cycle 3322) |
| bmb lint warnings | ✅ 178 non-recursive (pre-existing) |
| Z3 verify | ✅ 144/144 |
| P-track 7/7 | ✅ ALL ≤1.010× |
| B-axis Claude | ✅ 98.0% (stale: 2026-08-13) |
| B-axis GPUStack | ✅ 100.0% (2026-05-21) |

---

## 이번 세션 완료 (Cycles 3315-3323)

| 마일스톤 | 완료 사이클 | 내용 |
|---------|-----------|------|
| diagnose summary 섹션 | 3315 | json_extract_int_field + total_issues/effect_issues/contract_issues/lint_issues/duplicate_pairs |
| forbid_function 규칙 | 3316 | bc_check_forbid_fn + .bmb-contracts forbid_function = fn_name |
| P1 Phase 1 (effect_verify) | 3317 | count_caller_entries `{"type":` + eff_emit_viol_pair/pure_calls/missing_annot 형식 통일 |
| P1 Phase 2 (contracts+lint) | 3318 | count_rule_entries `{"type":` + 8개 "rule": → "type:" 교체 |
| P1 Phase 3 (semantic_dup) | 3319 | count_fn_a_entries `{"type":` + semdp_inner fn_a+fn_b → type+function+similar_to |
| 커밋 | 3320 | `03f6ec80` — 8 files, 256 ins |
| M15 Phase 6a | 3321 | enforce_module_caps contracts 규칙 (bc_check_module_cap_fn/bc_check_module_caps_scan) |
| Cross-gen FP S2==S3 | 3322 | `ee17c8e4` — 5 files, 135 ins |
| HANDOFF 갱신 | 3323 | 세션 종료 정리 |

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
    "total_issues":0,"effect_issues":0,"contract_issues":0,"lint_issues":0,"duplicate_pairs":0
  }
}
```

**이번 세션 신규 기능**:
- `summary` 섹션: AI가 단일 필드로 파일 품질 파악 (`total_issues=0` → 안전)
- `forbid_function` contracts 규칙: 특정 함수 직접 호출 금지
- **violations 형식 통일**: 모든 위반/경고가 `{"type":"...","function":"...",...}` 패턴
  - effect_verify: `effect_mismatch` / `pure_violation` / `missing_annotation`
  - contracts_check: `max_params` / `require_postcondition` / `forbid_effect` / `forbid_function` / `require_effect_annotation` / `module_capability`
  - lint_effects: `effect_pure_violation` / `effect_propagation` / `missing_effect_annotation`
  - semantic_duplicate: `semantic_duplicate` (function + similar_to)
- `enforce_module_caps = true`: module requires [...] 선언 기반 capability 강제

---

## 즉시 실행 가능한 다음 태스크

### [P1] enforce_module_caps 고도화

**배경**: Phase 6a 완료. Phase 6b: declared 필드를 JSON 배열로 개선 + diagnose 통합.
**구현**: `"declared":["IO","Net"]` 형식 (현재 `"declared":[IO]` — 문자열만)
**복잡도**: 1-2 사이클.

### [P2] M15 Phase 6b — diagnose에 module_capability 섹션 추가

**배경**: enforce_module_caps 결과를 diagnose JSON에 별도 섹션으로 포함.
**구현**: `"module_capability": {"total_violations":N,"violations":[...]}`
**복잡도**: 1-2 사이클.

### [P3] violations 형식 문서화

**배경**: 새 통일 형식을 LANGUAGE_REFERENCE 또는 ARCHITECTURE.md에 공식 문서화.
**복잡도**: 1 사이클.

### [P4] count_viol_entries 통합 리네이밍

**배경**: count_caller_entries/count_rule_entries/count_fn_a_entries가 동일한 `{"type":` 패턴 사용.
**구현**: 하나의 `count_viol_entries` 함수로 통합.
**복잡도**: 1 사이클 (단순 리팩토링).

### [P5] M15 Phase 6c — 런타임 capability sandbox

**배경**: 컴파일러가 module capability 선언을 강제하는 것을 넘어 실제 샌드박싱.
**복잡도**: 5-7 사이클.

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
- **Fixed Point**: `compiler_s1.exe emit-ir src out1.ll` 두 번 실행하여 동일성 확인.
- **Z3 경로**: `z3`는 C:/msys64/ucrt64/bin/z3.exe, PATH 접근 가능.
- **Stage 1 재빌드**: compiler.bmb 변경 후 반드시 `bootstrap/compiler.exe build bootstrap/compiler.bmb -o bootstrap/compiler_s1.exe` (BMB_ARENA_MAX_SIZE=32G).
- **violations 형식**: 이제 모든 위반/경고가 `{"type":"...","function":"...",...}` — 이전 `{"rule":...}`, `{"caller":...}`, `{"fn_a":...}` 형식은 모두 제거됨.

---

## 주요 파일

| 파일 | 역할 |
|------|------|
| `bootstrap/compiler.bmb` | 부트스트랩 컴파일러 (~47K+ LOC) |
| `ecosystem/bmb-mcp/mcp_server.bmb` | MCP 서버 (10 tools) |
| `claudedocs/ROADMAP.md` | 실무 앵커 (§ 6 AI-Native Pivot) |
