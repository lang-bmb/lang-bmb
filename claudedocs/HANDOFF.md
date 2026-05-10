# BMB Session Handoff -- 2026-05-10 (Cycles 2609-2618 -- Track S ~97% + CI 통합)

> **이전 HEAD**: `196a02e9` (Track S LSP 69-test + verify-host Z3 IPC)
> **새 HEAD**: TBD (push pending)
> **세션 성격**: 10-cycle run-cycle -- Track S 완성 + CI 통합
> **핵심 성과**: Track S ~92% → ~97%. LSP 92-test + verify-host 48-test. CI job 추가.

---

## 1. 이번 세션 산출물 (Cycles 2609-2618)

### Cycle 2609 -- LSP completion from current file symbols
- `handle_completion`: URI → `read_file` → `scan_for_completion` → 파일 심볼 추가
- `sk_to_ck`: 워크스페이스 심볼 kind → completion item kind 변환
- `scan_for_completion`: while-loop 기반 fn/struct/enum 스캔
- **79/79 PASS** (이전: 69/69)

### Cycle 2610 -- verify-host incremental verification (파일 레벨 캐시)
- `file_hash(content)`: djb2 while-loop 해시 (`hash:length` 형식)
- `cache_path`, `read_cache`, `write_cache`: `<file>.vh_cache` 캐시 시스템
- `main()` 수정: 파일 hash → cache hit/miss 분기
- **42/42 PASS** (이전: 33/33)

### Cycle 2611 -- verify-host proof database (Z3 conditions 캐시) + .gitignore
- `proof_db_path`: `<file>.vh_proofdb`
- `run_z3_direct` 수정: pre/post 조건 hash 기반 Z3 결과 캐시 (file_hash/read_cache/write_cache 재활용)
- `.gitignore`: `*.vh_cache`, `*.vh_proofdb` 추가
- **48/48 PASS** (이전: 42/42)

### Cycle 2612 -- LSP code actions (pre/post condition stubs)
- `codeActionProvider: true` capability 추가 (10 capabilities)
- `textDocument/codeAction` 핸들러 추가
- `line_at_loop`: while-loop 기반 라인 추출 (재귀 방지)
- `ca_make_edit`, `ca_make_action`: WorkspaceEdit 기반 code action JSON 생성
- `build_code_actions`: fn 라인 감지 → "Add pre/post condition" 제공
- **88/88 PASS** (이전: 79/79)

### Cycle 2613 -- CI 통합 + 플랫폼 독립 수정
- `lsp_test.py`, `verify_host_test.py`: `_EXE` 플랫폼 변수 추가 (Linux/macOS 지원)
- `.github/workflows/ci.yml`: `track-s-tests` job 추가
  - `ubuntu-latest`, `sudo apt-get install -y llvm`
  - `bmb build bootstrap/lsp.bmb -o bootstrap/lsp`
  - `python3 bootstrap/lsp_test.py` (88 tests)
  - `bmb build bootstrap/verify_host.bmb -o bootstrap/verify_host`
  - `python3 bootstrap/verify_host_test.py` (48 tests)

### Cycle 2614 -- LSP completion detail 개선 (함수 시그니처)
- `extract_fn_sig_at(src, fn_pos)`: fn 위치에서 직접 시그니처 추출 (sh_find_char_fwd 재활용)
- `scan_for_completion` 수정: 함수에 "user function" 대신 전체 시그니처 detail 제공
- **92/92 PASS** (이전: 88/88)

---

## 2. 현재 상태

### Track 스냅샷

| Track | % | 상태 |
|-------|---|------|
| M (Machine-First) | ~100% ✅ | 완료 |
| N (MCP Server) | ~99% ✅ | 13 tools / 4 resources / 3 prompts / 90 pytest |
| O (Context Pack) | ~95% ✅ | `uses` 의존성 그래프 포함 |
| Q (Ambiguity Audit) | ~92% ✅ | 10 checks, 90 pytest, CI "10 checks" |
| R (LLM Bench) | ~95% ✅ | run+analyze 파이프라인 완성, 30 pytest |
| S (BMB-rewrite) | ~97% ✅ | LSP ~99% + verify-host ~88% = 전체 ~97% |
| T (External Bindings) | ~95% ✅ | Node.js 5/5, npm-publish.yml |

### Track S 세부 현황

**LSP (bootstrap/lsp.bmb, ~1380 LOC)**:
- ✅ initialize (10 capabilities, codeActionProvider 포함)
- ✅ textDocument/hover (키워드 + 사용자 정의 함수 시그니처)
- ✅ textDocument/completion (37 static + file symbols, 함수 시그니처 detail)
- ✅ textDocument/publishDiagnostics (bmb check 연동)
- ✅ textDocument/documentSymbol
- ✅ textDocument/definition
- ✅ textDocument/references
- ✅ workspace/symbol (BMB_WORKSPACE env, while-loop 스캔)
- ✅ textDocument/signatureHelp (activeParameter, parameter labels)
- ✅ textDocument/codeAction ("Add pre/post condition" stubs)
- ✅ shutdown + exit
- **테스트**: 92/92 PASS

**verify-host (bootstrap/verify_host.bmb, ~575 LOC)**:
- ✅ BMB_FILE env var + stdin fallback
- ✅ `bmb check` → 에러/경고 파싱 → JSON
- ✅ `bmb verify` → verify_result 파싱 → JSON
- ✅ Z3 직접 IPC (pre/post 추출 → SMT-LIB2 → Z3 실행)
- ✅ incremental verification: 파일 레벨 캐시 (`<file>.vh_cache`)
- ✅ proof database: Z3 조건 레벨 캐시 (`<file>.vh_proofdb`)
- ✅ CI type check
- ✅ verify_host_test.py 48-test suite
- ⬜ 함수 레벨 incremental verification 미구현
- **테스트**: 48/48 PASS

### 테스트 현황

| 스위트 | 결과 |
|--------|------|
| `cargo nextest run --release` | ✅ 6210 passed |
| `python3 bootstrap/lsp_test.py` | ✅ 92 passed |
| `python3 bootstrap/verify_host_test.py` | ✅ 48 passed |
| `bmb-ai-bench pytest` | ✅ 30 passed |
| `bmb-mcp pytest` | ✅ 90 passed |

---

## 3. 다음 세션 우선순위

### 1차 -- npm publish 실행 (HUMAN)

GitHub Actions → "Publish npm packages" → `workflow_dispatch` → `dry_run: false`

### 2차 -- M3 showcase library 선정 (HUMAN Decision)

분석 문서: `claudedocs/m3-showcase-analysis.md`

| 순위 | 라이브러리 | 이유 |
|------|-----------|------|
| ★1순위 | **bmb-algo** | 성능 스토리 최강 (knapsack 6.8x > C) |
| ★2순위 | **bmb-json** | AI/LLM 도메인 정합, zero-copy |

### 3차 -- Track S 추가 개선 (자율, 선택적)

현재 ~97%. 선택적 개선:
- LSP: `textDocument/rename` 심볼 이름 변경 (2-3 cycles)
- CI: Z3 설치 추가 → verify_host Z3 IPC 테스트 CI에서 검증
- verify-host: 함수 레벨 incremental verification

### Backlog

| 작업 | 추정 | 우선도 |
|------|------|--------|
| Track R `run` 실제 LLM 실험 | API key 필요 | HUMAN |
| LSP rename symbol | 2-3 cycles | Medium |
| CI에 Z3 설치 → verify_host Z3 완전 테스트 | 1 cycle | Low |
| verify-host 함수 레벨 캐시 | 2-3 cycles | Low |
| M3 showcase 공식 벤치마크 측정 | 2 cycles | HUMAN 선택 후 |

---

## 4. 환경 노트

| 환경 | 상태 |
|------|------|
| LLVM | 21.1.8 MSYS2 UCRT64 |
| Node.js | v24.14.0 |
| Python | 3.12.10 |
| BMB workspace | `Cargo.toml workspace.version = "0.98.0"` |
| `target/release/bmb.exe` | 캐시 유효 |
| Branch | `main` |
| `bootstrap/lsp.bmb` | ~1380 LOC, 92-test suite |
| `bootstrap/verify_host.bmb` | ~575 LOC, 48-test suite |
| `bootstrap/lsp.exe` | gitignored (재빌드: `bmb build bootstrap/lsp.bmb -o bootstrap/lsp.exe`) |
| `bootstrap/verify_host.exe` | gitignored (재빌드: `bmb build bootstrap/verify_host.bmb -o bootstrap/verify_host.exe`) |

### 중요 운용 노트

**BMB_PATH 절대경로 필수**: `system_capture`이 Windows `cmd.exe`로 실행되어 `./` 상대경로 미지원.
- ✅ `BMB_PATH=D:/data/lang-bmb/target/release/bmb.exe`
- ❌ `BMB_PATH=./target/release/bmb.exe` (Windows에서 실패)

**lsp_test.py 실행**: `python3 bootstrap/lsp_test.py` (BMB_PATH 미설정시 DEFAULT_BMB 절대경로 자동 사용)

**verify_host_test.py 실행**: `python3 bootstrap/verify_host_test.py`

**BMB 소스 파일 em-dash 금지**: U+2014 사용 시 lexer 오류 → ASCII 하이픈으로 대체

**BMB codegen 버그**: `getenv()` 결과 let 바인딩 후 String concat 우측 피연산자로 사용 시 오동작 → helper function 파라미터로 전달하는 방식으로 우회

**캐시 파일**: `*.vh_cache`, `*.vh_proofdb` → `.gitignore` 등록됨

---

## 5. HUMAN-Decision (미결)

| 항목 | 현황 |
|------|------|
| npm publish 실행 | ⏳ `workflow_dispatch` |
| v0.100 버전 선언 | ⏳ 메인테이너 결정 |
| M3 showcase library 선정 | ⏳ 분석: `claudedocs/m3-showcase-analysis.md` |
| Track R `run` 실제 LLM 실험 | ⏳ API key + `BMB_BENCH_API_KEY` 환경변수 설정 필요 |

---

## 6. Push 상태

- 이전 세션: ✅ push 완료 (`2e4cadf7 → 196a02e9`)
- **이번 세션**: ⏳ push 대기 (commit pending)

### 다음 세션 시작 체크리스트

- [ ] `git push` 완료 확인
- [ ] `./target/release/bmb build bootstrap/lsp.bmb -o bootstrap/lsp.exe`
- [ ] `./target/release/bmb build bootstrap/verify_host.bmb -o bootstrap/verify_host.exe`
- [ ] `python3 bootstrap/lsp_test.py` → 92/92 PASS 확인
- [ ] `python3 bootstrap/verify_host_test.py` → 48/48 PASS 확인

---

**세션 종료**: 2026-05-10 (Cycles 2609-2618, 10 cycles)
