# ISSUE-20260512 — Sorting bench rebuild regression (~500× slowdown)

## 핵심 메타

**우선순위**: ~~P0~~ **CLOSED**
**영역**: `bmb/src/codegen/llvm_text.rs` — `MkTuple` 핸들러 missing store
**상태**: **RESOLVED (Cycle 2783)** — sorting bench 복원: 203ms + `403905348` ✅
**estimated_cycles**: 1 (실제 소요)

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

## 확인된 Root Cause (Cycle 2783 확정)

### 실제 원인: `MkTuple` 핸들러 missing store

`llvm_text.rs`의 `MkTuple` 명령어 처리에서 `insertvalue` 체인 이후 alloca에 store가 빠짐.

**수정 전 pre-opt IR (partition 함수 반환부)**:
```llvm
%_t19_0 = insertvalue { i64, i64 } undef, i64 %_t19_tuple_elem0, 0
%_t19   = insertvalue { i64, i64 } %_t19_0, i64 %_t19_tuple_elem1, 1
; ← store 없음!
%_t19_ret_load = load { i64, i64 }, ptr %_t19.addr   ; uninitialized alloca load → undef
ret { i64, i64 } %_t19_ret_load
```

**수정 후 pre-opt IR**:
```llvm
%_t19_0 = insertvalue { i64, i64 } undef, i64 %_t19_tuple_elem0, 0
%_t19   = insertvalue { i64, i64 } %_t19_0, i64 %_t19_tuple_elem1, 1
store { i64, i64 } %_t19, ptr %_t19.addr          ; ← fix: 올바른 값 저장
%_t19_ret_load = load { i64, i64 }, ptr %_t19.addr
ret { i64, i64 } %_t19_ret_load
```

### undef 전파 경로 (확정)

1. `partition` 함수: `%_t19.addr` alloca에 store 없음 → load → `undef` 반환
2. `quick_sort_helper` 인라인 후 (LLVM inline pass): `%_t19_ret_load.i` = load from `%_t19.addr.i` (store 없음)
3. SROA: store가 없으므로 alloca를 스칼라 SSA로 교체 → load를 `undef`로 대체
4. instcombine: `undef`가 `pi` 계산으로 전파 → `pi-1 = undef`, `pi+1 = undef`
5. 재귀 호출: `quick_sort_helper(arr, low, undef)` → 무한 루프/크래시

### 기각된 Hypothesis

**TCE+DAE 상호작용 (Cycle 2782)**: 부분 맞음 (최종 증상이 그것). 그러나 원인은 codegen missing store.  
**`willreturn mustprogress` 제거 (Cycle 2782)**: 기각 — undef 계속 출현.  
**`notail` prefix (Cycle 2783)**: 기각 — TCE 막지만 SROA+instcombine undef 전파는 막지 못함.

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

## ~~HUMAN 결정 필요~~

- ~~Option A 착수 승인~~: **완료** (Rule 6 P0 예외 적용, Cycle 2783)

## 종결 기준

- [x] sorting bench rebuild 결과가 Feb 9 build 와 동등 (`203ms` vs `234ms`, output `403905348` ✅)
- [ ] `verify_bench_outputs.py --tier 3` 결과 sorting PASS (다음 세션)
- [ ] CI에서 sorting 회귀 자동 감지 (다음 세션)

## Fix

- **파일**: `bmb/src/codegen/llvm_text.rs` — `MkTuple` 핸들러 (Cycle 2783)
- **변경**: `insertvalue` 체인 이후 `store {struct_type} %{dest_name}, ptr %{dest.name}.addr` 추가
  (단, `local_names`에 `dest.name`이 포함된 경우)
- **검증**: `cargo test --release` 전통과 + `main_fix.exe` → `403905348` / 203ms

## 메타

- 관련 ISSUE:
  - `ISSUE-20260512-bench-output-fairness-survey.md` (parent)
- 인용 cycle: cycle-2769.md (발견), cycle-2770.md (진단), cycle-2782.md (IR-level 증거), cycle-2783.md (fix)
- 외부 참조: `ecosystem/benchmark-bmb/benches/real_world/sorting/bmb/main.bmb`
