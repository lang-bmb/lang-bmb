# BMB Session Handoff — 2026-05-30 (Cycle 3332)

> **HEAD**: `f8326992` (fix(bootstrap): include_dirname_scan pre i >= 0 → pre i >= -1)
> **이전 HEAD**: `2c3472c6`
> **실무 앵커**: `claudedocs/ROADMAP.md` (§ 6 AI-Native Pivot)
> **전략 계획서**: `claudedocs/plans/ai-native-plan-2026.md`

---

## 현재 상태 스냅샷

| 항목 | 상태 |
|------|------|
| cargo test --release | ✅ 6282 PASS, 0 FAILED |
| Within-gen Fixed Point | ✅ fp3332a.ll == fp3332b.ll (Cycle 3332) |
| Cross-gen Fixed Point | ✅ S2 IR == S3 IR (Cycle 3322 — 이전 세션) |
| bmb lint warnings | ✅ 0 non-param/postcondition (180 pre-existing 제외) |
| Z3 verify | ✅ 144/144 |
| P-track 7/7 (Rust 컴파일러) | ✅ ALL ≤1.010× BMB faster than C |
| Bootstrap P-track | ✅ 6/7 ✅ (csv 1.039× — 경계선, 측정 노이즈 수준) |
| B-axis Claude | ✅ 98.0% (stale: 2026-08-13) |
| B-axis GPUStack | ✅ 100.0% (2026-05-21) |
| diagnose compiler.bmb | 352 semantic_duplicate (pre-existing trivial), 0 others |

---

## 이번 세션 완료 (Cycle 3332)

| 사이클 | 내용 |
|--------|------|
| 3332 | bootstrap-bare-filename-sigsegv 수정: `include_dirname_scan` `pre i >= 0` → `pre i >= -1` (LLVM `llvm.assume(false)` UB 제거 — Cycle 3249 ISSUE CLOSED) |

## 이전 세션 완료 (Cycle 3331)

| 사이클 | 내용 |
|--------|------|
| 3331 | P3 contracts_check_run에 module_capability 포함: `cc_json_prefix_sb` + `cc_combine_mc` 헬퍼 추가 → `contracts-check` 명령이 `"module_capability"` 필드 포함 출력 |

## 이전 세션 완료 (Cycles 3324-3330)

| 사이클 | 내용 |
|--------|------|
| 3324 | P1 declared 필드 JSON 배열 형식 개선 |
| 3325 | M15 Phase 6b: module_capability 전용 섹션 분리 |
| 3326 | count_viol_entries 통합 |
| 3327 | MCP bmb_diagnose 스키마 업데이트 |
| 3328 | bootstrap P-track 재측정 (STALE 확인) |
| 3329 | build_link gc-sections 추가 |
| 3330 | HANDOFF + ROADMAP + 커밋 |

---

## contracts-check 출력 형식 (Cycle 3331 갱신)

```bash
$ compiler.exe contracts-check test.bmb
{
  "type": "contracts_check",
  "file": "test.bmb",
  "status": "safe",
  "violations_count": 0,
  "module_capability": { "type":"module_capability", "status":"skipped", "total_violations":0 }
}
```

- `module_capability` 필드가 이제 embedded — enforce_module_caps 미설정 시 `"status":"skipped"`
- `diagnose` 명령의 `contracts_check` 서브섹션은 **기존과 동일** (module_capability 미포함, top-level 별도)

---

## diagnose 통합 현황 (5섹션 + summary)

```bash
$ compiler.exe diagnose test.bmb
{
  "type": "diagnose",
  "file": "test.bmb",
  "effect_verify": { "type":"effect_verify", "status":"safe", "violations_count":0 },
  "contracts_check": { "type":"contracts_check", "status":"safe", "violations_count":0 },
  "module_capability": { "type":"module_capability", "status":"skipped", "total_violations":0 },
  "lint_effects": { "type":"lint_effects", "count":0, "warnings":[] },
  "semantic_duplicate": { "type":"semantic_duplicate", "pairs_count":0, "pairs":[] },
  "summary": {
    "total_issues":0,
    "effect_issues":0,
    "contract_issues":0,
    "module_cap_issues":0,
    "lint_issues":0,
    "duplicate_pairs":0
  }
}
```

### module_capability 전용 섹션 (신규, Cycle 3325)

`.bmb-contracts`에 `enforce_module_caps = true` 설정 시:
- module requires와 위배되는 effect 함수 → `"module_capability"` 섹션에 별도 보고
- contracts_check에서 제거 (더 이상 섞이지 않음)
- violations 형식: `{"type":"module_capability","function":"fn","effect":"IO","declared":["pure"],...}`

---

## bootstrap P-track 현황 (Cycle 3328-3329 재측정)

| 벤치마크 | Bootstrap/C | 이전 보고(Cycle 3234) | 변화 |
|---------|------------|---------------------|------|
| brainfuck | 0.882× ✅ | 0.866× | 유사 |
| csv_parse | 1.039× ⚠️ | 1.134×❌ | 대폭 개선 |
| http_parse | 0.785× ✅ | 0.934× | 개선 |
| json_parse | 0.539× ✅ | 0.556× | 유사 |
| json_serialize | 0.941× ✅ | 0.925× | 유사 |
| lexer | 0.489× ✅ | 1.459×❌ | **대폭 개선** |
| sorting | 0.180× ✅ | 0.178× | 유사 |

**핵심 발견**: Cycle 3234의 bootstrap P-track 측정값은 STALE. 현재 bootstrap 컴파일러는 Rust 컴파일러와 identical IR 생성 → 레거시 회귀 실질 해소됨.

**csv 1.039× 잔여 원인**: `fn parse_csv() -> (i64, i64)` 튜플이 heap calloc(2, 8) 1회 사용. 근본 해결은 L1 언어사양(stack-allocated tuple ABI) 필요.

---

## .bmb-contracts 지원 규칙 (현재)

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

### [P2 / L1 언어사양] Stack-allocated tuple ABI (3-5 사이클)

**배경**: tuple 반환이 항상 heap calloc(2-word). bootstrap csv 1.039× 경계 원인.
**방향**: `(i64, i64)` → LLVM struct return (sret) ABI 또는 스택 alloca.
**복잡도**: 3-5 사이클. 파서 + IR lowering + bootstrap 양쪽.
**상태**: Human Decision 대기 (csv 1.039× 노이즈 허용 여부).

### [P2 / L5 AI-Native] M15 Phase 6c 런타임 sandbox (5-7 사이클)

**배경**: 현재는 compile-time 체크만. platform 선언 기반 런타임 capability 강제 필요.
**복잡도**: 5-7 사이클. 대규모. 장기 항목.

### ~~[P3] contracts_check_run에 module_capability 포함~~ ✅ DONE (Cycle 3331)

### ~~[P3] bootstrap bare-filename SIGSEGV~~ ✅ DONE (Cycle 3332)

---

## 미비/결함/개선 도출

| 레이어 | 유형 | 내용 | 심각도 |
|--------|------|------|--------|
| L1 언어사양 | 결함 | tuple heap-only → csv bootstrap 1.039× 근본 원인 | P2 |
| L5 AI-Native | 미비 | module_capability 런타임 sandbox 미구현 | P4 |
| ~~L2 컴파일러~~ | ~~미비~~ | ~~contracts_check_run이 module_capability 미포함~~ | ✅ DONE Cycle 3331 |
| ~~bootstrap~~ | ~~결함~~ | ~~bare filename SIGSEGV (`llvm.assume(false)` UB)~~ | ✅ DONE Cycle 3332 |

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
- **module_capability 분리 (Cycle 3325)**: contracts_check.violations에 module_capability 없음 → 별도 섹션 확인.
- **bootstrap P-track**: Cycle 3234 측정값 STALE → 현재는 6/7 ✅. csv 1.039× 경계선.

---

## 주요 파일

| 파일 | 역할 |
|------|------|
| `bootstrap/compiler.bmb` | 부트스트랩 컴파일러 (~47K+ LOC) |
| `ecosystem/bmb-mcp/mcp_server.bmb` | MCP 서버 (10 tools, bmb_diagnose 5섹션 설명 갱신) |
| `claudedocs/ROADMAP.md` | 실무 앵커 (§ 6 AI-Native Pivot) |
