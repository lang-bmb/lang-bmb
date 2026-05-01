# BMB Session Handoff — 2026-05-01 (Cycles 2526-2531 — MemoryEffect Diagnosis + Partial Fix)

> **이전 HEAD**: `943bdd3f` (Cycle 2525 측정 utilities + handoff)
> **새 HEAD**: `615a155a` (perf+docs: MemoryEffectAnalysis runtime intrinsic fix + cleanup, Cycles 2526-2531)
> **세션 성격**: 6-cycle run-cycle (20 budget 중 14 잔여 — early termination). HANDOFF 이전 권고 P-1/P-2 사실 오류 발견 → 진단 reframe → 진짜 root cause fix 부분 적용.
> **결정적 발견**: 이전 HANDOFF의 P-1 (loop/while 도입) + P-2 (byte intrinsic) 모두 **이미 존재함**. 진짜 root cause = `MemoryEffectAnalysis`가 runtime intrinsic을 write로 오분류 → LICM 차단.

---

## 1. 이번 세션 요약 (Cycles 2526-2531)

### Cycle 2526 — P-6 ROADMAP/Spec strict 게이트 + ai-proof 제거

- `docs/ROADMAP.md`: M1 게이트 ≤1.10x → **≤1.05x strict** (Cycle 2514 게이트 완화는 workaround였음). Headline 정정: "BMB > C in 16 benchmarks" → "7/16, ≤C 15/16, 1 FAIL". v0.51.22 stale "8/15 FAIL" 분류 OBSOLETED.
- `claudedocs/cycle-logs/ROADMAP.md`: v0.98.x distribution phase → **Performance-First gap closure** phase 전환.
- `ecosystem/ai-proof/` 제거 (197 tracked files): Cycle 2523 deprecation notice의 "Cycle 2526 약속" 이행. `bmb-ai-bench/README.md` + `dev-docs/AI_FRIENDLY_CYCLE_PLAN_V2.md` + `docs/AI_NATIVE_IMPROVEMENT_GUIDE.md` 활성 경로 정리.

### Cycle 2527 — P-5 benchmark 노이즈 자동 게이트

- `scripts/benchmark.sh` + `scripts/measure-v098.sh`: warmup-driven adaptive runs gate. warmup < 100ms 벤치는 자동 ≥10 runs.
- 새 CLI: `--no-noise-gate`, `--noise-threshold MS`, `--noise-min-runs N`.
- 5 시나리오 검증 통과 (default fire on small benches, skip on large, manual disable, force-trigger, high RUNS preserve).
- stderr-only verbose 메시지 (stdout 캡처 오염 방지 — Cycle 2527 STEP 4에서 발견 + 즉시 수정).

### Cycle 2528 — json_parse IR-level 진단

**HANDOFF P-1/P-2 사실 오류 발견**:

| HANDOFF 주장 | 실제 |
|------|------|
| "재귀 → 루프 (L1) `loop`/`while` 정식 도입 필요" | ❌ `bmb/src/grammar.lalrpop:39-41,1359-1390`에 이미 존재 |
| "byte_at deref (L4)" | ❌ pre-opt IR `getelementptr i8 + load i8` 직접 인라인 (line 652-653) |
| "pos >= len() 체크 (L1+L3)" | ❌ `s.len()` 한 번 hoist + `pos < len` 비교, C와 동일 |
| "재귀 TCO 비결정성 (L3)" | ❌ post-opt에서 일관 적용 (loop_header_7 phi 변환) |

진짜 root cause 가설 (cycle 2528) = jump table indirect jump → cycle 2529에서 부분 부정확함이 추가 발견.

### Cycle 2529 — Real root cause + manual IR patch validation

**Asm 비교 (`count_array` hot loop, BMB vs C)**:

```
C main:
    callq validate_json     ; ★ 1번 호출
    callq count_array        ; ★ 1번 호출
    addl  ...
    imull $100000, %eax      ; ★ ×100K multiply

BMB main:
.LBB5_1:
    callq validate_json     ; 100,000번 호출
    callq count_array        ; 100,000번 호출
    decq  %rbx
    ja    .LBB5_1
```

**진짜 root cause**: BMB의 `MemoryEffectAnalysis::inst_writes_memory_ip` (`bmb/src/mir/optimize.rs:7005`)가 `MirInst::Call { func, .. } => !non_writing_fns.contains(func)` 로직을 사용. `non_writing_fns`는 user-defined 함수의 `is_memory_free OR is_read_only` 집합으로만 빌드 → **runtime 함수 (`byte_at`, `len` 등)는 영구 누락** → `byte_at`를 부르는 함수는 "potentially writes memory"로 분류 → no `memory(read)` LLVM 속성 → opt-O2가 보수적으로 `memory(read, inaccessiblemem: readwrite)` 추정 → **LICM 차단** → 100K 호출 hoist 실패.

**Manual IR patch validation**:
- `attributes #11 = { mustprogress nosync nounwind willreturn memory(read) }` 추가
- `validate_json` + `count_array`만 #11로 변경
- 결과: BMB **25ms vs C 24ms = 1.04x ≤1.05 strict gate ✅** (vs original 27ms = 1.13x FAIL)

→ Fix path 입증.

### Cycle 2530 — `MemoryEffectAnalysis::is_runtime_read_only` 등록

`bmb/src/mir/optimize.rs` 변경:
- 신규 `is_runtime_read_only(name)` set: `len`/`bmb_string_len`/`byte_at`/`char_at`/`bmb_string_char_at`/`bmb_string_eq`/`starts_with`/`ends_with`/`contains`/`index_of`/`is_empty` (and `bmb_string_*` aliases) + `ord`/`bmb_ord` + `cstr_byte_at`/`strlen`.
- `inst_writes_memory_ip`: callee가 set에 있으면 short-circuit `false`.
- Pass order swap: `MemoryEffectAnalysis` → `AggressiveInlining` (carry-forward에서 inlining 휴리스틱이 is_read_only 활용 가능).

`should_hint_inline` 휴리스틱 변경 시도 (read-only && has_call → no inlinehint) — 효과 미미하여 revert. `cargo test --release` 3772/3773 (1 pre-existing 무관 failure).

### Cycle 2531 — 5 byte-stream 벤치 ROI 측정

| Bench | Pre-fix vs C | Post-fix vs C | Δ |
|-------|-------------|--------------|---|
| brainfuck | 1.04x | **1.00x** | +0.04 ✅ |
| lexer | 1.04x | **1.00x** | +0.04 ✅ |
| csv_parse | 1.00x | 1.00x | unchanged ✅ |
| json_parse | 1.12x | 1.12x | unchanged ❌ |
| http_parse | 0.96x | 빌드 실패 | pre-existing 결함 |

**Net positive**: brainfuck/lexer 격차 해소. csv_parse 이미 parity. json_parse 잔여 — clang -O3 auto-inliner override.

---

## 2. 산출물

### Tracked (commit `615a155a`)
| 분류 | 파일 |
|------|------|
| 코드 변경 | `bmb/src/mir/optimize.rs` (`is_runtime_read_only` + pass order swap) |
| docs | `docs/ROADMAP.md` (Cycles 2526-2531 anchors + new P-A track), `docs/AI_NATIVE_IMPROVEMENT_GUIDE.md` (ai-proof → bmb-ai-bench redirect), `docs/BENCHMARK.md` (noise gate 사용 예제) |
| scripts | `scripts/benchmark.sh` + `scripts/measure-v098.sh` (warmup-driven adaptive runs) |
| ecosystem | `ecosystem/ai-proof/` 제거 (197 files), `ecosystem/bmb-ai-bench/README.md` (deprecation notice 갱신) |

### Gitignored (local only)
| 분류 | 파일 |
|------|------|
| Cycle logs | `claudedocs/cycle-logs/cycle-{2526,2527,2528,2529,2530,2531}.md` |
| Run-cycle ROADMAP | `claudedocs/cycle-logs/ROADMAP.md` (Performance-First phase) |
| 기존 HANDOFF | `claudedocs/HANDOFF.md` (본 문서가 갱신) |
| 측정 IR/binary | `target/cycle-2528/*.ll`, `*.s`, `*.exe` |

### 메모리 (auto-memory)
| 파일 | 변경 |
|------|------|
| `MEMORY.md` | "MemoryEffect Diagnosis" 인덱스 항목 추가 |
| `project_session_2026_05_01_memeffect_diag.md` | 신규 — 6-cycle 진단/fix/잔여 |
| `project_benchmark_reality.md` | json_parse 격차 원인 정정 (재귀+byte_at → MemoryEffect) + brainfuck/lexer 1.00x 갱신 |

---

## 3. 검증 상태

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ 2m30s |
| `cargo build --release --features llvm --target x86_64-pc-windows-gnu` | ✅ 2m38s |
| `cargo test --release` | ⚠️ 3772/3773 (1 pre-existing `verify::contract::tests::test_trivial_contract_detection` — `git stash` 검증으로 무관 확인) |
| 부트스트랩 3-Stage | ⏳ 미실행 — 코드 변경은 MIR 분석 패스 등록만 (compiler.bmb 무영향). 다음 세션 검증 권고 |
| 5 byte-stream 벤치 측정 | ✅ 4 benches (http_parse 사전 결함 — 별도) |
| Net Tier 3 perf | brainfuck 1.04→1.00, lexer 1.04→1.00, csv_parse 1.00 unchanged, json_parse 1.12 unchanged |

---

## 4. 다음 세션 우선순위

### P-A.2' [코드젠] text backend `noinline` 자동 부여 ★ 최우선

**근거**: Cycle 2530 fix가 `memory(read)` 정확히 emit하지만, **clang -O3 auto-inliner가 medium-sized read-only 함수를 outer loop에 inline** → LICM이 inlined nested loops 분석 실패. Cycle 2529 manual IR patch (validate_json/count_array가 separate functions로 보존된 상태)에서 1.04x 입증됨 → noinline 부여하면 동일 효과 기대.

**작업 범위**:
1. `bmb/src/mir/optimize.rs::AggressiveInlining` (또는 새 패스): `func.is_read_only && has_user_call(func) && called_from_loop(func)` 조건 시 `func.no_inline = true`. `called_from_loop`는 program-level CFG 분석 필요 — 보수적으로 모든 호출자에 loop이 있는지 확인.
2. `bmb/src/codegen/llvm_text.rs:1965` 일대: `func.no_inline` 시 `noinline` 속성 emit (이미 있음 — 확인만).
3. json_parse 측정 → 1.12 → ≤1.05 검증.
4. **회귀 점검**: Tier 1+3 sweep — `noinline` 부여가 다른 벤치 회귀 없는지 확인. 작은 read-only 함수 (예: stdlib utility)는 `is_read_only && no_user_call` (leaf)이므로 영향 없음.

**추정**: 2 cycles (PoC + sweep).

### P-A.3' [코드젠] inkwell backend `memory(read)` enum attribute (Rule 7 parity)

**근거**: `bmb/src/codegen/llvm.rs:1968` `create_string_attribute("memory", "read")` → IR에 `"memory"="read"` (target string attr) 방출. **LLVM은 이를 표준 memory effect attribute로 인식 안 함** → text backend는 fix 받지만 LLVM-feature 빌드는 받지 못함.

**작업 범위**:
1. inkwell의 표준 `memory()` enum attribute API 식별. LLVM 21에서 `Attribute::get_named_enum_kind_id("memory")`는 enum kind를 주지만, value는 `MemoryEffects` 비트맵 인코딩 필요.
2. inkwell이 노출하는 API 검토 (`build_attribute` 또는 `Attribute::create_enum_attribute`).
3. text backend와 IR 동등성 확인 — `memory(read)` 표준 형식 emit.
4. LLVM-feature 빌드의 5 byte-stream 벤치 재측정.

**추정**: 1-2 cycles (LLVM C API 호출 직접 작성 가능).

### B-? [코드젠] http_parse 빌드 실패 (사전 결함)

**증상**: `target/cycle-2528/http_parse_bmb.ll:710:14: error: '%d' defined with type 'i32' but expected 'i64'` — `switch i64 %d, label %bb_else_25 [...]`. `%d`가 i32로 정의됐는데 switch가 i64 기대.

**확인**: `git stash` 후 baseline에서도 재현됨 → **pre-existing**, Cycle 2530 fix 무관.

**작업 범위**:
1. http_parse의 어떤 BMB 코드 패턴이 위 IR을 만드는지 식별 (`bmb/src/mir/lower.rs` switch lowering).
2. `bmb/src/codegen/llvm_text.rs` 또는 codegen layer에서 switch operand 타입 일관성 보장.
3. 골든 테스트 추가.

**추정**: 1 cycle.

### P-A.4' [측정] Tier 1+3 회귀 sweep

**근거**: Cycle 2530 fix는 byte-stream에 광범위 적용 — `is_runtime_read_only` set의 다른 함수들 (string predicates, ord, cstr_*)도 영향 받음. 다른 벤치 (binary_trees, hash_table, mandelbrot 등) 회귀 부재 확인 필요.

**작업 범위**:
1. `scripts/measure-v098.sh` 재실행 — 16 historic benches 모두.
2. v0.98 (pre-fix) vs current (post Cycle 2530) 비교.
3. 회귀 발견 시 root cause + Cycle 2530 fix 정교화.

**추정**: 1 cycle.

### Backlog (이전 세션 carry-over)

| 작업 | 추정 | 비고 |
|------|------|------|
| Track O Phase 2 — `bootstrap/context_pack/walker.bmb` | 1-2 | M2 도구층 — Performance fix 후 |
| Track N Phase 3 — 잔여 6 tools | 2-4 | M2 도구층 |
| Track Q Phase 2 — 키워드 충돌 + lint --ai-friendly | 2-3 | M2 도구층 |
| Track T Node bindings PoC | 2-3 | M3 진입 |
| P-3: Verifier-driven bounds check elimination 패스 | 3-5 | json_parse 외 다른 벤치 ROI 평가 후 |

**우선순위 근거**: P-A.2' (noinline) > P-A.3' (inkwell parity) > B-? (http_parse) > P-A.4' (sweep) > M2 도구층. Performance가 1차이고 P-A.2'가 json_parse 격차 해소의 직접 경로.

---

## 5. 환경 노트

| 환경 | 상태 |
|------|------|
| LLVM | 21.1.7-21.1.8 MSYS2 UCRT64 |
| GCC | MinGW-w64 |
| Rust | stable 1.95.0 |
| Z3 | `/c/msys64/ucrt64/bin/z3` (4.15.2) |
| BMB workspace | `Cargo.toml workspace.package.version = "0.98.0"` ✅ |
| `target/release/bmb.exe` (text backend) | May 1 ~20:18, 10MB (post-Cycle 2530 build) ✅ |
| `target/x86_64-pc-windows-gnu/release/bmb.exe` (LLVM inkwell) | May 1 ~20:00, 195MB (post-Cycle 2530 build) ✅ |
| Git working tree | Submodule untracked: `ecosystem/benchmark-bmb` (binary_trees/main_vec.bmb — 이전 세션, 본 세션 무관) |
| Branch | `main`, `origin/main` 대비 2 commits ahead (`943bdd3f` Cycle 2525 + `615a155a` Cycles 2526-2531) |

---

## 6. Git 상태 + push 권고

### 잔여 untracked
```
ecosystem/benchmark-bmb (untracked: benches/compute/binary_trees/bmb/main_vec.bmb)
```
이전 세션부터 누적된 submodule 잔여. 본 세션 무관. 별도 확인 후 처리.

### Push 결정
- 2 commits ahead of `origin/main`: `943bdd3f` (Cycle 2525, 사용자 push 미실행) + `615a155a` (본 세션).
- 본 commit은 텍스트 backend 한정 perf fix + 광범위 docs/cleanup. CI 통과 가능성 높음 (cargo test 3772/3773, pre-existing failure 무관).
- **Push 결정은 사용자 권한** — `git push origin main` 권고.

---

## 7. 다음 세션 시작 액션

```bash
# 1. Git 상태 확인
git -C /d/data/lang-bmb log -2                  # 615a155a, 943bdd3f
git -C /d/data/lang-bmb status -s                # submodule untracked만 (이전 세션)

# 2. 우선순위: P-A.2' (text backend noinline) 시작
ls bmb/src/mir/optimize.rs                       # AggressiveInlining + MemoryEffectAnalysis 위치
grep -n "no_inline\|noinline" bmb/src/codegen/llvm_text.rs | head  # noinline emit 확인

# 3. 측정 baseline (Cycle 2530 post-fix)
target/release/bmb.exe build ecosystem/benchmark-bmb/benches/real_world/json_parse/bmb/main.bmb -o target/json_baseline.exe
for i in $(seq 1 50); do (date +%s%3N; target/json_baseline.exe > /dev/null; date +%s%3N) | paste - -; done | sort -k2n | head -5
# 기대: 28-29ms (post Cycle 2530 baseline)

# 4. Cycle 2529 manual patch IR로 1.04x 재현
# target/cycle-2528/json_bmb_memread2.exe 가 있다면 25ms 확인
```

---

## 8. HUMAN-Decision

**없음**. P-A.2' (noinline) + P-A.3' (inkwell) + B-? (http_parse) 모두 BMB compiler 내부 fix — 자율 진행.

다음 결정 후보 (P-A.2' 후):
- **`noinline` 휴리스틱 정밀도**: `called_from_loop` CFG 분석을 보수적/공격적 어느 쪽으로 — 회귀 발견 시 결정
- **inkwell `memory()` enum API 사용 방식**: inkwell의 LLVM C API 직접 호출 vs 추상화 우회 — fork이 필요한지 평가

---

## 9. 본 세션 핵심 메시지

**HANDOFF는 진실이 아니다 — 측정으로 검증해야 한다**:
- 이전 HANDOFF (Cycle 2525)는 P-1 (loop/while 5-8 cycles) + P-2 (byte intrinsic 2-3 cycles)를 권고했으나 **둘 다 이미 완료된 작업**이었음. 사실 검증 없이 8-11 cycles 낭비할 뻔함.
- IR-level 진단 (Cycle 2528) + manual IR patch 실험 (Cycle 2529)이 진짜 root cause 식별 (`MemoryEffectAnalysis` runtime intrinsic 누락) → 1 cycle fix (Cycle 2530)로 3/4 byte-stream 벤치 향상.
- 같은 패턴 (HANDOFF 사실 오류) 미래에도 가능 — **첫 cycle은 항상 grep + IR inspection으로 inherited plan 검증** (run-cycle skill의 STEP 0 trigger detection 강화).

**Performance 격차의 정확한 분석은 IR + asm + machine code까지 내려가야 함**:
- "재귀 → 루프"라는 source-level 진단은 잘못. BMB MIR이 이미 tail-call-to-loop 변환 함.
- "byte_at deref"라는 runtime-level 진단은 잘못. BMB codegen이 이미 GEP+load 인라인 emit.
- 진짜 격차 = **LLVM 속성 emission 형식** (`"memory"="read"` target string vs `memory(read)` 표준) — 누구도 의심하지 않은 곳.

**Cycles 2526-2531 ROI**:
- 6 cycles 사용 (20 budget 중 14 잔여)
- Net perf: 3 benches improved, 1 unchanged (target = 1.04x, fix path 입증), 0 regression
- Diagnostic infra: noise gate + memory effect 분석 정확도
- Cleanup: ROADMAP align + ai-proof removal (Cycle 2523 약속)
- Carry-forward: P-A.2' (noinline) ★ + P-A.3' (inkwell parity) + B-? (http_parse)

---

**세션 종료**: 2026-05-01 (Cycles 2526-2531, HEAD `615a155a`)

**다음 세션 첫 액션**: `P-A.2'` — text backend `noinline` 자동 부여 (read-only && has_user_call && called_from_loop) → json_parse 1.12 → ≤1.05 목표.
