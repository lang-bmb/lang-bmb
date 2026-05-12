# ISSUE-20260512 — Sorting bench rebuild regression (~500× slowdown)

## 핵심 메타

**우선순위**: **P0** (UB — `i64 undef` 생성은 정의되지 않은 동작)
**영역**: `bmb/src/codegen/llvm_text.rs` — LLVM attribute 방출 (`willreturn`, `mustprogress`)
**상태**: Open — **IR-level root cause 확인 (Cycle 2782)**, fix 보류 (HANDOFF "진단 only")
**estimated_cycles**: **1-2** (fix: attribute 방출 조건 수정 + cargo test + sorting verify)

## 측정 stamp

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-05-12 (Cycle 2770) |
| `stale_after` | 2026-08-12 (3개월) |
| `measurement_source` | direct `./main.exe` (Feb 9 build) vs `./main_verify.exe` (May 12 rebuild) |
| `observed_rate` | **500× slowdown** (234ms → >120s) + **70% size bloat** (204KB → 350KB) + **no stdout output** |
| `scope` | `ecosystem/benchmark-bmb/benches/real_world/sorting/bmb` 단독 (다른 bench는 정상 rebuild) |
| `env_hash` | win32 / LLVM 21.1.8 / MSYS2 UCRT64 / target/release/bmb.exe build May 12 |

## 문제

`ecosystem/benchmark-bmb/benches/real_world/sorting/bmb/main.bmb` 재빌드 시 심각한 회귀:

| 측면 | main.exe (Feb 9, 동일 source) | main_verify.exe (May 12, 동일 source) |
|------|------------------------------|---------------------------------------|
| 빌드 명령 | `bmb build main.bmb` | `bmb build main.bmb -o main_verify.exe` |
| 파일 크기 | 204 KB | 350 KB (+71%) |
| Wall time | 234ms ✅ | **>120s** ❌ (timeout 안 끝남) |
| stdout | `403905348` ✅ | (empty) |
| Sections | 18 PE32+ | 19 PE32+ |

소스 코드 동일 (git untracked 변경 없음). **Rust compiler 변경이 회귀 원인**.

## 핵심 증거

### 재현 절차

```bash
cd ecosystem/benchmark-bmb/benches/real_world/sorting/bmb
# 옛 빌드 - 정상
./main.exe              # 234ms, "403905348"

# 신 빌드 - 회귀
rm -f main_verify.exe
export BMB_RUNTIME_PATH=d:/data/lang-bmb/bmb/runtime
target/release/bmb.exe build main.bmb -o main_verify.exe
./main_verify.exe       # >120s, no output
```

### Reproducibility

3 attempts (subprocess timeout=120): 100% reproducible hang.
Direct shell `./main_verify.exe`: > 120s wall, ~0.1s user CPU → IO bound or sleeping.

## 확인된 Root Cause (Cycle 2782 진단)

### IR 증거

**opt -O2 최적화 후** (`/tmp/sorting_opt.ll` 기준):

```llvm
; partition.exit 블록에서 — quick_sort_helper 재귀 호출 #1
%_t9.call = tail call fastcc i64 @quick_sort_helper(ptr nonnull %arr, i64 %low, i64 undef)
br label %bb_then_0
```

LLVM의 **Tail Call Elimination (TCE)** + **Dead Argument Elimination (DAE)** 상호작용:

1. `quick_sort_helper` 함수의 두 번째 재귀 호출 `quick_sort_helper(arr, pivot+1, high)` 이
   TCE 변환으로 loop-back이 됨 (`br label %bb_then_0`)
2. 첫 번째 재귀 호출 `quick_sort_helper(arr, low, pivot-1)` 은 tail position 아님 →
   DAE가 이 call-site의 `high` 인자를 "loop에서 결코 사용 안 됨"으로 판단 → **`undef` 대입**
3. 결과: `partition_exit` 블록에서 `quick_sort_helper(arr, low, undef)` 호출 → UB

### 구조적 버그: `bb_then_0`에 `%low` phi 없음

TCE 변환된 loop에서:
- `bb_then_0` (loop header): `%low`에 대한 **phi node 없음** — 항상 원본 함수 파라미터 사용
- TCE로 변환된 경로(`pivot+1`이 `%low` 역할 해야 함)에서 `%low`가 갱신되지 않음
- `%_t0_ptr.i = getelementptr inbounds i64, ptr %arr, i64 %high` — loop invariant로 preheader 호이스팅됨

이 구조적 결함이 DAE에게 `high` 인자를 `undef`로 대체할 빌미를 제공.

### 기각된 Hypothesis

**`willreturn mustprogress` 제거 (Cycle 2782 테스트)**:
- `llvm_text.rs`에서 recursive 함수에 한해 `willreturn mustprogress` 미방출하도록 변경
- `cargo build --release` 성공, sorting.exe 재빌드
- **결과: 여전히 hang** — `opt -O2` IR에서 `i64 undef` 계속 출현
- 결론: `willreturn`/`mustprogress`가 원인이 아님. 변경 revert 완료.

### 실제 fix 방향

`quick_sort_helper` 에 `willreturn`/`mustprogress`가 방출되는 자체는 문제가 아님.
문제는 LLVM이 TCE+DAE 후 `undef`를 삽입하는 것 — 이는 pre-opt IR의 **잘못된 phi 구조**가
DAE에게 정보를 주기 때문.

**Option 1 (codegen fix)**: `quick_sort_helper` 의 두 재귀 호출이 모두 다른 basic block에 있어
TCE loop 변환 후 phi가 올바르게 생성되도록 IR 방출 패턴 수정 (복잡도 높음)

**Option 2 (attribute 수정)**: `willreturn` / `mustprogress` 를 재귀 함수에서 제거 (이미 기각)

**Option 3 (pragmatic)**: `@noinline` 을 `quick_sort_helper`/`partition`에 부착하거나
opt pass flag (`-disable-tail-calls`) 사용 — Principle 2 위반 (workaround)

**추천 fix**: pre-opt IR을 수정해 TCE 후 phi가 올바르게 생성되도록 함 (Option 1).
구체적으로: `quick_sort_helper`의 두 재귀 call이 각각 독립 BB를 가지도록 IR 방출 구조 변경.

### 구 Hypothesis (무효화됨)

~~Hypothesis A — Cycle 2532 noinline pass~~: 기각.  
~~Hypothesis B — opt -O2 transformation regression~~: 부분 맞음 (TCE+DAE) — 단 codegen 문제가 아닌 IR 방출 구조 문제.  
~~Hypothesis C — 다른 cycle 변경~~: 기각.

## 영향 평가

| 영역 | 영향 |
|------|------|
| **M1 ≤1.05x 16/16 가설** | 🚨 sorting Tier 3 측정 차단 — 16/17 인지 16/17 인지 |
| Tier 3 measurement integrity | 🚨 sorting absolute time 측정 비교 불가 |
| 다른 bench | ⚠️ 다른 bench도 회귀 가능성 (cycle 2750 c2729 lexer 1.310 → c2751 1.000 변화는 c-side anomaly로 추정했으나 BMB-side rebuild 회귀일 가능성) |
| 부트스트랩 | ✅ 영향 없음 (compiler.bmb 별도 컴파일러) |

## 해결 방안 (Decision Framework)

### Option A: pre-opt IR 방출 구조 수정 (proper fix)
- `estimated_cycles`: 1-2
- 파일: `bmb/src/codegen/llvm_text.rs` (text backend) + `bmb/src/codegen/llvm.rs` (inkwell backend)
- 내용: `quick_sort_helper`처럼 두 개의 재귀 호출이 있는 함수에서 IR이 TCE+DAE 후
  phi가 올바르게 생성되도록 재귀 call 분기 구조 수정
- 검증: sorting bench rebuild → 234ms 복원 + `403905348` 출력 + cargo test --release

### Option B: `@noinline` 힌트 (workaround — 금지)
- Principle 2 명시 위반. 문서화만. 시행 불가.

### Option C: bootstrap-built sorting (전환)
- `estimated_cycles`: 0 (이미 동작 가능?)
- 트레이드오프: bootstrap의 stack overflow 문제(hash_table) 미해결 상태 — fair 비교 불가.

## HUMAN 결정 필요

- **Option A 착수 승인**: Rule 6 충돌 여부 — sorting regression이 "P0 correctness bug (UB)"에
  해당하므로 CLAUDE.md Rule 6 P0 예외 조항 적용 가능. 단, 사람의 명시 승인 후 진행 권장
- **우선순위 조정**: D2' fix가 다음 autonomous cycle에 포함되어야 하는가?

## 종결 기준

- [ ] sorting bench rebuild 결과가 Feb 9 build 와 동등 (234ms ≈ ratio, output 일치)
- [ ] `verify_bench_outputs.py --tier 3` 결과 sorting PASS
- [ ] CI에서 sorting 회귀 자동 감지

## 메타

- 관련 ISSUE:
  - `ISSUE-20260512-bench-output-fairness-survey.md` (parent)
- 인용 cycle: cycle-2769.md (발견), cycle-2770.md (진단), cycle-2782.md (IR-level root cause 확인)
- 외부 참조: `ecosystem/benchmark-bmb/benches/real_world/sorting/bmb/main.bmb`
