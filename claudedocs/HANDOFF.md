# BMB Session Handoff — 2026-05-23 (Cycles 3069-3074 — M6 완료 선언 + str_sb 추적 완전화)

> **HEAD**: `3827e001` (feat(cycles-3069-3074): M6 완료 선언 + bootstrap str_sb 추적 완전화)
> **이전 HEAD**: `032eae83` (chore: HANDOFF HEAD 최종 갱신)
> **3-Stage Fixed Point**: ✅ IR Fixed Point 확인 (Cycle 3073 — S3==S4 `745082F5`)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: M7 정의 또는 인간 주도 방향 설정 (M6 COMPLETE, Known Issues 모두 HUMAN-blocked)

---

## 이번 세션 작업 요약 (Cycles 3069-3074)

| Cycle | 제목 | 내용 |
|-------|------|------|
| 3069 | M6 완료 선언 | ROADMAP M6 ✅ COMPLETE 마킹 (P1/P2/P3 완료, playground 제외) |
| 3070 | method_to_runtime_fn allowlist | catch-all→allowlist 교체, substr 3개소 추가, Fixed Point ✅ |
| 3071 | gotgan BMB_PATH | `bmb_exe_path()` env var 우선 탐색 추가 |
| 3072 | native 검증 + 결함 문서화 | str_sb 사전 결함 발견, Cycle 3073으로 이관 |
| 3073 | is_string_returning_fn 완전화 | 20종 String-반환 함수 추가, native println 정상화, Fixed Point ✅ |
| 3074 | ROADMAP + 조기 종료 | 조기 종료 조건 충족 (액션 없음) |

### 핵심 성과: str_sb 추적 완전화 (Cycle 3073)

**발견**: `is_string_returning_fn` (bootstrap str_sb 추적 함수)에 런타임 String-반환 함수 20종 전부 누락.
- `bmb_string_reverse`, `bmb_string_substr`, `bmb_string_pad_left/right`, `bmb_string_trim/replace`, `bmb_string_to_upper/lower/repeat/join`, `bmb_f64_to_string`, `bmb_to_hex/binary/octal`, `bmb_getcwd`, `bmb_exec_output/system_capture/read_line/exec_with_stdin`, `bmb_svec_get/join`, `bmb_string_split`

**결과**: `println(s.reverse())`, `println(s.substr(6, 5))`, `println("hi".pad_left(5, 32))` 등 native 정상화.

**체크리스트** (향후 String-반환 함수 추가 시 5개소 동시 업데이트):
1. `method_to_runtime_fn` — 메서드→함수 이름 매핑
2. `get_call_arg_types` — 인수 타입 ("p", "i", "d")
3. `get_call_return_type` — 반환 타입 ("ptr", "i64", "double")
4. IR preamble — LLVM `declare` 추가
5. `is_string_fn_group*` — str_sb 추적 등록

### method_to_runtime_fn allowlist (Cycle 3070)

**변경**: catch-all `else { "bmb_" + method }` → `else { "__unknown_method_" + method }`
- 명시적 매핑 10종 추가: split/reverse/pad_left/pad_right/count/last_index_of/substr/abs/min/max
- 링크 에러 메시지 개선: `__unknown_method_unknown_name__` 형태로 즉시 진단 가능

---

## 테스트 상태

- `cargo test --release`: **6264 PASS** (3782 + 47 + 22 + 2390 + 23) ✅
- 3-Stage Fixed Point: `745082F5CA427CCDA06AB36A2C603953EA792701D84E5B1DBD6A94D4A65FB6B7` ✅
- native method_test (5종): `edcba`, `3`, `9`, `world`, `   hi` ✅

---

## 현재 로드맵 상태

| 마일스톤 | 상태 |
|---------|------|
| M1 | ✅ COMPLETE |
| M2 | ✅ COMPLETE |
| M3 | ✅ COMPLETE (2026-05-21) |
| M4 | ✅ COMPLETE |
| M5 | ✅ COMPLETE (Native Complete 포함) |
| M6 | ✅ COMPLETE (2026-05-23) |
| M7 | 미정의 — 다음 세션 방향 설정 필요 |

---

## Known Issues (Active, 모두 HUMAN-blocked)

- `ISSUE-20260326-external-problem-validation.md` — B축 외부 검증 방법론
- `ISSUE-20260326-integration-category-weakness.md` — 통합 카테고리 취약점
- `ISSUE-20260326-multi-model-validation.md` — 다중 모델 검증
- `ISSUE-20260326-problem-difficulty-bias.md` — 문제 난이도 편향
- `ISSUE-20260511-golden-flakiness-inttoptr.md` — 골든 테스트 비결정성

---

## 다음 세션 권장 사항

1. M7 마일스톤 방향 사용자 결정 필요
2. untracked golden tests 처리 (test_golden_extractor.bmb.out 포맷 불일치)
3. benchmark Tier 3 run 횟수 표준화 (5-run → 10-run 권고)
