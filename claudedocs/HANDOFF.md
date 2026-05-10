# BMB Session Handoff — 2026-05-11 (Cycles 2670-2679 — M5-5c/d 구현 → M5-5 7/7 완료)

> **HEAD**: `fe8dde38` (이전 세션 종료) + 마무리 commit (예정)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **이전 세션 핸드오프**: cycle-logs/cycle-2660~2669.md 참조

---

## 0. 이번 세션 작업 (Cycles 2670-2679)

### 세션 성과 요약

| 사이클 | 제목 | 성과 |
|--------|------|------|
| 2670-2673 | **M5-5c 구현 ✅** | `fn() -> Array<String>` dispatch — `parse_return_type` + `get_fn_return_scan` + `collect_string_fns` `A:` prefix + `llvm_gen_call` push_str_ptr_marker |
| 2674-2675 | **M5-5d 구현 ✅** | struct field `Array<String>` dispatch — `check_field_type` `~a` suffix + `is_field_str_array` + `llvm_gen_field_access` push_str_ptr_marker |
| 2676-2677 | ROADMAP/HANDOFF 갱신 + edge case (mut field) | M5-5 7/7 반영 |
| 2678 | 종합 commit (Cycles 2670-2677) | (다음) |
| 2679 | 세션 마무리 | (다음) |

---

## 1. 현재 상태

### 언어 갭 구현 현황 (M5-5 완성)

| 기능 | 이전 | 현재 |
|------|------|------|
| `let (a, b) = expr` | ❌ | ✅ M4-3 (Cycle 2621) |
| `Type::method(args)` | ❌ | ✅ M4-4 (Cycle 2620) |
| `Option::Some(x)` 표현식 | ❌ | ✅ M5-1 (Cycle 2633) |
| `println(String)` | ❌ | ✅ M5-4 (Cycle 2640) |
| `println(user_fn() -> String)` | ❌ | ✅ Cycle 2642 |
| `println(f64)` | ❌ | ✅ Cycle 2643 |
| `println(struct.string_field)` | ❌ | ✅ Cycle 2645 |
| `set b.string_field = x` (mut) | ✅ | ✅ Cycle 2646 |
| `println(arr[i])` of `["a","b"]` | ❌ | ✅ Cycle 2651 |
| `let arr2 = arr` alias dispatch | ❌ | ✅ Cycle 2652 |
| `let mut arr; set arr[i] = "x"` | ❌ | ✅ Cycle 2653 |
| `[s; N]` var-repeat | ❌ | ✅ Cycle 2664 |
| **`fn() -> Array<String>` 반환** | ❌ | ✅ **Cycle 2670-2673** |
| **`p.field[i]` struct array 필드** | ❌ | ✅ **Cycle 2674-2675** |

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
| M3 External Bindings | 🔄 ~96% (자율 100%, HUMAN publish 잔여) |
| M4 Adopted | 🔄 ~40% (M4-3/M4-4/M4-6 ✅, M4-1 미착수) |
| M5 Language Completeness | 🔄 M5-1~M5-4 ✅ + **M5-5 7/7 ✅** |

### 테스트 현황

| 스위트 | 결과 |
|--------|------|
| `cargo test --release` | ✅ 6210 passed |
| `bootstrap` 골든 | ✅ 총 **2857개** (M5-5c 3개 + M5-5d 3개 신규) |
| Stage 1 빌드 | ✅ OK (10.5s) |

---

## 2. 태스크 목록

### M5 완료 후 잔여 작업

| # | 태스크 | 성격 | 상태 |
|---|--------|------|------|
| M3-3 | **[HUMAN]** npm publish | 실행 | ⏳ workflow_dispatch dry_run=false |
| M3-4 | **[HUMAN]** PyPI publish | 실행 | ⏳ workflow_dispatch publish=true |
| M3-5 | **[HUMAN]** bmb-algo README baseline 명시 | 결정 | ⏳ |
| M4-1 | B 공식 측정 (1-shot LLM 성공률) | 자율 | ⏳ — `BMB_BENCH_API_KEY` 필요 |
| arena OOM | compiler.bmb self-compile 32G+ 초과 | 장기 | ⏳ — Fixed Point 검증 차단 중 |
| M6 | type-checker 분리 + AST inferred type attach | 장기 | ⏳ |
| nested struct array | `p.inner.tags[0]` 검증 | 자율 | ⏳ — get_field_ptr 인프라 활용 검토 |
| `Array<X>` 일반화 | i64, f64, ptr 등 (현재는 String만) | 자율 | ⏳ |

---

## 3. M5-5c/d 구현 핵심 사항 (이번 세션)

### M5-5c: `fn() -> Array<String>` (Cycle 2670-2673)

**4-point fix**:
1. `parse_return_type` (compiler.bmb 2617) — TK_IDENT("Array") + TK_LT + TK_STRING_TYPE + TK_GT 시퀀스 인식 → `"Array<String>"` 반환
2. `get_fn_return_scan` (compiler.bmb 6505) — sexp `"Array"` + `"<String>"` 두 토큰 합쳐 인식 (read_sexp_at가 `<`에서 끊김)
3. `collect_string_fns_acc` (compiler.bmb 13800) — ret_type=="Array<String>" → `"A:" + fn_name` tag (signature 변경 회피)
4. `llvm_gen_call_with_string_tracking_sb_reg` (compiler.bmb 15254) — `is_dynamic_string_array_fn` lookup → `push_str_ptr_marker`

**핵심 통찰**: 별도 `string_array_fns` 인자 추가 대신 기존 `string_fns` 리스트에 `A:` prefix 임베드 → caller chain ~20 함수 signature 변경 회피.

### M5-5d: struct field `Array<String>` (Cycle 2674-2675)

**4-point fix**:
1. `check_field_type` (compiler.bmb 2917) — Array<String> 토큰 시퀀스 인식 → 3 반환
2. `parse_struct_fields_to_registry` (compiler.bmb 2908) — type_info==3 → `~a` suffix
3. `is_field_str_array` / `check_field_is_str_array` 신규 — `is_field_string` 패턴 미러
4. `llvm_gen_field_access` (compiler.bmb 14894) — `field_is_str_arr` 체크 + `push_str_ptr_marker`

**핵심 통찰**: struct field type registry가 이미 카테고리화 (`~d`/`~s`/`~p-X`) → 새 카테고리 `~a` 추가만으로 일관 확장.

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

- **BMB_PATH 절대경로 필수**: `BMB_PATH=D:/data/lang-bmb/target/release/bmb.exe`
- **BMB 소스 em-dash 금지**: U+2014 → ASCII 하이픈
- **새 generic 타입 추가 시**: parse_return_type + get_fn_return_scan + check_field_type 모두 점검 (M5-5c/d 패턴 활용)
- **string_fns 카테고리화**: 추가 카테고리 (`R:` Result, `M:` Map 등) 확장 시 깨끗

---

## 5. 다음 세션 시작 체크리스트

- [ ] `claudedocs/ROADMAP.md` 읽기 (실무 앵커)
- [ ] `claudedocs/cycle-logs/cycle-2670.md` + `cycle-2674.md` 읽기 (이번 세션 핵심)
- [ ] `cargo test --release` → 6210/6210 확인
- [ ] M5-5c/d 신규 골든 검증:
  - `./target/bootstrap/bmb-stage1.exe build tests/bootstrap/test_golden_arr_str_fn_return.bmb -o /tmp/g && /tmp/g.exe` → "foo\nbar", exit 42
  - `./target/bootstrap/bmb-stage1.exe build tests/bootstrap/test_golden_arr_str_struct_field.bmb -o /tmp/g && /tmp/g.exe` → "red\ngreen\nblue", exit 42

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

- **B 공식 측정**: `BMB_BENCH_API_KEY` 설정 후 `bmb-ai-bench run` 1회 (M4-1)
- **nested struct field 검증**: `p.inner.tags[0]` 시나리오 (get_field_ptr 인프라 활용)
- **in-process timing benchmark-bmb 전체 적용** (다음 세션)

### 3순위 — 장기 아키텍처

- **arena OOM**: compiler.bmb self-compile 32G+ 초과 — Stage 2/3 검증 차단
- **type-checker 분리** (M6): AST inferred type attach + lookup_fn_ret_raw
- **Array<X> 일반화**: i64, f64, struct ptr — 현재는 String만

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

**세션 종료**: 2026-05-11 (Cycles 2670-2679 — M5-5c/d 구현 → M5-5 7/7 완료, 골든 2851 → 2857)
