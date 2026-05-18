# BMB Session Handoff — 2026-05-19 (Cycles 2928-2932 — http_parse flat + str_data P0 fix)

> **HEAD**: `7f1fbddc` (Cycles 2928-2932 완료)
> **이전 HEAD**: `07884ec1` (Cycles 2918-2927)
> **3-Stage Fixed Point**: ✅ IR Fixed Point 확인 (Cycle 2930) — GCC MinGW 링커 비결정성으로 binary hash 비교 불가, IR hash 비교로 방법론 정정. bootstrap/compiler_s3.exe IR == compiler_s4.exe IR
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: Cycle 2933

---

## 이번 세션 작업 요약 (Cycles 2928-2932)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 2928 | str_data builtin 추가 | bootstrap compiler.bmb에 `@str_data` 인라인 에미터 추가 (Stage 1 완료) |
| 2929 | csv_parse flat v2 | compound-cond 단일함수. 3300 µs (6.1% 개선). vs Clang 1.150× |
| 2930 | Bootstrap Fixed Point | IR Fixed Point 확인 + GCC 링커 비결정성 발견 + 방법론 정정 |
| 2931 | http_parse flat + P0 fix | flat 단일함수 1.099× + str_data literal crash P0 수정 (llvm_text.rs) |
| 2932 | str_data literal 테스트 | test_str_data_literal.bmb + .expected 신규. HANDOFF/ROADMAP 갱신 |

### 성능 현황 (tier3 inproc)

| 벤치마크 | BMB (µs) | C GCC (µs) | 비율 | 이전 비율 |
|---------|----------|-----------|------|---------|
| csv_parse flat v2 | ~3300 | ~2740 | **1.204×** | 1.283× (4.06× 이전) |
| http_parse flat v1 | ~2542 | ~2313 | **1.099×** | 1.186× |

### 핵심 변경 사항

**1. `str_data` builtin (Cycle 2928 — bootstrap)**:
- `bootstrap/compiler.bmb`에 `@str_data` 인라인 에미터 추가
- `llvm_gen_call` + `llvm_gen_call_reg` + `mlcse_safe_builtins`에 등록
- `tests/bootstrap/test_str_data_load_u8.bmb` 신규

**2. csv_parse flat v2 (Cycle 2929)**:
- `ecosystem/benchmark-bmb/benches/real_world/csv_parse/bmb/main_inproc.bmb` 재작성
- 단일 `parse_csv` 함수 + compound `and` while 조건
- 6.1% 개선. 잔여 갭 원인: i64 vs i32 arithmetic (~15%)

**3. http_parse flat v1 (Cycle 2931)**:
- `ecosystem/benchmark-bmb/benches/real_world/http_parse/bmb/main_inproc.bmb` 재작성
- request line skip (C와 동등), compound-cond 전체 inline
- `@inline fn tol`, `@inline fn is_content_length` 추가
- 1.186× → 1.099× (7% 비율 개선)

**4. str_data literal P0 fix (Cycle 2931 — llvm_text.rs)**:
- 버그: `let s = "literal"; str_data(s)` → SEGV (상수 전파된 Constant::String이 raw bytes 사용)
- 수정: `bmb/src/codegen/llvm_text.rs:5699` — `Constant::String` 분기 추가, `@.str.0.bmb` (struct) 사용
- 검증: `test_str_data_literal.bmb` + .expected

**5. Bootstrap Fixed Point 방법론 정정 (Cycle 2930)**:
- GCC MinGW-w64 링커 비결정적: 동일 소스 2회 빌드도 binary hash 다름
- 올바른 방법: IR hash 비교 (S3 IR == S4 IR for multiple test files ✓)
- CLAUDE.md 업데이트 권장 (아직 미완)

### 테스트 변화
6249+ tests (cargo test --release: 3778 + 2388 + 47 + 13 + 23), 0 FAILED.
test_str_data_literal.bmb 신규 (bootstrap + Rust backend 양쪽 확인).

---

---

## 이번 세션 작업 요약 (Cycles 2918-2925)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 2918 | lexer+brainfuck inproc (Phase 1) | `time_ns()` 기반 harness. lexer 0.169× ✅ PASS, brainfuck 1.21× ⚠️ 조건부 |
| 2919 | csv_parse+http_parse inproc (Phase 2) | csv_parse 4.06× FAIL, http_parse 1.255× ⚠️ 조건부 |
| 2920 | json_parse+json_serialize inproc (Phase 3) | json_parse 0.829× ✅ PASS, json_serialize 0.715× ✅ PASS |
| 2921 | sorting inproc (Phase 4) | C main_inproc.c 신규. BMB 0.156× ✅ PASS (6.41× faster) |
| 2922 | ISSUE close + 요약 문서 | ISSUE-20260512 CLOSED. tier3_inproc_summary 신규 |
| 2923 | csv_parse 최적화 | tuple return + 단일패스. 4.06× FAIL → 1.148× ⚠️ 조건부 |
| 2924 | http_parse 사전 할당 | 5 String 사전 생성. 1.255× → 1.186× ⚠️ 조건부 |
| 2925 | 회귀 검증 + ROADMAP 갱신 | cargo test 6249+ passed, 0 FAIL. ROADMAP 갱신 완료 |

### tier3 inproc 최종 결과

| 벤치마크 | BMB (µs) | C GCC (µs) | 비율 | 판정 |
|---------|----------|-----------|------|------|
| lexer | 1140 | 6740 | 0.169× | ✅ PASS (5.9× faster) |
| brainfuck | 2065 | 1707 | 1.21× | ⚠️ 조건부 (heap vs stack) |
| csv_parse | 3423 | 2982 | 1.148× | ⚠️ 조건부 (Cycle 2923 최적화) |
| http_parse | 2906 | 2451 | 1.186× | ⚠️ 조건부 (Cycle 2924 최적화) |
| json_parse | 2537 | 3062 | 0.829× | ✅ PASS (1.21× faster) |
| json_serialize | 467 | 653 | 0.715× | ✅ PASS (1.40× faster) |
| sorting | 471670 | 3023238 | 0.156× | ✅ PASS (6.41× faster) |

**요약**: 4 PASS / 3 조건부 / 0 FAIL — ISSUE-20260512 CLOSED

### 조건부 원인 분석 (구조적 한계)

| 벤치마크 | 원인 |
|---------|-----|
| brainfuck | heap malloc tape vs C stack array (언어 기능 필요) |
| csv_parse | `byte_at()` 간접 접근 overhead 누적 |
| http_parse | `byte_at()` 간접 접근 vs C `char*` 직접 포인터 |

### 변경 파일 (이번 세션)

- `ecosystem/benchmark-bmb/benches/real_world/sorting/c/main_inproc.c` — 신규 (C inproc harness)
- `ecosystem/benchmark-bmb/benches/real_world/csv_parse/bmb/main.bmb` — 전면 재작성 (tuple + 단일패스)
- `ecosystem/benchmark-bmb/benches/real_world/csv_parse/bmb/main_inproc.bmb` — 전면 재작성
- `ecosystem/benchmark-bmb/benches/real_world/http_parse/bmb/main_inproc.bmb` — 사전 할당 최적화
- `claudedocs/measurements/tier3_inproc_summary_2026-05-19.md` — 신규
- `claudedocs/issues/closed/ISSUE-20260512-tier3-spawn-overhead-methodology.md` — 이동+CLOSED
- `claudedocs/ROADMAP.md` — Cycles 2918-2924 갱신
- `claudedocs/cycle-logs/cycle-2918.md` ~ `cycle-2925.md` — 신규 8개

### 테스트 변화
6249+ tests (cargo test --release: 3778 + 2388 + 47 + 13 + 23), 0 FAILED.
bootstrap 변경 없음 → 3-Stage Fixed Point 유지.

---

## 이번 세션 작업 요약 (Cycles 2915-2917)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 2915 | Always FAIL 진단 1 | 15 placeholder problem.md 수정 (31-45) + 25/28/71/99 근본 원인 진단 |
| 2916 | Always FAIL 진단 2 | 79/89/90/91 진단+수정, bmb_reference 링 버퍼 패턴 추가 |
| 2917 | GPUStack 재측정 | Always FAIL 11 → 0 (100% pass), 추정 B축 85.0% → 96.0% |

### B축 현황

| 모델 | Success Rate | 측정일 | 비고 |
|------|-------------|--------|------|
| claude-sonnet-4-6 | **98.0%** | 2026-05-13 | 공식 baseline (stale 기한 2026-08-13) |
| qwen3.6-35b-a3b (Cycle 2914) | **85.0%** | 2026-05-18 | Always FAIL 11문제 포함 |
| qwen3.6-35b-a3b (Cycle 2917) | **96.0% (추정)** | 2026-05-18 | Targeted retest: 11문제 100%, 나머지 동일 가정 |

**Always FAIL 11문제 수정 목록**:
- 25_range_clamp: `clamp_val` 이름 충돌 경고
- 28_positive_factorial: main() contract 금지 설명
- 34, 39, 41: placeholder → 완전한 문제 설명
- 71_single_element: 설명 오류 완전 수정 (first/last/count)
- 79_mini_interpreter: op5=DUP, op6=print-without-pop 수정
- 89_topological_sort: O(n*m) BFS 알고리즘 힌트 추가
- 91_ring_buffer: overwrite-oldest 의미론 수정
- 90, 99: bmb_reference 강화 (`;` 패턴, vec_pop CRITICAL, 링 버퍼 패턴)

### 테스트 변화
2388 tests (변화 없음). bootstrap 변경 없음.

---

---

## 이번 세션 작업 요약 (Cycles 2908-2914)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 2908 | bmb-algo C 바인딩 | `bindings/c/` 생성 (76 tests / 55 함수) |
| 2909 | bmb-compute C 바인딩 | `bindings/c/` 생성 (56 tests / 33 함수) |
| 2910 | bmb-crypto C 바인딩 | `bindings/c/` 생성 (23 tests / 14 함수) + arena-free 규칙 발견 |
| 2911 | bmb-text C 바인딩 | `bindings/c/` 생성 (33 tests / 23 함수) |
| 2912 | bmb-json C 바인딩 | `bindings/c/` 생성 (28 tests / 12 함수) |
| 2913 | ROADMAP 갱신 + Rule 9 | M4-14 항목 추가 + .gitignore + 조기 종료 |
| 2914 | GPUStack B축 측정 | qwen3.6-35b-a3b 85.0% (255/300) + bmb-ai-bench GPUSTACK_* 연동 |

### 테스트 변화
2388 tests (변화 없음). C 바인딩: 216개 C 테스트 (별도 GCC 빌드).

---

## C 바인딩 현황

| 라이브러리 | 파일 | 테스트 | 함수 수 |
|-----------|------|--------|---------|
| bmb-algo   | ✅ Cycle 2908 | 76 | 55 |
| bmb-compute | ✅ Cycle 2909 | 56 | 33 |
| bmb-crypto | ✅ Cycle 2910 | 23 | 14 |
| bmb-text   | ✅ Cycle 2911 | 33 | 23 |
| bmb-json   | ✅ Cycle 2912 | 28 | 12 |

총 **216 C tests** (5개 라이브러리). GCC + DLL 직접 링크.

### M4 ④ 바인딩 완성도

| 언어 | 상태 | 완료 Cycle |
|------|------|-----------|
| Python | ✅ | Cycle 2649 |
| Node.js | ✅ | Cycle 2556 |
| C#     | ✅ | Cycle 2897 |
| Java   | ✅ | Cycle 2904 |
| **C**  | ✅ | **Cycle 2908-2912** |

---

## arena-free 규칙 (신규 확립 — Cycle 2910)

C 바인딩에서 `@export` 반환 String은 arena-backed:
- **입력** (`bmb_ffi_cstr_to_string` 결과) → `bmb_ffi_free_string` 호출 필수
- **출력** (`@export` 함수 반환값) → `bmb_ffi_free_string` 절대 금지, `bmb_ffi_end()` 전에 데이터 읽기

위반 시 `STATUS_HEAP_CORRUPTION (0xC0000374)` 발생.
C 바인딩 README 각각에 CRITICAL 섹션으로 문서화.

---

## 변경 파일 (이번 세션)

**C 바인딩 신규** (각 4파일 × 5라이브러리 = 20파일):
- `ecosystem/bmb-{algo,compute,crypto,text,json}/bindings/c/Makefile`
- `ecosystem/bmb-{algo,compute,crypto,text,json}/bindings/c/example.c`
- `ecosystem/bmb-{algo,compute,crypto,text,json}/bindings/c/test.c`
- `ecosystem/bmb-{algo,compute,crypto,text,json}/bindings/c/README.md`

**설정 갱신**:
- `.gitignore`: `ecosystem/bmb-*/bindings/c/*.dll/.so/.dylib` 추가

**문서 갱신**:
- `claudedocs/ROADMAP.md`: M4-14 C 바인딩 ✅ 항목 추가, 헤더 갱신
- `claudedocs/cycle-logs/cycle-2908.md` ~ `cycle-2913.md`

---

## 다음 세션 우선순위

### Carry-Forward (Actionable)
- **없음** — str_data P0 fix + http_parse flat + Bootstrap Fixed Point 완료

### Pending Human Decisions
- **GPUStack B축 실제 재측정**: `.env.local` 필요. qwen3.6-35b-a3b Cycle 2917 추정 96.0% (실측 필요).
- **Claude B축 재측정**: Stale 기한 2026-08-13 (아직 유효).
- **i32 타입 추가**: ≤1.05× 유일한 경로 (언어 스펙 변경, Level 1 Decision). Pending Human 확인 필요.

### 다음 자율 작업 권장 (Cycle 2933+)
- **언어 갭 추가 해소** — 고차함수/제너릭 등 미구현 BMB 언어 기능 (HANDOFF 최우선 자율 작업)
- **brainfuck stack array** (Long-term): 언어 기능 추가 필요 (고정 크기 stack array)
- **CLAUDE.md Fixed Point 방법론 정정**: binary hash → IR hash 비교 (Cycle 2930 발견)

---

## 세션 종료 정리 (2026-05-19 최신 — Cycle 2932 포함)

### 최종 커밋 이력
| SHA | 내용 |
|-----|------|
| `07884ec1` | chore: 세션 종료 정리 — HANDOFF/ROADMAP 최종 갱신 (Cycle 2927) |
| *(이번 세션)* | feat(cycles-2928-2932): str_data builtin+http_parse flat+P0 fix+Bootstrap IR Fixed Point |

### 테스트 상태
- `cargo test --release`: 6249+ passed, 0 failed ✅
- IR Fixed Point: S3 IR == S4 IR (3 test files verified, Cycle 2930)
- str_data literal P0 fix: test_str_data_literal.bmb 신규 ✅

### 다음 세션 진입 체크리스트
- [ ] `claudedocs/HANDOFF.md` HEAD 확인 (최신 커밋 SHA)
- [ ] Cycle 2933 시작 — 언어 갭 해소 (고차함수/제너릭) 또는 i32 타입 추가 (HUMAN 결정 있으면)
- [ ] Pending Human Decisions 재확인:
  - GPUStack B축 실측 (`.env.local` 필요)
  - Claude B축 재측정 (stale 기한 2026-08-13)
  - npm/PyPI publish (M3-3/M3-4)
