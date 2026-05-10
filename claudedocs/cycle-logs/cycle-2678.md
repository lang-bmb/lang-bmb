# Cycle 2678-2679: M5-5 완성 — 10-사이클 종합 정리
Date: 2026-05-11

## Re-plan
이전 Carry-Forward (Cycle 2675/2677): M5-5d 구현 완료 + 갱신 완료. 종합 commit + 세션 마무리.

## Scope & Implementation

### 10-사이클 (2670-2679) 종합 요약

| Cycle | 핵심 작업 | 결과 |
|-------|---------|------|
| 2670 | M5-5c 진단 — `parse_return_type` Array<String> fallback i64 발견 | 4-point 원인 (parse_return_type + get_fn_return_scan + collect + dispatch) |
| 2671 | M5-5c 구현 (4-point fix) | `Array<String>` parse + collect `A:` prefix + push_str_ptr_marker |
| 2672 | M5-5c 첫 골든 + 동작 확인 | `arr_str_fn_return` exit 42 ✅ |
| 2673 | M5-5c 변형 검증 (alias, for-loop) | 2개 골든 추가, cargo test 6210 회귀 없음 |
| 2674 | M5-5d 진단 — struct field type registry `~a` 필요 | 동형 패턴 확인 (M5-5c와 동일) |
| 2675 | M5-5d 구현 ✅ + 골든 2개 | struct field `~a` suffix + is_field_str_array |
| 2676 | ROADMAP 갱신 (M5-5 7/7) | 골든 2856 |
| 2677 | mut struct field edge case 검증 | 골든 2857 (`arr_str_struct_field_mut`) |
| 2678 | 종합 commit (현재) | (다음) |
| 2679 | 세션 마무리 | (다음) |

### 세션 핵심 산출물

**M5-5c 구현 (Cycle 2670-2673)**:
- `parse_return_type` — `Array<String>` 토큰 시퀀스 인식 → ret_type 문자열 emit
- `get_fn_return_scan` — sexp 두 토큰 ("Array" + "<String>") 합쳐 인식
- `collect_string_fns_acc` — `A:fn_name` prefix 임베드 (caller chain signature 변경 회피)
- `is_dynamic_string_array_fn` + `llvm_gen_call dispatch` — push_str_ptr_marker 발행
- 골든 3개: `arr_str_fn_return`, `arr_str_fn_return_alias`, `arr_str_fn_return_loop`

**M5-5d 구현 (Cycle 2674-2675)**:
- `check_field_type` — Array<String> field 인식 → 3 반환
- `parse_struct_fields_to_registry` — `~a` suffix 추가
- `is_field_str_array` / `check_field_is_str_array` — `is_field_string` 패턴 미러
- `llvm_gen_field_access` — `field_is_str_arr` 체크 + push_str_ptr_marker
- 골든 3개: `arr_str_struct_field`, `arr_str_struct_field_loop`, `arr_str_struct_field_mut`

**핵심 설계 결정**:
- **임베드 vs 별도 인자**: M5-5c는 `A:` prefix로 기존 `string_fns`에 임베드 → caller chain ~20 함수 signature 변경 회피. 깨끗.
- **카테고리화 확장**: M5-5d는 struct field type registry의 기존 카테고리화 (`~d`/`~s`/`~p-X`)에 `~a` 추가 → 일관 확장 패턴.

## Verification & Defect Resolution

**최종 상태**:
- `cargo test --release`: ✅ 6210 passed (회귀 없음)
- 골든 테스트: ✅ 2857/2857 (이전 2851 + 신규 6)
- Stage 1 빌드: ✅ OK (~10.5s)
- M5-5 매트릭스: **7/7 ✅ 완료**
- 변형 시나리오 모두 PASS:
  - M5-5c: 직접 호출 indexing, alloca + indexing, alias propagation, for-loop iter
  - M5-5d: 직접 field indexing, for-loop iter, mut field reassignment

## Reflection

**10-사이클 평가**:

**달성**:
- **M5-5 7/7 ✅ 완료** — Array<String> 모든 시나리오 (literal/alias/while/mut/var-repeat/fn-return/struct-field)
- 두 거대한 미지원 패턴 (M5-5c, M5-5d) — 한 세션에 정확한 layer 수정으로 해결
- 4-point fix 패턴 검증 — generic type 인식이 parse → AST scan → MIR registry → codegen 모든 layer에서 정합 필요
- 골든 6개 신규 — 회귀 가드 안정

**미달성 / Defer**:
- nested struct array (`p.inner.tags[0]`) — 미검증, 다음 세션
- Array<X> 일반화 (i64, f64, struct ptr) — 현재는 String만
- arena OOM Fixed Point 검증 — 여전히 차단 중

**Philosophy 점검**:
- "Workaround 금지, 근본 해결" — 4-point fix가 각 layer 정확 위치 수정. AST sexp 합성 (`Array` + `<String>`), registry 카테고리화 (`~a`) 모두 정형 ✅
- "복잡도는 기피 사유 아니다" — 두 미지원 패턴 둘 다 시도, 둘 다 한 세션에 완료 ✅
- "AI-native 언어 확장" — LLM이 자연스럽게 작성할 패턴 `["a","b"]` 반환 함수 + struct { tags: Array<String> } 자동 dispatch ✅
- "출력 디폴트 = AI 친화" — string_fns `A:` prefix는 기계 파싱 친화 ✅

**도그푸딩 활동**:
- compiler.bmb 자체가 dispatch 추가 → bootstrap 컴파일러 도그푸딩
- struct field type registry 패턴 일관성 = "AI가 쉽게 추론 가능한 코드" 증명

**Roadmap impact**:
- M5-5 7/7 완료 → 다음 세션은 HUMAN 결정 또는 M4-1 (B 공식 측정) 또는 nested struct array 검증
- M4 ~40% 유지 (M4-1만 잔여 자율 작업)
- M3 ~96% 유지 (HUMAN publish 잔여)

**Structural Improvement Proposals (Carry-Forward)**:
- 공통 type signature parsing helper (parse_return_type + check_field_type 통합) — M6 후보
- `Array<X>` 일반화 (i64, f64, struct ptr) — 현재 패턴 확장
- `string_fns` 카테고리화 추가 (`R:` Result, `M:` Map 등) — 일관 패턴
- `lookup_fn_ret_raw`(raw BMB type) — registry format 확장 (M6 큰 변경)

## Carry-Forward (다음 세션)

### 1순위 — HUMAN 결정 (불변)
- npm publish, PyPI publish, README baseline 명시, v0.100 선언

### 2순위 — 자율 (작은 사이클)
- B 공식 측정 (M4-1) — `BMB_BENCH_API_KEY` 설정 후
- nested struct array (`p.inner.tags[0]`) 검증
- in-process timing benchmark-bmb 전체 적용

### 3순위 — 장기 아키텍처
- arena OOM 근본 해결 (compiler.bmb self-compile)
- type-checker 분리 (M6) — AST inferred type attach
- `Array<X>` 일반화

## 세션 종료
2026-05-11 (Cycles 2670-2679, 10-사이클 — **M5-5 7/7 완료**)
