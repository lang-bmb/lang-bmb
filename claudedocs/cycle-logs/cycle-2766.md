# Cycle 2766: HashMap 진단 — root cause 식별

Date: 2026-05-12

## Re-plan

진입 — cycle 2765에서 Phase A 축소 + HashMap 한 칸 앞당김. Trigger ⚪ NONE. Scope = HashMap profiling.

## Scope & Implementation

### Step 1: stdlib vs bench 분리

`stdlib/core/hashmap.bmb` 는 thin wrapper (`hashmap_*` C 런타임 intrinsic). 그러나 `ecosystem/benchmark-bmb/benches/compute/hash_table/bmb/main.bmb` 는 **자체 구현** (`hash_i64`, `hm_insert/get/remove`). bench는 BMB 언어 성능 자체 (memory access patterns, branch, fn call) 측정.

### Step 2: 알고리즘 동등성 확인

| 항목 | BMB | C |
|------|-----|---|
| Hash function | `(key * 0x517cc1b727220a95) ^ (h >> 32)` | 동일 |
| Probing | Open addressing + linear probing | 동일 |
| Capacity | 131072 (2^17) | 동일 |
| Entry layout | `*i64` 3-word `[key, value, state]` | `Entry { key, value, state }` |
| Bench iterations | 30 | 30 |

→ **알고리즘 동일**. 갭은 codegen 품질에서.

### Step 3: IR-level diagnosis (smoking gun)

`bmb build main.bmb --emit-ir` 결과:
```
@hm_insert  ...  inlinehint  nosync ...
@hm_get     ...  noinline    nosync ... memory(read) ...
@hm_remove  ...  inlinehint  nosync ...
```

**`hm_get` 만 `noinline`**. `benchmark_lookup` hot loop에서 매 iteration 호출 → call overhead 매번 발생. `hm_insert/remove` 는 `inlinehint` → LLVM 자동 inline.

### Step 4: 원인 코드 식별

`bmb/src/mir/optimize.rs:6874` `should_no_inline_for_licm`:
- v0.99 Cycle 2532 추가 (`v0.99 (Cycle 2532): noinline pass to enable LICM on read-only functions called from loops`)
- 조건: read_only + ≥10 MIR insts + in-loop callee
- 목적: json_parse `count_array`/`validate_json` invariant-call hoist (1.12x → 1.0x)

**부작용**: `hm_get`도 read-only + ≥10 insts + in-loop → noinline 적용. 하지만 `benchmark_lookup`은 매 iter `key` 인자 변동 → LICM hoist 불가 → 순수 overhead.

### Step 5: Rule 6 / 7 conflict

- Rust `optimize.rs` 에 pass 존재
- `bootstrap/compiler.bmb` 에 **동일 pass 없음** (`grep noinline bootstrap/compiler.bmb` → 0)
- Rule 7 parity 위반 가능성 — Rust vs bootstrap 컴파일 결과 IR 차이
- Rule 6 (Rust frozen): 새 기능은 bootstrap. 이 pass는 Rust 잔존, bootstrap port 필요

### Step 6: ISSUE-hashmap-perf 갱신

`claudedocs/issues/ISSUE-20260413-hashmap-perf.md` 에 Cycle 2766 diagnosis 추가 (4-level analysis + Rule 6 conflict + 다음 cycle 측정 분기).

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| `cargo test --release` | ✅ 변경 없음 (문서/IR 분석만) |
| IR 분석 정합 (3 fn 각 inline attribute 확인) | ✅ `--emit-ir` 출력으로 확인 |
| BMB ↔ C 알고리즘 등가성 | ✅ 양쪽 main.c, main.bmb 직접 비교 |
| 원인 코드 식별 (optimize.rs:6874) | ✅ |
| Rule 6/7 영향 평가 | ✅ 부트스트랩 grep 확인 |

**Defects**: 없음 (진단 cycle).
**Diagnosis findings**: ISSUE 갱신 + 다음 cycle 측정 분기 정의 (carry-forward).

## Reflection

### advisor leverage

Cycle 중반 advisor 호출이 결정적. expectation ("1.04x → 0.95x")이 **근거 없는 추정**임을 지적 + 분기 ① 측정 우선 권고. 추가 발견:
- 1.04x 갭 자체가 **measurement noise** 가능성 (1.027 → 1.040 = +1.2pp, Cycle 2725 vs 2750 noise 범위 안)
- bootstrap 측정으로 진단 검증 가능 (cheapest verification path)
- Phase B' 4-cycle budget이 비현실적 (현실 5-7 cycles per advisor)

### 외부 관점 — 6 dimensions

1. **Scope fit**: 적합. cycle 2766은 진단 cycle, root cause 식별까지 도달.
2. **Latent defects**: Rust vs bootstrap parity 위반 (Rule 7) — 별도 이슈 후보. 하지만 Rule 6 (Rust frozen)와의 충돌 자체가 메타-issue.
3. **Structural improvement opportunities**:
   - `claudedocs/issues/_template.md` 에 "cycle estimate은 검증 전까지 가설" 메타 필드 추가 (advisor 메타 권고, 두 cycle 연속 같은 패턴 발견)
   - bootstrap에 smarter no_inline pass 구현 시 call-site argument invariance 검사 추가 (compiler engineering 깊이 작업)
4. **Philosophy drift**: 없음. workaround (bench-level `@inline`) 거부 + 부트스트랩 proper fix 경로 선택은 Principle 2 정합.
5. **Roadmap impact**:
   - cycle-logs `ROADMAP.md` Phase B' 4-cycle → 실제 분기 ① 측정 (1 cycle) → 분기 ② 5-7 cycles 또는 carry-forward
   - 또 한 번의 ISSUE estimate 갭 → 패턴화 (cycle 2765, 2766 연속)
6. **User-facing quality**: N/A.

### 진단 결과 요약 (one-liner)

`hm_get` 만 v0.99 Cycle 2532 `MemoryEffectAnalysis` pass 부착 `noinline`을 받아 hot loop call overhead 발생. `benchmark_lookup`의 key 인자 변동 → LICM hoist 불가 → noinline이 의도된 LICM 이득 없이 순수 손실.

## Carry-Forward

### Actionable (다음 cycle)

- **Cycle 2767**: bootstrap-built hash_table 측정 (`bootstrap/compiler.exe`로 빌드) + phase split 측정 (lookup-only / insert-only / delete-only). 다음 분기 결정:
  - bootstrap 우월 + lookup hot path → 분기 ② (bootstrap에 smarter no_inline pass, 5-7 cycles, Phase B' 잔여 cycles 모두 소비)
  - 갭 작거나 lookup non-hot → carry-forward + 다른 P-track ISSUE 전환

### Structural Improvement Proposals

- **bootstrap port of `should_no_inline_for_licm` 패턴 + 강화 (call-site argument variance check)**: Rule 6 정합. 5-7 cycles. 핵심: invariant args만 noinline 적용 → hash_table 영향 없이 json_parse 이득 유지.
- **`_template.md` 에 cycle estimate 가설 필드 추가**: 메타-improvement. 1 cycle. cycle 2765 + 2766 연속 ISSUE 추정 갭 패턴이 신호.
- **bench output verification CI** (cycle 2765 carry-forward 유지): BMB ↔ C 출력 diff. 1-2 cycles.

### Pending Human Decisions

- M3-3/M3-4 publish dispatch (HUMAN, 이전 세션부터 누적)
- M4-1 BMB_BENCH_API_KEY (HUMAN, 이전 세션부터 누적)
- **신규 (Rule 6 conflict)**: Rust 잔존 pass (`should_no_inline_for_licm`) 을 bootstrap port 시점 + scope. Rule 6 vs Rule 7 충돌 — HUMAN 정책 결정.

### Roadmap Revisions

cycle-logs `ROADMAP.md`:
- Phase B' 4-cycle → 1 cycle 진단 (cycle 2766) + 1 cycle 측정 (cycle 2767) + 결과 분기:
  - 분기 ②: 5-7 cycles bootstrap port — Phase B' + or-chain (Phase C') 합산 budget 초과, or-chain skip
  - carry-forward path: 분기 ② 외 또는 가설 거부

### Next Recommendation

**Cycle 2767**: bootstrap-built hash_table 측정 (분기 ① per advisor). 절차:
1. `bootstrap/compiler.exe` 로 hash_table/bmb/main.bmb 빌드
2. 직접 측정 (Python perf_counter_ns × 10 runs)
3. Phase split: insert-only / lookup-only / delete-only 분리 측정 (각 단계 단독 main 만들기 또는 시간 측정 추가)
4. 결과로 분기 ② 진입 여부 결정

## Files

| 변경 | 위치 | 추적 |
|------|------|------|
| ISSUE-hashmap-perf 갱신 (Diagnosis 1-4 + Rule 6 conflict) | `claudedocs/issues/ISSUE-20260413-hashmap-perf.md` | tracked |
| 본 cycle log | `claudedocs/cycle-logs/cycle-2766.md` | gitignored |
