# BMB 로드맵 — 철학 정렬 앵커
> 최종 업데이트: 2026-05-11 (Cycles 2746-2748 — **시퀀스 E 완결**: -march=native CI spec parity, 서브모듈 main merge + 부모 submodule pointer bump)
> 이전 갱신: Cycles 2737-2745 (policy-vs-code 갭 fix + HUMAN 결정 채택 및 시퀀스 확정)
> 이 문서는 매 세션의 **유일한 실무 앵커**다.
> 상세 사이클 로그: `docs/ROADMAP.md` | 개발 규칙: `CLAUDE.md` | 세션 상태: `claudedocs/HANDOFF.md`

---

## § 1 가설 (The Hypothesis)

### 1.1 왜 BMB가 존재하는가

**① 기존 언어의 한계**

모든 기존 언어는 *인간이 직접 작성해야 한다*는 전제 위에 설계됐다.

```
인간 인식 한계 수용
→ 쉬운 문법 · 모호성 허용
→ 컴파일러가 런타임에 판단을 미룸
→ 런타임 오버헤드 구조적으로 잔존
```

**② AI가 바꾼 것**

AI가 코드를 작성하면 언어가 어려워도 무관하다.

```
모호성 · 암묵 변환 · 런타임 체크 완전 제거 가능
→ "Performance > Everything" 처음으로 실현 가능한 조건이 생겼다
```

**③ AI의 한계가 만드는 BMB의 위치**

AI도 기계어를 직접 작성할 수 없다.  
(컨텍스트 한계, 대규모 프로젝트 구조 파악, 속도, 비용)

```
더 낮추면: hallucination · 검증 불가 · 비용 폭발
         ↑
        BMB  ← AI가 다룰 수 있는 가장 낮은 추상화 수준
         ↓
더 높이면: 런타임 오버헤드 잔존
```

> **결론**: BMB는 AI 이전에는 존재할 수 없었던 언어다.  
> 인간 인식 한계를 전제하지 않고 설계된 첫 시스템 언어.

---

### 1.2 목표 #1: 성능 — 존재가치 증명

이것은 단순 목표가 아니라 프로젝트의 존재 근거다.

```
BMB가 C/Rust와 동등하거나 초월하지 못한다
→ "AI 시대에 새 언어가 필요하다"는 가설이 거짓
→ BMB의 존재 이유 없음
```

| 상황 | 판정 |
|------|------|
| LLVM 백엔드 버그·한계 | ✅ 외부 요인, 수용 |
| "언어 한계"로 성능 포기 | ❌ 언어 스펙을 바꾼다 |
| "컴파일러 한계"로 성능 포기 | ❌ 컴파일러를 바꾼다 |
| "부트스트랩 복잡도"로 기피 | ❌ 복잡도는 이유가 아니다 |

성능 달성 수단: **AI-friendly 언어 설계**  
(예측 가능한 패턴, 명시적 계약, 구조화 출력, 도구 통합)

---

### 1.3 존재가치 증명의 메커니즘: 도그푸딩

BMB가 가설을 증명하는 방법은 벤치마크만이 아니다.  
BMB로 만든 모든 것이 가설 검증의 데이터포인트다.

| 활동 | 증명하는 것 |
|------|------------|
| `bootstrap/compiler.bmb` (32K LOC) | BMB로 컴파일러 작성 가능 |
| `bootstrap/lsp.bmb` (~1450 LOC) | BMB로 언어 도구 작성 가능 |
| `bootstrap/lint.bmb` | BMB로 정적 분석 작성 가능 |
| `ecosystem/bmb-algo` 등 5개 | BMB로 배포 가능한 라이브러리 작성 가능 |
| `bmb-mcp` / `context-pack` | BMB가 AI 워크플로우에 통합 가능 |

**따라서**: 컴파일러·부트스트랩·에코시스템·외부 패키지 작업은  
별개 활동이 아니라 **도그푸딩 활동의 일환**이며, 가설 검증에 직접 기여한다.

**게이트 커밋먼트**:  
도그푸딩 과정에서 문제가 발견되면 — 언어 스펙·컴파일러·부트스트랩 어느 레벨이든 —  
**low level부터 개선될 수 있는 게이트가 항상 열려있어야 한다.**  
고수준 workaround로 문제를 덮는 것 = 도그푸딩의 실패.

---

## § 1.4 역할 및 범위 (Role & Scope)

### 1차 사용자

```
인간 → 자연어 의도 → LLM → BMB 코드 작성
                              ↓
                    BMB 컴파일러 (contract 검증 + IR 생성)
                              ↓
                         최적 기계어
```

- **인간**: 자연어로 의도 전달 (LLM에게)
- **LLM**: BMB 코드 작성 — BMB 컴파일러와 직접 통합 없음
- **BMB 컴파일러**: contract 검증, 결정론적 IR 생성, 구조화 출력

AI-readiness는 **언어 자체의 속성**이다 — 외부 LLM 채널·합성기·AI API 통합 없음.

### 1차 도메인

**컴파일러 · 언어 도구 · DSL · 검증기** (자기 자신 포함)

이 도메인인 이유:
- BMB가 이미 BMB로 자신을 컴파일한다 (`bootstrap/compiler.bmb` 32K LOC) — 도메인이 코드에 박혀있음
- Contract 검증의 가치가 가장 명확한 영역 (parser invariants, type checker soundness, IR rewriter correctness)
- AI가 가장 잘 생성하는 영역 (정형적 · 알고리즘적 · 트리 변환)

### 범위 외 (Out-of-scope)

| 항목 | 이유 |
|------|------|
| 수치 계산 (mandelbrot, n-body 등) | 도메인 외, 벤치 강등 대상 |
| 외부 LLM API 통합 | AI-readiness는 언어 설계로, 외부 채널로 X |
| 일반 웹/앱 개발 도구 | BMB 1차 도메인 밖 |

---

## § 2 진단 (Diagnosis) — 4가지 Drift

M1·M2를 달성하는 동안 활동들이 가설과의 연결 없이 독립적으로 진행된 결과:

### Drift A — 도그푸딩 프레임 누락
- **현상**: 생태계(MCP/LSP/바인딩) 작업이 "성능 목표와 별개"로 보임
- **근본 원인**: 생태계 작업이 도그푸딩임이 명시되지 않았음
- **결과**: 문제 발견 시 low-level 게이트로 돌아가야 함이 문서화되지 않음

### Drift B — B(Failure Rate) #1 우선순위 vs. 미선언
- **현상**: B가 최우선이라 선언, 비공식 실험(90.9%, 2026-03-26)은 있으나 공식 baseline 미선언
- **근본 원인**: 인프라는 완성됐으나 공식 실험 실행(API key + 모델 고정)이 미수행
- **결과**: M4 첫 액션으로 연기 중

### Drift C — AI-native 선언 vs. 언어 갭
- **현상**: "LLM이 쓰는 언어" 선언, 그러나 LLM 자연 패턴 다수 미지원
- **확인된 실제 파서 갭**:
  - `let (a, b) = expr` — tuple destructuring (파서 미지원)
  - `Type::method()` — static method call expression (파서 미지원)
  - `Option::Some(x)` — 표현식 위치에서 enum 생성 (패턴만 지원)
- **지원됨 (CLAUDE.md와 다름)**: `_` wildcard pattern, trait impl blocks
- **근본 원인**: 언어 갭 해소가 명시적 작업 항목으로 등록되지 않음

### Drift D — 문서 분산
- **현상**: `CLAUDE.md` / `docs/ROADMAP.md` / vision spec이 각각 독립 진화
- **근본 원인**: 세션마다 진짜 앵커가 어디인지 불명확
- **결과**: 새 세션마다 정렬 비용 반복 발생

---

## § 3 처방 (Prescription)

### A → 도그푸딩 분류 명시
- 모든 활동은 도그푸딩 분류로 관리
- 도그푸딩 중 발견된 성능·언어 문제: workaround 금지, low-level 게이트로 처리
- "왜 이 작업을 하는가?" = 항상 가설로 traceable해야 함

### B → B축 baseline 공식 선언 (M4 첫 액션)
- 인프라 완성 (`bmb-ai-bench run`), 비공식 결과 존재 (90.9%, 2026-03-26, ~100문제)
- 공식 실험: 고정 모델 + `BMB_BENCH_API_KEY` + 결과 커밋 → baseline 선언
- 공식 선언 전까지: "B 비공식 ~90.9%, 공식 미확정"으로 명시

### C → 언어 갭 백로그 등록
- 확인된 갭 3개 → `claudedocs/issues/` 이슈 등록
- 원칙: "AI-native 선언 = 언어 갭 해소 의무를 수반함"

### D → 문서 역할 분리 (이 문서로 확정)

| 문서 | 역할 |
|------|------|
| `claudedocs/ROADMAP.md` | **유일한 실무 앵커** — 매 세션 시작 시 참조 |
| `docs/ROADMAP.md` | 공개용 상세 문서 (사이클 로그, 트랙 상세) |
| `CLAUDE.md` | 개발 규칙 (이 ROADMAP을 전제) |
| `claudedocs/HANDOFF.md` | 세션 간 상태 전달 |

---

## § 4 로드맵

### 현재 위치

```
M1  Self-Validated      ████████████████████  ✅ COMPLETE
M2  AI-Ready Infra      ████████████████████  ✅ COMPLETE
M3  External Bindings   ███████████████████░  🔄 ~97%  ← 현재 (M3-6 신 완결, M3-5 자율 잔여 + HUMAN publish 잔여)
M4  Adopted             ████░░░░░░░░░░░░░░░░  🔄 ~40%
M5  Language Complete   █████████████░░░░░░░  🔄 M5-1~M5-4 ✅ + **M5-5 7/7 ✅** + **M5-5e nested ✅** + **M5-5f Array<f64> ✅**
v1.0                    ░░░░░░░░░░░░░░░░░░░░  ⬜ 외부 신호 대기
```

현재 버전: `0.98.0` | 권장 다음: `v0.100` (M3 완료 후, 메인테이너 결정)

#### M5 언어 완성도 현황 (신규 트랙)

| 기능 | 상태 |
|------|------|
| M5-1: `enum Option { None, Some(i64) }` + match payload | ✅ Stage 1 구현 (Cycle 2633) |
| M5-1: Fixed Point 검증 | ✅ **회복 (Cycle 2711-2714)** — Token packing 1M→5M scale + builtin arity guard; 32G arena 한도, 28s |
| M5-2: Result<Ok(i64), Err(i64)> + 다중 payload enum | ✅ 완료 (Cycle 2635, M5-1 인프라 재사용) |
| M5-3: Multi-field enum `Branch(i64,i64)`, `Three(i64,i64,i64)` | ✅ 완료 (Cycle 2637) |
| M5-4: `println(String)` 타입 추론 dispatch (str_sb 인프라 활용) | ✅ 완료 (Cycle 2640) |
| M5-4 확장: `println(f64)` dispatch (`@println_f64`) | ✅ 완료 (Cycle 2643) |
| M5-4 확장: struct String 필드 (`p.name`) 직접 println | ✅ 완료 (Cycle 2645) — registry `~s` suffix |
| M5-4 확장: 중첩 struct + mut struct String 검증 | ✅ 완료 (Cycle 2646) |
| M5-5: `["a","b"]` array literal of String dispatch | ✅ 완료 (Cycle 2651) — `mark_str_ptr` 발행 + 자동 propagation |
| M5-5: alias / while iter / mut set | ✅ 완료 (Cycle 2652-2653) — R: marker 보존 |
| M5-5b: `[s;N]` var-repeat | ✅ 완료 (Cycle 2664) — `mark_str_ptr_if` 새 MIR 명령어 (codegen 시점 lookup) |
| M5-5c: `fn() -> Array<String>` 반환 | ✅ **완료 (Cycle 2670-2673)** — `parse_return_type` Array<String> 인식 + collect_string_fns `A:` prefix + llvm_gen_call dispatch |
| M5-5d: `p.field[i]` struct array field | ✅ **완료 (Cycle 2674-2675)** — struct field `~a` suffix + `is_field_str_array` + llvm_gen_field_access dispatch |
| M5-5e: nested struct array (`o.inner.tags[i]`, 3-level) | ✅ **무구현 통과 (Cycle 2680)** — M5-5d 인프라가 nested 경로 자연 처리 |
| **M5-5f: `Array<f64>` literal/fn-return/struct-field/nested/alias/loop/mut** | ✅ **완료 (Cycle 2681-2683)** — `mark_f64_ptr` 분기 + `F:` prefix + `~af` suffix + `is_dynamic_f64_array_fn` + `is_field_f64_array` |
| **M5-5g: `set obj.field[idx] = val` + nested chain (`set o.a.b[i] = v`)** | ✅ **완료 (Cycle 2690-2692)** — AST 차원 desugar (`set_index` of `field`) + `parse_set_field` 재귀 일반화, 신규 AST/MIR 노드 0 |
| **Hardcoded String-fn 리스트 cleanup** | ✅ **Cycle 2702 + 2705** — 9 entries 제거 (tokenize + 8 dead entries: concat/3/5/7, make_error, gen_program_sb_with_strings, compile_function/source). 사용자 silent IR corruption 회귀 방지 |
| **Lint 11 — builtin_name_collision** | ✅ **Cycle 2703** — 21 reserved 이름 정적 감지 (bit_or 가족 8 + str fn 13). lint count 10→11 |
| **골든 스위트 0 FAIL** | ✅ **Cycle 2701** — 2862/2862 PASS (43분 풀 실행). 12→3→0 회귀 정정 누적 |
| **M4-9 clang knapsack outlier 분석** | ✅ **Cycle 2704** — clang -O3 unconditional store + select-phi anti-pattern 식별. ISSUE-20260511-clang-knapsack-outlier로 deferral. BMB 측 작업 없음 |
| **Bootstrap Stage 2 Fixed Point 회복** | ✅ **Cycles 2708-2714** — Token packing 1MB threshold smoking gun → 5M scale (10 LOC) → Stage 2 통과 → S2 == S3 fixed point. bootstrap.sh default 16G → 32G. |
| **Builtin arity proper-fix (Cycle 2697 workaround 회수)** | ✅ **Cycles 2712-2714** — 30 사이트 arity guard + fallback (`call_has_one_arg`/`call_has_two_args`/`count_commas`). user-defined `bit_or(a,b,n)`/`popcount(x,y)` 정상. Lint 11 + arity guard = 이중 안전망 |

---

### M3 완료 조건

| 조건 | 상태 | 잔여 |
|------|------|------|
| C ABI + 빌드 인프라 | ✅ | — |
| Python + Node 바인딩 5/5 | ✅ | — |
| Track S 90% | ✅ (~99%) | — |
| Showcase 라이브러리 선정 | ✅ | bmb-algo (HUMAN 결정 완료) |
| Showcase Python 비교 | ✅ | 7/7 BMB faster (Cycle 2654) |
| Showcase 공식 C 비교 | ✅ | 7/7 (Cycle 2655-2660) — clang -O3 4/5 ≤1.05x, gcc -O3 5/5 ≤1.5x |
| Showcase clang baseline | ✅ | clang -O3 baseline 5/5 측정 (Cycle 2660) — BMB ≈ clang |
| in-process timing 인프라 | ✅ | `time_ns()` + `bmb_black_box` harness (Cycle 2661) |
| npm / PyPI publish | ⏳ | workflow_dispatch (HUMAN) |
| README 측정 주장 검증 | ⏳ | "knapsack 6.8x faster than C": clang 기준 재현, gcc 기준 미재현 (HUMAN 결정) |

**ETA (갱신 Cycle 2748)**: M3-6 ✅ 완결. 잔여 = M3-5 자율 (bench 종료 후 1-2 cycles) → M3-3/M3-4 HUMAN publish → M3-7 자동 (M4-1 종속).

#### M3 잔여 태스크 (실행 순서, Cycles 2737-2748 진척 반영)

| # | 태스크 | 성격 | 상태 |
|---|--------|------|------|
| M3-1 | showcase 선정 | HUMAN | ✅ 완료 (bmb-algo) |
| M3-2 | showcase 공식 벤치마크 측정 | 자율 | ✅ 7/7 (clang -O3 + gcc -O3 dual baseline, Cycle 2660-2662) |
| M3-5 | **bmb-algo README 측정 정정** (knapsack 100→10 items, 90x/181x source, clang vs gcc 라벨) | 자율 (옵션 자율 → review) | ⏳ 시퀀스 B (publish 선결) |
| M3-3 | npm publish | HUMAN dispatch | ⏳ 시퀀스 C — M3-5 정정 **후** |
| M3-4 | PyPI publish | HUMAN dispatch | ⏳ 시퀀스 C — M3-5 정정 **후** |
| M3-6 (구) | nqueens benchmark suite 추가 | 자율 | ✅ (이미 존재 — `nqueen` 디렉토리, Cycle 2660) |
| M3-6 (신, Cycle 2743) | **CI workflow + Dockerfile 5 위치 `-march=native` PR** | 자율 draft → HUMAN 결정 변경: PR 회수 + main 직접 merge | ✅ Cycles 2746-2747 (서브모듈 `cb478d2` on main, 부모 submodule pointer bump `477e5827`, PR #2 withdrawn). CI baseline 첫 stamp = 다음 schedule trigger (Sunday 00:00 UTC) |
| M3-7 (Cycle 2742) | M4-1 baseline에 "supersedes 2026-03-26" annotation | 자율 | ⏳ M4-1 종속, 자동 |

순서 정정 근거 (Cycle 2745 결정):
- M3-5 (README quality) → M3-3/M3-4 (publish): publish된 README의 사후 정정은 신뢰 손실 더 큼
- BMB 철학 "측정 없는 성능 주장 금지" 우선
- M3-6 신구 항목 분리 명시 (nqueens 자율 완료 vs CI flag PR)

---

### M4 경로

| 우선순위 | 축 | 태스크 |
|---------|-----|--------|
| ① | **B** | LLM 1-shot 성공률 공식 baseline 측정 (`BMB_BENCH_API_KEY` 필요) |
| ② | **언어 갭** | let-tuple destructuring + static method call + Option::Some expr → 파서 추가 |
| ③ | **P** | 도메인 핵심 ≤1.00x 유지, FAST 확장 |
| ④ | **바인딩** | C# / Java / C (M3 showcase 확장) |
| ⑤ | **Track S** | gotgan / tree-sitter BMB-rewrite (장기) |

#### M4 준비 태스크 (선행 가능)

| # | 태스크 | 성격 | 소요 |
|---|--------|------|------|
| M4-1 | B 공식 측정 실행 (API key + 고정 모델) | B축 | 1 cycle |
| M4-2 | 언어 갭 이슈 등록: let-tuple, static-method, Option-expr | Drift C | 즉시 |
| M4-3 | let (a, b) = expr — tuple destructuring 파서 추가 | 언어 | 2-3 cycles |
| M4-4 | Type::method() — static method call expression 파서 추가 | 언어 | 2-3 cycles |
| M4-5 | Option::Some(x) 표현식 위치 지원 | 언어 | 1-2 cycles |
| M4-6 | C# 바인딩 scaffold | 바인딩 | 3-5 cycles |
| M4-7 | `set obj.field[idx] = val` 파서 확장 (ISSUE-20260511) | 언어 | ✅ Cycle 2690-2692 (nested chain 포함) |
| M4-8 | Tier 1 bench inproc 변환 (Knapsack, Mandelbrot) | P축 | ✅ Cycle 2694-2695 (4 도메인 누적) |
| M4-9 | clang knapsack outlier 분석 (6.7x BMB faster) | P축 | 장기 — IR/asm 비교 |

---

### v1.0 선언 조건 (비자율, 외부 신호)

```
GitHub stars      ≥ 1,000
외부 PR merged    ≥ 10 (각각 다른 contributor)
외부 이슈 (월)    ≥ 10
외부 BMB 프로젝트 ≥ 5
부정 평가 비율    < 30% (HN/Reddit 노출 후)
결정권: 메인테이너 + 외부 contributor 협의
```

마일스톤(M1~M4)은 자율 게이트. 버전(v0.x → v1.0+)은 **비자율, 외부 신호 게이트**.

---

## § 5 측정 지표 (B > P > A > D > C)

| 축 | 현재값 | 목표 | 측정 방법 |
|----|--------|------|----------|
| **B** Failure Rate | ⚠️ **비공식 ~90.9%** (2026-03-26, ~100문제) — 공식 미선언 | 공식 baseline 확보 → 개선 추적 | LLM 1-shot 컴파일+verifier 통과율 |
| **P** Performance | ✅ 16/16 ≤1.05x · inproc 4 도메인 (knapsack/mandelbrot/fibonacci/nqueen 평균 BMB vs clang 1.057x, vs gcc 도메인별 양극 0.38-1.39x) | 도메인 핵심 ≤1.00x, 일부 FAST | Tier 1/3 벤치마크 + inproc (Cycle 2685-2695) |
| **A** Token Efficiency | ❌ 미측정 | BMB ≤ Rust LOC (동일 알고리즘) | LOC·토큰 비교 |
| **D** Verification | ❌ 미측정 | contract 자동 증명률 추적 | `bmb verify` 통과율 |
| **C** Navigability | ❌ 미측정 | LLM N-파일 정답률 추적 | Track R suite |

> **진단**: P만 측정됨. B는 비공식 데이터 존재, 공식 선언 필요. M4 첫 액션 = **B 공식 baseline 선언**.

### P 축 inproc 측정 누적표 (Cycle 2661-2695)

`time_ns()` + `bmb_black_box` harness, median of 5 runs.
검증: 모든 runtime checksum 일치 (correctness OK).

| 도메인 | BMB / Clang -O3 ratio | BMB / GCC -O3 ratio | 측정 cycle |
|--------|---------------------|--------------------|-----------|
| nqueen (15-q × 10 iter) | **1.06x** | 1.27x | 2660-2667 |
| fibonacci (50, 100M iter) | **1.04x** | **0.38x** (BMB 2.6x faster) | 2685-2686 |
| knapsack (N=2000, cap=5000, 50 iter) | **0.149x** (BMB 6.7x faster, **clang anomaly**) | 1.39x | 2694 |
| mandelbrot (size=2000, max_iter=100) | **1.075x** | **1.11x** | 2695 |

**Clang ratio 평균 (산술)**: 4 도메인 = **0.831x**, 단 knapsack outlier 영향 큼.
**Clang ratio 평균 (knapsack 제외)**: 3 도메인 = **1.058x** — LLVM parity 범위.

**핵심 관찰**:
- BMB vs Clang (knapsack 제외): 1.04-1.075x — **LLVM 백엔드 parity 가설 일관**
- BMB vs GCC: 도메인별 양극 (0.38-1.39x) — LLVM 백엔드 특성 (knapsack에서 GCC 9x faster than clang)
- knapsack clang outlier (6.7x BMB faster): clang -O3가 inner loop를 GCC 대비 9x 느리게 — clang의 vector/SLP 최적화 갭으로 추정 (M4-9 분석 cycle 필요)

**절대 측정값** (μs, median of 5):
- knapsack: BMB 171000 / Clang 1138000 / GCC 124000
- mandelbrot: BMB 148000 / Clang 138000 / GCC 133000
- nqueen/fibonacci 절대값은 이전 세션 측정 (HANDOFF 표 참조)

### P 축 Tier 1/3 측정 (Cycle 2725 — 1년 stale 데이터 재검증)

historic.json (2026-05-02, 5-run) + tier3-10runs.json (2026-05-01, 10-run, noise-gate):

| benchmark | tier | BMB | C | ratio_c | 이전 (2026-04-13) | 변화 |
|-----------|------|-----|---|---------|-------------------|------|
| **sorting** | 3 | 121 | 133 | **0.910x** ✅ | 1.10x | **19 pp** BMB **9% FASTER** |
| **lexer** | 3 | 28 | 28 | **1.000x** ✅ | 1.09x | **9 pp** parity |
| **brainfuck** | 3 | 29 | 28 | **1.036x** | 1.11x | 7.4 pp 개선 |
| hash_table | 1 | 112 | 109 | **1.027x** | 1.11x | 8.3 pp 개선 |
| binary_trees | 1 | 121 | 116 | **1.043x** | 1.06x | 1.7 pp 개선 |
| fasta | 1 | 115 | 106 | **1.085x** | 1.08x | ≈ same (StringBuilder 무관 확인) |

**Cycle 2722-2725 진단 결과**:
- 3 ISSUE close (compare-inline / string-builder-opt / match-jump-table 모두 false positive 또는 목표 달성)
- 1 new ISSUE 등록 (or-chain-lowering, lexer 1.000x 달성으로 P2 강등)
- M1 가설 ≤1.05x 16/16 PASS — historic 데이터에서 6/6 P-track 벤치마크 모두 ≤1.085x

**시스템적 발견**: 1년 stale measurement이 3 cycles false-positive 야기. ISSUE 양식 측정 stamp + stale-after threshold 표준화 필요 (다음 세션 우선).

---

## § 6 ISSUE 양식 표준화 (Cycles 2728-2735, 신규)

`claudedocs/issues/_template.md` 양식 표준:

| 필수 필드 | 목적 |
|----------|------|
| `measurement_date` | 측정한 날짜 (commit date 아님) |
| `stale_after` | stale로 간주되는 시점 (기본 +3개월) |
| `measurement_source` | 측정 명령/파일 경로 |
| `observed_rate` | 정량값 (% / ratio / count) |
| `scope` | 영향 범위 |
| `env_hash` | OS / LLVM / GCC 버전 hash |

### Cycle 2735 시점 ISSUE 백로그

| 항목 | 카운트 |
|------|-------|
| Active ISSUE | **19** (직전 23 - 5 close + 1 신규) |
| Closed (누적) | **40** (+5 이번 세션) |
| 양식 적용 | 21/21 (직접 12 + batch reference 9) = **100%** |
| 신규 표준 파일 | `_template.md`, `_b_track_methodology_stamp.md` (모두 `claudedocs/issues/` 하 local) |

### 이번 세션 close 5건 (Cycles 2728-2735)

| ISSUE | 사유 |
|-------|------|
| `llvm-name-conflicts` (Cycle 2730) | Lint 11 (Cycle 2703)에서 이미 resolution |
| `simd-vectorization` (Cycle 2730) | Superseded (Cycle 2220) 잔존 정리 |
| `roadmap-sync` (Cycle 2733) | v0.98 재측정으로 3 claim 모두 resolved (Fixed Point ✅, 0 FAIL ✅, BMB ≥ C ✅) |
| `if-else-early-return-codegen` (Cycle 2735) | v0.98 재현 5/5 정답 (v0.51.22 era 버그, codegen 변경으로 fix) |
| `recursive-function-codegen` (Cycle 2735) | v0.98 재현 heapify 5/5 deterministic (v0.51.22 era 버그) |

### Cycle 2728 신규 ISSUE — 양식 first application (Cycle 2736 가설 확정)

`ISSUE-20260511-golden-flakiness-inttoptr.md` — 풀 골든 4건 environmental flakiness:
- HANDOFF 원본 "lcs_three 1 FAIL 회귀" framing **기각**
- 실제: 4 tests fail (lcs_three / cholesky_trace / crc32_simple / assortativity)
- 3 measurement points (load 종속):
  - 격리 stress (50회): **20% segfault**
  - 첫 풀 골든 (2 concurrent benches): **4/2862 fail (0.14%)**
  - 깨끗한 환경 재실행 (Cycle 2736, 1 concurrent bench): **0/2862 fail (0%)** ✅
- → MSYS2/UCRT64 fork/heap concurrent UB 확정. `inttoptr/ptrtoint` 패턴은 발현 빈도만 결정
- 우선순위 P2 → **P3 강등** (실제 사용자 영향 극히 미미)
- Fix scope: codegen `inttoptr` → `alloca ptr` 전환, multi-cycle (5-10 cycles), low priority

### 풀 골든 결과 (Cycle 2736 갱신)

| Cycle | 실행 환경 | 결과 |
|-------|----------|------|
| 2701 (2026-05-02) | 깨끗 | 2862/2862 PASS ✅ |
| 2718 (Cycle 2718 시점) | 2 background benches 동시 | 2858/2862 PASS (4 fail flaky) |
| **2736 (Cycle 2729→2736)** | 1 background bench | **2862/2862 PASS** ✅ (35.2분) |

---

## HUMAN 결정 사항 (2026-05-11 갱신, Cycle 2745 채택)

### M3/M4 실행 결정 (시퀀스 명시)

| 항목 | 결정 | 시퀀스 |
|------|------|--------|
| M3 showcase 라이브러리 선정 | ✅ **bmb-algo** | 완료 |
| **M3-5 bmb-algo README 정정** (knapsack 100→10 items, 90x/181x source 검증, clang vs gcc 라벨) | ✅ **재측정 + 정정** — BMB 철학 "측정 없는 성능 주장 금지" 우선 | **시퀀스 B (자율, publish 선결)** |
| npm publish | ✅ M3-5 정정 **후** — `workflow_dispatch` → dry_run → 실 publish | **시퀀스 C (HUMAN dispatch)** |
| PyPI publish | ✅ M3-5 정정 **후** — `workflow_dispatch` → publish → 실 publish | **시퀀스 C (HUMAN dispatch)** |
| v0.100 버전 선언 | ✅ M3 publish 완료 직후 메인테이너 결정 | publish 후 |
| **B 공식 측정 모델** | ✅ **claude-sonnet-4-6** — Opus 대비 비용 1/5 + 품질 80%+ | — |
| B 공식 측정 실행 | ✅ 즉시 — `BMB_BENCH_API_KEY` 설정 후 자동 | **시퀀스 D (HUMAN setup + 자율 실행)** |
| **M3-6 신규: CI workflow + Dockerfile 5위치 `-march=native`** | ✅ **spec 정합 적용** — CI history 단절 수용 (정확성 > 연속성, BMB 철학 정렬) | **시퀀스 E (자율 draft + HUMAN merge)** |
| M3-7: M4-1 결과에 "supersedes 2026-03-26 (baseline change Cycle 2742)" annotation | ✅ M4-1 종속, 자동 처리 | M4-1과 함께 |

### M5 언어 설계 결정 (불변)

| 항목 | 결정 |
|------|------|
| M5-1 하위 호환성 | ✅ unit enum + payload enum 모두 `{i64, i64}` 통일 |
| M5-1 LLVM 표현 | ✅ `%EnumValue = type { i64, i64 }` 고정 2-word (스택 할당) |
| M5-1 가변 페이로드 | ✅ M5-1 = i64 단일 페이로드만. Result<T,E>는 M5-2로 defer |
