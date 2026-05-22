# BMB Session Handoff — 2026-05-22 (Cycles 3038-3043 — M6-P1 scripts BMB 포팅 완료)

> **HEAD**: `78719ac8` (feat(cycle-3041): run-all-bench-tests.bmb — 1230/1230 (100%) pass)
> **이전 HEAD**: `2e9c4910` (chore(M6-P1): bmb-mcp BMB 포팅 완료)
> **3-Stage Fixed Point**: ✅ IR Fixed Point 확인 (Cycle 2930)
> **실무 앵커**: `claudedocs/ROADMAP.md`
> **다음 세션 진입점**: Cycle 3044

---

## 이번 세션 작업 요약 (Cycles 3038-3043)

### 주요 변경 사항

| Cycle | 제목 | 내용 |
|-------|------|------|
| 3038 | exec_with_stdin 1차 | file_mtime builtin + rebuild scripts BMB 포팅 |
| 3039 | exec_with_stdin 완성 | script_args builtin + check-version-sync + scripts 업그레이드 |
| 3040 | run-bench-tests.bmb | exec_with_stdin + JSON 파서 + 테스트 러너 완성 |
| 3041 | run-all-bench-tests.bmb | 100문제 일괄 실행 — **1230/1230 (100%) PASS** |
| 3042 | ROADMAP 업데이트 | M6-P1 완료 표시, M6 현황 갱신 |
| 3043 | HANDOFF 업데이트 | 세션 종료 정리 |

### 핵심 성과: M6-P1 scripts 완전 완료

**`exec_with_stdin` 빌트인** (신규):
- 시그니처: `exec_with_stdin(cmd: String, args: String, stdin: String) -> String`
- 구현: `bmb/src/interp/eval.rs` (Rust subprocess + piped stdin)
- 타입 등록: `bmb/src/types/mod.rs`, codegen (`llvm_text.rs`, `llvm.rs`)
- 런타임: `bmb/runtime/bmb_runtime.c` (Windows CreatePipe/CreateProcess + POSIX fork/pipe)

**새 BMB 스크립트** (scripts/):
- `run-bench-tests.bmb` — JSON 파서 + exec 기반 테스트 러너 (15/15 × 3문제 검증)
- `run-all-bench-tests.bmb` — 100문제 일괄 실행 + 집계 (1230/1230 PASS)
- `rebuild-runtime.bmb` — C 런타임 재빌드 자동화 (Cycle 3038)
- `rebuild-bootstrap-exe.bmb` — Bootstrap exe 재빌드 자동화 (Cycle 3038)
- `check-version-sync.bmb` — 버전 동기화 검사 (Cycle 3039)

**핵심 버그 및 해결**:
- `s.char_at(pos)` 메서드가 `String` 반환 (NOT i64) → `s.char_code_at(pos)` 사용 (i64 반환)
- `file_read` 미등록 → `read_file` (올바른 빌트인 이름)
- `SvecHandle` 파라미터는 타입 어노테이션에 `SvecHandle` 명시 필요 (i64와 구별)
- `read_int()` 줄 단위 파싱 — 공백 구분 stdin은 `str_replace(s, " ", "\n")` 사전 변환 필요

---

## 이전 세션 작업 요약 (Cycles 3034-3037)

### M6-P1 bmb-mcp Python→BMB 포팅 완료

**`ecosystem/bmb-mcp/mcp_server.bmb`** (~650줄):
- stdio JSON-RPC 2.0 + Content-Length 프레이밍 (LSP 패턴 동일)
- 9종 도구: bmb_check/run/lint/ir/verify + bmb_spec_lookup/example + bmb_context_pack
- 전체 MCP 프로토콜: initialize/tools/resources/prompts/shutdown
- `exec_output(bmb, args)` 사용 (`system_capture`는 interpreter-only 미지원)
- `getenv("BMB_BINARY")` + `getenv("BMB_REPO_ROOT")` 환경변수

---

## M6 현황 (2026-05-22)

| 컴포넌트 | 상태 | 비고 |
|---------|------|------|
| `bootstrap/compiler.bmb` | ✅ 완료 | 3-Stage Fixed Point |
| `bootstrap/lsp.bmb` | ✅ 완료 | |
| `bootstrap/lint.bmb` | ✅ 완료 | |
| `ecosystem/bmb-mcp/` | ✅ 완료 | Cycle 3037, 9종 도구 |
| `scripts/*.bmb` (핵심 5종) | ✅ 완료 | Cycles 3038-3041 |
| `ecosystem/bmb-ai-bench/` | ❌ 미이식 | P2 다음 단계 |
| `gotgan/` | ❌ 미이식 | P3 장기 |
| `ecosystem/playground/` | ❌ (WASM 일부) | |

**M6-P1 완료** ✅ — bmb-mcp + scripts 핵심 5종 모두 BMB 자체구현 달성

---

## 다음 세션 (Cycle 3044+)

### 권장 우선순위

1. **M6-P2: bmb-ai-bench Python→BMB 이식** — HTTP 클라이언트 + JSON + 파일 I/O (복잡, 3-5 cycles)
2. **exec_with_stdin codegen 검증** — bmb_runtime.c 구현 존재하나 네이티브 실행 미검증
3. **B축 Claude 재측정** — 98.0% stale 기한 2026-08-13 (아직 여유)

### M6-P2 착수 전 필요 언어 기능 체크

bmb-ai-bench 이식에 필요한 기능들:
- HTTP 클라이언트 (`http_post` 등) — 미구현 (신규 builtin 필요)
- JSON 직렬화/역직렬화 — `bmb-json` 라이브러리 or BMB 자체 구현
- 환경 변수 처리 (`getenv` ✅)
- 파일 I/O (`read_file`, `write_file`, `file_exists`, `list_dir` ✅)
- 프로세스 실행 (`exec_with_stdin` ✅)

### 알려진 BMB 언어 특성 (중요도 순)

- `s.char_at(pos)` → `String` 반환, `s.char_code_at(pos)` → `i64` 반환 (혼동 주의)
- `else if` 체인 세미콜론: statement 위치에서 `};` 필수 (Cycle 2984 발견)
- `fn main() -> i64 = { ... };` 끝에 `;` 필수 (Cycle 2986 발견)
- `SvecHandle`: 함수 파라미터에 타입 명시 필수 (`entries: SvecHandle` NOT `entries: i64`)
- `read_int()` 줄 단위 파싱 — 공백 구분 입력은 `str_replace(s, " ", "\n")` 변환 필요
- `&&`/`||` short-circuit: ✅ 완전 지원 (Cycle 2965)
- `memset_fill(ptr, val, count)`: ✅ native-only builtin (v0.100.1 신규)

### ISSUE 현황 (Active 5개)

| ISSUE | 상태 | 우선순위 |
|-------|------|---------|
| multi-model-validation | PARTIALLY RESOLVED | MEDIUM |
| external-problem-validation | PARTIALLY RESOLVED | MEDIUM |
| integration-category-weakness | PARTIALLY RESOLVED | LOW |
| problem-difficulty-bias | OPEN | LOW |
| golden-flakiness-inttoptr | OPEN | P3 |

### B-axis 상태

| 모델 | 마지막 측정 | 상태 |
|------|-----------|------|
| Claude (claude-sonnet-4-6) | 98.0% (2026-05-13) | 고정 베이스라인 (stale 기한: 2026-08-13) |
| GPUStack qwen3.6-35b-a3b | **100.0% (2026-05-21)** | **최신 공식 측정** |
