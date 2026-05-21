# BMB Session Handoff — 2026-05-21 (Cycles 3017-3026 — P-track 7/7 전부 BMB faster)

> **HEAD**: `72ddb300` (Cycles 3017-3026 — P-track 전체 최적화: brainfuck+csv+http 대폭 개선)
> **3-Stage Fixed Point**: ✅ IR Fixed Point 확인 (Cycle 2930)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: Cycle 3027

---

## 이번 세션 작업 요약 (Cycles 3017-3026)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 3017 | P-track 측정 + 최적화 기회 탐색 | 7/7 PASS, brainfuck 1.037× borderline, memset_fill 기회 발견 |
| 3018 | memset_fill 빌트인 추가 | runtime.c + 4개 Rust 소스, brainfuck 1.037×→0.974× |
| 3019 | P-track 전체 재측정 + ROADMAP 갱신 | 7/7 PASS 확인, ROADMAP §5 갱신 |
| 3020 | brainfuck match dispatch 최적화 | if-else chain → match (switch IR) + byte_at 직접 접근, 0.958× |
| 3021 | brainfuck band 255 branchless | % 256 → band 255, if v==0 제거, 1.037×→**0.956×** (누적) |
| 3022 | csv_parse single-load 최적화 | and-chain 이중 load → break-based, 1.018×→**0.891×** |
| 3023 | http_parse single-load 최적화 + 전체 재측정 | 4곳 이중-load 제거, 0.938×→**0.909×**. 7/7 전부 BMB faster |
| 3024 | MIR CSE ISSUE 등록 + 탐색 | ISSUE-20260521-mir-cse-and-chain.md (P2) |
| 3025 | lexer 알고리즘 분석 + P-track 검증 | lexer 0.169× = 알고리즘 우위 확인 |
| 3026 | HANDOFF 갱신 + 최종 commit | (이 문서) |

### 핵심 결과

| 항목 | 세션 시작 | 세션 종료 |
|------|---------|---------|
| brainfuck | 1.037× ⚠️ | **0.956×** ✅ BMB faster |
| csv_parse | 1.018× ⚠️ | **0.891×** ✅ BMB faster |
| http_parse | 0.938× ✅ | **0.909×** ✅ 개선 |
| lexer | 0.175× ✅ | **0.169×** ✅ 안정 |
| json_parse | 0.815× ✅ | **0.822×** ✅ 안정 |
| json_serialize | 0.701× ✅ | **0.668×** ✅ 안정 |
| sorting | 0.154× ✅ | **0.154×** ✅ 안정 |

**P-track 7/7: 세션 전후 모두 PASS. 3개 벤치마크 대폭 개선 (brainfuck -8.1pp, csv -12.7pp, http -2.9pp).**

### 신규 빌트인: memset_fill(ptr, val, count) -> i64

| 파일 | 변경 내용 |
|------|---------|
| `bmb/runtime/bmb_runtime.c` | `bmb_memset` + `memset_fill` alias 추가 |
| `bmb/src/types/mod.rs` | 타입 등록 `[I64, I64, I64] -> I64` |
| `bmb/src/codegen/llvm_text.rs` | IR 선언 `nocallback nounwind` |
| `bmb/src/mir/mod.rs` | 반환 타입 등록 |
| `bmb/src/codegen/llvm.rs` | inkwell backend 선언 (Rule 7 패리티) |

### 최적화 패턴 (재사용 가능)

1. **memset_fill 단일 alloc 패턴**: calloc×N → calloc×1 + memset_fill×N (per-iter 할당 제거)
2. **match dispatch 패턴**: chained if-else → `match c { 62 => ..., _ => 0 }` (LLVM switch)
3. **band 255 wrapping**: `(v + 1) % 256` → `(v + 1) band 255` (and i64 255, 분기 제거)
4. **single-load break 패턴**: `while _ and load_u8(ptr+p) != X and load_u8(ptr+p) != Y` → break-based 단일 load

### 신규 ISSUE

| ISSUE | 우선순위 | 요약 |
|-------|---------|------|
| ISSUE-20260521-mir-cse-and-chain | P2 | BMB `and/or` 체인 내 동일 load_u8 CSE 최적화 |

---

## 다음 세션 (Cycle 3027+)

### 권장 우선순위

1. **MIR CSE 개선** (P2) — ISSUE-20260521-mir-cse-and-chain: `and/or` 체인 내 동일 subexpression 자동 CSE → 사용자가 break-based 패턴 강제 없이도 자동 최적화
2. **M4 채택 지표** — GitHub stars, 외부 PR, 외부 프로젝트 추적 (HUMAN-blocked 잔여)
3. **B축 Claude 재측정** — 98.0% stale 기한 2026-08-13 (아직 여유)
4. **Tier 1 벤치마크** — binary_trees/fasta/mandelbrot 등 최신 측정 확인

### 알려진 HUMAN-blocked 항목

- GPT-4o 실험 (multi-model-validation)
- golden-flakiness-inttoptr Option A/B/C
- problem-difficulty-bias 신규 hard 문제 20개
- crosslang 측정 (stale)

### ISSUE 현황 (Active 6개)

| ISSUE | 상태 | 우선순위 |
|-------|------|---------|
| **mir-cse-and-chain** | OPEN | **P2** (신규) |
| multi-model-validation | PARTIALLY RESOLVED | MEDIUM |
| external-problem-validation | PARTIALLY RESOLVED | MEDIUM |
| integration-category-weakness | PARTIALLY RESOLVED | LOW |
| problem-difficulty-bias | OPEN | LOW |
| golden-flakiness-inttoptr | OPEN | P3 |

### 알려진 BMB 언어 특성 (중요도 순)

- `else if` 체인 세미콜론: statement 위치에서 `};` 필수 (Cycle 2984 발견)
- `fn main() -> i64 = { ... };` 끝에 `;` 필수 (Cycle 2986 발견)
- `match`: integer literal, char literal, OR pattern (`a | b => ...`) 지원
- `match` arm body: block + comma 필요 (`{ expr }` 후 `,` 필수, 마지막 arm 제외)
- `band`/`bor`/`bxor`/`bnot`: bitwise 연산자 지원
- `break`/`continue`/`return`: ✅ 지원 (단, break는 while에서만)
- `&&`/`||` short-circuit: ✅ 완전 지원 (Cycle 2965)
- `memset_fill(ptr, val, count)`: ✅ native-only builtin (v0.100.1 신규)

### B-axis 상태

| 모델 | 마지막 측정 | 상태 |
|------|-----------|------|
| Claude (claude-sonnet-4-6) | 98.0% (2026-05-13) | 고정 베이스라인 (stale 기한: 2026-08-13) |
| GPUStack qwen3.6-35b-a3b | **100.0% (2026-05-21)** | **최신 공식 측정** |
