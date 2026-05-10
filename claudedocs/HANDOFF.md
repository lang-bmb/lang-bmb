# BMB Session Handoff — 2026-05-11 (Cycles 2650-2657 — M5-5 array dispatch + M3-2 정식 측정 5개)

> **HEAD**: `d9855b04` (커밋 예정 — cycle 2650-2657 변경 누적)
> **실무 앵커**: `claudedocs/ROADMAP.md`

---

## 0. 이번 세션 작업 (Cycles 2619-2649)

### 세션 성과 요약

| 사이클 | 제목 | 성과 |
|--------|------|------|
| 2619 | 위생 + M4 이슈 등록 | 3개 이슈 등록, 기준선 확인 |
| 2620 | M4-4 Static Method Call | `Type::method(args)` 파서 구현 ✅ |
| 2621 | M4-3 Let-Tuple Destructuring | `let (a, b) = expr` 파서 구현 ✅ |
| 2622 | M4-5 스코프 분석 | payload enum = M5-1로 재분류 |
| 2623 | CLAUDE.md Rule 2 업데이트 | 지원 문법 목록 갱신, 사이드 이펙트 명시 |
| 2624 | 엣지 케이스 골든 테스트 | 2개 테스트 추가 (let-tuple-advanced, static-method-advanced) |
| 2625 | M5-1 아키텍처 설계 문서 | `DESIGN-M5-1-payload-enum.md` 작성 |
| 2626 | M4 통합 테스트 | `test_golden_m4_integration.bmb` → 42 |
| 2627 | HANDOFF + ROADMAP 갱신 | — |
| 2628 | M4 최종 커밋 | 완료 |
| 2629-2632 | C# 바인딩 + PyPI 수정 | M4-6 C# 5/5 ✅, PyPI windows-2022 수정 |
| **2633** | **M5-1 payload enum 구현** | **`enum Option { None, Some(i64) }` + match ✅** |
| **2634** | **Rule 문서화 + OOM 분석** | **wildcard 지원 확인 + CLAUDE.md Rule 2/3 업데이트 ✅** |
| **2635** | **M5-2 Result enum 검증** | **`Result<Ok,Err>` + 3-variant + 체이닝 골든 테스트 3개 ✅** |
| **2636** | **HANDOFF + M5-3 설계** | **DESIGN-M5-3 설계 문서 + HANDOFF 갱신 ✅** |
| **2637** | **M5-3 다중 필드 enum** | **`Branch(i64,i64)` + `Three(i64,i64,i64)` 구현 ✅** |
| **2638** | **CLAUDE.md Rule 2 업데이트** | **M5-3 문서화 + HANDOFF 갱신 ✅** |
| **2639** | **Dead Code 제거** | **`resolve_payload_extracts` 2개 함수 제거 ✅** |
| **2640** | **M5-4 println(String) 구현** | **`str_sb` dispatch → `@println_str` 자동 선택 ✅** |
| **2641** | **HANDOFF + ROADMAP M5-4 반영** | **— 문서 갱신** |
| **2642** | **println(user_fn()) 체이닝 검증** | **string_fns 경로 골든 테스트 ✅** |
| **2643** | **println(f64) dispatch** | **is_double_var_sb 인프라 활용 → @println_f64 ✅** |
| **2644** | **enum String payload 통합 테스트** | **Message::Text(String) → match → println 종합 ✅** |
| **2645** | **struct String 필드 타입 추적** | **registry `~s` suffix + is_field_string ✅** |
| **2646** | **중첩 + mut struct String 검증** | **set_field 경로 영향 없음 확인 ✅** |
| **2647** | **HANDOFF + ROADMAP + CLAUDE.md 종합 갱신** | **— 문서 갱신** |
| **2648** | **dispatch 갭 탐색 + M5-4 매트릭스** | **`arr[i]` of String 미지원 명확화** |
| **2649** | **M5-5 후보 등록 + 세션 마무리** | **— 세션 종료 정리** |
| **2650** | **tuple String 갭 발견 + M5-5 인프라 조사** | **`let (s,n)=...` String component 미지원 확정** |
| **2651** | **M5-5 array literal of String dispatch ✅** | **`lower_array_literal_sb` mark_str_ptr 발행** |
| **2652** | **M5-5 alias / while iter 검증 ✅** | **R: marker propagation 정상** |
| **2653** | **M5-5 mut set 검증 ✅ + struct field array ❌** | **M5-5 매트릭스 4/7 ✅ 확정** |
| **2654** | **bmb-algo Python 벤치 7/7 BMB faster** | **6/7 FAST ≥2x speedup** |
| **2655** | **knapsack BMB vs C 정식 측정** | **BMB 1.22x slower (1417 vs 1152 ms)** |
| **2656** | **추가 4개 알고리즘 C 비교** | **fibonacci 1000x BMB faster (@pure), sieve/lcs ≈ C, edit_dist 1.50x slower** |
| **2657** | **checksum DIFFER 조사 + HANDOFF 갱신** | **5/5 알고리즘 checksum 동일 (trailing newline만 차이)** |
| **2658** | **floyd_warshall 측정 + 종합 정리** | **BMB 1.73x faster (591 vs 1021 ms), 6개 알고리즘 매트릭스 ✅** |

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
| M5-5 | `[String; N]` array element 타입 추적 | 언어 (인프라 확장) | ⬜ 후보 — 큰 변경 |

---

## 3. 다음 세션 우선순위

### 1순위 — M5-5 또는 M3-2 (자율 실행 가능)

1. **M5-5 array element 타입 추적**: `arr[i]` of `[String; N]` dispatch
   - 인프라: 배열 element 타입 registry 신규 도입 (struct registry 패턴 미러링)
   - 영향 범위: `lower_index_sb` (MIR), `llvm_gen_index` (codegen), 새 함수 `is_array_element_string`
   - 소요: 3-5 cycles
2. **M3-2 showcase 벤치마크 측정**: bmb-algo (이미 선정) 공식 벤치 측정
   - 조건: BMB --release + opt -O2 vs C -O2 -march=native (Rule 4)
   - 산출: M3 완료 마지막 자율 게이트
   - 소요: 1-2 cycles

### 2순위 — M6 계획 수립 (장기 아키텍처)

3. **arena OOM 근본 해결**: `compiler.bmb` self-compile 시 32G+ 초과
   - 원인: 문자열 기반 AST의 O(n²) 성장 (Cycle 2634 확인)
   - 방향: 문자열 AST → 구조체 전환 (큰 변경, M6 메이저)
   - Fixed Point (S2 == S3) 복원 차단 해소

### 3순위 — HUMAN 결정 대기

| # | 작업 | 트리거 |
|---|------|------|
| M3-3 | npm publish | `workflow_dispatch` dry_run=false |
| M3-4 | PyPI publish | `workflow_dispatch` publish=true (windows-2022 수정 push 선행 필요) |
| M3-5 | NuGet publish 5 C# 패키지 | M4-6 완료 후 |
| M4-1 | B 공식 측정 | `BMB_BENCH_API_KEY` 필요 |
| v0.100 | 메이저 버전 선언 | M3 publish 직후 메인테이너 결정 |

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
- [ ] `claudedocs/cycle-logs/cycle-2649.md` 읽기 (마지막 Carry-Forward)
- [ ] `cargo test --release` → 6210/6210 확인
- [ ] M5-4 신규 골든 테스트 4개 검증:
  - `./target/bootstrap/bmb-stage1 build tests/bootstrap/test_golden_println_string.bmb -o /tmp/g && /tmp/g.exe` → "hello", exit 42
  - `./target/bootstrap/bmb-stage1 build tests/bootstrap/test_golden_struct_str_field.bmb -o /tmp/g && /tmp/g.exe` → "Bob\n25", exit 42
- [ ] (선택) `./target/release/bmb build bootstrap/lsp.bmb -o bootstrap/lsp.exe` + `python3 bootstrap/lsp_test.py` → 100/100

---

---

## 8. M5-1 구현 핵심 사항

### 페이로드 enum LLVM 표현

```
heap calloc(2, 8) → 2-word struct:
  word 0: tag (i64)    — variant index (0-based)
  word 1: payload (i64) — 값 (unit variant = 0)
```

### 두 lowering 시스템 동시 처리 필수 (Rule 추가 예정)

| 시스템 | 위치 | 목적 |
|--------|------|------|
| recursive | `lower_expr_sb` | 표현식 내 중첩 eval |
| iterative | `step_expr` | 함수 body `let` 체인 |

신규 AST 노드 추가 시 **두 곳 모두** 수정 필수. (struct_init, lambda, enum_val 선례)

### bootstrap Arena OOM (pre-existing, 사이클 2237 이후)

`compiler.bmb` (~20K LOC) → Stage 2 빌드 시 arena 한계(16G) 초과.  
M5-1과 무관한 사전 존재 문제. Fixed Point 복원에 32G+ arena 또는 증분 컴파일 필요.

**세션 종료**: 2026-05-10 (Cycles 2619-2633 — M4 언어 갭 + M5-1 payload enum)
