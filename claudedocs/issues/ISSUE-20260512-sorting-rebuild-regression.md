# ISSUE-20260512 — Sorting bench rebuild regression (~500× slowdown)

## 핵심 메타

**우선순위**: **P1** (실측 ratio 측정 차단, M1 P-track 신뢰도 위협)
**영역**: codegen / opt pipeline / bmb/src/codegen/llvm_text.rs (or .rs)
**상태**: Open — 진단 cycle (Cycle 2770)
**estimated_cycles**: **3-7** (hypothesis — bisect + fix + bootstrap verify)

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

## 추정 root cause

**Hypothesis A — Cycle 2532 noinline pass**: read-only + ≥10 insts + in-loop → noinline 부착. sorting의 `partition`/`quick_sort_helper`/`array_new` 등이 영향?

**Hypothesis B — opt -O2 transformation regression**: 다른 codegen 변경 (text/inkwell parity, readonly enum 등)이 sorting 패턴 trigger.

**Hypothesis C — 다른 cycle 변경**: e.g., M5-1~M5-5g enum/string 인프라가 부작용.

미진단: git bisect로 회귀 commit 식별 필요.

## 영향 평가

| 영역 | 영향 |
|------|------|
| **M1 ≤1.05x 16/16 가설** | 🚨 sorting Tier 3 측정 차단 — 16/17 인지 16/17 인지 |
| Tier 3 measurement integrity | 🚨 sorting absolute time 측정 비교 불가 |
| 다른 bench | ⚠️ 다른 bench도 회귀 가능성 (cycle 2750 c2729 lexer 1.310 → c2751 1.000 변화는 c-side anomaly로 추정했으나 BMB-side rebuild 회귀일 가능성) |
| 부트스트랩 | ✅ 영향 없음 (compiler.bmb 별도 컴파일러) |

## 해결 방안 (Decision Framework)

### Option A: bisect + fix (proper)
- `estimated_cycles`: 3-7 **(hypothesis — verify via 진단 cycle)**
- 절차:
  1. git log on bmb/src/codegen/ + bmb/src/mir/ 사이 Feb-May commits
  2. bisect: 각 commit으로 sorting build + time
  3. 회귀 commit 식별
  4. fix 또는 노이즈 추가
- 리스크: Rule 6 (Rust frozen) 충돌. 새 코드 추가 금지, 회귀 fix 부분만 허용 가능?
- 검증: 회귀 commit 식별 → revert vs 새 fix vs settings change

### Option B: bench source workaround (immediate)
- `estimated_cycles`: 1
- 절차: sorting bench source에 `@noinline` 또는 `@inline` hint 추가 → 옛 IR 패턴 회복
- 트레이드오프: workaround (Principle 2). 진짜 root cause 미해결.

### Option C: bootstrap-built sorting (전환)
- `estimated_cycles`: 0 (이미 동작 가능?)
- 절차: framework가 bootstrap/compiler.exe로 sorting bench 빌드
- 트레이드오프: 다른 bench도 같이 전환해야 fair. bootstrap의 stack overflow 문제 (cycle 2767 발견) 검증 필요.

## HUMAN 결정 필요

- **Option A vs B vs C 선택**
- bisect 시작 시점 (이번 세션 budget 부족, 다음 세션 또는 분리 phase)
- Rule 6 충돌 (Rust 회귀 fix이 "부트스트래핑 차단"에 해당하는가?) — 측정 차단이지만 부트스트랩은 영향 없음

## 종결 기준

- [ ] sorting bench rebuild 결과가 Feb 9 build 와 동등 (234ms ≈ ratio, output 일치)
- [ ] `verify_bench_outputs.py --tier 3` 결과 sorting PASS
- [ ] CI에서 sorting 회귀 자동 감지

## 메타

- 관련 ISSUE:
  - `ISSUE-20260512-bench-output-fairness-survey.md` (parent)
- 인용 cycle: cycle-2769.md (발견), cycle-2770.md (진단)
- 외부 참조: `ecosystem/benchmark-bmb/benches/real_world/sorting/bmb/main.bmb`
