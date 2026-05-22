# Cycle 3044: M6-P2 feasibility probe + exec_with_stdin codegen 검증
Date: 2026-05-22

## Re-plan
Carry-forward (Cycle 3043): exec_with_stdin codegen 검증 + M6-P2 착수 결정.
이번 사이클: (1) probe-llm-pipeline.bmb 작성으로 ai-bench 파이프라인 전체 검증,
(2) native 빌드로 exec_with_stdin codegen 검증.

## Scope & Implementation

**probe-llm-pipeline.bmb** (신규, `scripts/`):
- 풀 파이프라인: curl → GPUStack LLM API → JSON parse → ```bmb 블록 추출 → write_file → bmb check → 에러 감지
- `find_jval` (기존 run-bench-tests.bmb 패턴 재사용): `"content":` 필드 추출
- `extract_code_block`: ` ```bmb\n...``` ` 패턴 탐지 + str_trim
- `build_request`: OpenAI-compatible JSON body 빌더
  - GPUStack qwen3 필수: `"chat_template_kwargs":{"enable_thinking":false}`, `max_tokens:16384`
- `has_error`: `"type":"error"` JSON 패턴으로 compile error 감지 (exit-code 대체)
- `print_check_output`: helper fn 분리 (codegen 버그 우회)

**test-exec-with-stdin-native.bmb** (신규):
- exec_with_stdin native 빌드 최소 검증 스크립트
- curl --version + curl health endpoint 두 가지 테스트

## Verification & Defect Resolution

**인터프리터 모드 결과**:
1. LLM API (GPUStack qwen3): ✅ (응답 1019-1026 bytes, enable_thinking:false 필수)
2. JSON content 필드 추출: ✅ (jstr_loop로 \n/\"/\\ 이스케이프 처리)
3. ```bmb 코드 블록 추출: ✅ (283 chars fibonacci 코드)
4. write_file + bmb check: ✅ (임시파일 작성 + 체크 실행)
5. 에러 감지: ✅ (`"type":"error"` 패턴으로 컴파일 에러 정확 감지)

**native 빌드 결과**:
- exec_with_stdin codegen: ✅ BUILD SUCCESS + RUNTIME OK
  - `curl --version` → 380 bytes (exec_with_stdin 정상 동작)
  - `curl health` → 22 bytes (HTTP 통신 정상)

**발견된 defect — P0 codegen 버그**:
- 패턴: `let var = builtin_fn(...)` (getenv/exec_with_stdin) 이후 `"literal" + var` concat 시
- 증상: `getelementptr inbounds i64, ptr "literal", i64 %var_offset_load` — GEP 잘못 생성
- 원인: text backend step_expr 반복 lowering에서 String 변수 주소를 i64로 load
- 영향: native 빌드만 실패 (인터프리터 정상)
- 우회: var를 함수 파라미터로 전달하면 ptr로 올바르게 처리됨
- 조치: ISSUE 등록 (Cycle 3045에서 처리)

## Reflection
- exec_with_stdin codegen: 검증 완료 — 기본 구조 (compile + link + runtime) 정상
- Exit-code gap 평가: `bmb check` 출력에서 `"type":"error"` 패턴으로 충분히 대체 가능
  - exec_with_stdin_ex (extended builtin) 불필요
- GPUStack qwen3: thinking mode disabled 필수 — Python bmb_ai_bench와 동일 설정 확인
- 전체 파이프라인이 BMB 단일 스크립트로 동작 — M6-P2 착수 가능 확인

## Carry-Forward
- Actionable: P0 codegen 버그 ISSUE 등록 (`String builtin-var + literal concat → GEP`)
- Structural Improvement Proposals:
  - probe-llm-pipeline.bmb: `.env.local` 파싱 추가 (M6-P2 본구현에서)
  - JSON escape function 구현 필요 (problem.md 내용을 JSON body에 주입 시)
- Pending Human Decisions: 없음
- Roadmap Revisions: exec_with_stdin codegen 검증 ✅ 완료 (잔여 ISSUE: native String concat bug)
- Next Recommendation: Cycle 3045 — codegen ISSUE 등록 + M6-P2 bmb-ai-bench 포팅 설계 착수
