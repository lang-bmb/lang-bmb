# BMB Session Handoff — 2026-05-29 (Cycles 3296-3305)

> **HEAD**: `a6543120` (chore(docs): 세션 종료 정리 — Cycles 3296-3305)
> **실무 앵커**: `claudedocs/ROADMAP.md` (§ 6 AI-Native Pivot + M12-M15 진척)
> **전략 계획서**: `claudedocs/plans/ai-native-plan-2026.md`

---

## 현재 상태 스냅샷

| 항목 | 상태 |
|------|------|
| cargo test --release | ✅ 3800+2390+23 tests, 0 FAILED |
| Within-gen Fixed Point | ✅ fp3302a.ll == fp3302b.ll (Cycle 3302) |
| bmb lint warnings | ✅ 178 non-recursive (pre-existing) |
| Z3 verify | ✅ 144/144 |
| P-track 7/7 | ✅ ALL ≤1.010× |
| B-axis Claude | ✅ 98.0% (stale: 2026-08-13) |
| B-axis GPUStack | ✅ 100.0% (2026-05-21) |

---

## 이번 세션 완료 (Cycles 3296-3305)

| 마일스톤 | 완료 사이클 | 내용 |
|---------|-----------|------|
| P2 버그 수정 | 3296 | index_source/query_source platform 블록 스킵 |
| P3 set-equality | 3297 | eff_set_equals — module-suggest 순서 무관 비교 |
| P1 diagnose CLI | 3298 | effect-verify + contracts-check 통합 JSON |
| Fixed Point | 3299 | within-gen FP ✅, 커밋 `78ed63b7` |
| ROADMAP | 3300 | § 6 타임라인 P1-P3 완료 마킹 |
| lint_effects | 3301 | diagnose에 lint_effects 섹션 (3종 JSON 빌더) |
| Fixed Point + 커밋 | 3302 | `cc01c81d` |
| bmb-mcp | 3303 | bmb_diagnose MCP tool (도구 10번째) |
| 커밋 | 3304 | `dabb82be` |
| 메모리/세션 종료 | 3305 | MEMORY.md + HANDOFF + 최종 커밋 `a6543120` |

---

## 신규 기능 요약

### diagnose CLI (`compiler.exe diagnose src.bmb`)

```bash
$ compiler.exe diagnose test.bmb
{
  "type": "diagnose",
  "file": "test.bmb",
  "effect_verify": {"type":"effect_verify","status":"safe","z3":"sat"},
  "contracts_check": {"type":"contracts_check","status":"safe"},
  "lint_effects": {
    "type": "lint_effects",
    "warnings": [
      {"rule":"missing_effect_annotation","function":"foo","inferred_effect":"IO"}
    ]
  }
}
```

**내부 구조**:
- `eff_verify_build_json(input, entries, eff_map, transitive_map) -> String`
- `cc_build_json(input, src, entries, eff_map, transitive_map, contracts) -> String`
- `lint_effects_build_json(input, entries, eff_map, transitive_map) -> String`
  - `lint_eff_pure_viol_sb` + `lint_eff_propagation_sb` + `lint_missing_eff_sb`
- entries/eff_map/transitive_map **1회 계산** (성능)

### bmb-mcp: bmb_diagnose 도구 (ecosystem/bmb-mcp)

```json
{"name": "bmb_diagnose", "description": "Unified effect diagnostics: effect-verify + contracts-check + lint-effects"}
```

### index/query platform 버그 수정

`index_source`와 `query_source`에 TK_IDENT "platform" → `skip_platform_block` 분기 추가.

### module-suggest eff_set_equals

`eff_subset(a,b,pos)` + `eff_set_equals(a,b)` — "IO File" == "File IO" → 순서 무관 비교.

---

## 즉시 실행 가능한 다음 태스크

### [P1] diagnose 경고 count 필드 추가

**배경**: `lint_effects.warnings` 배열 길이를 AI가 파악하려면 카운트 필요.
**구현**: `lint_effects_build_json` 에서 count 집계 + `"count":N` 필드 추가.

### [P2] M12 Z3 lattice 확장 — missing_annotation formal

**배경**: 현재 `missing_effect_annotation`은 heuristic transitive scan. Z3 formal certification 없음.
**방향**: `transitive_map` 기반 SMT 모델 확장 → Z3가 annotation 누락도 formal 검증.
**복잡도**: 2-3 사이클.

### [P3] cross-gen Fixed Point 검증 (HANDOFF P5)

**배경**: Within-gen FP ✅. Cross-gen(S2 IR vs S3 IR) 은 `-1` vs `18446744073709551615` 표현 차이 존재.
**방향**: `sed` 정규화 후 비교.
**필요**: llc + gcc 링크 (~40분).

### [P4] .bmb-contracts max_params 규칙 구현

**배경**: `contracts_check_run`에 `max_params` 파싱은 있으나 실제 체크 없음.
**수정**: `bc_check_max_params_scan(entries, max_n, viol_sb, isfirst)` 추가.

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

---

## 주요 파일

| 파일 | 역할 |
|------|------|
| `bootstrap/compiler.bmb` | 부트스트랩 컴파일러 (~47K+ LOC) |
| `ecosystem/bmb-mcp/mcp_server.bmb` | MCP 서버 (10 tools) |
| `claudedocs/ROADMAP.md` | 실무 앵커 (§ 6 AI-Native Pivot) |
