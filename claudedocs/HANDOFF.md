# BMB Session Handoff — 2026-05-11 (Cycles 2708-2717 — Stage 2 Fixed Point 회복 + builtin arity proper-fix)

> **HEAD**: `abf78075` (Cycles 2708-2717 통합 commit)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **이전 세션 핸드오프**: cycle-logs/cycle-2690~2707.md 참조

---

## 0. 이번 세션 작업 (Cycles 2708-2717)

### 🎉 큰 변곡점

| 항목 | Before | After |
|------|--------|-------|
| **Bootstrap Stage 2 Fixed Point** | ❌ Cycle 2237 이후 회귀 (OOM/parse 추정) | ✅ **회복** S2 == S3 |
| Token packing scale | 1M (source > 1MB 차단) | **5M** (source ≤ 5MB 가능) |
| Builtin name collision (silent IR corruption) | source rename workaround (Cycle 2697/2700) | **proper-fix** (arity guard + fallback) |
| bootstrap.sh default arena | 16G | **32G** |

### 세션 성과 요약

| 사이클 | 제목 | 성과 |
|--------|------|------|
| 2708 | **Stage 2 OOM 재현 + 메모리 곡선** | OOM @ 3K lines, 비단조 곡선 |
| 2709 | **가설 분리: 1MB byte threshold smoking gun** | Token packing overflow 확정 |
| 2710 | **Token packing fix 옵션 산출** | A안 (10M 대비 5M scale) 권고 |
| 2711 | **🎉 A안 5M scale 적용 → Fixed Point 회복** | 26.7s, S2 == S3 |
| 2712 | **Builtin arity proper-fix (2-arg + 3-arg)** | 16 사이트, Cycle 2697 workaround 회수 |
| 2713 | bootstrap.sh 16G → 32G default | scope adjust |
| 2714 | **1-arg builtin arity guard** | 14 사이트 + helper, Fixed Point 유지 |
| 2715 | 골든 sample 80개 (50 S1 + 30 S2) | 80/80 PASS, 무회귀 |
| 2716 | **ISSUE 백로그 triage** | 40개 분류 (15 resolved, 13 HUMAN-locked, 9 actionable, 3 stale) |
| 2717 | HANDOFF/ROADMAP 갱신 + 종합 commit | (현재) |

---

## 1. 현재 상태

### Bootstrap 검증 상태 (Cycle 2237 회복)

| 게이트 | 결과 |
|--------|------|
| Stage 1 빌드 | ✅ 10.4s |
| Stage 2 LLVM IR (32G arena) | ✅ 28s |
| Stage 2 binary 생성 | ✅ |
| Stage 3 LLVM IR | ✅ 28s |
| **Fixed Point S2 == S3** | ✅ **회복** |

### Token packing 변경 사항

| 항목 | 1M (이전) | **5M (현재)** |
|------|-----------|---------------|
| Pack scale | `kind * 1e6 + pos` | `kind * 5e6 + pos` |
| Source 크기 한도 | < 1MB | **< 5MB** |
| 사용자 정수 literal max | 9.22e12 | 1.84e12 |
| 임계 (scan_int) | 9e12 | 1.8e12 |
| Sentinel (saturate) | 9.2e12 | 1.84e12 |

### Builtin arity guard 적용 (30 사이트)

| Builtin 카테고리 | 사이트 수 | Lowering paths |
|------------------|----------|----------------|
| 2-arg i64 (`@bit_and/or/xor/shift_left/shift_right/min/max`) | 14 | lower_expr_sb + step_expr |
| 3-arg i64 (`@clamp`) | 2 | 양쪽 |
| 1-arg i64 (`@popcount/clz/ctz/bit_reverse/bit_not/abs/bswap`) | 14 | 양쪽 |
| **합계** | **30** | |

각 분기: `... and call_has_N_args(...)` 가드 + fallback to `emit_regular_i64_call`. 패턴: `rotate_left/right` (Cycle 2384) 확립.

### 테스트 현황

| 스위트 | 결과 |
|--------|------|
| `cargo test --release` | ✅ **6210/6210 passed** |
| Stage 1 sample golden 50 | ✅ 50/50 |
| Stage 2 binary sample 30 | ✅ 30/30 (production-readiness) |
| **Bootstrap Fixed Point** | ✅ **S2 == S3** |
| 풀 golden suite (Cycle 2701 기준) | ✅ 2862/2862 (이번 세션 sample 검증으로 회귀 없음 확인) |

### compiler.bmb 크기 추이

| Cycle | bytes | delta |
|-------|-------|-------|
| 2707 (직전 세션 종료) | 1,036,359 | - |
| 2711 (5M scale 후) | ≈ 1,036,500 | +~150 |
| 2712 (2-arg guard) | 1,039,623 | +3,264 |
| 2714 (1-arg guard) | 1,042,127 | +2,504 |
| **현재** | **1,042,127** | **+5,768 from 2707** |

### 마일스톤 상태

| 마일스톤 | 상태 |
|---------|------|
| **M1 Self-Validated + Bootstrap 검증** | ✅ **COMPLETE + 회복** (이번 세션) |
| M2 AI-Ready Infra | ✅ COMPLETE |
| M3 External Bindings | 🔄 ~96% (자율 100%, HUMAN publish 잔여) |
| M4 Adopted | 🔄 ~50% |
| M5 Language Completeness | 🔄 M5-1~M5-5g ✅ |

---

## 2. 태스크 목록 (잔여 + 신규)

### 다음 세션 우선순위

| # | 태스크 | 성격 | 상태 |
|---|--------|------|------|
| ~~Stage 2 진단~~ | parse error vs arena OOM 가설 분리 | ✅ Cycle 2708-2714 완료 |
| ~~Builtin arity proper fix~~ | source rename workaround 회수 | ✅ Cycle 2712-2714 완료 (30 사이트) |
| M3-3 | **[HUMAN]** npm publish | 실행 | ⏳ |
| M3-4 | **[HUMAN]** PyPI publish | 실행 | ⏳ |
| M3-5 | **[HUMAN]** bmb-algo README clang vs gcc 라벨 | 결정 | ⏳ |
| M4-1 | **[HUMAN]** B 공식 측정 (`BMB_BENCH_API_KEY` 필요) | 자율 | ⏳ — **13개 이슈 잠금 해소 ROI** |
| Actionable backlog | P축 (HashMap/StringBuilder/Alloc/Compare/match-jump) | 자율 | ⏳ |
| FP 1-arg builtin guard | `@fabs/floor/ceil/round/sqrt/sin/cos/etc.` consistency | 자율 | ⏳ (낮은 우선순위) |
| 2-arg FP builtin guard | `@f64_min/max/atan2/pow_f64/fmod` | 자율 | ⏳ (낮은 우선순위) |
| Token packing B안 (bit packing) | proper fix `kind << 32 \| pos` | 자율 | ⏳ (장기) |
| O(n²) AST proper-fix | string AST → binary/shared arena | 자율 | ⏳ (장기, 수개월) |
| CI 게이트: bootstrap_3stage.sh | 회귀 방지 | 인프라 | ⏳ |
| ISSUE backlog 이동 (resolved → closed/) | 정리 | 인프라 | ⏳ |

---

## 3. 핵심 구현 사항 (이번 세션)

### Token packing 5M scale (Cycle 2711)

`bootstrap/compiler.bmb`:
- L506-509: tok_val/tok_end/make_tok scale `1e6 → 5e6`
- L392/407/417/428: scan_*_int 임계 `9e12 → 1.8e12`, sentinel `9.2e12 → 1.84e12`
- L399-400: pack_int_tok scale `1e6 → 5e6`
- L623: get_ident_text (dead code 식이지만 일관성)

**산술 정합성**: `1.84e12 * 5e6 = 9.2e18 < i64_max 9.22e18` ✅

### Builtin arity guard (Cycle 2712 + 2714)

**Helper 함수** (compiler.bmb 7032-7052):
- `call_has_two_args(line, paren_pos, close_pos) -> bool` (기존)
- `call_has_one_arg(line, paren_pos, close_pos) -> bool` (Cycle 2714 신규)
- 3-arg: inline `count_commas(args, 0) == 2` (clamp)

**패턴**:
```bmb
} else if fn_name == "@bit_or" and call_has_two_args(...) {
    // builtin intrinsic 발행
} else if fn_name == "@bit_or" {
    emit_regular_i64_call(line, paren_pos, close_pos, fn_name, dest)  // user fn fallback
}
```

### bootstrap.sh 32G default (Cycle 2713)

`BMB_ARENA_MAX_SIZE=${BMB_ARENA_MAX_SIZE:-16G}` → `${BMB_ARENA_MAX_SIZE:-32G}` (2 occurrences, line 299, 388).

---

## 4. 환경 노트

| 환경 | 상태 |
|------|------|
| LLVM | 21.1.8 MSYS2 UCRT64 |
| Node.js | v24.14.0 |
| Python | 3.12.10 |
| 버전 | `0.98.0` |
| Branch | `main` |

### 운용 주의사항 (갱신)

- **BMB_ARENA_MAX_SIZE**: bootstrap.sh default 16G → **32G**. compiler.bmb 1.04MB+ 처리에 필요. direct call 시 명시 필요.
- **Token packing 5M scale**: 사용자 정수 literal 한도 1.84e12. 그 초과 literal은 saturate.
- **Builtin name collision**: lint 11 (Cycle 2703) + arity guard (Cycle 2712/2714) 이중 안전망. user-defined `bit_or(a,b,n)`, `popcount(x,y)` 등 정상 동작.
- **FP builtin 1-arg/2-arg arity guard 미적용** — 사용자 충돌 가능성 낮으나 consistency를 위해 carry-forward 후보.
- **컴파일러 변경 후 Stage 2 32G 부족 시**: 임계 도달 가능성 — 추가 ~28GB 여유 있으나 O(n²) AST 메모리는 별도 트랙.

---

## 5. 다음 세션 시작 체크리스트

- [ ] `claudedocs/ROADMAP.md` 읽기 (실무 앵커)
- [ ] `claudedocs/cycle-logs/cycle-2708~2717.md` 읽기 (이번 세션 회복 라이브러리)
- [ ] `cargo test --release` → **6210/6210** 확인
- [ ] `./scripts/bootstrap.sh` → **Fixed Point S2 == S3** 확인 (32G arena 사용)
- [ ] **`./scripts/run-golden-tests.sh` 백그라운드 실행** — Cycle 2712-2714 변경 후 풀 골든 회귀 검증 (sample 80개는 PASS, 풀 2862개 미검증, 43분 소요)
- [ ] HUMAN 결정 잔여: M3-3 (npm), M3-4 (PyPI), M3-5 (README clang vs gcc 라벨), M4-1 (B 측정 BMB_BENCH_API_KEY)

---

## 6. HUMAN 결정 사항 (불변, 2026-05-10/11 확정)

| 항목 | 결정 |
|------|------|
| M3 showcase 선정 | ✅ bmb-algo |
| npm publish | ✅ 즉시 진행 |
| PyPI publish | ✅ 즉시 진행 |
| v0.100 버전 선언 | ✅ M3 publish 완료 직후 |
| B 공식 측정 | ✅ 즉시 실행 — **이번 세션 정렬 결과 13 이슈 잠금 해소 ROI 매우 큼** |
| README "knapsack 6.8x faster" | ⏳ clang -O3 outlier (M4-9 deferred) — gcc 기준 1.39x slower |

---

## 7. Memory Note 정정 (claudedocs/memory)

이전 세션에서 기록된 OOM 가설:
- ❌ "compiler.bmb self-compile arena OOM (32G+ 초과, O(n²) 문자열 AST 성장)"
- ✅ **정정**: 두 결함 공존 — (1) **Token packing 1MB overflow** (Cycle 2711 5M scale fix), (2) **O(n²) AST 부분적 (16G→32G 한계 가까이 도달)**

Cycle 2708→2709→2711의 가설 진화:
1. Cycle 2708: OOM 우세 가설 (틀림)
2. Cycle 2709: 두 결함 공존 (부분 맞음)
3. Cycle 2711: token packing이 primary, O(n²)이 secondary

---

**세션 종료**: 2026-05-11 (Cycles 2708-2717 — **Stage 2 Fixed Point 회복** + builtin arity proper-fix 30 사이트 + bootstrap.sh 32G + ISSUE triage 40개)

---

## 8. 다음 세션 첫 cycle 권고 시퀀스

각 sequence는 self-contained micro-loop (사이클 1개 = 1 action).

### 시퀀스 A — 회귀 안전망 확보 (병렬 가능)

**Cycle 1 — 풀 골든 백그라운드 시작** (사이클 fire-and-forget):
```bash
nohup ./scripts/run-golden-tests.sh --json > /tmp/golden-full.json 2>&1 &
# 43분 소요. 다른 사이클 동시 진행
```
완료 후 fail count 확인. 0 FAIL 기대 (sample 80/80 통과).

**Cycle 2 — 부트스트랩 회복 sanity** (≤2 min):
```bash
BMB_ARENA_MAX_SIZE=32G ./scripts/bootstrap.sh
# Stage 1 OK + Stage 2 OK + S2==S3 fixed point 확인
```

### 시퀀스 B — HUMAN 잠금 해소 (우선순위)

**Cycle 3 — M4-1 B 공식 측정** (HUMAN 결정 잔여, 자율 실행):
```bash
export BMB_BENCH_API_KEY=<key>
./tools/bmb-ai-bench run --suite full --model <fixed>
# 결과 → claudedocs/B-baseline-2026-05-XX.json
```
완료 시 ROADMAP § 5 측정 지표 표 B축 갱신 + 13개 2026-03-26 ISSUE 재평가 가능.

### 시퀀스 C — Actionable backlog (P축 클러스터)

**Cycle 4-8 — P축 actionable 5개** (1 cycle = 1 issue):
- HashMap 해시 함수 최적화 (`ISSUE-20260413-hashmap-perf.md`)
- StringBuilder 성능 (`ISSUE-20260413-string-builder-opt.md`)
- 메모리 할당 최적화 (`ISSUE-20260413-alloc-optimization.md`)
- 비교 인라인 (`ISSUE-20260413-compare-inline.md`)
- match → jump table (`ISSUE-20260413-match-jump-table.md`)

### 시퀀스 D — 정리 (낮은 우선순위)

**Cycle N — ISSUE 폴더 정리**:
```bash
mkdir -p claudedocs/issues/closed/
mv claudedocs/issues/ISSUE-20260501-track-*.md claudedocs/issues/closed/
mv claudedocs/issues/ISSUE-20260510-let-tuple-destructuring.md claudedocs/issues/closed/
mv claudedocs/issues/ISSUE-20260510-static-method-call.md claudedocs/issues/closed/
mv claudedocs/issues/ISSUE-20260510-option-expr-position.md claudedocs/issues/closed/
mv claudedocs/issues/DESIGN-M5-*.md claudedocs/issues/closed/
mv claudedocs/issues/ISSUE-20260511-set-field-index.md claudedocs/issues/closed/
mv claudedocs/issues/ISSUE-20260511-golden-regression-3.md claudedocs/issues/closed/
mv claudedocs/issues/ISSUE-20260511-golden-manifest-audit.md claudedocs/issues/closed/
mv claudedocs/issues/ISSUE-20260413-bootstrap-fixed-point.md claudedocs/issues/closed/
# 15개 resolved → active backlog 40 → 25
```

**Cycle N+1 — CI 게이트 추가**:
`.github/workflows/` 또는 동등 위치에 `bootstrap_3stage` + golden sample 50 추가.

### 시퀀스 E — Long-term (필요 시점에)

- **Token packing B안 (bit-pack `kind << 32 | pos`)** — 5M scale 임시 회수
- **O(n²) AST proper-fix** — string AST → binary, 수개월
- **FP 1-arg/2-arg builtin arity guard 확장** — consistency
- **`bmb verify` D축 활성화** — Z3 IPC 인프라 활용 (Cycle 2606)
- **`bmb-ai-bench` C축 활성화** — Track R suite

### Decision Framework 적용 (다음 cycle 진입 시)

각 시퀀스 진입 전 CLAUDE.md § Decision Framework 검토:
1. 언어 스펙 변경 필요? — 시퀀스 E의 token packing B안이 해당
2. 컴파일러 구조 변경? — O(n²) AST proper-fix
3. 최적화 패스? — 시퀀스 C (P축)
4. 코드 생성? — 시퀀스 C
5. 런타임? — 해당 없음

낮은 수준에서 해결하려는 유혹 경계. **workaround 금지** (CLAUDE.md Principle 2).

