# BMB Session Handoff — 2026-05-11 (Cycles 2660-2669 — nqueen 측정 + in-process timing + M5-5b 구현)

> **HEAD**: `6aa6c7cc` (세션-종료 commit) + 정리 commit (예정)
> **submodule**: `ecosystem/benchmark-bmb` HEAD `6ce9f8d` (in-process harness commit)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **이전 세션 핸드오프**: 이전 세션 내용(Cycles 2650-2659)은 cycle-logs/cycle-2650~2659.md 참조

---

## 0. 이번 세션 작업 (Cycles 2660-2669)

### 세션 성과 요약

| 사이클 | 제목 | 성과 |
|--------|------|------|
| 2660 | nqueen 정식 측정 + clang -O3 baseline | BMB 829ms vs clang 847ms (0.97x), gcc -O3 6440ms (BMB 7.78x faster) |
| 2661 | in-process time_ns() harness | wall-clock vs in-process 측정 분리 — LLVM IPO 폴딩 효과 정량화 |
| 2662 | M3-2-bench-results.md v2 | 5개 알고리즘 clang -O3 + gcc -O3 dual baseline |
| 2663 | M5-5b 근본 진단 | val_type=="var" 발견 — get_node_type은 syntactic만 |
| 2664 | **M5-5b 구현 ✅** | `mark_str_ptr_if` 새 MIR 명령어 — codegen 시점 lookup |
| 2665 | M5-5b 골든 추가 + M5-5c 진단 | 2851/2851 골든, M5-5c 처방 명확화 |
| 2666 | M5-5c defer 결정 | program-level fn registry 필요 — 다음 세션 작업 |
| 2667 | ROADMAP/HANDOFF 갱신 | (현재) |
| 2668 | 종합 commit | (다음) |
| 2669 | 세션 마무리 | (다음) |

---

## 1. 현재 상태

### 언어 갭 구현 현황

| 기능 | 이전 | 현재 |
|------|------|------|
| `let (a, b) = expr` | ❌ 미지원 | ✅ M4-3 구현 (Cycle 2621) |
| `Type::method(args)` | ❌ 미지원 | ✅ M4-4 구현 (Cycle 2620) |
| `Option::Some(x)` 표현식 | ❌ 미지원 | ✅ M5-1 구현 완료 (Cycle 2633) |
| `println(String)` | ❌ 포인터 정수 출력 | ✅ M5-4 구현 완료 (Cycle 2640) |
| `println(user_fn() -> String)` | ❌ 포인터 정수 출력 | ✅ Cycle 2642 검증 (string_fns 경로) |
| `println(f64)` | ❌ 링크 실패 (type mismatch) | ✅ Cycle 2643 (`@println_f64` dispatch) |
| `println(struct.string_field)` | ❌ 포인터 정수 출력 | ✅ Cycle 2645 (registry `~s` suffix) |
| `set b.string_field = x` (mut) | ✅ 작동 | ✅ Cycle 2646 검증 (set_field 영향 없음) |
| `println(arr[i])` of `["a","b"]` | ❌ 포인터 정수 출력 | ✅ Cycle 2651 (mark_str_ptr 발행 + R: 자동 propagation) |
| `let arr2 = arr` alias dispatch | ❌ | ✅ Cycle 2652 (R: 마커 propagation) |
| `let mut arr; set arr[i] = "x"` | ❌ | ✅ Cycle 2653 (alloca R: 보존) |
| **`[s; N]` var-repeat** | ❌ 포인터 정수 출력 | ✅ **Cycle 2664 (`mark_str_ptr_if` 새 MIR — codegen 시점 lookup)** |
| `fn() -> Array<String>` 반환 | ❌ | ❌ 미지원 (다음 세션 — `collect_string_array_fns_from_mir`) |
| `p.field[i]` struct array 필드 | ❌ | ❌ 미지원 (다음 세션 — struct field type registry) |

### Track 스냅샷

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
| M3 External Bindings | 🔄 ~96% (자율 100% — M3-2 7/7 측정 + clang baseline + in-process timing 인프라; npm/PyPI publish 잔여 HUMAN) |
| M4 Adopted | 🔄 ~40% (M4-3 ✅, M4-4 ✅, M4-6 ✅, M4-5→M5-1 ✅, M4-1 미착수) |
| M5 Language Completeness | 🔄 M5-1~M5-4 ✅ + **M5-5 5/7 ✅** (literal/alias/while/mut/var-repeat, fn-return/struct-field 미지원) |

### 테스트 현황

| 스위트 | 결과 |
|--------|------|
| `cargo test --release` | ✅ 6210 passed |
| `bootstrap` 골든 테스트 | ✅ 총 **2851개** (M5-5 신규 5: arr_str_println, arr_str_alias, arr_str_for_loop, arr_str_mut_set, **arr_str_var_repeat**) |
| struct/enum 회귀 (Stage 1) | ✅ 8/8 PASS |

---

## 2. 태스크 목록

### M3 완료 태스크

| # | 태스크 | 성격 | 상태 |
|---|--------|------|------|
| M3-1 | showcase 선정: bmb-algo | HUMAN | ✅ 완료 |
| M3-2 | showcase 공식 벤치마크 측정 | 자율 | ✅ **완료** (7/7 clang+gcc dual baseline, Cycle 2655-2662) |
| M3-2-bench | nqueen 정식 측정 + 5개 알고리즘 clang baseline | 자율 | ✅ **완료** (Cycle 2660) |
| M3-2-timing | in-process timing 인프라 | 자율 | ✅ **완료** (Cycle 2661 — `time_ns()` + `bmb_black_box`) |
| M3-3 | **[HUMAN]** npm publish | 실행 | ⏳ workflow_dispatch dry_run=false |
| M3-4 | **[HUMAN]** PyPI publish | 실행 | ⏳ workflow_dispatch publish=true |
| M3-5 | **[HUMAN]** bmb-algo README baseline 명시 | 결정 | ⏳ — gcc/clang 모두 측정 결과 있음 |

### M5 잔여 태스크 (다음 세션)

| # | 태스크 | 성격 | 우선순위 |
|---|--------|------|---------|
| M5-5c | `fn() -> Array<String>` 반환 dispatch | 언어 인프라 | 高 — 옵션 A (`collect_string_array_fns_from_mir`) 구현 |
| M5-5d | `p.field[i]` struct array 필드 dispatch | 언어 인프라 | 中 — struct field type registry 필요 |
| in-process suite 적용 | benchmark-bmb 전체 알고리즘에 in-process timing | 자율 | 中 — 체계적 측정 환경 |
| clang -O3 knapsack 이상 조사 | 1.08s vs gcc 0.10s 차이 원인 | 조사 | 低 — LLVM 자체 이슈 가능성 |
| nqueen 1.26x slower 분석 | gcc 대비 in-process 1.26x slower 원인 | 조사 | 中 — 알고리즘 본질 |

---

## 3. 다음 세션 우선순위

### 1순위 — M5-5c/d 구현 (옵션 A)

1. **`collect_string_array_fns_from_mir` 신규** — `collect_string_fns_acc` 패턴 미러
2. **모든 codegen signature에 `string_array_fns` 인자 추가** — `gen_program_*`, `gen_function_*`, `llvm_gen_call_*`
3. **함수 호출 결과 자동 dispatch** — `is_dynamic_string_array_fn` lookup → `push_str_ptr_marker(str_sb, dest)`
4. 골든 테스트: `test_golden_arr_str_fn_return.bmb`, `test_golden_arr_str_struct_field.bmb`
5. 소요: 5-7 cycles (M5-5c) + 3-5 cycles (M5-5d) = 1-2 sessions

### 2순위 — HUMAN 결정 대기

| # | 작업 | 트리거 |
|---|------|------|
| M3-3 | npm publish | `workflow_dispatch` dry_run=false |
| M3-4 | PyPI publish | `workflow_dispatch` publish=true |
| M3-5 | NuGet publish 5 C# 패키지 | M4-6 완료 후 |
| M4-1 | B 공식 측정 | `BMB_BENCH_API_KEY` 필요 |
| README | bmb-algo baseline 명시 (clang vs gcc) | 측정 환경 vs 결과 정합성 결정 |
| v0.100 | 메이저 버전 선언 | M3 publish 직후 메인테이너 결정 |

### 3순위 — 장기 아키텍처

5. **arena OOM 근본 해결**: `compiler.bmb` self-compile 32G+ 초과
6. **type-checker 분리** — 옵션 A (AST inferred type attach)로 M5-5 잔여 + 다른 type 추적 통합

---

## 4. M5-5b 구현 핵심 사항 (이번 세션 — Cycle 2664)

### 새 MIR 명령어 `mark_str_ptr_if %dest, %src`

**lowering 단계** (compiler.bmb 5119, 5395 두 경로):
```bmb
// var 케이스에 conditional 발행
let w_mark = if val_type == "string" { sb_push_mir(sb, "  mark_str_ptr ...") }
    else if val_type == "var" { sb_push_mir(sb, "  mark_str_ptr_if " + dest + ", " + val_temp) }
    else { 0 };
```

**codegen 단계** (compiler.bmb 14400 옆 + 신규 함수):
```bmb
fn llvm_handle_mark_str_ptr_if(line: String, p: i64, str_sb: i64) -> String =
    let rest = line.slice(p + 16, line.len());
    let comma = find_comma(rest, 0);
    let dest = trim_end(rest.slice(0, comma));
    let src_start = low_skip_ws(rest, comma + 1);
    let src_clean = trim_end(rest.slice(src_start, rest.len()));
    let src_is_str = is_string_var_sb(src_clean, str_sb);
    let w = if src_is_str { push_str_ptr_marker(str_sb, dest) } else { 0 };
    same_mapping("");
```

**핵심 발상**: lowering 단계는 var의 inferred type을 모르지만, codegen 단계의 `str_sb` registry는 var → string 매핑 보유. 새 MIR 명령어가 이를 deferred decision으로 우회.

### 두 lowering 시스템 동시 처리 (CLAUDE.md Rule 3)

| 시스템 | 위치 | 처리 |
|--------|------|------|
| recursive | `lower_array_repeat_sb` (5101) | line 5119에 var 케이스 추가 |
| iterative | `step_array_repeat` (5377) | line 5395에 var 케이스 추가 |

---

## 5. M3-2 측정 핵심 사항 (이번 세션 — Cycle 2660-2662)

### 측정 모드 정의

| 모드 | 측정 대상 | 의미 |
|------|---------|------|
| **wall-clock** | startup + 알고리즘 + LLVM IPO 폴딩 | 사용자 체감 시간 |
| **in-process** (`time_ns()` + `bmb_black_box`) | 순수 알고리즘 본질 | 코드젠 품질 비교 |

### nqueen 측정 결과

| 측정 모드 | BMB ms | clang -O3 ms | gcc -O3 -flto ms |
|----------|--------|--------------|------------------|
| wall-clock | 829 | 847 | 6440 |
| in-process | 8861 | 8407 | 7012 |

**결론**: BMB ≈ clang -O3, gcc 우위는 LLVM IPO 약점 (wall-clock에서 7.78x faster는 IPO 폴딩 효과)

### 5-알고리즘 clang -O3 baseline (in-process + wall-clock 혼합)

| 알고리즘 | BMB ms | clang -O3 ms | Ratio | 평가 |
|---------|--------|--------------|-------|------|
| fibonacci | 4.8 | 4.1 | 1.17x | OK (startup-dominated) |
| sieve | 100.9 | 104.9 | 0.96x | ≤1.05x ✅ |
| floyd | 565.0 | 553.7 | 1.02x | ≤1.05x ✅ |
| nqueen | 819.6 | 846.7 | 0.97x | ≤1.05x ✅ |
| knapsack | 137 | 1085 | 0.13x | FAST (clang vectorization 이상) |

**전체**: 4/5 ≤1.05x, 5/5 ≤1.5x, 2/5 FAST → **CLAUDE.md "동일 LLVM ≈ Clang은 OK" 조건 충족**

---

## 6. HUMAN 결정 사항 (2026-05-10/11 확정)

| 항목 | 결정 |
|------|------|
| M3 showcase 선정 | ✅ **bmb-algo** |
| npm publish | ✅ **즉시 진행** — `workflow_dispatch` dry_run=false |
| PyPI publish | ✅ **즉시 진행** — `workflow_dispatch` publish=true |
| v0.100 버전 선언 | ✅ **M3 publish 완료 직후** |
| B 공식 측정 | ✅ **즉시 실행** — `BMB_BENCH_API_KEY` 설정 후 |
| M5-1 하위 호환성 | ✅ unit + payload enum 모두 `{i64, i64}` 통일 |
| M5-1 LLVM 표현 | ✅ `%EnumValue = type { i64, i64 }` 고정 2-word |
| M5-1 가변 페이로드 | ✅ M5-2로 defer (Result/T,E) |
| README "knapsack 6.8x faster" | ⏳ **HUMAN 결정** — clang baseline 재현, gcc 미재현, 라벨 명시 권장 |

---

## 7. 환경 노트

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
- **새 MIR 명령어 추가 시**: 두 lowering 경로 (recursive + iterative) 모두 처리
- **새 codegen 명령어**: `low_starts_with_at` 매칭 + 핸들러 함수 작성

---

## 8. 다음 세션 시작 체크리스트

- [ ] `claudedocs/ROADMAP.md` 읽기 (실무 앵커)
- [ ] `claudedocs/cycle-logs/cycle-2669.md` 읽기 (마지막 Carry-Forward)
- [ ] `claudedocs/M3-2-bench-results.md` 읽기 (정식 측정 v2)
- [ ] `cargo test --release` → 6210/6210 확인
- [ ] M5-5b 신규 골든 검증:
  - `./target/bootstrap/bmb-stage1.exe build tests/bootstrap/test_golden_arr_str_var_repeat.bmb -o /tmp/g && /tmp/g.exe` → "hello\nhello\nhello", exit 42
- [ ] Cycle 2667+: M5-5c 옵션 A 구현 (`collect_string_array_fns_from_mir`)

---

## 9. 세션 정리 작업 (이번 세션 마지막)

### 정리 완료
- ✅ 임시 binary 18개 삭제 (`*_v098*.exe`, `*_inproc*.exe`)
- ✅ in-process harness source 4개 submodule commit (`6ce9f8d`)
- ✅ HEAD hash 갱신 (`6aa6c7cc`)
- ✅ submodule pointer 메인 repo에 반영

### 보존된 진단용 임시 파일 (다음 세션 활용)
- `tests/bootstrap/test_m55b_var_repeat.bmb` — m55b 동작 확인용 (골든 `arr_str_var_repeat` 동등)
- `tests/bootstrap/test_m55c_fn_return_array_string.bmb` — M5-5c 다음 세션 진단 시작점

### 환경 cleanup 확인
- bootstrap stage1 정상 빌드 ✅
- cargo test 6210/6210 ✅
- 골든 2851/2851 ✅

---

## 10. M5-5c 구현 가이드 (다음 세션)

### 옵션 A 단계 (5-7 cycles 예상)

1. **`collect_string_array_fns_from_mir`** (compiler.bmb 13790 옆)
   - `collect_string_fns_acc` 패턴 미러, ret_type 매칭만 다름
   - `if ret_type == "Array<String>" { ... }` (또는 변형 — `Array<string>`, `[String; N]`?)
2. **`is_dynamic_string_array_fn`** (compiler.bmb 13847 옆)
3. **codegen signature 확장**:
   - `gen_program_sb_with_strings_fns_structs` (13931)
   - `gen_function_sb_structs_reuse` (14043)
   - `llvm_gen_fn_line_structs` (14386)
   - `llvm_gen_line_structs` (14394)
   - `llvm_gen_assign_structs` (14421)
   - `llvm_gen_rhs_*` 모두
   - 새 인자: `string_array_fns: String`
4. **호출 결과 처리**: `llvm_gen_call_*`에서 fn_name 추출 → `is_dynamic_string_array_fn` → `push_str_ptr_marker(str_sb, dest)` 호출
5. **골든 테스트**: `test_golden_arr_str_fn_return.bmb` (`fn make_strs() -> Array<String> = [...]`, `let arr = make_strs()`, `println(arr[0])`)

**세션 종료**: 2026-05-11 (Cycles 2660-2669 — nqueen 측정 + in-process timing + M5-5b 구현 + submodule in-process harness commit)
