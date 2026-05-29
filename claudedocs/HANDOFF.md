# BMB Session Handoff — 2026-05-29 (Cycles 3306-3314)

> **HEAD**: `04520838` (chore(docs): 세션 종료 정리 Cycles 3306-3314)
> **실무 앵커**: `claudedocs/ROADMAP.md` (§ 6 AI-Native Pivot)
> **전략 계획서**: `claudedocs/plans/ai-native-plan-2026.md`

---

## 현재 상태 스냅샷

| 항목 | 상태 |
|------|------|
| cargo test --release | ✅ 3800+2390+23 tests, 0 FAILED |
| Within-gen Fixed Point | ✅ fp3314a.ll == fp3314b.ll (Cycle 3314) |
| Cross-gen Fixed Point | ✅ S2 IR == S3 IR (Cycle 3310) |
| bmb lint warnings | ✅ 178 non-recursive (pre-existing) |
| Z3 verify | ✅ 144/144 |
| P-track 7/7 | ✅ ALL ≤1.010× |
| B-axis Claude | ✅ 98.0% (stale: 2026-08-13) |
| B-axis GPUStack | ✅ 100.0% (2026-05-21) |

---

## 이번 세션 완료 (Cycles 3306-3314)

| 마일스톤 | 완료 사이클 | 내용 |
|---------|-----------|------|
| P1 count 필드 | 3306 | lint_effects_build_json count_rule_entries 카운터 |
| P4 max_params | 3307 | bc_check_max_params_scan + count_top_commas + count_sig_params |
| FP 검증 + 커밋 | 3308 | within-gen FP ✅ + 커밋 `da068fb4` |
| P2 Z3 formal | 3309 | eff_z3_gen_missing_anno_sb contradiction pairs + transitive_map 통합 |
| P3 cross-gen FP | 3310 | S2==S3 IR ✅ + 커밋 `2c4e35e7` |
| ROADMAP 갱신 | 3311 | 진척 현황 표 + 타임라인 갱신 |
| violations_count | 3312 | effect_verify+contracts_check violations_count:N + count_caller_entries |
| semantic_duplicate | 3313 | semdp_build_json + diagnose 4번째 섹션 통합 |
| FP + 커밋 | 3314 | within-gen FP ✅ + 커밋 `80fb861e` |

---

## diagnose 통합 현황 (4섹션)

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
  }
}
```

**새 기능 요약**:
- `lint_effects.count`: 경고 총 수
- `effect_verify.violations_count`: 위반 총 수  
- `contracts_check.violations_count`: 규칙 위반 총 수
- `semantic_duplicate.pairs_count`: 중복 쌍 수
- `.bmb-contracts max_params = N`: 파라미터 수 제한 검사
- `eff_z3_gen_missing_anno_sb`: Z3 formal missing_annotation 인증

---

## 즉시 실행 가능한 다음 태스크

### [P1] violations 형식 통일 (기술부채)

**배경**: 현재 violations 형식 불일치:
- `effect_verify`: `{"caller":"...", "callee":"...",...}`
- `contracts_check`: `{"rule":"...", "function":"...",...}`
- `semantic_duplicate`: `{"fn_a":"...", "fn_b":"...",...}`

**방향**: 모두 `{"type":"...", "function":"...",...}` 형식으로 통일.
**복잡도**: 3-4 사이클 (포맷 변경 + 카운터 함수 통합).

### [P2] diagnose summary 섹션

**배경**: AI가 단일 필드로 파일 품질 요약 파악 가능.
**구현**: `"summary": {"total_issues": N, "effect_issues": N, "contract_issues": N, "lint_issues": N, "duplicate_pairs": N}`

### [P3] M15 Phase 6: capability enforcement

**배경**: `module X requires [IO, Net]` 선언 기반 런타임 capability 강제.
**복잡도**: 5-7 사이클.

### [P4] contracts-check 새 규칙: forbid_function

**배경**: 특정 함수 직접 호출 금지 (e.g. `forbid_function = println_str` → wrapper 사용 강제).
**구현**: `bc_check_forbid_fn_scan` + `.bmb-contracts forbid_function = fn_name`.
**복잡도**: 1 사이클.

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

---

## 주요 파일

| 파일 | 역할 |
|------|------|
| `bootstrap/compiler.bmb` | 부트스트랩 컴파일러 (~47K+ LOC) |
| `ecosystem/bmb-mcp/mcp_server.bmb` | MCP 서버 (10 tools) |
| `claudedocs/ROADMAP.md` | 실무 앵커 (§ 6 AI-Native Pivot) |
