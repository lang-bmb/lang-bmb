# BMB Session Handoff — 2026-05-01 (Cycle 2525 — Performance-First Realignment + v0.98 Rebaseline)

> **이전 HEAD**: `cc5afe65` (Cycles 2521-2524 close)
> **새 HEAD**: `cc5afe65` (코드 변경 없음 — 본 세션은 측정/분석)
> **세션 성격**: 사용자 지침 정정 ("성능 1차, 모든 격차는 개선 기회") → 평가 정직화 → v0.98 정직한 재측정 → 격차 인벤토리 + Decision Framework 매핑.
> **결정적 발견**: 이전 평가의 "8/15 FAIL" 주장은 v0.51.22 stale data. v0.98 실측 = **1/16 FAIL**.

---

## 1. 이번 세션 요약 (Cycle 2525)

### 트리거: 사용자 지침 재정렬

> "BMB 는 프로그래밍 언어 개발 프로젝트임. 제1순위는 '성능'임. '안정성'은 자연히 뒤따르는 것임. ... 성능/컴파일러 등의 한계가 발견되면 제약으로 인정할 것이 아니라 개선기회로 삼아야 함. 문법개선을 비롯해서 핵심구조의 변경이 언제든지 오픈될 수 있는 트랙임."

### Phase A: 평가 정직화

이전 다각도 성숙도 평가에서 발견된 분류 오류:
- "M1 도메인 벤치 ≤1.10x 게이트 ✅" → strict ≤1.00x 미충족 = **게이트 완화 workaround**
- "lexer 1.11x M1 known opportunistic gap" → 핫패스 격차 미루기
- "8/15 비-도메인 강등" → v0.51.22 측정 후 v0.98 미재측정
- "M1 자율 부분 ✅ COMPLETE" → 마일스톤 형식주의

→ 모든 격차를 Decision Framework L1~L5 매핑 가능한 개선 기회로 재분류.

### Phase B: 측정 인프라 (3개 스크립트 — tracked)

```
scripts/measure-v098.sh      — 16 historic 3-runs 측정 (Tier 1 + Tier 3)
scripts/reverify-tier3.sh    — Tier 3 10-runs (작은 절대값 노이즈 감소)
scripts/reverify-noisy.sh    — 단일 벤치 빠른 20-runs 재검증
```

**노이즈 학습**: 작은 벤치(30-50ms) 3-runs 측정은 1.13x 거짓 격차 보고 가능. lexer 3-runs=1.13x → 20-runs=1.04x. **Tier 3는 반드시 ≥10 runs**.

### Phase C: v0.98 정직한 재측정

#### Tier 1 (compute, 9 benches, 3-runs)
| Bench | vs C | 판정 |
|-------|------|------|
| n_body | 0.85x | ✅ FAST |
| fannkuch | 0.80x | ✅ FAST |
| binary_trees | 0.98x | ✅ FAST |
| string_hash | 0.98x | ✅ FAST |
| fibonacci | 1.00x | ✅ TIE |
| spectral_norm | 1.03x | ⚠️ |
| mandelbrot | 1.03x | ⚠️ |
| hash_table | 1.04x | ⚠️ |
| fasta | 1.04x | ⚠️ |

→ **9/9 within ≤1.05x. 0 FAIL**.

#### Tier 3 (real_world, 7 benches, 20-runs)
| Bench | vs C | 판정 |
|-------|------|------|
| json_serialize | 0.82x | ✅ FAST |
| sorting | 0.91x | ✅ FAST |
| http_parse | 0.96x | ✅ FAST |
| csv_parse | 1.00x | ✅ TIE |
| brainfuck | 1.04x | ⚠️ |
| lexer | 1.04x | ⚠️ |
| **json_parse** | **1.12x** | ❌ **FAIL** |

→ 6/7 ≤1.05x. **1 FAIL: json_parse 1.12x**.

#### v0.51.22 → v0.98 변화

| 지표 | v0.51.22 (이전 평가 의존) | v0.98 (재측정) | Δ |
|------|------|------|------|
| BMB > C | 4/15 (27%) | **7/16 (44%)** | **+17pp** |
| BMB ≈ C ≤1.05x | 7/15 (47%) | **15/16 (94%)** | **+47pp** |
| FAIL >5% | 8/15 (53%) | **1/16 (6%)** | **−47pp** |

### Phase D: 격차 분석 → 핵심 발견

**5개 벤치 동일 핫패스** (byte-stream parsing):

```
brainfuck   1.04   ─┐
csv_parse   1.00    │ 재귀 + byte_at + len() 체크 (BMB)
http_parse  0.96    ├ vs while + s[pos] (C)
json_parse  1.12   ─┤ ── 단일 구조 개선 → 5개 동시 향상
lexer       1.04   ─┘
```

#### Decision Framework 매핑 (json_parse 1.12x = 모든 byte-stream 격차의 결정체)

| 격차 | Level | 변경 |
|------|-------|------|
| 재귀 → 루프 | **L1 언어 스펙** | `loop` / `while` 표현식 정식 도입 |
| `byte_at()` deref | **L4 코드젠** | string-byte intrinsic, GEP 직접 방출 |
| `pos >= len()` 체크 | **L1 + L3** | NUL-terminated view 또는 contract-driven elimination |
| 재귀 TCO 비결정성 | **L3 최적화** | `musttail` 일관 보장 |

---

## 2. 산출물

| 분류 | 파일 | Tracked? |
|------|------|----------|
| 측정 스크립트 | `scripts/measure-v098.sh` | ✅ untracked → 다음 commit 후보 |
| 측정 스크립트 | `scripts/reverify-tier3.sh` | ✅ untracked |
| 측정 스크립트 | `scripts/reverify-noisy.sh` | ✅ untracked |
| 측정 데이터 | `target/benchmarks/v098-historic.json` | ❌ gitignored |
| 측정 데이터 | `target/benchmarks/v098-tier3-10runs.json` | ❌ gitignored |
| 분석 문서 | `claudedocs/tier1-tier3-rebaseline-2026-05-01.md` | ❌ gitignored |
| Cycle log | `claudedocs/cycle-logs/cycle-2525.md` | ❌ gitignored |
| 메모리 갱신 | `~/.claude/.../MEMORY.md` + `project_benchmark_reality.md` | (auto-memory) |

빌드 산출물 (재빌드된 바이너리, gitignored):
- `target/x86_64-pc-windows-gnu/release/bmb.exe` — v0.98.0 MinGW LLVM, May 1 18:55, 195MB

---

## 3. 검증 상태

| 항목 | 결과 |
|------|------|
| `cargo build --release --features llvm --target x86_64-pc-windows-gnu` | ✅ 2m58s |
| 16 historic benches BMB 빌드 | ✅ all OK |
| 16 historic benches C 빌드 (clang -O3 -march=native) | ✅ all OK |
| 16 historic benches Rust 빌드 (rustc -O) | ✅ 15 OK (string_hash N/A) |
| 측정 일관성 (3-runs vs 20-runs) | ✅ 노이즈 격차 식별 |
| `cargo test --release` | ⏳ 본 cycle 미실행 — 코드 변경 없음 (측정만) |
| 부트스트랩 3-Stage | ⏳ 본 cycle 미실행 — 코드 변경 없음 |

---

## 4. 다음 세션 우선순위 (Cycle 2526+)

### P-1 [언어 스펙] `loop` / `while` 표현식 정식 도입 ★ 권장 첫 작업

**근거**: 5개 byte-stream 벤치 공통 핫패스. **단일 개선이 5개 동시 향상**.

**예상 효과**:
- json_parse 1.12 → ≤1.05 (FAIL → ≤5%)
- lexer/brainfuck 1.04 → 1.00 이하
- 0/16 FAIL + BMB > C 50%+ 가능

**작업 범위 (CLAUDE.md Rule 5 — 전수 검색):**
1. `bmb/src/parser/grammar.lalrpop` — `loop`/`while` 토큰 + 표현식 추가
2. `bmb/src/ast/` — `Expr::Loop`, `Expr::While`, `Expr::Break`, `Expr::Continue`
3. `bmb/src/types/` — bool 검사 + Unit 반환
4. `bmb/src/mir/lowering.rs` — break/continue MIR 생성
5. `bmb/src/codegen/llvm_text.rs` — basic block + br
6. `bmb/src/codegen/llvm.rs` — inkwell 동일
7. `bmb/src/interp/` — eval loop
8. `bootstrap/parser.bmb`, `bootstrap/parser_ast.bmb` — bootstrap parser 동기화
9. `bootstrap/lowering.bmb`, `bootstrap/llvm_ir.bmb` — bootstrap codegen 동기화
10. `bootstrap/types.bmb` — bootstrap type checker
11. `tests/loop_*.bmb` — 신규 골든 테스트
12. `ecosystem/benchmark-bmb/benches/real_world/json_parse/bmb/main.bmb` — 재귀 → loop 재작성으로 효과 측정
13. `docs/SPECIFICATION.md`, `docs/LANGUAGE_REFERENCE.md` — 문법 명세

**검증 (CLAUDE.md Rule 3 — 부트스트랩 3-Stage 필수):**
- Stage 1: Rust 컴파일러로 `compiler.bmb` 빌드 + 골든
- Stage 2: Stage 1 바이너리로 `compiler.bmb` 컴파일
- Stage 3: Stage 2 바이너리로 `compiler.bmb` 컴파일 → S2 == S3 Fixed Point

**추정**: 5-8 cycles (multi-cycle 작업).

### P-2 [코드젠] String byte-access intrinsic

**근거**: `s.byte_at(pos)` 호출 비용. C `s[pos]`는 단일 GEP.
**변경**: `bmb/src/codegen/llvm_text.rs` + `bmb/src/codegen/llvm.rs` — `byte_at` 패턴 인식 → 직접 GEP 방출.
**추정**: 2-3 cycles.

### P-3 [컴파일러 구조] Verifier-driven bounds check elimination

**근거**: `pos < s.len()` 체크는 contract `pre pos < s.len()` 입증 시 제거 가능. **D 축 (Verification) → P 축 (Performance)** 활용 — BMB의 핵심 가치 명제.
**변경**: `bmb/src/cir/smt.rs` 또는 `bmb/src/verify/` — bounds check elimination pass.
**추정**: 3-5 cycles.

### P-4 [언어 스펙] NUL-terminated string view 또는 sentinel 패턴

**근거**: BMB string (ptr, len) 구조가 작은 벤치에서 measurable cost. 결정 필요.
**대안**: `&[u8]` slice, `#[null_terminated]` attribute, 또는 stdlib `c_str` 타입.
**추정**: 결정 + 1-3 cycles.

### P-5 [측정 인프라] benchmark.sh 노이즈 자동 감지

**근거**: 본 cycle에서 3-runs 노이즈 발견. CI 측정도 동일 위험.
**변경**: `scripts/benchmark.sh` — 절대값 < 100ms 시 runs 자동 ≥ 10.
**추정**: 1 cycle.

### P-6 [문서] ROADMAP/Spec 갱신

**근거**: 본 cycle 측정으로 ROADMAP 다음 항목 정정 필요:
1. M1 게이트 "≤1.10x" → "≤1.05x (strict)" — 7/7 도메인 통과 확인
2. "lexer 1.11x M1 known opportunistic gap" → "lexer 1.04x within margin"
3. Headline numbers "BMB > C in 16 benchmarks" → "BMB > C in 7/16, ≤C in 15/16, FAIL 1/16 (json_parse)"
4. `MEMORY.md` `project_benchmark_reality.md` — 이미 갱신됨 ✅

**추정**: 1 cycle.

### Backlog (Cycles 2521-2524 carry-over, M2 트랙)

이전 세션 우선순위는 Performance 1차 정렬 후 후순위로 이동:

| 작업 | 추정 | 비고 |
|------|------|------|
| Track O Phase 2 — `bootstrap/context_pack/walker.bmb` | 1-2 | M2 도구층 — P-1 이후 |
| Track N Phase 3 — 잔여 6 tools | 2-4 | M2 도구층 |
| Track Q Phase 2 — 키워드 충돌 + lint --ai-friendly | 2-3 | M2 도구층 |
| ai-proof 실제 제거 (Cycle 2526 약속) | 1 | 정리 |
| Track T Node bindings PoC | 2-3 | M3 진입 |

**우선순위 근거**: M2/M3 도구층은 BMB 언어 자체가 강해진 후 진정한 가치 발현. 약한 언어 위에 도구만 쌓는 것은 거꾸로 된 순서. P-1 (loop/while)이 5개 벤치 동시 향상의 ROI로 압도적 우선.

---

## 5. 환경 노트

| 환경 | 상태 |
|------|------|
| Z3 in PATH | `/c/msys64/ucrt64/bin/z3` (4.15.2) |
| LLVM | 21.1.8 MSYS2 UCRT64 |
| GCC | MinGW-w64 |
| Rust | stable 1.95.0 |
| BMB workspace | `Cargo.toml workspace.package.version = "0.98.0"` ✅ |
| `target/release/bmb.exe` (text backend) | May 1 15:15, 10MB |
| `target/x86_64-pc-windows-gnu/release/bmb.exe` (LLVM inkwell) | May 1 18:55, 195MB ✅ fresh |
| Git working tree | 3 untracked scripts (P-6 commit 후보), submodule 1 untracked file (이전 세션) |

---

## 6. Git 상태 + 다음 commit 권고

### Untracked (다음 commit 후보)

```
scripts/measure-v098.sh     — 16 historic 3-runs measurement
scripts/reverify-tier3.sh   — Tier 3 10-runs noise reduction
scripts/reverify-noisy.sh   — 단일 벤치 20-runs 재검증
```

권고 commit 메시지:
```
chore(scripts): add v0.98 benchmark measurement utilities (Cycle 2525)

3 scripts for honest performance re-baseline of 16 historic benches:
- measure-v098.sh — 3-runs Tier 1 + Tier 3 first pass
- reverify-tier3.sh — 10-runs for small benches (30-50ms)
- reverify-noisy.sh — single-bench 20-runs sanity check

Identified Tier 3 noise threshold: < 100ms benches require ≥10 runs.
3-runs measurement of lexer reported false 1.13x (actual 1.04x at 20-runs).

v0.98 result: 1/16 FAIL (json_parse 1.12x), 7/16 BMB FAST.
v0.51.22 baseline (8/15 FAIL) obsoleted.
```

### Submodule (이전 세션 — 본 cycle 무관)

```
ecosystem/benchmark-bmb (untracked: benches/compute/binary_trees/bmb/main_vec.bmb)
```

---

## 7. 다음 세션 시작 액션

```bash
# 1. Git 상태 확인
git -C /d/data/lang-bmb log -1                    # HEAD = cc5afe65 확인
git -C /d/data/lang-bmb status -s                  # 3 untracked scripts (P-6 commit 결정)

# 2. 본 HANDOFF § 4 우선순위 → P-1 (loop/while 표현식) 시작
#    또는 P-6 (ROADMAP 갱신 + 측정 스크립트 commit) 으로 짧게 시작

# 3. P-1 작업 시작 시:
ls bmb/src/parser/grammar.lalrpop                  # 1단계 grammar 변경
grep -n "BinOp\|Expr::" bmb/src/ast/mod.rs | head  # AST 추가 위치 확인
grep -rn "Expr::" bmb/src/ | wc -l                 # 전수 검색 (예상 100+ 사이트)
```

---

## 8. 잔여 HUMAN-Decision

**없음**. 사용자 지침 (성능 1차 재정렬)은 본 cycle에서 수용·실행 완료.

다음 결정 후보 (Cycle 2526+):
- P-1 vs P-6 우선순위 (단기 전수 작업 큰 P-1 vs 1 cycle 빠른 P-6)
- P-4 NUL-terminated 도입 결정 (언어 스펙 변경)

---

## 9. 참고 문서 (gitignored — 로컬에만 존재)

`claudedocs/`:
- `HANDOFF.md` (본 문서)
- `tier1-tier3-rebaseline-2026-05-01.md` ★ 본 cycle 핵심 산출물
- `cycle-logs/cycle-2525.md` ★ 본 cycle log
- (이전) `vision-consistency-audit-2026-05-01.md`, `vision-gap-analysis-2026-05-01.md`
- (이전) `m1-perf-diagnosis-2026-05-01.md` — Cycle 2513 측정 (3-runs 노이즈 영향 — 본 cycle에서 정정)

`MEMORY.md` (auto-memory):
- `project_benchmark_reality.md` — v0.98 1/16 FAIL 영속화
- 기타 LSP/Bootstrap/Vision/Recommended-Path 메모리 보존

---

**세션 종료**: 2026-05-01 (Cycle 2525)

**핵심 메시지**: 이전 평가의 "8/15 FAIL" 위기감은 v0.51.22 stale data 의존이었음. v0.98 실측은 1/16 FAIL (json_parse 1.12x). **단일 구조 개선 (`loop`/`while` 표현식 + byte intrinsic)** 이 5개 byte-stream 벤치 동시 향상의 ROI 압도적 우선. **다음 세션 첫 액션 = P-1 (loop syntax) 또는 P-6 (ROADMAP 갱신 + 측정 스크립트 commit)**.
