# BMB Session Handoff — 2026-05-13 (Cycles 2803-2807 — playground-wasm close + bootstrap CI script)

> **HEAD**: `78c7b8e8`
> **이번 세션 commits**: playground-wasm ISSUE close (C2805) + rebuild-bootstrap-exe.sh (C2806) + session-close (C2807)
> **3-Stage Fixed Point**: ✅ S2 == S3 (Cycle 2792, 이번 세션 bootstrap 미변경)
> **이전 세션 핸드오프**: Cycles 2793-2802 (`71055ef3`)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: Cycle 2808 — Active ISSUE 11개 아래 우선순위 참고
> **이번 세션 cycle logs**: cycle-2803.md ~ cycle-2807.md (claudedocs/cycle-logs/)

---

## 다음 세션 우선순위 (Cycle 2808+)

### Active ISSUE 11개 현황 (Cycle 2807 기준)

| ISSUE | 우선순위 | 상태 | 비고 |
|-------|---------|------|------|
| `ISSUE-20260512-tier3-spawn-overhead-methodology` | P2 | 🔴 HUMAN 결정 | Option A/B/C 선택 필요 |
| `ISSUE-20260511-golden-flakiness-inttoptr` | P3 | 환경 의존 | 부하 높을 때만 발현 |
| `ISSUE-20260511-clang-knapsack-outlier` | low | 외부 | Clang upstream 이슈 |
| B-track `ISSUE-20260326-*` (8개) | HIGH/MED | 🔴 API 필요 | benchmark 재측정 키 필요 |

### 자율 착수 가능 작업

1. **bootstrap parser 재귀→iterative 전환** — P3 장기 개선 (multi-cycle, 3-5 사이클 예상)
2. **`--check-only` CI 연동** — `scripts/rebuild-bootstrap-exe.sh --check-only` 를 GitHub Actions step에 연결 (P4, 1 사이클)
3. **ROADMAP Active ISSUE 탐색** — 다른 자율 착수 가능 항목 확인

### 다음 사이클 권장

**Cycle 2808**: ROADMAP.md 검토 후 자율 항목 선택
- bootstrap parser iterative conversion 착수 여부 결정
- 또는 ROADMAP의 다른 P2/P3 항목 선택

---

## 0. 이번 세션 작업 (Cycles 2803-2807)

### ✅ playground-wasm ISSUE close (Cycles 2803-2805)

`ISSUE-20260413-playground-wasm` → `claudedocs/issues/closed/` 이동. Active ISSUE: 12 → 11. Closed: 55 → 56.

완료 기준 3/3 달성:
1. Playground가 실제 BMB 컴파일러 실행 (JS 인터프리터 아님) ✅
2. 10 예제 검증 (5 live via Playwright + 5 code inspection) ✅
3. URL share + WASM 실행 결합 ✅

배포 노트: `npm run build` dist에 `bmb_wasm_bg.wasm` 자동 복사 안 됨 → 프로덕션 배포 시 `bmb-wasm/pkg/` → `playground/public/` 수동 복사 필요.

### ✅ bootstrap compiler.exe CI 재빌드 스크립트 (Cycle 2806)

`scripts/rebuild-bootstrap-exe.sh` 신규 (75 LOC):
- Staleness check: `compiler.bmb` mtime vs `compiler.exe` mtime
- Rebuild: `bmb build bootstrap/compiler.bmb --fast-compile` (~13s)
- Stack verification: Python PE32+ 파싱으로 `SizeOfStackReserve ≥ 32 MB` 확인 (Windows only)
- 3가지 모드: `--check-only` (CI gate), `--force`, `--json`

`scripts/bootstrap.sh` 통합: Stage 1 시작 전 exe freshness 자동 확인

Bug fixed: `log()` 함수 `set -e` trap (`|| true` 추가) + build JSON 오염 (`>&2 2>&1` 수정)

Cycle 2807에서 Linux OSTYPE guard 추가: `[ "$STACK_MB" -gt 0 ] && [ "$STACK_MB" -lt 32 ]` — 0 반환 시 (PE 헤더 없음) 경고 발화 방지

### ✅ 조기 종료 (Cycle 2807)

HANDOFF 자율 목록 4개 중 3개 완료. P3 multi-cycle (bootstrap parser iterative) 는 시작 후 부분 커밋이 더 나쁨 → 다음 세션으로 이월. 조기 종료 3 조건 모두 충족.

---

## [PREV] 이번 세션 추가 작업 (Cycles 2800-2802)

### ✅ Rule 20 bare_panic false positive fix (Cycle 2800)

`bootstrap/lint/lint.bmb`: `check_bare_panic`에서 `line_contains` → `find_pattern_outside_str` + ident-char guard
- Root cause: `fn check_bare_panic(` 같은 식별자가 `panic(` 패턴을 포함해 오탐
- Fix: panic_pos 앞 byte가 ident char면 skip
- `line_contains_outside_str` NOTE comment 추가 (`"` 자체 매칭 불가 설계 문서화)
- Result: 104 → 102 warnings (2 FP 제거)

### ✅ SIMD P1 ISSUE close (Cycle 2801)

`ISSUE-20260413-simd-codegen`: Cycles 2215-2283 구현, Cycle 2801 검증
- `fadd fast <4 x double>` IR emit ✅, SIMD ≥3x speedup ✅, Fixed Point ✅
- `claudedocs/issues/closed/`로 이동. Active ISSUE: 14 → 13.

### ✅ bootstrap stack overflow P3 fix (Cycle 2802)

`ISSUE-20260512-bootstrap-parser-stack-overflow`:
- Root cause: `bootstrap/compiler.exe`의 SizeOfStackReserve = 2MB (Cycle 2780 D2 패치 이전 빌드)
- Fix: `bmb build bootstrap/compiler.bmb --release` 재빌드 → 64MB 스택 확인
- `hash_table bench` 빌드 및 실행 성공 ✅
- `cargo test --release` 2377/2377 PASS ✅
- `claudedocs/issues/closed/`로 이동. Active ISSUE: 13 → 12.

---

## [PREV] 이번 세션 작업 (Cycles 2793-2799)

### ✅ lint Rules 15-17 + UTF-8 boundary fix (Cycle 2798)

`bootstrap/lint/lint.bmb` 14→17 rules:
- **Rule 15** (negated_comparison): `not(a == b)` → `a != b`
- **Rule 16** (long_line): 100자 초과 경고
- **Rule 17** (fn_too_many_params): 6+ params 경고
- **UTF-8 fix**: `line_contains_outside_str`에서 멀티바이트 UTF-8 문자(em dash `—`) 경계에서 슬라이스 패닉 수정
  → start/end char boundary guard 추가 (byte < 128 or >= 192)

Result: cargo test 6211/6211 PASS, lint self-test 86 warnings 패닉 없음

### ✅ lint Rules 18-20 + line_contains 버그 수정 (Cycle 2799)

`bootstrap/lint/lint.bmb` 17→20 rules:
- **Rule 18** (string_chain_concat): 4+ `+` concatenation chain 경고
- **Rule 19** (dual_negation): `not(a) and not(b)` → De Morgan 제안
- **Rule 20** (bare_panic): `panic()` call → `pre` contract 제안
- **Bug fix**: `check_string_chain_concat`에서 `line_contains(_, _, _, "\"")` 항상 false 반환 버그 수정
  Root cause: `line_contains` → `line_contains_outside_str`는 `"` 구분자 자체를 매칭하지 않음
  Fix: raw byte scan (`while qi < nl and byte_at(qi) != 34`)으로 교체

Result: cargo test 6211/6211 PASS, lint self-test 104 warnings, 3 new rules 모두 발화 확인

### ✅ ISSUE-20260413-linter-enhancement Closed

완료 기준 "20+ 린트 규칙" 달성 → `claudedocs/issues/closed/`로 이동.
Active ISSUE: 16 → 14. Closed (누적): 52 → 53.

---

## [PREV] 이번 세션 작업 (Cycles 2788-2792)

### ✅ or/and 단락 평가(Short-Circuit) 부트스트랩 fix (Cycle 2792)

`bootstrap/compiler.bmb` 3개 위치 수정:
1. `is_pure_expr`: `or`/`and`를 impure로 표시
2. `lower_binop_sb` (recursive): `or`/`and` → `lower_if_branch_sb` 디스패치
3. `step_binop_start` (iterative): `or`/`and` → `IT` work item 디스패치

검증: `false and expensive(42)` → expensive 미호출 ✅, `false or expensive(42)` → 호출 ✅

Result: S2==S3 Fixed Point ✅, cargo test 6211/6211 PASS

---

## [PREV] 이번 세션 작업 (Cycles 2788-2791, bench output fairness)

### ✅ lexer benchmark 6-bug fix (Cycle 2788)

`ecosystem/benchmark-bmb/benches/real_world/lexer/bmb/main.bmb` 전면 수정:
1. `is_keyword_at`: "return" vs "result" 구분 위해 3번째 문자 체크 (`peek(src,start+2)==116`) 추가
2. 단일 i64 packing → (i64, i64) 튜플 반환 (op=17→19 overflow 해결)
3. `count_tokens_loop`: `new_str`/`new_comment` 추적 추가
4. `main()`: Strings, Comments 출력 추가 + 빈 줄 추가
5. `count_tokens` 반환 타입 (i64, i64) 변경

Result: verify PASS (Small: Identifiers:20/Numbers:9/Keywords:12/..., Large total:8900 — C와 정확히 일치)

### ✅ csv_parse skip_ws zero-position 버그 수정 (Cycle 2788)

`ecosystem/benchmark-bmb/benches/real_world/csv_parse/bmb/main.bmb` skip_ws 수정:
- **Root cause**: pos=0에서 비공백 문자를 만나면 exit trick `len+0=len`, decode `len>len=false`로 `len` 반환
- 영향: 첫 행의 모든 필드가 "1 field, 0 chars"로 잘못 집계 (소규모: 41 fields vs 44, 대규모: 9991 vs 10000)
- 수정: `else { len + p }` → `else { len + p + 1 }`, `if p > len { p - len }` → `if p > len { p - len - 1 }`

Result: verify PASS (Rows:11/Fields:44/Quoted:4/Total chars:274 — C와 정확히 일치)

### 최종 verify 결과: **17/17 PASS** (epsilon 1e-6, fair comparison)

```
python scripts/verify_bench_outputs.py --tier all --epsilon 1e-6 --rebuild
→ Matched: 17/17, Mismatched: 0, Failed: 0, Time: 43.6s
```

fibonacci fair fix (Cycle 2791):
- BMB: `fibonacci_iter(bmb_black_box(50))` → LLVM @pure constant-fold 방지
- C: `__attribute__((noinline))` → GCC hoisting/constant-propagation 방지
- 시간 12.7s → 43.6s: fibonacci 이제 C+BMB 모두 실제 ~2s 실행 (fair)

### ISSUE 정리

- `ISSUE-20260512-bmb-lexer-bench-zero-tokens.md` → **closed/** 이동 (Cycle 2789 재확인)
- `ISSUE-20260512-sorting-rebuild-regression.md` → **closed/** 이동 (Cycle 2789 verify PASS 확인)
- `ISSUE-20260512-bootstrap-stack-depth-hash_table.md` → **closed/** 이동 (Cycle 2784 해결)
- `ISSUE-20260512-bench-output-fairness-survey.md` → fibonacci P3 scope 노트 추가 (Cycle 2789)

Active ISSUE: **16** / Closed: **51** (Cycle 2792: or-chain-lowering → closed/)

**Cycle 2792 ISSUE 정리**:
- `ISSUE-20260511-or-chain-lowering.md` → **closed/** (short-circuit 시맨틱 수정 완료)

---

## [PREV] 이번 세션 작업 (Cycles 2783-2787, 5 cycles, P0+P1 correctness fixes)

### ✅ D2' sorting P0 fix (Cycle 2783)

`MkTuple` handler in `llvm_text.rs`에서 `insertvalue` chain 이후 alloca에 store가 빠짐 →
SROA가 load를 `undef`로 대체 → `partition` 함수 재귀 호출 인수가 `undef` → 무한 루프 (~500×).
Fix: `if local_names.contains(&dest.name) { store struct_type %dest_name, ptr %dest.name.addr }` 추가.
Result: sorting 203ms + `403905348` ✅ (ref Feb 9 = 234ms).

### ✅ P1 bootstrap int_to_string i64::MIN fix (Cycle 2784)

`bootstrap/compiler.bmb`의 `int_to_string`에서 `0 - i64::MIN = i64::MIN` (wrap-around) →
무한 재귀 → STATUS_STACK_OVERFLOW on hash_table (226 LOC).
Root cause는 parser recursion이 아님 (3 else-if chains ≠ 64MB overflow) — bisection으로 확인.
Fix: `int_to_string_neg` helper 추가 (음수를 negation 없이 직접 처리).
Result: `stage1.exe build hash_table/main.bmb` ✅ (was STATUS_STACK_OVERFLOW).
Issue document 근본 수정 (hypothesis 오류 정정).

### ✅ D5-B epsilon FP tolerance (Cycle 2785)

`full-cycle.sh` verify step에 `--epsilon 1e-6` 추가 → n_body FP precision 차이 (~7e-7 rel)
가 false MISMATCH 유발하던 문제 해결. Tier 1 verify: 9/10 matched (fibonacci C timeout = 기존).

### ✅ int_to_string fix 6 modular files (Cycle 2786)

Rule 5 (전수 검색): mir.bmb / optimize.bmb / lowering.bmb / llvm_ir.bmb / parser_ast.bmb /
types.bmb 모두 동일 `0 - n` 패턴 수정. build_unified_compiler.sh + CI 커버.

### ✅ Tier 3 verify sorting confirmed (Cycle 2787)

`verify_bench_outputs.py --tier 3 --rebuild --epsilon 1e-6`: sorting ✅ (5/7 matched).
csv_parse + lexer MISMATCH = pre-existing tracked issues.

### ✅ 3-Stage Fixed Point verified (post-Cycle 2787)

`stage2_test.exe build compiler.bmb → stage3.ll` vs `stage3_test.exe build compiler.bmb → s3.ll`:
**S2 == S3** — only ModuleID/source_filename header comments differ (embedded binary name, not code).
int_to_string fix is deterministic across self-compilation rounds. Rule 3 compliance confirmed.

---

## [PREV] 이전 세션 작업 (Cycles 2765-2773, 9 cycles, infrastructure + diagnosis 중심)

### 🚨 중대 발견 — **P0 store_u8 silent correctness bug**

Cycle 2772에서 발견: `store_u8(buf + pos, c)` 패턴이 함수 인자 buf + pos 케이스에서 잘못된 base 선택 (pos을 base ptr로 inttoptr)으로 LLVM이 store를 UB로 간주하여 제거. **compile success + run success + 잘못된 출력**.

- 영향: json_serialize bench `Array: {1,2,3,4,5]` (정상: `[1,2,3,4,5]`)
- 잠재 영향: 다른 bench/bootstrap에서 동일 패턴 사용 시 silent 회귀
- ISSUE: `claudedocs/issues/ISSUE-20260512-store_u8-null-ptr-base.md` (P0, estimated 3-5 cycles, Rule 6 충돌 검토 필요)

### 🛠️ 신규 인프라 — bench output verification

Cycle 2769에서 `scripts/verify_bench_outputs.py` 작성 (240 LOC):
- BMB ↔ C bench 출력 정합 자동 검사 (Tier 1/3 17 benches)
- 1차 측정에서 **6개 결함 즉시 발견** (도구의 가치 입증)
- Cycle 2771: `scripts/full-cycle.sh` 에 Step 3.5로 통합 (non-blocking)

### Cycle-by-cycle 요약

| 사이클 | 제목 | 성과 |
|--------|------|------|
| 2765 | Tier 3 workload amplification POC | Option A 한계 발견 (BMB CSE/purity inference로 outer-loop DCE). lexer + brainfuck 부분 적용. ISSUE-tier3-spawn-overhead 갱신 + 새 ISSUE bmb-lexer-0-token 등록 |
| 2766 | HashMap 진단 | `hm_get` 만 noinline (Cycle 2532 MemoryEffectAnalysis pass). Rule 6 충돌 발견 (bootstrap에 동일 pass 없음) |
| 2767 | HashMap 측정 검증 | **갭은 noise였음** (advisor 가설 우월). 실측 1.020x ≈ parity. bootstrap-built 시도 → STATUS_STACK_OVERFLOW (별도 ISSUE) |
| 2768 | ISSUE-hashmap close + 양식 강화 | P1 → P3 강등, closed/ 이동. `_template.md` 에 `estimated_cycles` + hypothesis 필드 추가 (cycle estimate 갭 패턴 회귀 방지) |
| 2769 | verify 도구 작성 + 1차 측정 | 17/17 측정 → 11 PASS, 4 unfairness, 2 fail. **6 결함 즉시 catch** |
| 2770 | sorting 재빌드 회귀 진단 | **P1 ~500× 슬로다운** (Rust compiler 회귀, Feb 9 main.exe 정상 vs May 12 rebuild 시 hang) |
| 2771 | verify CI 통합 | `full-cycle.sh` Step 3.5 추가 (`--skip-verify` opt, non-blocking 동작) |
| 2772 | json_serialize char bug | **P0 store_u8 silent UB 발견**. workaround 불가, multi-cycle fix 필요 |
| 2773 | HANDOFF/ROADMAP 갱신 | 본 cycle (commit 직전) |

### advisor leverage (세 번째 메타-패턴 등 4건)

- **Cycle 2765**: Option A 비현실성 + scope 축소 + HashMap 우선순위 권고
- **Cycle 2766**: HashMap "1.040x → 0.95x" expectation 근거 부재 지적 + measurement-first 권고
- **Cycle 2767**: 분기 ① 측정 후 부정 결과 → bootstrap port ROI 부정 결정
- **Cycle 2772 (메타)**: 도구가 P0 bug 즉시 catch — measurement integrity infrastructure 효과 누적 검증

**Meta-pattern**: ISSUE 본문 cycle estimate은 검증 전까지 가설. 3 cycle 연속 같은 패턴 → `_template.md` 메타 필드 추가 (cycle 2768).

---

## 1. 현재 상태

### Bootstrap 검증 상태 (변경 없음)

| 게이트 | 결과 (Cycle 2718) |
|--------|------------------|
| Stage 1 | ✅ 10.8s |
| Stage 2 (32G arena) | ✅ 29.2s |
| Stage 3 | ✅ 36.7s |
| **Fixed Point S2 == S3** | ✅ **유지** |

### 테스트 현황

| 스위트 | 결과 |
|--------|------|
| `cargo test --release` | ✅ (BMB compiler 변경 없음) |
| **verify_bench_outputs.py** | ✅ **16/17 PASS** (--epsilon 1e-6, Cycle 2788+ 갱신) |

### 마일스톤 상태

| 마일스톤 | 상태 |
|---------|------|
| M1 Self-Validated + Bootstrap | ✅ COMPLETE + 회복 |
| M2 AI-Ready Infra | ✅ COMPLETE |
| M3 External Bindings | 🔄 ~99% (HUMAN publish dispatch 잔여) |
| M4 Adopted | 🔄 ~50% |
| M5 Language Completeness | 🔄 M5-1~M5-5g ✅ |

### ISSUE 백로그 변화 (Cycles 2765-2789)

| 시점 | active | closed |
|------|--------|--------|
| 시작 (Cycle 2764 종료) | 16 | 44 |
| 종료 (Cycle 2772) | 22 | 45 |
| 종료 (Cycle 2788) | 20 | 47 |
| **종료 (Cycle 2789)** | **18** | **49** |

**Close 1건**: `hashmap-perf` (실측 1.020x ≈ parity)

**신규 등록 6건** (이번 세션):
| ISSUE | 우선순위 | scope |
|-------|---------|-------|
| `bmb-lexer-bench-zero-tokens` | P2 | lexer count_tokens 0-token correctness bug (cycle 2765) |
| `bootstrap-parser-stack-overflow` | P3 | hash_table size source가 bootstrap STATUS_STACK_OVERFLOW (cycle 2767) |
| `bench-output-fairness-survey` | P2 | 통합 ISSUE — verify 도구 발견 결함 6건 종합 (cycle 2769) |
| `sorting-rebuild-regression` | **P1** | sorting 재빌드 시 ~500× 슬로다운 (cycle 2770) |
| `store_u8-null-ptr-base` | **P0** | silent UB: pos가 base로 선택, pos=0 시 null base GEP (cycle 2772) |

추가로 `_template.md` 양식 강화 (cycle 2768): `estimated_cycles` + hypothesis 필드 + 양식 보존 가이드 #6.

---

## 2. 태스크 목록 (다음 세션 진입) — **결정사항 D1-D8 권장 옵션 채택 (2026-05-12)**

> Decision matrix는 § 6 참조. **권장 ordering** = 다음 세션 진입 시 순차 실행.

### 진입 ordering (갱신 2026-05-12, Cycles 2783-2787 반영)

| # | 결정 | 태스크 | 성격 | 상태 |
|---|------|--------|------|------|
| 1 | D6 | submodule revert | 자율 | ✅ 이전 세션 완료 |
| 2 | D4 | `.gitignore` exceptions | 자율 | ✅ 이전 세션 완료 |
| 3 | **D7** | M3 publish dispatch (npm + PyPI) | **HUMAN dispatch** | ⏳ |
| 4 | D1 | P0 `store_u8` fix | 자율 | ✅ Cycle 2777 완료 (closed) |
| 5 | D5-B | epsilon FP tolerance | 자율 | ✅ **Cycle 2785 완료** |
| 6 | **D5-A** | GitHub Actions verify workflow | HUMAN approval | ⏳ |
| 7 | D2 | bootstrap parser stack fix | 자율 | ✅ **Cycle 2784 완료** (int_to_string root cause fix) |
| 8 | D2' | sorting P0 UB fix | 자율 | ✅ **Cycle 2783 완료** |
| 9 | **D8** | M4-1 B baseline | **HUMAN setup** | ⏳ |
| 10 | D3 | Rule 6 정책 CLAUDE.md | 자율 | ✅ Cycle 2781 완료 |

### 자율 진입 가능 (HUMAN 차단 없음)

D6 / D4 / D1 / D5-B / D2 / D2' / D3 모두 즉시 진입 가능. D6 → D4 → D1 순서 권장 (clean state + 영속화 → P0 fix).

### HUMAN 결정 후 진입

- **D7 (publish)**: M3-5 완결, 즉시 dispatch 가능. `gh workflow run npm-publish.yml -f dry_run=true` → 검증 → `dry_run=false`.
- **D8 (M4-1)**: `BMB_BENCH_API_KEY` 환경변수 setup 후 `bmb-ai-bench --all --runs 3 --model claude-sonnet-4-6` (이전 세션 모델 결정 완료).
- **D5-A**: GitHub workflow 추가는 maintainer approval 필요 (CI 변경).

### 장기 carry-forward (이번 진입 plan 외)

| 항목 | scope | 우선순위 |
|------|-------|---------|
| Tier 3 workload amp 잔여 5 benches (Option A+B 결합) | 5-10 cycles | 별도 phase |
| sub-ISSUE 처리 (csv_parse / lexer / fibonacci / n_body) | 각 2-5 cycles | sub-ISSUE 우선순위 별 |
| bench output golden test 통합 (D5-D) | 2-3 cycles | P0 fix 후 |
| clang knapsack outlier 분석 | multi | 장기 |
| bootstrap → Rust deprecation 가속화 | multi-cycle phase | D3 정책 채택 후 |

---

## 3. 핵심 산출물 (이번 phase, Cycles 2765-2773)

### Code 산출

- `scripts/verify_bench_outputs.py` (신규, 240 LOC) — BMB ↔ C bench 출력 정합 검사
- `scripts/full-cycle.sh` — Step 3.5 verify 통합 (`--skip-verify` opt, non-blocking)
- `ecosystem/benchmark-bmb/benches/real_world/lexer/c/main.c` — 100x→1000x scaling + buffer 500K
- `ecosystem/benchmark-bmb/benches/real_world/lexer/bmb/main.bmb` — 100x→1000x scaling
- `ecosystem/benchmark-bmb/benches/real_world/brainfuck/c/main.c` — 9→99 outer loop
- `ecosystem/benchmark-bmb/benches/real_world/brainfuck/bmb/main.bmb` — 9→99 outer loop

### Documentation 산출

- 6 신규 ISSUE 등록 (P0 1, P1 1, P2 3, P3 1)
- `claudedocs/issues/_template.md` 양식 강화 (estimated_cycles + hypothesis)
- `claudedocs/issues/closed/ISSUE-20260413-hashmap-perf.md` 이동 (P1 → P3 close)
- `claudedocs/ROADMAP.md` 갱신 (TBD: ROADMAP 미갱신, cycle 2773에서 진행)

### Measurement 산출

- Tier 1 verify: 8/10 PASS (`claudedocs/measurements/verify_tier1_2026-05-12_c2769.json`)
- Tier 3 verify: 3/7 PASS (`claudedocs/measurements/verify_tier3_2026-05-12_c2769.json`)
- hash_table A/B 측정: BMB orig 82.2ms / @inline 82.1ms / C 80.6ms (median, 10-run)

> 향후 재측정 시 baseline diff 가능 — `claudedocs/measurements/` 디렉토리 정착 권고.

---

## 4. 환경 노트 (변경 없음)

| 환경 | 상태 |
|------|------|
| LLVM | 21.1.8 MSYS2 UCRT64 |
| Node.js | v24.14.0 |
| Python | 3.12.10 |
| 버전 | `0.98.0` |
| Branch | `main` (origin 4 commits ahead) |

### 운용 주의사항 (NEW 이번 세션)

- **`scripts/verify_bench_outputs.py` 도구**: BMB compiler 변경 후 측정 전 `python3 scripts/verify_bench_outputs.py --tier all --rebuild` 실행 권장 (silent 회귀 catch)
- **`store_u8(arg + pos, c)` 패턴 회피**: P0 bug 잔존 시까지. 가능한 경우 `store_u8(arg + non_zero_offset, c)` 또는 local var 통해 사용
- **bootstrap parser stack 한계**: deeply nested AST 회피 (hash_table 패턴) — bootstrap port 시 STATUS_STACK_OVERFLOW
- 이전 세션 운용 주의사항 그대로:
  - bmb-algo bench median-of-N (`bench_algo.py --runs=5`)
  - Tier 3 spawn overhead methodology (ISSUE-20260512)
  - bmb-algo submodule 아님
  - BMB_ARENA_MAX_SIZE default 32G
  - Token packing 5M scale
  - FP builtin arity guard 미적용

---

## 5. 다음 세션 시작 체크리스트 — **D1-D8 권장 옵션 채택 ordering**

### 기본 검증
- [ ] `claudedocs/ROADMAP.md` 읽기 (실무 앵커)
- [ ] `cargo test --release` (옵션, BMB compiler 변경 없음 시)

### Cycle 2775 — D6 submodule revert (1 cycle)
- [ ] `cd ecosystem/benchmark-bmb && git checkout -- benches/real_world/lexer/ benches/real_world/brainfuck/`
- [ ] verify: `git status` clean (mandelbrot inproc untracked는 prior session work 그대로)
- [ ] parent repo: submodule pointer 변화 없음 확인

### Cycle 2776 — D4 `.gitignore` 정책 (1 cycle)
- [ ] `.gitignore` 에 `!claudedocs/issues/` + `!claudedocs/measurements/` 추가
- [ ] `git ls-files claudedocs/issues/` 확인 (force-add 없이 tracked 상태)

### Cycle 2777+ — D1 P0 `store_u8` fix (3-5 cycles, bootstrap path)
- [ ] `bootstrap/compiler.bmb` `store_u8` lowering 위치 grep
- [ ] first-operand = base 휴리스틱 적용 (source order 보존)
- [ ] minimal repro test (`store_u8(arg_a + 0, c)` pattern) PASS
- [ ] Stage 1 build + 골든 테스트 추가
- [ ] Stage 2/3 Fixed Point 검증
- [ ] full Tier 1/3 verify PASS

### Cycle 27XX — D5-B + D5-A verify 확장 (2 cycles)
- [ ] `verify_bench_outputs.py --epsilon 1e-6` arg 추가 (n_body 정상화)
- [ ] `.github/workflows/` 에 verify step PR (HUMAN approval)

### Cycle 27XX — D2 bootstrap parser stack fix (2-3 cycles)
- [ ] Windows linker `-Wl,--stack=64M` 적용 + bootstrap rebuild
- [ ] hash_table bench bootstrap build 성공 확인
- [ ] Stage 2/3 Fixed Point 유지 검증
- [ ] (분리) sorting Rust bisect — 진단 only, fix 안 함

### HUMAN dispatch — D7 publish (즉시 진입 가능)
- [ ] `gh workflow run npm-publish.yml -f dry_run=true`
- [ ] dry_run artifact 검증
- [ ] `dry_run=false` 재실행
- [ ] PyPI 동일 절차
- [ ] 24h 모니터링 (npm metadata + README rendering)

### HUMAN setup — D8 M4-1 B baseline
- [ ] `.env.local` 에 `BMB_BENCH_API_KEY=...` 설정
- [ ] `bmb-ai-bench doctor` PASS
- [ ] `bmb-ai-bench --all --runs 3 --model claude-sonnet-4-6` (8-12h)
- [ ] 결과 commit + B-track 갱신

### Cycle 27XX — D3 Rule 6 정책 명시 (1 cycle)
- [ ] CLAUDE.md `Rule 6 (Rust frozen)` 섹션에 보강:
  > "Rust 잔존 결함은 부트스트랩 port + Rust deprecation으로 해소. fix는 bootstrap에서."
- [ ] bootstrap port roadmap 명시 (M5-X 트랙 또는 별도)

---

## 6. HUMAN 결정 사항 누적

### 기존 결정 (불변)

| 항목 | 결정 |
|------|------|
| M3 showcase | ✅ bmb-algo |
| npm publish | ✅ 즉시 dispatch 가능 |
| PyPI publish | ✅ 즉시 dispatch 가능 |
| v0.100 버전 선언 | ✅ M3 publish 완료 직후 |
| B 공식 측정 모델 | ✅ claude-sonnet-4-6 |
| M3-5 README/CHANGELOG | ✅ Cycles 2760-2764 완결 |

### 이번 세션 신규 결정 (D1-D8, 권장 옵션 채택 — 2026-05-12)

| # | 결정 | 채택 옵션 | 정합 |
|---|------|----------|------|
| **D1** | P0 `store_u8` silent UB fix | ✅ **bootstrap fix + 결정론 휴리스틱 (first-operand = base)** | Decision Framework Level 2, AI 친화 예측 가능성 |
| **D2** | P1 sorting rebuild 회귀 | ✅ **Accept (Rust frozen) + bootstrap parser stack fix 가속** | Rule 6 strict + 도그푸딩 게이트 |
| **D3** | Rule 6 (Rust frozen) 정책 | ✅ **유지 + Rust deprecation 가속화 명시** | 도그푸딩 § 1.3, single source of truth |
| **D4** | `.gitignore` ISSUE 영속화 | ✅ **`!claudedocs/issues/` + `!measurements/` 예외** | 휴먼 마찰 제거, 의도 명시 |
| **D5** | bench verify 확장 | ✅ **A (CI workflow) + B (FP epsilon) 즉시 + D (golden) 장기** | Verification Principle |
| **D6** | submodule working tree | ✅ **REVERT cycle 2765 변경** | Principle 2 (Workaround 금지), effect 없는 변경 회피 |
| **D7** | M3 publish dispatch | ✅ **즉시 dispatch** (npm + PyPI) | M3-5 완결, 차단 사유 없음 |
| **D8** | M4-1 B baseline | ✅ **API key setup + 자율 실행** | M4 첫 액션 (vision § 3 처방 B) |

### 자율 결정 (이전 세션 + 이번 세션)

| 항목 | 결정 |
|------|------|
| `_template.md` 양식 강화 (estimated_cycles + hypothesis) | ✅ Cycle 2768 |
| `scripts/verify_bench_outputs.py` 신규 도구 | ✅ Cycle 2769 |
| `scripts/full-cycle.sh` Step 3.5 통합 | ✅ Cycle 2771 |
| `hashmap-perf` ISSUE close (P3) | ✅ Cycle 2768 |
| `claudedocs/measurements/` 디렉토리 신설 + 영속화 | ✅ Cycle 2774 cleanup |
| ~~lexer/brainfuck workload amp partial~~ | ❌ **D6 revert 예정** (cycle 2775) |

---

## 7. 이번 phase의 메타 통찰

### 1. measurement integrity infrastructure의 누적 효과

| Cycle | 단계 | 결과 |
|-------|------|------|
| 2768 | ISSUE 양식 강화 (estimated_cycles + hypothesis) | 메타 회귀 방지 |
| 2769 | verify 도구 (240 LOC) | 즉시 6 결함 catch |
| 2771 | CI 통합 (Step 3.5) | 회귀 자동 alert |
| 2772 | 도구 효과 **P0 silent bug 식별** | **존재 자체가 정당화** |

3 cycles 작업이 **P0 발견**으로 이어짐. ROI 매우 높음.

### 2. advisor의 가설 거부 leverage

cycle 2766 advisor: "HashMap 4% 갭 자체가 measurement noise일 가능성"
cycle 2767 측정: 1.040x → 1.020x (real). 가설 정확. 5-7 cycles compiler work 비용 회피.

이는 advisor가 **expectation 추정 거부**의 시스템적 패턴. 향후 ISSUE estimate 평가 시 적용.

### 3. P-track 측정 신뢰도의 메타-우려

verify 도구 발견:
- **4/17 = 24% benches가 unfair comparison** (csv_parse, json_serialize, lexer, n_body)
- **2/17 = 12% benches가 build/run 회귀** (sorting hang, fibonacci C fail)

기존 P-track 측정 통합 (`tier_all_c2729.json` 등)이 이런 unfairness 위에 구축됨. **재측정 + verify 통과 확인 후 ratio 활용** 표준 권고.

### 4. Rule 6 / Rule 7 한계 — **D3 결정: Rule 6 유지 + Rust deprecation 가속**

이번 세션 3 가지 결함이 Rule 6 충돌:
- `should_no_inline_for_licm` (Rust 잔존) — 결국 ROI 부정 (cycle 2767)
- `sorting` 재빌드 회귀 (Rust codegen) — fix 차단
- `store_u8` silent bug (Rust codegen) — P0 fix 차단

**D3 결정 (2026-05-12)**: Rule 6 (Rust frozen) 그대로 유지 + 새 정책 추가 (CLAUDE.md cycle 27XX):
> "Rust 잔존 결함은 부트스트랩 port + Rust deprecation으로 해소. fix는 bootstrap에서, 검증은 양쪽 측정 비교."

근거: 두 컴파일러 divergence 누적 = workaround (Principle 2). proper fix = single source of truth (bootstrap). BMB는 자기 자신을 컴파일하는 언어 (도그푸딩 § 1.3) — bootstrap이 canonical이 되는게 자연 결말.

### 5. 진단 cycle 우선 정책의 가치

원 plan: cycle 2767 HashMap fix attempt (3-5 cycles).
실제: cycle 2766 진단 + cycle 2767 측정 검증 → 가설 거부 → 2 cycle만 소비. 5-7 cycles 회피.

advisor 권고 "1 cycle 진단 cycle 먼저" 패턴 검증. `_template.md` 양식 강화 (cycle 2768) 정합.

---

## 8. 다음 세션 첫 cycle 권고 — **D1-D8 권장 옵션 채택**

### Cycle 2775 — D6 submodule revert (자율, 즉시)

```bash
cd ecosystem/benchmark-bmb
git checkout -- benches/real_world/lexer/ benches/real_world/brainfuck/
# mandelbrot inproc (prior session) 그대로 untracked 유지
git status  # confirm clean except mandelbrot
```

### Cycle 2776 — D4 `.gitignore` 정책 (자율, 즉시)

`.gitignore` 갱신:
```
claudedocs/
!claudedocs/issues/
!claudedocs/measurements/
!claudedocs/HANDOFF.md
!claudedocs/ROADMAP.md
# cycle-logs는 계속 ignore (intentional ephemeral)
```

검증: `git ls-files claudedocs/issues/` 모든 새 ISSUE tracked 확인.

### Cycle 2777-2781 — D1 P0 store_u8 fix in bootstrap (3-5 cycles)

`bootstrap/compiler.bmb` lowering에 first-operand-is-base 휴리스틱 적용. minimal repro + 골든 테스트 + 3-Stage Fixed Point.

### 병행 HUMAN 진입 가능

- **D7 publish** (즉시): `gh workflow run npm-publish.yml -f dry_run=true`
- **D8 M4-1**: `BMB_BENCH_API_KEY` setup

---

**세션 종료**: 2026-05-12 (Cycles 2765-2774 — bench verify infrastructure + P0 store_u8 bug 진단 + D1-D8 권장 옵션 채택). HEAD `c14f2265` (final session-close 후 `?` 갱신).
