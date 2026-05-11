# BMB Session Handoff — 2026-05-11 (Cycles 2680-2689 — nested + Array<f64> 일반화 + inproc 측정 표준화)

> **HEAD**: `796a55b2` → (예정 통합 commit Cycles 2680-2687)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **이전 세션 핸드오프**: cycle-logs/cycle-2670~2679.md 참조

---

## 0. 이번 세션 작업 (Cycles 2680-2689)

### 세션 성과 요약

| 사이클 | 제목 | 성과 |
|--------|------|------|
| 2680 | **nested struct array 검증** ✅ | M5-5d 인프라가 nested 경로 자연 처리 — 무구현 통과, 골든 3개 |
| 2681 | **Array<f64> literal dispatch** ✅ | `mark_f64_ptr` 1-line fix — array literal 동작 |
| 2682 | **Array<f64> fn return + struct field** ✅ | 4+4 point fix (M5-5c/d f64 변형) — `F:` prefix + `~af` suffix |
| 2683 | **Array<f64> 변형 시나리오** ✅ | alias/loop/nested/mut set 4개 무구현 통과 — 골든 4개 |
| 2684 | **set field-index 파서 갭 발견** | `set s.values[0] = x` 미지원 → ISSUE-20260511-set-field-index 등록 |
| 2685 | **In-process timing 표준 패턴 문서화** | `INPROC_TIMING_GUIDE.md` 신규 + nqueen 측정 재확인 |
| 2686 | **fibonacci inproc 변환 + 측정** | BMB vs clang 1.04x, BMB vs gcc 0.38x (BMB 2.6x faster) |
| 2687 | **종합 회귀 + 안정성 검증** | 11개 신규 골든 모두 PASS + cargo test 6210 |
| 2688 | ROADMAP/HANDOFF 갱신 (현재) | (다음) |
| 2689 | 통합 commit + 세션 마무리 | (다음) |

---

## 1. 현재 상태

### 언어 갭 구현 현황 (Array<X> 일반화 확장)

| 기능 | 이전 | 현재 |
|------|------|------|
| `let (a, b) = expr` | ❌ | ✅ M4-3 (Cycle 2621) |
| `Type::method(args)` | ❌ | ✅ M4-4 (Cycle 2620) |
| `Option::Some(x)` 표현식 | ❌ | ✅ M5-1 (Cycle 2633) |
| `println(String)` | ❌ | ✅ M5-4 (Cycle 2640) |
| **Array<String> 전체 시나리오** | ❌ | ✅ M5-5 7/7 (Cycle 2651-2675) |
| **nested struct Array<String> field** | 미검증 | ✅ Cycle 2680 (무구현 통과) |
| **Array<f64> literal/fn-return/struct-field** | ❌ | ✅ **Cycle 2681-2682** |
| **Array<f64> alias/loop/nested/mut set** | 미검증 | ✅ **Cycle 2683** (모두 무구현 통과) |

### Track 스냅샷 (변화 없음)

| Track | % | 상태 |
|-------|---|------|
| M (Machine-First) | ~100% ✅ | 완료 |
| N (MCP Server) | ~99% ✅ | 완료 |
| O (Context Pack) | ~95% ✅ | 완료 |
| Q (Ambiguity Audit) | ~92% ✅ | 완료 |
| R (LLM Bench) | ~95% ✅ | 완료 |
| S (BMB-rewrite) | ~99% ✅ | 완료 |
| T (External Bindings) | ~95% ✅ | 완료 |

### 마일스톤 상태

| 마일스톤 | 상태 |
|---------|------|
| M1 Self-Validated | ✅ COMPLETE |
| M2 AI-Ready Infra | ✅ COMPLETE |
| M3 External Bindings | 🔄 ~96% (자율 100%, HUMAN publish 잔여) |
| M4 Adopted | 🔄 ~40% (M4-3/M4-4/M4-6 ✅, M4-1 미착수) |
| M5 Language Completeness | 🔄 M5-1~M5-5 ✅ + **M5-5e nested ✅** + **M5-5f Array<f64> ✅** |

### 테스트 현황

| 스위트 | 결과 |
|--------|------|
| `cargo test --release` | ✅ 6210 passed |
| `bootstrap` 골든 | ✅ 총 **2868개** (이전 2857 + 신규 11) |
| Stage 1 빌드 | ✅ OK (10.7s) |

### 측정 현황 (in-process timing)

| Bench | BMB vs clang -O3 | BMB vs gcc -O3 | 판정 |
|-------|------------------|----------------|------|
| nqueen (15-queens × 10 iter) | 1.06x (BMB +5.8%) | 1.27x (BMB +26%) | LLVM parity, gcc 특화 알려진 케이스 |
| fibonacci (50, 100M iter) | 1.04x (BMB +4.3%) | 0.38x (**BMB 2.6x faster**) | LLVM iter loop 최적 |

---

## 2. 태스크 목록 (잔여 + 신규)

### 다음 세션 우선순위

| # | 태스크 | 성격 | 상태 |
|---|--------|------|------|
| **NEW** | **`set obj.field[idx] = val` 파서 확장** | 자율 (2-3 cycles) | ⏳ ISSUE-20260511 |
| M4-1 | B 공식 측정 (1-shot LLM 성공률) | 자율 | ⏳ — `BMB_BENCH_API_KEY` 필요 |
| M3-3 | **[HUMAN]** npm publish | 실행 | ⏳ workflow_dispatch dry_run=false |
| M3-4 | **[HUMAN]** PyPI publish | 실행 | ⏳ workflow_dispatch publish=true |
| M3-5 | **[HUMAN]** bmb-algo README baseline 명시 | 결정 | ⏳ |
| **NEW** | **Tier 1 bench inproc 변환** (Knapsack, Mandelbrot) | 자율 | ⏳ |
| **NEW** | **BMB vs gcc IR 비교 사이클** (도메인별 갭 분석) | 자율 | ⏳ |
| arena OOM | compiler.bmb self-compile 32G+ 초과 | 장기 | ⏳ — Fixed Point 검증 차단 중 |
| M6 | type-checker 분리 + AST inferred type attach | 장기 | ⏳ |
| `Array<X>` 추가 일반화 | bool, char (낮은 우선순위) | 장기 | ⏳ |

---

## 3. 핵심 구현 사항 (이번 세션)

### Array<f64> 5+4 point fix (Cycle 2681-2682)

**Part 1: fn return** (M5-5c f64 변형)
1. `parse_return_type` (line 2649) — Array + LT + F64 + GT 토큰 시퀀스 → `"Array<f64>"`
2. `get_fn_return_scan` (line 6537) — sexp 두 토큰 "Array" + "<f64>" 합쳐 인식
3. `collect_string_fns_acc` (line 13836) — ret_type=="Array<f64>" → `F:` prefix
4. `is_dynamic_f64_array_fn` + `check_f64_array_fn_in_list` (line 13937, 신규)
5. `llvm_gen_call` dispatch (line 15363) — `is_f64_array_fn` → push_f64_ptr_marker

**Part 2: struct field** (M5-5d f64 변형)
1. `check_field_type` (line 2918) — Array<f64> → type_info==4
2. `parse_struct_fields_to_registry` (line 2906) — `~af` suffix
3. `is_field_f64_array` + `check_field_is_f64_array` (신규)
4. `llvm_gen_field_access` (line 14909) — `field_is_f64_arr` → push_f64_ptr_marker

**핵심 통찰**: M5-5c/d 패턴과 완전 동형. `mark_f64_ptr` MIR 명령어는 이미 존재 — 인프라 0 추가, 인식 + dispatch 채널 연결만.

### In-process timing 표준 패턴 (Cycle 2685-2686)

**산출물**:
- `ecosystem/benchmark-bmb/docs/INPROC_TIMING_GUIDE.md` — BMB/C/Rust harness 패턴
- `fibonacci/bmb/main_inproc.bmb` + `fibonacci/c/main_inproc.c` 샘플 (nqueen 외 첫 변환)

---

## 4. 환경 노트 (변화 없음)

| 환경 | 상태 |
|------|------|
| LLVM | 21.1.8 MSYS2 UCRT64 |
| Node.js | v24.14.0 |
| Python | 3.12.10 |
| 버전 | `0.98.0` |
| Branch | `main` |

### 운용 주의사항

- **BMB_PATH 절대경로 필수**: `BMB_PATH=D:/data/lang-bmb/target/release/bmb.exe`
- **BMB 소스 em-dash 금지**: U+2014 → ASCII 하이픈
- **새 generic 타입 추가 시**: parse_return_type + get_fn_return_scan + check_field_type 모두 점검 (M5-5c/d/f 패턴)
- **string_fns 카테고리 (`A:` / `F:`)**: 다음 카테고리 추가 시 동일 패턴 (`B:` bool 등)

---

## 5. 다음 세션 시작 체크리스트

- [ ] `claudedocs/ROADMAP.md` 읽기 (실무 앵커)
- [ ] `claudedocs/cycle-logs/cycle-2681.md` + `cycle-2682.md` 읽기 (Array<f64> 5+4 point fix)
- [ ] `claudedocs/issues/ISSUE-20260511-set-field-index.md` 읽기 (다음 자율 작업)
- [ ] `cargo test --release` → 6210/6210 확인
- [ ] 신규 11개 골든 검증:
  - `arr_f64_*` 시리즈 (literal/fn_return/struct_field/alias/for_loop/nested_struct/mut_set)
  - `arr_str_nested_struct{,_loop,_triple_nested}` 시리즈

---

## 6. 다음 세션 우선순위

### 1순위 — HUMAN 결정 (불변)

| # | 작업 | 트리거 |
|---|------|------|
| M3-3 | npm publish | `workflow_dispatch` dry_run=false |
| M3-4 | PyPI publish | `workflow_dispatch` publish=true |
| M3-5 | bmb-algo README baseline | clang vs gcc 명시 권장 |
| v0.100 | 메이저 버전 선언 | M3 publish 직후 메인테이너 결정 |

### 2순위 — 자율 (작은 사이클)

- **set field-index 파서 확장** (ISSUE-20260511, 2-3 cycles) — 신규 갭, AI-native 패턴
- **B 공식 측정**: `BMB_BENCH_API_KEY` 설정 후 `bmb-ai-bench run` (M4-1)
- **Tier 1 bench inproc 변환** (Knapsack, Mandelbrot, JSON parse) — 회귀 가드 강화

### 3순위 — 장기 아키텍처

- **BMB vs gcc IR 비교 사이클** — 도메인별 갭 분석 (nqueen 1.27x, fibonacci 0.38x 양극)
- **arena OOM**: compiler.bmb self-compile 32G+ 초과 — Stage 2/3 검증 차단
- **type-checker 분리** (M6): AST inferred type attach + lookup_fn_ret_raw
- **Array<X> 추가 일반화**: bool, char (낮은 우선순위, 사례 부족)

---

## 7. HUMAN 결정 사항 (불변, 2026-05-10/11 확정)

| 항목 | 결정 |
|------|------|
| M3 showcase 선정 | ✅ bmb-algo |
| npm publish | ✅ 즉시 진행 |
| PyPI publish | ✅ 즉시 진행 |
| v0.100 버전 선언 | ✅ M3 publish 완료 직후 |
| B 공식 측정 | ✅ 즉시 실행 |
| README "knapsack 6.8x faster" | ⏳ clang baseline 재현, gcc 미재현 — 라벨 명시 권장 |

---

**세션 종료**: 2026-05-11 (Cycles 2680-2689 — nested + Array<f64> 일반화 + inproc 측정 표준화, 골든 2857 → 2868)
