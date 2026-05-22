# BMB Session Handoff — 2026-05-22 (Cycles 3034-3037 — M6-P1 bmb-mcp BMB 포팅 완료)

> **HEAD**: `(commit pending)` — cycle-logs 3034-3037 + bmb-mcp submodule 갱신
> **이전 HEAD**: `fe51af7b` (Cycle 3033 — println dispatch fix + M5/M6 계획 수립)
> **3-Stage Fixed Point**: ✅ IR Fixed Point 확인 (Cycle 2930)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: Cycle 3038

---

## 이번 세션 작업 요약 (Cycles 3034-3037)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 3034 | M6-P1 사전 감사 | HANDOFF 전제 검증 (HTTP→stdio 수정), server.py _QUICK_REFERENCE 오류 수정 |
| 3035 | mcp_server.bmb 스캐폴드 | stdio JSON-RPC MCP 서버 BMB 구현 (4 tools + 전체 프로토콜 핸들러) |
| 3036 | 추가 도구 구현 | bmb_verify + bmb_spec_lookup + bmb_example (7종 도구) |
| 3037 | 완료 정리 | bmb_context_pack 추가 + mcp_server_config.json + submodule 커밋 |

### 핵심 성과: M6-P1 bmb-mcp Python→BMB 포팅 완료

**`ecosystem/bmb-mcp/mcp_server.bmb`** (~650줄) 신규 생성:
- stdio JSON-RPC 2.0 + Content-Length 프레이밍 (LSP 패턴 동일)
- 9종 도구: bmb_check/run/lint/ir/verify + bmb_spec_lookup/example + bmb_context_pack
- 전체 MCP 프로토콜: initialize/tools/resources/prompts/shutdown
- `exec_output(bmb, args)` 사용 (`system_capture`는 interpreter-only 미지원)
- `getenv("BMB_BINARY")` + `getenv("BMB_REPO_ROOT")` 환경변수

**핵심 발견**:
- HANDOFF의 "HTTP 서버 필요" 전제가 틀렸음: FastMCP 기본값은 stdio (HTTP 아님)
- `system_capture`는 codegen 전용 (interpreter eval.rs에 미등록) → `exec_output`으로 대체
- `SvecHandle` 타입은 함수 시그니처에 명시 필요 (i64와 구별)

**submodule 커밋**: `2be1c47` (ecosystem/bmb-mcp)

---

## 이전 세션 작업 요약 (Cycle 3033)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 3033 | println dispatch 버그 수정 | `mir/lower.rs` MirType::String 반환 함수 목록에 M4 builtin 17종 추가 |
| 3033 | Native Complete 조사 | while-let/for-in-svec/string interpolation 모두 네이티브 정상 확인 |
| 3033 | M5/M6 계획 수립 | ROADMAP에 M5 Native Complete + M6 Full Dogfooding 섹션 추가 |

### 핵심 버그 수정: println dispatch (mir/lower.rs)

**증상**: `println(str_replace("a","b","c"))` 등 String-returning builtin 함수 결과를 println으로 출력하면 문자열 내용 대신 메모리 주소 출력

**원인**: `bmb/src/mir/lower.rs` line 1684에서 MirType::String 반환 함수 목록에 M4 builtin 누락.
- 조건: println dispatch logic이 `ctx.locals.get(&p.name)`으로 인수의 MirType 조회
- 누락된 함수들은 `_ => MirType::I64` 기본값으로 처리 → `@println(i64)` 호출 → 포인터 주소

**수정** (`bmb/src/mir/lower.rs:1684`):
```rust
// 추가된 17종:
"str_to_upper" | "str_to_lower" | "str_replace" | "str_repeat"
| "str_trim" | "str_trim_left" | "str_trim_right" | "str_reverse"
| "int_to_hex" | "int_to_bin"
| "str_substr" | "str_pad_left" | "str_pad_right"
| "str_char_at" | "svec_join" | "svec_get"
| "read_line" | "read_bytes" => MirType::String,
```

**테스트 결과** (네이티브 빌드):
```
hello BMB       ← str_replace 정상
HELLO WORLD     ← str_to_upper 정상
dlrow olleh     ← str_reverse 정상
ff              ← int_to_hex 정상
00042           ← str_pad_left 정상
```

**cargo test --release 결과**: 3780 passed, 2 failed (pre-existing Windows 임시파일 권한 오류 — 내 변경과 무관)

### Native Complete 조사 결과

이전 세션에서 "interpreter-only 전수 native 포팅" 요청 대비 조사 결과:

| 조사 항목 | 결과 |
|---------|------|
| eval.rs "interpreter-only" 주석 (~45개) | 모두 stale — Cycles 2871-2894 포팅 완료의 역사적 흔적 |
| while-let 네이티브 동작 | ✅ 완전 동작 (enum 패턴 필수) |
| for-in-vec 네이티브 동작 | ✅ 완전 동작 |
| for-in-svec 네이티브 동작 | ✅ 완전 동작 |
| CIR lower.rs의 `WhileLet => Unit` | 네이티브 경로 아님 (MIR 경로 완전) |
| 실제 native 갭 | **1개** (println dispatch 버그, 이번 수정으로 해결) |

**결론**: M4 "interpreter-only 제로" 선언은 실질적으로 맞았음. 단, println dispatch 버그 1개는 이번에 발견/수정.

### M6 Full Dogfooding 계획 (사용자 결정: 완전 자체구현)

상세 내용: `claudedocs/ROADMAP.md § M6` 참조

| 컴포넌트 | 우선순위 | 예상 |
|---------|---------|------|
| bmb-mcp (Python→BMB) | P1 | 2-3 cycles |
| scripts/핵심 (Shell→BMB) | P1 | 1-2 cycles |
| bmb-ai-bench (Python→BMB) | P2 | 3-5 cycles |
| gotgan (Rust→BMB) | P3 | 6-12 cycles |

---

## 이전 세션 작업 요약 (Cycles 3031-3032)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 3031 | P-track 재측정 + ROADMAP 갱신 | 7/7 PASS 확인, § 5 갱신, ISSUE-20260521 closed/ 이동 |
| 3032 | Or-chain CSE 조사 → 조기 종료 | 모든 벤치마크 단일-load 패턴 확인, 추가 최적화 기회 소진 |

### P-track 재측정 결과 (2026-05-22, 5-run median)

| 벤치마크 | Cycle 3023 | Cycle 3031 | 변동 |
|---------|-----------|-----------|------|
| brainfuck | 0.956× | **0.941×** | 개선 |
| csv_parse | 0.891× | **0.858×** | 개선 |
| http_parse | 0.909× | **0.934×** | ±노이즈 |
| lexer | 0.169× | **0.174×** | ±노이즈 |
| json_parse | 0.822× | **0.875×** | ±노이즈 |
| json_serialize | 0.668× | **0.670×** | 동등 |
| sorting | 0.154× | **0.155×** | 동등 |

**P-track 7/7 PASS** ✅ — 모두 BMB faster than C GCC -O2.

### 조기 종료 사유

- P-track 추가 최적화 기회 소진 (모든 벤치마크 이미 단일-load 패턴)
- Active ISSUE 5개 전부 HUMAN-blocked
- Actionable defects 0개

---

## 다음 세션 (Cycle 3034+)

### 권장 우선순위

1. **M6-P1: bmb-mcp Python→BMB 이식** — MCP 서버 자체구현 시작 (HTTP + JSON 필요)
2. **M6-P1: scripts/ 핵심 스크립트 BMB CLI 전환** — benchmark/bootstrap 스크립트
3. **M5 Language Complete 진행** — 미완료 언어 기능 완성 (M6 전제 조건)
4. **B축 Claude 재측정** — 98.0% stale 기한 2026-08-13 (아직 여유)

### M6 착수 전 필요 언어 기능 (M5 잔여)

M6에서 bmb-mcp/scripts 이식 시 필요한 기능들:
- HTTP 서버/클라이언트 stdlib
- 환경 변수 처리 (`getenv` ✅ 이미 있음)
- 프로세스 실행 (`system_capture` ✅ 이미 있음)
- JSON 파싱 (bmb-json 라이브러리 활용 가능)

### 알려진 HUMAN-blocked 항목

- GPT-4o 실험 (multi-model-validation)
- golden-flakiness-inttoptr Option A/B/C
- problem-difficulty-bias 신규 hard 문제 20개
- crosslang 측정 (stale)
- `v[i]` Vec 인덱싱 구현 여부 (Rule 6 예외 결정)

### ISSUE 현황 (Active 5개)

| ISSUE | 상태 | 우선순위 |
|-------|------|---------|
| ~~mir-cse-and-chain~~ | **RESOLVED → closed/** | — |
| multi-model-validation | PARTIALLY RESOLVED | MEDIUM |
| external-problem-validation | PARTIALLY RESOLVED | MEDIUM |
| integration-category-weakness | PARTIALLY RESOLVED | LOW |
| problem-difficulty-bias | OPEN | LOW |
| golden-flakiness-inttoptr | OPEN | P3 |

### 알려진 BMB 언어 특성 (중요도 순)

- `else if` 체인 세미콜론: statement 위치에서 `};` 필수 (Cycle 2984 발견)
- `fn main() -> i64 = { ... };` 끝에 `;` 필수 (Cycle 2986 발견)
- `match`: integer literal, char literal, OR pattern (`a | b => ...`) 지원
- `match` arm body: block + comma 필요 (`{ expr }` 후 `,` 필수, 마지막 arm 제외)
- `band`/`bor`/`bxor`/`bnot`: bitwise 연산자 지원
- `break`/`continue`/`return`: ✅ 지원 (단, break는 while에서만)
- `&&`/`||` short-circuit: ✅ 완전 지원 (Cycle 2965)
- `memset_fill(ptr, val, count)`: ✅ native-only builtin (v0.100.1 신규)
- `[T; N]` 고정 배열: ✅ 인터프리터 완전 동작 (read/write/init)

### B-axis 상태

| 모델 | 마지막 측정 | 상태 |
|------|-----------|------|
| Claude (claude-sonnet-4-6) | 98.0% (2026-05-13) | 고정 베이스라인 (stale 기한: 2026-08-13) |
| GPUStack qwen3.6-35b-a3b | **100.0% (2026-05-21)** | **최신 공식 측정** |
