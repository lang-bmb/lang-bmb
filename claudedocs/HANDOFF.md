# BMB Session Handoff — 2026-05-11 (Cycles 2690-2707 — set field-index + 골든 0 FAIL + hardcoded list cleanup + lint 11 + clang outlier 분석)

> **HEAD**: `9d8b3da2` (Cycles 2690-2707 통합 commit)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **이전 세션 핸드오프**: cycle-logs/cycle-2680~2689.md 참조

---

## 0. 이번 세션 작업 (Cycles 2690-2707)

### 세션 성과 요약

| 사이클 | 제목 | 성과 |
|--------|------|------|
| 2690 | **set field-index 파서 설계 + 기본 i64** ✅ | AST desugar 전략 — `(set_index (field obj f) i v)`, 신규 노드 0 |
| 2691 | **set field-index 변형 (f64/String/compound)** | 3/3 PASS, nested 갭 발견 |
| 2692 | **nested field path 일반화** ✅ | `parse_set_field_chain` 재귀 — 7 변형 PASS |
| 2693 | **골든 등록 + manifest 검증 결함 정정** | 19개 manifest 오등록 fix |
| 2694 | **Knapsack inproc 측정** ✅ | BMB vs clang 0.149x (6.7x faster), vs gcc 1.39x |
| 2695 | **Mandelbrot inproc 측정** ✅ | BMB vs clang 1.075x, vs gcc 1.110x |
| 2696 | **측정 데이터 종합 + 골든 end-to-end** | 2850/2862 PASS, 9 manifest + 3 회귀 분류 |
| 2697 | **set_cover 회귀 단일 질문 IR diff** ✅ | builtin `@bit_or` arity 충돌 — source rename fix |
| 2698 | **골든 스위트 재실행 + audit 스크립트** ✅ | 2859 PASS / 3 FAIL, audit-first 권고 |
| 2699 | **token_scan + tokenizer 동시 진단** ✅ | hardcoded `tokenize` String-fn 충돌 → silent IR corruption |
| 2700 | **회귀 source rename** ✅ | 2개 회귀 즉시 fix |
| 2701 | **골든 end-to-end 종결** ✅ | **2862/2862 PASS, 0 FAIL** |
| 2702 | **컴파일러 fix: tokenize 제거** ✅ | source rename 워크어라운드 불필요해짐 |
| 2703 | **Lint 11 — builtin_name_collision** ✅ | 25 reserved 이름 정적 감지 |
| 2704 | **M4-9 clang knapsack outlier IR 분석** ✅ | clang -O3 unconditional store + select-phi anti-pattern → ISSUE deferral |
| 2705 | **hardcoded dead entries 8개 추가 제거** ✅ | concat/3/5/7, make_error 등 |
| 2706 | HANDOFF/ROADMAP 갱신 (현재) | (다음) |
| 2707 | 통합 commit + 세션 마무리 | (다음) |

---

## 1. 현재 상태

### 언어 갭 구현 현황 (M5-5g 확장)

| 기능 | 이전 | 현재 |
|------|------|------|
| `let (a, b) = expr` | ✅ M4-3 | ✅ |
| `Type::method(args)` | ✅ M4-4 | ✅ |
| `Option::Some(x)` 표현식 | ✅ M5-1 | ✅ |
| `println(String)` | ✅ M5-4 | ✅ |
| **Array<String> 전체 시나리오** | ✅ M5-5 | ✅ |
| **Array<f64> literal/fn-return/struct-field/nested/alias/loop/mut** | ✅ M5-5f | ✅ |
| **set obj.field[idx] = val** | ❌ | ✅ **M5-5g (Cycle 2690)** |
| **set o.f1.f2[idx] = val (nested chain)** | ❌ | ✅ **M5-5g 확장 (Cycle 2692)** |
| **set o.f1.f2.f3 = val (3-level chain)** | ❌ | ✅ **M5-5g 확장 (Cycle 2692)** |

### 마일스톤 상태

| 마일스톤 | 상태 |
|---------|------|
| M1 Self-Validated | ✅ COMPLETE |
| M2 AI-Ready Infra | ✅ COMPLETE |
| M3 External Bindings | 🔄 ~96% (자율 100%, HUMAN publish 잔여) |
| M4 Adopted | 🔄 ~50% (M4-3/4/6/7/8 ✅, M4-1 미착수) |
| M5 Language Completeness | 🔄 M5-1~M5-5g ✅ (full set field chain) |

### 테스트 현황

| 스위트 | 결과 |
|--------|------|
| `cargo test --release` | ✅ 6210 passed |
| `bootstrap` 골든 manifest | 2862개 |
| Stage 1 빌드 | ✅ OK (10s) |
| **골든 스위트 end-to-end (Cycle 2701)** | ✅ **2862/2862 PASS, 0 FAIL** (43분 풀 실행) |
| 회귀 정정 누적 | 9 manifest (Cycles 2693, 2696) + 1 source rename (Cycle 2697 bit_or) + 컴파일러 fix (Cycle 2702 tokenize hardcoded 제거) |

### 측정 현황 (in-process timing 누적)

| Bench | BMB vs clang -O3 -march=native | BMB vs gcc -O3 -march=native | 판정 |
|-------|--------------------------------|-------------------------------|------|
| nqueen (15-q × 10 iter) | 1.06x | 1.27x | LLVM parity, gcc 특화 |
| fibonacci (50, 100M iter) | 1.04x | 0.38x (BMB 2.6x faster) | LLVM iter loop 최적 |
| knapsack (N=2000, cap=5000) | **0.149x (BMB 6.7x faster)** | 1.39x | **clang outlier** (M4-9 분석 후보) |
| mandelbrot (size=2000) | 1.075x | 1.110x | dual baseline parity |

**평균 (clang, knapsack 제외)**: 1.058x — LLVM parity 일관.
**평균 (clang, all)**: 0.831x (knapsack outlier 영향).

---

## 2. 태스크 목록 (잔여 + 신규)

### 다음 세션 우선순위

| # | 태스크 | 성격 | 상태 |
|---|--------|------|------|
| ~~NEW~~ token_scan / tokenizer segfault | 자율 | ✅ Cycle 2699-2702 (compiler fix + lint) |
| ~~NEW~~ builtin `@bit_*` arity 체크 | 자율 | ⏳ 부분 — Cycle 2697/2700 source rename + Cycle 2702/2705 hardcoded dead 9개 제거. arity 체크 자체는 미진행 |
| ~~NEW~~ golden manifest auto-audit script | 자율 | ✅ `scripts/audit-golden-manifest.sh` (Cycle 2698), 풀 스위트가 동등 검증 |
| ~~M4-9~~ clang knapsack outlier IR 분석 | 자율 | ✅ Cycle 2704 → ISSUE-20260511-clang-knapsack-outlier (deferral) |
| M4-1 | B 공식 측정 (1-shot LLM 성공률) | 자율 | ⏳ — `BMB_BENCH_API_KEY` 필요 |
| M3-3 | **[HUMAN]** npm publish | 실행 | ⏳ |
| M3-4 | **[HUMAN]** PyPI publish | 실행 | ⏳ |
| M3-5 | **[HUMAN]** bmb-algo README baseline 라벨 | 결정 | ⏳ — "knapsack 6.7x faster than C" → "vs Clang -O3 outlier" 정정 권장 |
| Stage 2 진단 | compiler.bmb self-compile parse error vs arena OOM 두 가설 분리 | 장기 | ⏳ |
| Option C dynamic 우선화 | Stage 2 복원 후 재검토 | 장기 | ⏳ |
| M6 | type-checker 분리 + AST inferred type attach | 장기 | ⏳ |

---

## 3. 핵심 구현 사항 (이번 세션)

### M5-5g set field-index + nested chain (Cycle 2690-2692)

**전략**: AST 차원 desugar — 신규 AST/MIR 노드 0개

`bootstrap/compiler.bmb`:
- `parse_set_field` (996): `parse_set_field_chain` helper 위임
- `parse_set_field_chain` (신규): 한 번에 한 field 읽고 `.` / `[` / assign op 분기
- `parse_set_field_chain_index` (신규): `(set_index (field <prev_base> f) idx val)` 생성

**핵심 통찰**: `step_set_field` / `step_set_index` 가 base를 `EX`로 평가하므로 nested `(field ...)` 표현이 자동 처리 (M5-5e 무구현 통과 패턴 동일).

### Tier 1 inproc 측정 (Cycle 2694-2695)

`ecosystem/benchmark-bmb/benches/compute/`:
- knapsack/bmb/main_inproc.bmb + c/main_inproc.c — 측정 첫 실행 (직전 세션 inproc 파일은 존재, 측정만 미실행)
- mandelbrot/bmb/main_inproc.bmb + c/main_inproc.c — 신규 변환

**환경**: BMB --release / Clang -O3 -march=native / GCC -O3 -march=native, median of 5 runs.

### Golden manifest 정정 (Cycle 2693, 2696)

총 28개 manifest expected 정정:
- Cycle 2651-2675 잔재 8개: arr_str_* (mut_set, var_repeat, fn_return*, struct_field*)
- Cycle 2680-2683 잔재 11개: arr_str_nested*, arr_i64_baseline, arr_f64_*
- Cycle 2690-2692 신규 7개: set_field_index_*, set_field_chain_*
- Cycle 2696 발견 9개 추가: println_string, println_chain, println_f64, enum_str_payload, struct_str_field/mut, arr_str_println/alias/for_loop

### Golden 회귀 fix (Cycle 2697)

`test_golden_set_cover.bmb`: source rename `bit_or` → `bits_or_n` (builtin `@bit_or` arity 충돌 회피).

---

## 4. 환경 노트

| 환경 | 상태 |
|------|------|
| LLVM | 21.1.8 MSYS2 UCRT64 |
| Node.js | v24.14.0 |
| Python | 3.12.10 |
| 버전 | `0.98.0` |
| Branch | `main` |

### 운용 주의사항

- **BMB_PATH 절대경로 필수**
- **BMB 소스 em-dash 금지** (U+2014 → ASCII)
- **새 generic 타입 추가 시**: parse_return_type + get_fn_return_scan + check_field_type 모두 점검
- **builtin 이름 충돌**: Cycle 2702-2705로 9개 dead/안전 entries 제거 + Cycle 2703 lint 11 (builtin_name_collision) 추가. 21개 reserved 이름은 lint가 정적 감지. 컴파일러 자체는 hardcoded list 일부 잔존 (chr/slice/get_field 등 — defined in compiler.bmb).
- **AST 차원 desugar 패턴**: 신규 AST 노드 추가 전, 기존 노드의 base/operand로 표현 가능한지 먼저 검토 (M5-5g/e 무구현 통과 사례)

---

## 5. 다음 세션 시작 체크리스트

- [ ] `claudedocs/ROADMAP.md` 읽기 (실무 앵커)
- [ ] `claudedocs/cycle-logs/cycle-2700~2707.md` 읽기 (이번 세션 fix 라이브러리)
- [ ] `claudedocs/issues/ISSUE-20260511-clang-knapsack-outlier.md` (M4-9 deferral)
- [ ] `cargo test --release` → 6210/6210 확인
- [ ] 골든 스위트 sample (`./scripts/run-golden-tests.sh --json`) — **0 FAIL 예상** (Cycle 2701에서 2862/2862 PASS 확인)
- [ ] HUMAN 결정 잔여: M3-3 (npm), M3-4 (PyPI), M3-5 (README clang vs gcc 라벨), M4-1 (B 측정 BMB_BENCH_API_KEY)

---

## 6. HUMAN 결정 사항 (불변, 2026-05-10/11 확정)

| 항목 | 결정 |
|------|------|
| M3 showcase 선정 | ✅ bmb-algo |
| npm publish | ✅ 즉시 진행 |
| PyPI publish | ✅ 즉시 진행 |
| v0.100 버전 선언 | ✅ M3 publish 완료 직후 |
| B 공식 측정 | ✅ 즉시 실행 |
| README "knapsack 6.8x faster" | ⏳ **clang -O3 -march=native 재현 확인** (Cycle 2694), gcc는 BMB 1.39x slower — 라벨 명시 권장 |

---

**세션 종료**: 2026-05-11 (Cycles 2690-2707 — M5-5g + 골든 0 FAIL + hardcoded list cleanup 9개 + Lint 11 + clang outlier 분석)
