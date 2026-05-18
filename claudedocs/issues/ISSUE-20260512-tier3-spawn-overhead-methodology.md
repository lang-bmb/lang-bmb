# ISSUE-20260512 — Tier 3 measurement methodology — wall-time dominated by OS process spawn overhead

## 핵심 메타

**우선순위**: P2 (P-track measurement integrity)
**영역**: ci / benchmarks
**상태**: Open — Option B 확정 (2026-05-18, Cycle 2914), Phase 1 대기

## 측정 stamp

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-05-12 (Cycle 2752) |
| `stale_after` | 2026-08-12 (3개월) |
| `measurement_source` | Python `perf_counter_ns` × 10-run direct exec (Cycle 2752 진단) |
| `observed_rate` | Tier 3 실제 workload **~7ms**, framework 측정값 **30-130ms** (spawn overhead 80-95% 비중) |
| `scope` | 모든 Tier 3 real_world 벤치마크 (`scripts/benchmark.sh` `run_tier_3` 경로) |
| `env_hash` | win32 / Python 3.12.10 / bash subshell / clang-built C exes |

**측정 추이**:

| date | source | observed | 변화 |
|------|--------|----------|------|
| 2026-05-12 | Cycle 2752 direct measurement | 7-8ms workload vs 30-130ms framework | (신규 진단) |
| 2026-05-12 | Cycle 2765 amplification 시도 (lexer/brainfuck) | Option A 한계 노출 — 아래 § "Cycle 2765 empirical findings" | (현실성 검증) |

## Cycle 2765 empirical findings

POC 시도 결과 Option A 추정이 비현실적임이 확인됨. 실증 데이터:

### (a) lexer outer-loop amplification (BMB)

| 시도 | BMB 측정 | C 측정 | 진단 |
|------|---------|--------|------|
| 100× outer loop, plain | 6.2ms (orig: 6.5ms) | 21.3ms (orig: 5.5ms) | BMB 측 loop 완전 DCE (LLVM CSE/purity inference) |
| 100× outer loop + `bmb_black_box` on return | 8.2ms (+1.7ms) | 21.3ms | BMB `bmb_black_box` return-only로는 hoist 차단 불충분 |
| 1000× input scaling, single call | 6.8ms (+0.3ms) | 12.9ms (+7.4ms) | C 측 효과; BMB는 **count_tokens 0-token 버그** (별도) |

**핵심**: BMB의 LLVM optimizer는 `bmb_black_box(pure_fn(invariant_input))` 패턴에서 pure_fn을 invariant로 인식하여 hoist함. Tier 1 inproc 패턴이 작동했던 이유는 `bmb_black_box(input)`을 매 iter마다 호출하여 input invariance 자체를 막았기 때문.

### (b) brainfuck outer-loop amplification

| 시도 | BMB 측정 | C 측정 |
|------|---------|--------|
| 9 → 99 (10×) | 6→7.6ms (+1.6ms) | 6→6.9ms (+0.9ms) |

brainfuck `interpret()`는 heap mutation을 동반하여 DCE 회피되나 inner work 자체가 <0.1ms/iter → 10× 증가가 workload dominance 달성 불충분. 100× 또는 1000× 필요.

### (c) BMB 측 추가 issue (carry-forward)

**lexer count_tokens 0-token 버그** (pre-existing, Cycle 2765에서 발견):
- small source: BMB가 모든 token 카운트 0 출력, C는 정상 20 idents / 12 keywords 출력
- large source: BMB total tokens 0, C total 8900-89000
- 측정 fairness 영향 — BMB 측이 "0 work" 상태이므로 framework Tier 3 lexer 1.000x 측정이 부적절 fair comparison

### (d) 새 권고: Option A는 inproc 패턴과 결합 필요

Option A (workload amplification) 단독으로는 BMB CSE 회피 불가. **Option B (inproc port)와 결합**이 실제 작동:
- per-iter `bmb_black_box(input_seed)` 적용
- `time_ns()` 직접 측정 (framework wall-time 의존 회피)

→ Tier 1 inproc 패턴 그대로 Tier 3로 포팅하는 것이 정합

## 갱신된 권고 (Cycle 2765)

## 문제

`scripts/benchmark.sh` 의 `run_benchmark` → `time_cmd` 가 측정하는 wall-time은 다음 합:
1. Bash subshell fork
2. exe spawn
3. stdout/stderr redirection setup
4. **실제 workload** (Tier 3 short benches: 6-8ms)
5. Process teardown

Tier 3 단위 워크로드 (lexer/json_parse/brainfuck/csv_parse/http_parse/json_serialize/sorting) 측정 시:
- **실제 algorithmic time ≈ 7ms** (직접 Python subprocess timing 검증)
- **Framework 보고값 ≈ 28-135ms** (시스템 부하에 따라)
- **80-95%가 OS spawn overhead**

이로 인해:
- **Tier 3 절대 ms 값에 진단적 의미 부여 시 false positive 가능** (예: Cycle 2750 c2729 lexer 1.310x → Cycle 2751 c2751 1.000x 변화의 absolute 차이 28→41ms는 workload 아닌 spawn overhead 노이즈)
- **BMB:C ratio는 fair (양쪽 동일 spawn overhead 부담)** → 1.000x verdict는 정확하나 절대 값은 무의미
- **5-run vs 10-run 차이는 noise-floor 평탄화 효과만 — 본질적 측정 정확도 향상 아님**

## 핵심 증거

### 직접 측정 (Cycle 2752, 10 consecutive runs per bench)

| bench | BMB direct min | C direct min | ratio direct | framework c2729 | framework c2751 |
|-------|---------------|-------------|--------------|-----------------|-----------------|
| lexer | 7ms | 7ms | 1.000 | bmb=28 c=28 | bmb=41 c=41 |
| http_parse | 6ms | 6ms | 1.000 | bmb=45 c=47 | bmb=135 c=137 |
| brainfuck | 6ms | 6ms | 1.000 | bmb=42 c=45 | bmb=42 c=133 |

→ direct 측정에서 일관 7ms parity. framework는 ~5-20× amplification.

### bench source 확인 (예: lexer)

`ecosystem/benchmark-bmb/benches/real_world/lexer/c/main.c:151-159`:
- 100KB source string (100x replication of ~1KB seed)
- single-pass tokenization
- Output: 9 token type counts + printf

Algorithmic time: ~1-3ms.
Process startup: ~3-5ms.
Total binary wall time: 6-8ms.

→ 측정값 30-130ms는 이 합과 일치하지 않음. spawn fork overhead가 추가.

## 추정 root cause

**Confirmed (high confidence)**: bash `$(...)` 명령 substitution + subshell fork + exe spawn 의 합산 overhead가 Tier 3 short workload time보다 큼. Tier 1 (workload 100ms+) 에서는 비중이 작아 영향 미미.

## 영향 평가

| 영역 | 영향 |
|------|------|
| CI | ✅ ratio_c 자체는 fair (BMB와 C 모두 동일 overhead) — false negative/positive 가능성은 absolute 값 사용 시만 |
| 부트스트랩 | ✅ 영향 없음 |
| **사이클 분석** | ⚠️ Cycle 2750 처럼 absolute ms 변화로 회귀 판단 시 false alarm 가능 (5-10 cycles 헛수고 직전 회피한 사례) |
| **M축 (P)** | Tier 1 ≤1.05x 16/16 PASS는 유효 (workload 시간이 충분히 큼). Tier 3는 ratio만 유효, absolute는 의미 부여 회피 |

## 해결 방안 (옵션 비교)

### Option A: workload amplification (현 bench source 직접 변경) — **수정 (Cycle 2765)**
- ~~scope: 1-2 cycles~~ → **실측: 5-10 cycles (각 bench마다 DCE 회피 패턴 다름)**
- 절차: 각 Tier 3 bench의 `large_source` replication factor 100 → 1000 또는 main loop 추가 N-iter
- ~~효과: workload 7ms → ~70ms 또는 ~700ms, spawn overhead 비중 90% → 10%~~
- 실측 효과 (Cycle 2765):
  - lexer: 1000× input scaling으로 C 측 +7ms (효과 있음). BMB는 별도 0-token 버그로 측정 불가
  - brainfuck: 10× outer loop으로 BMB +1.6ms / C +0.9ms (효과 작음, inner work <0.1ms/iter)
- 트레이드오프: bench source 변경 + bench 별 패턴 분석 필요
- **권고 수정**: Option B와 결합 필수 (BMB CSE 회피 위해)

### Option B: inproc timing port (Tier 1 inproc 패턴 Tier 3에)
- scope: 5-10 cycles
- 절차: Cycle 2661 Tier 1 inproc (`time_ns()` + `bmb_black_box` harness) 패턴을 Tier 3 7 benches에 포팅
- 효과: framework wall-time 의존성 완전 제거, in-process timing
- 트레이드오프: BMB+C 양쪽 모두 main() 변경 필요, 큰 작업
- **권고**: 장기 옳은 방향, 단 multi-cycle phase

### Option C: framework run_benchmark `effective_workload_ratio` 보고
- scope: 1 cycle
- 절차: warmup time에서 spawn overhead 측정 (e.g., empty workload exe) → ratio 분리 보고
- 효과: 동일한 wall-time을 받지만 "workload < spawn" 케이스 explicit warning 출력
- 트레이드오프: 측정 정확도 향상 아님, awareness만
- **권고**: Option A 보완 가능

## 결정 (2026-05-18, Cycle 2914)

**선택: Option B — inproc timing port**

근거:
- Option C는 awareness 개선뿐 — 측정 정확도 향상 없음 (CLAUDE.md Principle 2: Workaround 금지)
- Option A 단독은 BMB LLVM DCE 회피 불가 (Cycle 2765 실증)
- Option B는 Tier 1 inproc 패턴(`time_ns()` + `bmb_black_box`)을 그대로 Tier 3에 포팅 — framework wall-time 의존 완전 제거
- 작업량이 크다는 것은 하지 않을 이유가 아님 (CLAUDE.md Principle 3)
- 성능 주장은 측정으로 증명 (CLAUDE.md Verification Principle)

**Phase 실행 계획** (각 Phase = 1-2 cycles):

| Phase | 대상 벤치마크 | 우선도 |
|-------|------------|--------|
| 1 | `lexer`, `brainfuck` | 최우선 (DCE 문제 실증된 케이스) |
| 2 | `csv_parse`, `http_parse` | 2순위 |
| 3 | `json_parse`, `json_serialize` | 3순위 |
| 4 | `sorting` | 4순위 (BMB faster 이미 확인, 노이즈 제거로 신뢰도 향상) |

각 Phase 진입 시:
1. BMB + C 양쪽 main() → `time_ns()` 직접 측정 harness로 교체
2. `bmb_black_box(input_seed)` per-iter 적용 (DCE 회피)
3. 기존 wall-time 측정값 archive (`measurements/tier3_legacy_*.json`)
4. 새 inproc 측정값으로 ROADMAP P-track row 갱신

## 종결 기준

- [ ] Tier 3 측정값 absolute time이 workload-dominated (overhead < 20%) 확인
- [ ] Tier 3 noise rate measurement (run-to-run variance) < 5%
- [ ] CI tier_all default 측정 신뢰도 회복 (5-run으로 충분)
- [ ] 골든 자동 검사 또는 ad-hoc cycle에서 false-positive 회귀 alert 0건

## 메타

- 관련 ISSUE:
  - `ISSUE-20260511-or-chain-lowering.md` (Cycle 2750 false-positive 회귀 후보의 root cause 이 spawn overhead)
  - Cycle 2661 (Tier 1 inproc harness 패턴, Option B 참조)
- 인용 cycle: cycle-2752.md (root cause 발견), cycle-2750.md/cycle-2751.md (sequencing)
- 외부 참조: `scripts/benchmark.sh:141-182` (`time_cmd` + `run_benchmark`)
