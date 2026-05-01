# BMB Session Handoff — 2026-05-01 (Cycles 2532-2535 — M1 ≤1.05 Strict Gate 16/16 PASS ★)

> **이전 HEAD**: `615a155a` (Cycle 2531 closure, P-A track carry-forward)
> **새 HEAD**: `c960edbb` (perf: M1 ≤1.05 strict gate 16/16 PASS, Cycles 2532-2535)
> **세션 성격**: 4-cycle run-cycle (10 budget 중 4 사용, **자율 종료**). HANDOFF P-A.2'/P-A.3'/B-?/P-A.4' 모두 자율 처리.
> **결정적 결과**: **M1 ≤1.05 strict gate 16/16 PASS, 0 FAIL** 첫 달성 + Bootstrap 3-Stage Fixed Point 안정 + http_parse build 결함 close.

---

## 1. 이번 세션 요약 (Cycles 2532-2535)

### Cycle 2532 — P-A.2' text backend `noinline` 자동 부여

**구현** (`bmb/src/mir/optimize.rs` `AggressiveInlining` 확장):
- 새 `collect_in_loop_callees(program)` — 프로그램 전역 CFG 분석으로 back-edge 기반 loop 검출 후 그 안의 callees 수집.
- 새 `should_no_inline_for_licm(func, in_loop_callees)` — 조건: `is_read_only && !is_memory_free && !always_inline && !is_recursive && inst_count >= 10 && called_from_loop`.
- `run_on_program` 확장: 기존 alwaysinline/inlinehint 결정 후 별도 패스로 `func.no_inline = true` (inline_hint override).

**임계값 튜닝**: 첫 시도 (≥20 instr)는 `validate_json`만 noinline → count_array 미적용 → 1.136x. 임계값 10으로 낮춤 → 양 함수 noinline → opt -O2가 hoist → C와 동일 패턴 (`imulq $100000, %rax`).

**측정**: json_parse text backend 1.12 → **1.04 ✅** (안정 측정, 50-run × 3-round 검증).

### Cycle 2533 — P-A.3'/P-A.4' inkwell Rule 7 parity (RE-PLAN)

**Trigger 🟠 RE-PLAN**: sweep 1차 결과 inkwell이 cycle 2532 변경 미반영 발견 → inkwell 빌드 재컴파일. 그러나 sweep 2차에서 `csv_parse 1.087, lexer 1.07, json_parse 1.16` — text backend 대비 큰 격차. 분석 결과 inkwell이 `func.no_inline`/`func.inline_hint` 둘 다 emit 안 함 + `"memory"="read"` 문자열 속성이 LLVM 16+에 무시됨. → cycle 2535 (Rule 7 parity) 작업을 본 cycle에 통합.

**구현** (`bmb/src/codegen/llvm.rs:1955-1981`):
- `noinline` enum 추가 — cycle 2532 효과 inkwell mirror.
- `readonly` enum 추가 — `memory(read)` compatibility shim (LLVM 16+).

**시도-revert** (mandelbrot 회귀 회피):
- `inlinehint` enum 추가 → mandelbrot 1.00 → 1.07 ❌ → revert (코멘트 보존).
- `readnone` enum 추가 → mandelbrot 1.06 잔존 → revert + `"memory"="none"` 문자열 유지.

**측정**: json_parse inkwell 1.16 → **1.04 ✅**. mandelbrot 안정.

### Cycle 2534 — B-? http_parse 빌드 결함 (text-backend-specific)

**재현 검증**: cycle 2531 보고된 사전 결함 = "자체-해결 또는 간헐적" 가설 → 부정. **text backend에서만 재현**, inkwell은 정상.

**Root cause**: `bmb/src/codegen/llvm_text.rs:8512` Switch terminator가 i64 hardcode. `LoopBoundedNarrowing`이 `digit_char(d: i64)` → `digit_char(i32 noundef %d)` narrow했으나 switch는 `switch i64 %d` → clang IR validation 실패. inkwell은 `disc_int.get_type()` 동적 추출으로 정상.

**Fix**: text backend에 동일 narrow-aware 처리 적용. `narrowed_param_names` 검사로 disc_ty `i32`/`i64` 결정.

**측정**: http_parse text backend 빌드 ✅ + 0.926×C **FAST**.

### Cycle 2535 — Bootstrap 3-Stage 검증

**검증**: `BMB_ARENA_MAX_SIZE=16G bash scripts/bootstrap.sh` ✅
- Stage 1 (10.6s) + Stage 2 (26.6s) + Stage 3 (33.5s) = 71s
- **Fixed Point S2 == S3** ✅

cycle 2530-2534 변경 누적 (MemoryEffect + noinline + parity + switch narrow)이 self-hosting 결정성 보존 입증.

---

## 2. 산출물

### Tracked (commit `c960edbb`)
| 분류 | 파일 |
|------|------|
| 코드 변경 | `bmb/src/mir/optimize.rs` (AggressiveInlining: collect_in_loop_callees + should_no_inline_for_licm + run_on_program 확장) |
| 코드 변경 | `bmb/src/codegen/llvm.rs` (noinline + readonly enum 추가, inlinehint/readnone revert reasoning 코멘트) |
| 코드 변경 | `bmb/src/codegen/llvm_text.rs` (Switch terminator narrow-aware operand type) |

### Gitignored (local only)
| 분류 | 파일 |
|------|------|
| Cycle logs | `claudedocs/cycle-logs/cycle-{2532,2533,2534,2535}.md` |
| Run-cycle ROADMAP | `claudedocs/cycle-logs/ROADMAP.md` (M1 strict gate 16/16 PASS marking) |
| 기존 HANDOFF | `claudedocs/HANDOFF.md` (본 문서가 갱신) |
| 측정 binaries | `target/cycle-2532/*.{exe,ll,opt.ll,s}`, `target/benchmarks/*` |
| Bootstrap | `target/bootstrap/{bmb-stage1,bmb-stage2,bmb-stage3}*` |

### 메모리 (auto-memory)
| 파일 | 변경 |
|------|------|
| `MEMORY.md` | "M1 Strict Gate" 인덱스 항목 추가 |
| `project_session_2026_05_01_m1_strict_gate.md` | 신규 — 4-cycle 결과 |
| `project_benchmark_reality.md` | 16/16 ≤1.05 0 FAIL 갱신 |
| `project_bootstrap_status.md` | Cycle 2535 추가 (71s, S2==S3) |

---

## 3. 검증 상태

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ 2m 27s (text backend) |
| `cargo build --release --features llvm --target x86_64-pc-windows-gnu` | ✅ 2m 36s (inkwell, 2회) |
| `cargo test --release --lib` | ⚠️ 3772/3773 (1 pre-existing `verify::contract::tests::test_trivial_contract_detection` — Cycle 2530-2531 검증으로 무관) |
| 부트스트랩 3-Stage | ✅ S2 == S3 (71s) |
| Tier 1+3 sweep (16 historic) | ✅ **16/16 ≤1.05 strict gate, 0 FAIL** |
| http_parse text backend 빌드 | ✅ (was: linker error) |

### 정밀 측정 (50-run top-3, 안정 후)

| Bench | BMB | C | 판정 |
|-------|-----|---|------|
| json_parse (text) | 25 | 24 | **1.04 ✅** (was 1.12) |
| json_parse (inkwell) | 25 | 24 | **1.04 ✅** (was 1.16) |
| http_parse (text) | 25 | 27 | **0.93 ✅ FAST** (was: linker error) |
| mandelbrot (inkwell) | 163 | 160 | 1.02 ✅ |
| 나머지 12 벤치 | 모두 ≤1.05 | 회귀 0 |

---

## 4. 다음 세션 우선순위

### P-A.3'' [코드젠] inkwell `inlinehint`+`readnone`+`speculatable` combo 튜닝 (별도 session)

**근거**: Cycle 2533에서 `inlinehint` 단독 추가 시 mandelbrot 1.07, `readnone` 단독 추가 시 1.06 회귀. text backend는 둘 다 + `speculatable` 함께 emit해서 mandelbrot 1.01. inkwell도 `speculatable` 추가 시 회귀 회피 가능성.

**작업 범위**:
1. `bmb/src/codegen/llvm.rs`에 `func.is_memory_free && !has_user_call` 조건 시 `speculatable` enum 추가.
2. `inlinehint` + `readnone` 동시 추가 후 mandelbrot 측정.
3. 회귀 시 trade-off 정밀 분석 (어떤 조합이 perf-neutral인지).

**추정**: 1-2 cycles.

### P-A.5 [코드젠] inkwell runtime decls Rule 7 parity (별도 session)

**근거**: `bmb/src/codegen/llvm.rs:890` `let memory_read_attr = self.context.create_string_attribute("memory", "argmem: read");` — 동일 string-attr 버그 패턴. 영향은 user functions 대비 작음 (runtime declarations는 LLVM이 inline하지 않음).

**작업 범위**:
1. `argmem: read` 표현 enum 인코딩 — LLVM 21에서 ArgMem-only Ref bitfield (= 1).
2. 또는 호환 shim `argmemonly` + `readonly` 조합 emit.
3. 측정 검증.

**추정**: 0.5-1 cycle.

### narrowed-locals switch handling (별도 session)

**근거**: Cycle 2534 fix는 param-only. Local이 `narrow_local`로 narrow된 경우 동일 패턴 가능. 현재 영향 0 (http_parse는 param case).

**작업 범위**: text backend의 `local_names.contains` arm에서 narrowed-locals tracking. 또는 `place_types` 활용.

**추정**: 0.5 cycle.

### Backlog (M2 도구층)

| 작업 | 추정 | 비고 |
|------|------|------|
| Track O Phase 2 — `bootstrap/context_pack/walker.bmb` | 1-2 | M2 도구층, Performance gap closed → 우선순위 상승 |
| Track N Phase 3 — 잔여 6 tools | 2-4 | M2 도구층 |
| Track Q Phase 2 — 키워드 충돌 + lint --ai-friendly | 2-3 | M2 도구층 |
| Track T Node bindings PoC | 2-3 | M3 진입 |

**우선순위 근거**: Performance-First phase **종료**. 후속 polish (P-A.3''/P-A.5)는 별도 micro-session. 도구층 (M2)으로 전환할 적기.

---

## 5. 환경 노트

| 환경 | 상태 |
|------|------|
| LLVM | 21.1.7-21.1.8 MSYS2 UCRT64 |
| GCC | MinGW-w64 |
| Rust | stable 1.95.0 |
| Z3 | `/c/msys64/ucrt64/bin/z3` (4.15.2) |
| BMB workspace | `Cargo.toml workspace.package.version = "0.98.0"` ✅ |
| `target/release/bmb.exe` (text backend) | May 1 ~22:08, 10MB (post-Cycle 2534 build) ✅ |
| `target/x86_64-pc-windows-gnu/release/bmb.exe` (LLVM inkwell) | May 1 ~22:00, 195MB (post-Cycle 2533 build) ✅ |
| Git working tree | Submodule untracked: `ecosystem/benchmark-bmb` (binary_trees/main_vec.bmb — 이전 세션, 본 세션 무관) |
| Branch | `main`, `origin/main` 대비 3 commits ahead (`943bdd3f` + `615a155a` + `996e343e` + `c960edbb`) |
| BMB_ARENA_MAX_SIZE | 부트스트랩에 16G 필수 (4G/8G OOM) |

---

## 6. Git 상태 + push 권고

### 잔여 untracked
```
ecosystem/benchmark-bmb (untracked: benches/compute/binary_trees/bmb/main_vec.bmb)
```
이전 세션부터 누적된 submodule 잔여. 본 세션 무관.

### Push 결정
- 4 commits ahead of `origin/main`: `943bdd3f` + `615a155a` + `996e343e` + `c960edbb`.
- 본 commit은 Performance-First phase 핵심 milestone — M1 ≤1.05 strict gate 16/16 PASS.
- CI 통과 가능성 높음 (cargo test 3772/3773, pre-existing failure 무관).
- **Push 결정은 사용자 권한** — `git push origin main` 권고.

---

## 7. 다음 세션 시작 액션

```bash
# 1. Git 상태 확인
git -C /d/data/lang-bmb log -4                  # c960edbb, 996e343e, 615a155a, 943bdd3f
git -C /d/data/lang-bmb status -s                # submodule untracked만

# 2. Performance-First phase 종료 확인
# 다음 priorities:
# (a) P-A.3'' inkwell inlinehint+readnone+speculatable combo (perf polish)
# (b) M2 도구층 (Track O/N/Q) — Performance gap closed → 도구로 전환

# 3. 부트스트랩 검증 (변경 시)
BMB_ARENA_MAX_SIZE=16G bash scripts/bootstrap.sh   # ~71s

# 4. Tier 1+3 sweep
bash scripts/measure-v098.sh
```

---

## 8. HUMAN-Decision

**없음**. P-A.3''/P-A.5/M2 도구층 모두 BMB 내부 작업 — 자율 진행.

다음 결정 후보:
- **Phase 전환 시점**: Performance-First → 도구층 (M2) 본격 진입 시점 결정 (P-A.3'' polish 먼저 완료 vs 바로 M2)
- **`docs/ROADMAP.md` major version 표시**: M1 ≤1.05 strict gate 16/16 PASS이 v1.0 release 전 필수 조건이었는지 — 외부 신호 (커뮤니티 사용 여부) 함께 평가 필요

---

## 9. 본 세션 핵심 메시지

**Performance-First phase 종료 — 16/16 ≤1.05, 0 FAIL 첫 달성**:
- HANDOFF 4 priorities (P-A.2', P-A.3', B-?, P-A.4')를 4 cycles로 완료. 자율 처리.
- json_parse 단 하나 outlier (1.12)가 cycle 2532 noinline으로 close → 16/16 strict gate.
- Bootstrap 3-Stage Fixed Point 보존 — codegen/MIR 변경 누적이 self-hosting 안정.

**RE-PLAN 1회**: cycle 2533이 sweep → inkwell parity로 자율 확장. 트리거 매트릭스 따라 ROADMAP.md 갱신 + 측정 → revert 사이클로 trade-off 격리. 별도 cycle 분할 없이 통합 처리.

**측정 변동성 관리**:
- sweep (10-run) vs 정밀 측정 (50-run top-3) 차이 인지.
- system load 영향 — interleaved 200-run으로 변동성 수치화.
- mandelbrot 1.06 → 1.02 진동은 system 변동, 코드 회귀 아님 입증.

**Cycles 2532-2535 ROI**:
- 4 cycles 사용 (10 budget 중 6 잔여 — 자율 종료)
- Net perf: json_parse 1.12 → 1.04 (text + inkwell 둘 다), http_parse 빌드 close + FAST, 14 다른 벤치 회귀 0
- Phase milestone: M1 ≤1.05 strict gate 16/16 PASS 첫 달성
- Carry-forward: P-A.3'' (polish), P-A.5 (parity), M2 도구층 (별도 session)

---

**세션 종료**: 2026-05-01 (Cycles 2532-2535, HEAD `c960edbb`)

**다음 세션 첫 액션**: 사용자 결정 — `git push origin main` 후 (a) P-A.3'' polish 별도 1-2 cycles 또는 (b) M2 도구층 전환.
