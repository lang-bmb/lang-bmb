# BMB Session Handoff — 2026-05-11 (Cycles 2650-2659 — M5-5 array dispatch + M3-2 정식 측정 6개)

> **HEAD**: `cec09e90` (Cycle 2659 세션 마무리 + 추가 정리 commit 예정)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **이전 세션 핸드오프**: 이전 세션 내용(Cycles 2619-2649)은 cycle-logs/cycle-2619~2649.md 참조

---

## 0. 이번 세션 작업 (Cycles 2650-2659)

### 세션 성과 요약

| 사이클 | 제목 | 성과 |
|--------|------|------|
| 2650 | tuple String 갭 발견 + M5-5 인프라 조사 | `let (s,n)=...` String component 미지원 확정 |
| 2651 | **M5-5 array literal of String dispatch ✅** | `lower_array_literal_sb` mark_str_ptr 발행 (4 lines) |
| 2652 | M5-5 alias / while-iter 검증 | R: marker auto-propagation ✅ |
| 2653 | M5-5 mut set 검증 ✅ + struct field array ❌ | 매트릭스 4/7 ✅ 확정 |
| 2654 | bmb-algo Python 벤치 측정 | 7/7 BMB faster, 6/7 FAST ≥2x speedup |
| 2655 | knapsack BMB vs C 정식 측정 | BMB 1.22x slower (1393 vs 1140 ms, ITERS=500) |
| 2656 | 추가 4개 알고리즘 C 비교 (fib/sieve/lcs/edit_dist) | fibonacci 1000x BMB faster (@pure), sieve/lcs ≈ C, edit_dist 1.50x slower |
| 2657 | checksum DIFFER 조사 + HANDOFF 갱신 | 모두 동일 (trailing newline만 차이) |
| 2658 | **floyd_warshall 측정 + 종합 정리** | BMB 1.73x faster (591 vs 1021 ms), `M3-2-bench-results.md` 작성 |
| 2659 | 종합 commit + 세션 종료 정리 | 10-사이클 통합 커밋 `5f366e93` + 마무리 `cec09e90` |

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
| `[s; N]` var-repeat / `fn() -> Array<String>` / `p.field[i]` | ❌ | ❌ 미지원 (lower-time type registry 필요, M6 후보) |

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
| M3 External Bindings | 🔄 ~93% (showcase 5/7 정식 측정 ✅, npm/PyPI publish 잔여 HUMAN) |
| M4 Adopted | 🔄 ~40% (M4-3 ✅, M4-4 ✅, M4-6 ✅, M4-5→M5-1 ✅, M4-1 미착수) |
| M5 Language Completeness | 🔄 M5-1~M5-4 ✅ + M5-5 핵심 4/7 ✅ (literal/alias/while/mut, [s;N]/fn-return/struct-field 미지원) |

### 테스트 현황

| 스위트 | 결과 |
|--------|------|
| `cargo test --release` | ✅ 6210 passed |
| `bootstrap` 골든 테스트 | ✅ 총 2850개 (M5-5 신규 4: arr_str_println, arr_str_alias, arr_str_for_loop, arr_str_mut_set) |
| struct/enum 회귀 (Stage 1) | ✅ 8/8 PASS (enum_match, enum_variant, enum_payload, struct_complex, struct_method, nested_struct, mut_struct, struct_fn) |

---

## 2. 태스크 목록

### M3 완료 태스크 (변경 없음 — HUMAN 결정 대기)

| # | 태스크 | 성격 | 소요 |
|---|--------|------|------|
| M3-1 | **[HUMAN]** showcase 선정: bmb-algo vs bmb-json | 결정 | 즉시 |
| M3-2 | showcase 공식 벤치마크 측정 (v0.98 기준) | 자율 | 1-2 cycles |
| M3-3 | **[HUMAN]** npm publish | 실행 | 즉시 |
| M3-4 | **[HUMAN]** PyPI publish | 실행 | 즉시 |

### M4 태스크 상태

| # | 태스크 | 상태 |
|---|--------|------|
| M4-1 | **[HUMAN+KEY]** B 공식 측정 | ⏳ API key 필요 |
| M4-2 | 언어 갭 이슈 등록 | ✅ 완료 (Cycle 2619) |
| M4-3 | `let (a, b) = expr` tuple destructuring | ✅ 완료 (Cycle 2621) |
| M4-4 | `Type::method()` static method call | ✅ 완료 (Cycle 2620) |
| M4-5 | `Option::Some(x)` 표현식 | → **M5-1**로 재분류 |
| M4-6 | C# 바인딩 scaffold | ⏳ 미착수 |

### M5 준비 태스크 (신규)

| # | 태스크 | 성격 | 소요 |
|---|--------|------|------|
| M5-1 | payload enum 구현 | 언어 아키텍처 | ✅ **완료** (Cycle 2633) |
| M5-2 | Result enum + 다중 payload | 언어 | ✅ **완료** (Cycle 2635, M5-1 인프라 재사용) |
| M5-3 | Multi-field enum `Branch(i64,i64)` | 언어 | ✅ **완료** (Cycle 2637) |
| M5-4 | `println(String/f64)` + struct String 필드 종합 dispatch | 언어 | ✅ **완료** (Cycle 2640-2646) |
| M5-5 | `[String; N]` array element 타입 추적 | 언어 (인프라 확장) | 🔄 **부분 완료** (Cycle 2651-2653) — 매트릭스 4/7 ✅ |
| M5-5a | array literal of String dispatch (`["a","b"]`) | 언어 | ✅ Cycle 2651 |
| M5-5a | alias / while-iter / mut-set | 언어 | ✅ Cycle 2652-2653 |
| M5-5b | `[s; N]` var-repeat dispatch | 언어 | ❌ lower-time type registry 필요 |
| M5-5c | `fn() -> Array<String>` 반환 dispatch | 언어 | ❌ array-fn signature analysis 필요 |
| M5-5d | `p.field[i]` struct array 필드 dispatch | 언어 | ❌ struct field type registry 확장 필요 |

### M3 잔여 태스크

| # | 태스크 | 성격 | 상태 |
|---|--------|------|------|
| M3-2 (showcase C 비교) | knapsack/floyd/sieve/lcs/fibonacci/edit_distance 정식 측정 | 자율 | ✅ 6/7 완료 (Cycle 2655-2658) |
| M3-2 (잔여) | nqueens benchmark suite 추가 + 측정 | 자율 | ⏳ benchmark-bmb suite에 nqueens 부재 |
| README 검증 | bmb-algo "knapsack 6.8x faster than C" 주장 검증 | HUMAN | ⏳ 본 측정에서 미재현 (1.22x slower) |
| in-process timing | `time_ns()` 인프라로 wall-clock 한계 해소 | 자율 | ⬜ 미착수 |

---

## 3. 다음 세션 우선순위

### 1순위 — M6 (장기 아키텍처) 또는 잔여 자율 작업

1. **M6 lower-time type registry** (M5-5 잔여 + 통합 인프라)
   - 영향: M5-5b/c/d 잔여 3개 동시 해결 + tuple String 컴포넌트 (M5-4-A)
   - 인프라: lowering 단계의 var-type / fn-return-type / struct-field-type 추적 통합
   - 소요: 5-10 cycles (큰 변경, 단계별 진행)
2. **in-process timing** (`time_ns()` 활용 BMB 벤치 인프라)
   - 영향: M3-2 측정 정확도 향상, README 주장 재검증 가능
   - 소요: 2-3 cycles
3. **nqueens benchmark suite 추가** (M3-2 7/7 달성 마무리)
   - 영향: M3-2 정식 측정 완전 완료
   - 소요: 1 cycle

### 2순위 — README 정확성 (HUMAN 결정 후 진행)

4. **bmb-algo README 검증/갱신**: "knapsack 6.8x faster than C" 주장
   - 본 세션 측정: knapsack BMB 1.22x slower than C
   - 결정 옵션: 측정 환경 차이 인정 / 주장 갱신 / 재측정
   - 트리거: HUMAN 결정 → 자율 측정 또는 문서 갱신

### 3순위 — HUMAN 결정 대기

| # | 작업 | 트리거 |
|---|------|------|
| M3-3 | npm publish | `workflow_dispatch` dry_run=false |
| M3-4 | PyPI publish | `workflow_dispatch` publish=true |
| M3-5 | NuGet publish 5 C# 패키지 | M4-6 완료 후 |
| M4-1 | B 공식 측정 | `BMB_BENCH_API_KEY` 필요 |
| README | bmb-algo 측정 주장 검증/갱신 | 측정 환경 vs 결과 정합성 결정 |
| v0.100 | 메이저 버전 선언 | M3 publish 직후 메인테이너 결정 |

### 4순위 — 장기 아키텍처

5. **arena OOM 근본 해결**: `compiler.bmb` self-compile 32G+ 초과
   - 원인: 문자열 기반 AST O(n²) 성장 (Cycle 2634 확인)
   - 방향: 문자열 AST → 구조체 전환 (큰 변경, M6 메이저)
   - Fixed Point (S2 == S3) 복원 차단 해소

---

## 4. M5-4 dispatch 인프라 (Cycle 2640-2646 정리)

신규 println dispatch 함수: `llvm_try_println_str_dispatch` (compiler.bmb 라인 ~15220).

**호출 순서** (`llvm_gen_call_with_string_tracking_sb_reg`):
1. `extract_call_fn_name` → fn_name 추출
2. `llvm_try_println_str_dispatch` → fn_name이 print/println/eprint/eprintln + arg가 String/f64이면 dispatch IR 반환
3. dispatch가 비어있지 않으면 즉시 반환, 아니면 기존 `llvm_gen_call_reg` 경로

**str_sb 마커 종류**:
| 접두 | 의미 | push 시점 |
|-----|------|---------|
| `S:` | String 변수 | 문자열 리터럴, string_fns 호출, `~s` struct 필드 load |
| `D:` | f64 변수 | float 리터럴, double-반환 함수 호출 |
| `F:` | f64 ptr | (별도 추적) |

**struct registry 신규 suffix** (Cycle 2645):
- `~d` = f64 (기존)
- `~s` = String (신규)
- `~p-Type` = pointer to struct (기존)
- (없음) = i64 (기본)

**미해결 갭**:
- `arr[i]` of `[String; N]` — 배열 element 타입 추적 부재 (M5-5 후보)
- 워크어라운드: `fn elem(a: ..., i: i64) -> String = a[i];` 함수 래핑 → string_fns 경로

---

## 5. HUMAN 결정 사항 (2026-05-10 확정)

| 항목 | 결정 |
|------|------|
| M3 showcase 선정 | ✅ **bmb-algo** (알고리즘·CPU bound → 성능 가설 직접 증명) |
| npm publish | ✅ **즉시 진행** — `workflow_dispatch` dry_run=false |
| PyPI publish | ✅ **즉시 진행** — `workflow_dispatch` publish=true, repository=pypi |
| v0.100 버전 선언 | ✅ **M3 publish 완료 직후** 메인테이너 결정 |
| B 공식 측정 | ✅ **즉시 실행** — `BMB_BENCH_API_KEY` 설정 후 `bmb-ai-bench run` |
| M5-1 하위 호환성 | ✅ **전체 마이그레이션** — unit enum도 `{i64, i64}` 로 통일 (이중 코드젠 경로 금지) |
| M5-1 LLVM 표현 | ✅ **고정 2-word** — `%EnumValue = type { i64, i64 }` (heap alloc 없음) |
| M5-1 가변 페이로드 | ✅ **M5-2로 defer** — M5-1 범위 = i64 단일 페이로드 + Option/패턴 매칭 |

---

## 6. 환경 노트

| 환경 | 상태 |
|------|------|
| LLVM | 21.1.8 MSYS2 UCRT64 |
| Node.js | v24.14.0 |
| Python | 3.12.10 |
| 버전 | `0.98.0` |
| Branch | `main` |

### 운용 주의사항

- **BMB_PATH 절대경로 필수**: `BMB_PATH=D:/data/lang-bmb/target/release/bmb.exe`
- **lsp.exe 재빌드**: `./target/release/bmb build bootstrap/lsp.bmb -o bootstrap/lsp.exe`
- **verify_host.exe 재빌드**: `./target/release/bmb build bootstrap/verify_host.bmb -o bootstrap/verify_host.exe`
- **BMB 소스 em-dash 금지**: U+2014 → ASCII 하이픈
- **캐시 파일**: `*.vh_cache`, `*.vh_proofdb` → `.gitignore` 등록됨

---

## 7. 다음 세션 시작 체크리스트

- [ ] `claudedocs/ROADMAP.md` 읽기 (실무 앵커)
- [ ] `claudedocs/cycle-logs/cycle-2659.md` 읽기 (마지막 Carry-Forward)
- [ ] `claudedocs/M3-2-bench-results.md` 읽기 (정식 측정 매트릭스)
- [ ] `cargo test --release` → 6210/6210 확인
- [ ] M5-5 신규 골든 테스트 4개 검증:
  - `./target/bootstrap/bmb-stage1.exe build tests/bootstrap/test_golden_arr_str_println.bmb -o /tmp/g && /tmp/g.exe` → "hello\nworld", exit 42
  - `./target/bootstrap/bmb-stage1.exe build tests/bootstrap/test_golden_arr_str_alias.bmb -o /tmp/g && /tmp/g.exe` → "ant\nbee", exit 42
  - `./target/bootstrap/bmb-stage1.exe build tests/bootstrap/test_golden_arr_str_for_loop.bmb -o /tmp/g && /tmp/g.exe` → "foo\nbar\nbaz", exit 42
  - `./target/bootstrap/bmb-stage1.exe build tests/bootstrap/test_golden_arr_str_mut_set.bmb -o /tmp/g && /tmp/g.exe` → "alpha\nNEW\ngamma", exit 42
- [ ] (선택) `./target/release/bmb build bootstrap/lsp.bmb -o bootstrap/lsp.exe` + `python3 bootstrap/lsp_test.py` → 100/100

---

---

## 8. M5-5 구현 핵심 사항 (이번 세션)

### Array of String dispatch — 기존 인프라 재활용

**유일한 코드 변경** (`bootstrap/compiler.bmb` 5246 `lower_array_literal_sb`, 4 lines):
```bmb
// M5-5: Mark array of String for dispatch (first element type detection)
let first_elem = if count > 0 { get_child(ast, 0) } else { "" };
let first_type = if first_elem == "" { "" } else { get_node_type(first_elem) };
let w_mark = if first_type == "string" { sb_push_mir(sb, "  mark_str_ptr " + result_tmp) } else { 0 };
```

**자동 propagation 체인** (수정 불필요):
1. `mark_str_ptr %arr` → arr를 R: 마킹
2. `let arr2 = arr` → assign 시 R: propagation (라인 14461, 14490)
3. `arr[i]` → MIR `gep arr, idx` → `llvm_gen_gep_sb`에서 base R:이면 dest R: 마킹 (라인 6664)
4. `load_ptr gep_dest` → `llvm_gen_load_ptr_sb`에서 R:이면 dest를 S: 마킹 (라인 6649)
5. `println(load_dest)` → `llvm_try_println_str_dispatch` S: 인식 → `@println_str` dispatch

### M5-5 매트릭스 (4/7 ✅)

| 케이스 | 상태 | 메커니즘 / 한계 |
|--------|------|---------------|
| `["a","b"]` literal | ✅ | mark_str_ptr |
| `let arr2 = arr` alias | ✅ | R: marker propagation |
| `while ... arr[i]` loop | ✅ | block-internal R: persist |
| `let mut; set arr[i] = "x"` | ✅ | mut alloca R: 보존 |
| `[s; N]` var-repeat | ❌ | val_type="var" 추적 부재 |
| `fn() -> Array<String>` | ❌ | string_fns ret_type "String" 한정 |
| `p.field[i]` struct-array | ❌ | struct field type=i64 저장 (registry 부재) |

### M3-2 정식 C 비교 매트릭스 (6/7 ✅)

`claudedocs/M3-2-bench-results.md` 참조. 종합:
- ≤1.05x (목표): 4/6 (fibonacci, floyd, sieve, lcs)
- ≤2x (OK): 6/6
- BMB faster: 2/6 (fibonacci 1000x via @pure, floyd 1.73x)
- README "knapsack 6.8x faster than C" 미재현 (1.22x slower)

### 두 lowering 시스템 동시 처리 필수 (CLAUDE.md Rule 3)

| 시스템 | 위치 | 목적 |
|--------|------|------|
| recursive | `lower_expr_sb` | 표현식 내 중첩 eval |
| iterative | `step_expr` | 함수 body `let` 체인 |

본 세션의 M5-5 변경은 `step_array_literal`이 `lower_array_literal_sb`를 위임 → 단일 수정으로 양쪽 처리됨.

### bootstrap Arena OOM (pre-existing, 사이클 2237 이후)

`compiler.bmb` (~20K LOC) → Stage 2 빌드 시 arena 한계(16G) 초과. Fixed Point 복원에 32G+ arena 또는 증분 컴파일 필요.

**세션 종료**: 2026-05-11 (Cycles 2650-2659 — M5-5 array dispatch + M3-2 정식 측정 6개)
