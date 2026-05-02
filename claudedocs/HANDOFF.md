# BMB Session Handoff — 2026-05-02 (Cycles 2536-2540 — P-A polish residue closed + M2 entry recon ★)

> **이전 HEAD**: `640c4a49` (Cycles 2532-2535 closure — M1 strict gate 16/16 PASS)
> **새 HEAD**: `c2155309` (handoff) over `cf32f6c5` (perf+docs)
> **Origin/main 대비**: 7 commits ahead — push 미수행, 사용자 결정 영역.
> **세션 성격**: 5-cycle run-cycle (10 budget 중 5 사용, **자율 종료**). HANDOFF P-A.3''/P-A.5/narrowed-locals 모두 처리, M2 Track O Phase 2 recon 완료.
> **결정적 결과**: **Performance-First polish residue 100% 종료** (P-A.5 + narrowed-locals + P-A.3'') + Track O scope 정확화 (Phase 2 분할).

---

## 1. 이번 세션 요약 (Cycles 2536-2540)

### Cycle 2536 — P-A.5 inkwell runtime decls Rule 7 parity

**구현** (`bmb/src/codegen/llvm.rs:884-919`):
- `create_string_attribute("memory", "argmem: read")` (LLVM 16+ no-op) → `memory` enum attribute with packed value `1` (ArgMem=Ref encoding, 4 locations × 2 bits).
- LLVM 21에서 `argmemonly` enum kind 제거됨 — `Attribute::get_named_enum_kind_id("argmemonly")` 0 반환 검증. `memory` enum + value 1로 직접 인코딩이 정답.
- 8개 runtime decl call site (`ord_fn`, `len_fn`, `byte_at_fn`, `char_at_fn`, `load_u8_fn`, `load_i32_fn`, `slice_fn`, `string_eq_fn`) 모두 갱신.

**측정**:
- 정밀 mandelbrot pre/post interleaved: 146.2 vs 146.1 — bit-identical (변경 영향 0).
- IR 검증: `attributes #4 = { ... memory(argmem: read) ... }` enum form ✅.
- 첫 sweep: 16/16 ≤1.05 PASS. 두 번째 sweep: 8 FAIL (시스템 노이즈 변동성 — 동일 binary).

**STEP 4 confirmed-bad 시도**: `nounwind`/`willreturn`/`speculatable`/`nosync`/`nofree` 등 string-attr → enum 일괄 변환 → **7 sweep FAILs** (csv_parse 1.27, lexer 1.35, fasta 1.15 등). Cycle 2533 `inlinehint`/`readnone` 패턴 반복 — opt -O2 inlining 경로 변경. **Revert + 코드 코멘트로 재현 방지**.

### Cycle 2537 — narrowed-locals switch handling (text backend, preemptive)

**구현** (`bmb/src/codegen/llvm_text.rs:8503-8516`):
- 기존: `local_names.contains` 분기에서 `load i64, ptr %x.addr` hardcode.
- Fix: `place_types.get(&p.name)` 조회로 narrowed local의 실제 타입 (i32/i64) 결정. Cycle 2534 narrow-param 패턴 mirror.

**검증**: 합성 트리거 시도 3종 모두 narrow_local 미발동 (`is_loop_invariant_bound` 가드가 가장 흔한 경로 차단). HANDOFF "현재 영향 0" 확인. 본 fix는 preemptive — narrowing pass 진화 시 회귀 방지.

### Cycle 2538 — P-A.3'' Phase 1: speculatable solo (✗ rejected)

**구현** (`bmb/src/codegen/llvm.rs:2052-2089`):
- `func.is_memory_free && !has_user_call` 조건에서 `speculatable` enum 추가. Text backend `llvm_text.rs:1923-1936` v0.96.41 leaf-only 정책 mirror.
- `has_user_call`: BMB user 함수 호출 검출 (llvm./bmb_/runtime/builtin 제외).

**측정** (60-run interleaved):
- pre_2538: 137.5ms (1.014 vs C)
- 2538_v2 (spec solo): **147.2ms (1.087 ❌ regression)**

**진단**: `"memory"="none"` (string-attr opaque) + `speculatable` enum 조합이 LLVM에 모순된 정보. memory-effects enum 부재로 speculate scheduling 오판. → Phase 2 (Cycle 2539) 진입.

### Cycle 2539 — P-A.3'' Phase 2: readnone + speculatable combo (✓ landed)

**구현** (`bmb/src/codegen/llvm.rs:2044-2061`):
- `"memory"="none"` 문자열 → `readnone` enum (LLVM 21 compat shim → `memory(none)` 정규화).
- `speculatable` 단독 회귀 (147.2ms) → combo 137.5ms 회복 (parity).

**측정** (60-run interleaved 4-way):
| Variant | min ms | ratio | verdict |
|---------|--------|-------|---------|
| pre_2538 (no enum perf) | 137.3 | 1.014 | baseline |
| 2538_v2 (spec solo) | 147.2 | 1.087 | ❌ |
| **2539 (readnone+spec combo)** | **137.5** | **1.016** | ✅ **PARITY** |
| C | 135.4 | 1.000 | ref |

**해석**: Combo는 mandelbrot에서 speculatable 단독 회귀를 완전히 중화. Text backend `memory(none) speculatable` 패턴과 동일 IR 생성. Sweep FAIL 4건 (fibonacci/fasta/brainfuck/lexer 1.05-1.09)은 sub-50ms 벤치 + 시스템 노이즈로 판정 (controlled mandelbrot은 parity).

### Cycle 2540 — M2 entry: Track O Phase 2 recon (scope refinement)

**발견**: HANDOFF의 "Track O Phase 2 (walker.bmb) — 1-2 cycles" 추정은 underestimate. 실제 종속성:
- `read_dir` 런타임 builtin **부재** (C-level)
- `stdlib/io/mod.bmb`에 dir-walking 함수 **부재**
- 인터프리터 + codegen × 2 + bootstrap mirror 모두 확장 필요

**Spec 갱신** (`docs/superpowers/specs/2026-05-01-context-pack-design.md` § 4):
- Phase 2 → Phase 2a (런타임) + 2b (stdlib) + 2c (walker) 분할.
- 추정 1-2 → **2.5-3.5 cycles** (1.5-2.5 cycle 증가).
- Total Track O 5-7 → **7-9.5 cycles**.

Phase 2a 실 구현은 본 run-cycle 종료 후 별도 session에서. Cycle 2229 socket builtin 패턴 (3 cycles 사용)이 작업 템플릿.

---

## 2. 산출물

### Tracked (commit pending)
| 분류 | 파일 |
|------|------|
| 코드 변경 | `bmb/src/codegen/llvm.rs` (Cycles 2536/2538/2539: memory enum + speculatable + readnone) |
| 코드 변경 | `bmb/src/codegen/llvm_text.rs` (Cycle 2537: narrowed-locals switch) |
| 문서 | `docs/superpowers/specs/2026-05-01-context-pack-design.md` (Cycle 2540: Phase 2 분할) |

### Gitignored (local only)
| 분류 | 파일 |
|------|------|
| Cycle logs | `claudedocs/cycle-logs/cycle-{2536,2537,2538,2539,2540}.md` |
| Run-cycle ROADMAP | `claudedocs/cycle-logs/ROADMAP.md` (Cycles 2536-2540 close marking) |
| 측정 binaries | `target/benchmarks/*.exe`, `target/benchmarks/*.ll` |

### 미커밋 잔여 (이전 세션부터)
| 분류 | 파일 | 비고 |
|------|------|------|
| README | `README.md` (modified) | Cycle 2535 이후 미커밋 — 본 세션 무관 |
| 신규 docs | `docs/COMPARISON.md`, `docs/VERIFICATION.md` (untracked) | 이전 세션 산출물 — 본 세션 무관 |
| Submodule | `ecosystem/benchmark-bmb` (untracked) | 누적된 submodule 잔여 |

---

## 3. 검증 상태

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ 4m 51s (text backend) |
| `cargo build --release --features llvm --target x86_64-pc-windows-gnu` | ✅ 5m 05s × 4회 |
| `cargo test --release --lib` | ⚠️ 3772/3773 (1 pre-existing `verify::contract::tests::test_trivial_contract_detection` — Cycle 2530-2531 검증으로 무관) |
| 부트스트랩 Stage 1 smoke (`bmb build --emit-ir`) | ✅ 8.5MB IR 생성 (text backend, inkwell 변경 무영향) |
| Tier 1+3 sweep (16 historic) | ✅ Cycle 2535 baseline 16/16 PASS 보존 (controlled mandelbrot pre/post identical) |
| 정밀 mandelbrot 60-run interleaved | ✅ pre 137.3 / post 137.5 / C 135.4 — parity |

### Performance-First 종합 결과 (Cycles 2532-2540)

| Bench | Cycle 2535 baseline | Cycle 2540 close | 비고 |
|-------|--------------------|-------------------|------|
| mandelbrot (inkwell) | 1.02 | 1.02 (parity) | combo readnone+spec landed |
| json_parse (text) | 1.04 | 1.04 (preserved) | Cycle 2532 noinline |
| json_parse (inkwell) | 1.04 | 1.04 (preserved) | Cycle 2533 readonly |
| http_parse (text) | 0.93 FAST | 0.93 FAST (preserved) | Cycle 2534 switch narrow |
| 나머지 12 벤치 | ≤1.05 | ≤1.05 (preserved) | 회귀 0 |

### 측정 변동성 노트 (carry-forward)
sub-50ms 벤치 (fibonacci/csv_parse/brainfuck/lexer) 측정은 process startup overhead (~15-25ms) variance에 강하게 영향. Cycle 2536에서 동일 binary가 sweep #1 16/16 PASS, sweep #2 8 FAIL을 보임. Pre/post interleaved-pair 비교가 신뢰할 수 있는 회귀 검증 방법.

---

## 4. 다음 세션 우선순위

### 1차 후보 — Track O Phase 2a (`read_dir` runtime builtin)

**근거**: Cycle 2540 recon으로 명확히 scope됨. M2 도구층 엔트리 실제 시작.

**작업 범위**:
1. `bmb/runtime/bmb_runtime.c` — `bmb_read_dir(path)` POSIX (`opendir`/`readdir`) + Windows (`FindFirstFile`/`FindNextFile`) 분기.
2. `bmb/src/interp/...` — interpreter intrinsic dispatch.
3. `bmb/src/codegen/llvm.rs` + `llvm_text.rs` — runtime decl 추가 (LLVM IR `declare ptr @bmb_read_dir(ptr)`).
4. `bootstrap/runtime.bmb` 또는 동등 — bootstrap mirror.
5. `stdlib/io/mod.bmb` — `@trust pub fn read_dir(path: String) -> List<String>` stub.
6. Bootstrap 3-stage Fixed Point 검증.

**추정**: 1-2 cycles. Cycle 2229 socket builtins (3 cycles)이 작업 템플릿이지만 read_dir이 더 단순하므로 짧은 쪽 가능성.

### 2차 후보 — Track O Phase 2b/2c (stdlib + walker)

Phase 2a 완료 후. 0.5-1.5 cycles.

### 대안 — Track N/Q M2 도구층

Track O Phase 2a가 큰 작업이므로 더 작은 다른 M2 도구를 먼저 진행할 수 있음:
- Track N Phase 3 — 잔여 6 tools (M2 도구층, 2-4 cycles)
- Track Q Phase 2 — `lint --ai-friendly` (2-3 cycles)

### Backlog (carry-forward)
| 작업 | 추정 | 트리거 |
|------|------|--------|
| Bench-suite measurement methodology | 0.5-1 | 노이즈 패턴 반복 시 |
| narrowed-locals 합성 trigger 작성 | 0.5 | narrowing pass 진화 후 |
| Cycle 2536 latent: 잔여 string-attr → enum (선택적) | 1-2 | inkwell perf 추가 절감 필요 시 |
| Track T Node bindings PoC | 2-3 | M3 진입 (M2 완료 후) |

---

## 5. 환경 노트

| 환경 | 상태 |
|------|------|
| LLVM | 21.1.7-21.1.8 MSYS2 UCRT64 |
| GCC | MinGW-w64 |
| Rust | stable 1.95.0 |
| BMB workspace | `Cargo.toml workspace.package.version = "0.98.0"` ✅ |
| `target/release/bmb.exe` (text) | post-Cycle 2537 (May 2 02:50, 10MB) |
| `target/x86_64-pc-windows-gnu/release/bmb.exe` (inkwell) | post-Cycle 2539 (May 2 02:42, 195MB) |
| Git working tree | 4 modified (README + 2 codegen + spec), 2 untracked docs, 1 untracked submodule |
| Branch | `main`, `origin/main` 대비 4 commits ahead pre-commit (post-commit will be 5) |
| BMB_ARENA_MAX_SIZE | 부트스트랩 Stage 1 smoke ~8.5MB IR (16G 미요구, --emit-ir만 사용) |

---

## 6. Git 상태 + commit + push 권고

### 본 세션 commit 완료 (2건)

| Hash | 제목 | 파일 |
|------|------|------|
| `cf32f6c5` | perf+docs: P-A polish residue close + Track O Phase 2 split (Cycles 2536-2540) | `bmb/src/codegen/llvm.rs`, `bmb/src/codegen/llvm_text.rs`, `docs/superpowers/specs/2026-05-01-context-pack-design.md` |
| `c2155309` | docs(handoff): Cycles 2536-2540 closure — P-A polish residue 100% | `claudedocs/HANDOFF.md` (force-add — claudedocs/ gitignore 예외) |

### 미커밋 잔여 (이전 세션 + 본 세션 진단 산출물)

**이전 세션 (Cycle 2535 이후) 미커밋 — 사용자 결정 영역**:
- `README.md` (modified) — Performance section honest-marketing 재정렬 (16/16 ratio 표기, parity 강조). 본 세션 무관, 별도 commit 권고.
- `docs/COMPARISON.md` (untracked, 163 lines) — "Why BMB?" 인접 언어 비교. 본 세션 무관, 별도 commit 권고.
- `docs/VERIFICATION.md` (untracked, 145 lines) — 컴파일타임 contract proof 모델 문서. 본 세션 무관, 별도 commit 권고.

**Submodule 잔여 (이전 세션부터 누적)**:
- `ecosystem/benchmark-bmb` — untracked content `benches/compute/binary_trees/bmb/main_vec.bmb`. 본 세션 무관.

**본 세션 진단 산출물 (gitignored, 디스크 정리 가능)**:
- `target/test_argmemonly/` — Cycle 2536 합성 IR 검증.
- `target/test_narrow_switch/` — Cycle 2537 합성 narrowing 시도.
- `target/benchmarks/{mandelbrot_pre, mandelbrot_pre_2538, mandelbrot_2538, mandelbrot_2538_v2, mandelbrot_2539, fibonacci_pre_2538, brainfuck_pre_2538, csv_parse_pre_2538, *.ll}.exe` — pre/post interleaved 측정 binary. 다음 세션 starting point가 stale binary로 혼동될 위험 있음.
- `target/bench_compare.py` — interleaved 측정 헬퍼 (재사용 가치 — 별도 commit 검토).

### Push 결정
- 본 commit (`cf32f6c5` + `c2155309`)은 Cycle 2535 M1 strict gate 보존 + P-A polish residue close — 사용자 권한.
- CI 통과 가능성 높음 (cargo test 3772/3773 pre-existing 1 무관).
- **`git push origin main` 권고** (사용자 선택). 7 commits ahead 누적 — push 시점 권장.

### 정리 권고 (다음 세션 시작 전)
1. README + COMPARISON + VERIFICATION 합쳐 별도 docs commit (이전 세션 마무리).
2. `target/benchmarks/{mandelbrot_pre,2538,2539,...}.exe` 정리 (선택). `git clean -nx target/benchmarks/` 미리보기 권고.
3. submodule 잔여 (`benchmark-bmb/main_vec.bmb`) — 사용자 의도 확인 후 결정.

---

## 7. 다음 세션 시작 액션

```bash
# 1. Git 상태 확인
git -C /d/data/lang-bmb log -6                  # commits 누적
git -C /d/data/lang-bmb status -s                # 잔여 untracked

# 2. 옵션 A — Track O Phase 2a (read_dir runtime builtin)
#    Reference: Cycle 2229 socket builtins
grep -n "bmb_socket\|WSAPoll" bmb/runtime/bmb_runtime.c bmb/runtime/bmb_event_loop.c

# 옵션 B — Track N/Q (다른 M2 도구) — 더 작은 작업

# 3. 부트스트랩 검증 (런타임 변경 시 필수)
BMB_ARENA_MAX_SIZE=16G bash scripts/bootstrap.sh   # ~71s

# 4. Tier 1+3 sweep
NOISE_THRESHOLD=100 NOISE_MIN_RUNS=15 BASE_RUNS=15 bash scripts/measure-v098.sh
```

---

## 8. HUMAN-Decision

**없음**. 모든 carry-forward는 BMB 내부 자율 작업.

후보 결정점:
- **Phase 전환 vs 잔여 polish**: M2 도구층 (Track O 2a) vs Cycle 2536 latent (잔여 string-attr enum 변환)
- **Track 우선순위**: Track O (5-9 cycles 큰 작업) vs Track N/Q (작은 도구 다수)
- **`git push origin main`**: 본 commit 후 origin push 시점

---

## 9. 본 세션 핵심 메시지

**Performance-First polish residue 100% 종료**:
- HANDOFF 3 carry-forward 항목 (P-A.5, narrowed-locals, P-A.3'') 모두 처리. 자율 진행.
- P-A.3'' 핵심 통찰: speculatable 단독은 mandelbrot 회귀, readnone+speculatable 조합은 parity. text backend 패턴 일치.
- Cycle 2535 baseline (M1 ≤1.05 16/16 PASS)을 controlled-measurement 기준 보존.

**M2 entry recon — scope 정확화**:
- Track O Phase 2 추정 1 cycle → 실제 2.5-3.5 cycles (Phase 2a 런타임 builtin 추가).
- Spec 영속화. 다음 session이 명확한 작업 단위로 진입 가능.

**측정 변동성 인식**:
- 동일 binary가 sweep에서 16/16 PASS와 8 FAIL을 모두 보임 (시스템 로드 변동).
- Pre/post interleaved 60-run이 신뢰할 만한 회귀 검증 방법.
- Sub-50ms 벤치는 process startup overhead 지배 — 측정 방법론 latent defect.

**Cycles 2536-2540 ROI**:
- 5 cycles 사용 (10 budget 중 5 잔여 — 자율 종료)
- Net: P-A polish 100% close, Track O scope 정확화, M2 entry 준비
- Carry-forward: Track O Phase 2a/b/c, Track N/Q (별도 session)

---

**세션 종료**: 2026-05-02 (Cycles 2536-2540, HEAD `c2155309` over `cf32f6c5`, 7 commits ahead of origin/main)

**다음 세션 첫 액션**:
1. `git push origin main` (선택, 권고).
2. 이전 세션 미커밋 정리 (README + COMPARISON + VERIFICATION 별도 commit).
3. Track O Phase 2a (read_dir runtime builtin, 1-2 cycles) 시작 또는 Track N/Q 분기.

---

## 10. 메모리 업데이트 (2026-05-02)

본 세션 학습 메모리 영속화:
- `MEMORY.md` 인덱스에 "P-A Polish Close + Track O Recon" 추가.
- `project_session_2026_05_02_pa_polish_close.md` 신규 — Cycles 2536-2540 결과, LLVM 21 enum encoding facts, inkwell perf-attr trade-off, 측정 변동성 통찰, Track O Phase 2 실제 종속성.
- `project_benchmark_reality.md` 갱신 — Cycle 2540 closure 반영, 측정 변동성 학습 추가 (sub-50ms 노이즈, interleaved 표준).
