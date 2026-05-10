# BMB Session Handoff -- 2026-05-10 (Cycles 2603-2608 -- Track S LSP + verify-host)

> **이전 HEAD**: `2e4cadf7` (Track S LSP references + 45-test)
> **새 HEAD**: TBD (push pending)
> **세션 성격**: 6-cycle run-cycle -- Track S LSP 기능 확장 + verify-host Z3 IPC
> **핵심 성과**: LSP 69-test suite (69/69 PASS), verify-host Z3 직접 IPC, verify_host_test.py 33-test

---

## 1. 이번 세션 산출물 (Cycles 2603-2608)

### Cycle 2603 -- workspace/symbol 구현 + 10 tests
- `workspaceSymbolProvider: true` capability 추가
- `handle_workspace_symbol`: BMB_WORKSPACE env → dir 명령 → 파일별 심볼 스캔
- `scan_file_lines` while-loop 기반 (스택 오버플로 방지)
- `build_ws_cmd` helper (BMB codegen 버그 우회)
- `path_to_file_uri`, `qs_contains`, `mk_ws_symbol`
- **55/55 PASS** (이전: 45/45)

### Cycle 2604 -- verify-host Z3 직접 IPC 구현
- `run_z3_query`: 임시파일 기반 Z3 IPC (`vh_z3_tmp.smt2` → `z3 <file>`)
- `extract_pre_conds` / `extract_post_conds`: BMB 소스 `pre`/`post` 추출
- `expr_to_smt2`: infix BMB 비교 → SMT-LIB2 prefix 변환
- `collect_all_vars` / `gen_declarations` / `gen_asserts_from_conds`
- `run_z3_direct`: sat → "consistent", unsat → "unsatisfiable", 미발견 → "skipped"
- `z3_direct` 필드 JSON 출력에 추가
- **6210/6210 cargo nextest PASS** 확인

### Cycle 2605 -- LSP signatureHelp 구현 + 11 tests
- `signatureHelpProvider: {"triggerCharacters":["(","," ]}` capability 추가
- `sh_find_call_open_r`: 후방 스캔 → 콜 컨텍스트 `(` 위치
- `sh_count_commas_r`: depth-0 쉼표 카운트 → activeParameter
- `sh_fn_name_before`: paren 직전 identifier 추출
- `sh_extract_sig_from_file` + `sh_sig_scan`: 파일 내 `fn name(` 스캔 → 시그니처
- `sh_sig_builtin`: 내장 함수 18개 시그니처 테이블
- `sh_param_labels` + `sh_build_param_ranges`: parameter label ranges 생성
- **66/66 PASS** (이전: 55/55)

### Cycle 2606 -- verify_host_test.py 33-test suite 신규 작성
- `test_valid_file()`: status ok + 스키마 검증 (10 assertions)
- `test_type_error_file()`: 타입 오류 → type_error 상태 (4 assertions)
- `test_no_contracts()`: pre/post 없음 → z3_direct skipped (1 assertion)
- `test_contracts_consistent()`: pre 조건 → z3_direct consistent (3 assertions)
- `test_stdin_fallback()`: BMB_FILE 미설정 → stdin 폴백 (2 assertions)
- `test_no_file_error()`: 존재하지 않는 파일 → 에러 JSON (2 assertions)
- `test_output_schema()`: 전체 JSON 스키마 타입 검증 (11 assertions)
- **33/33 PASS**

### Cycle 2607 -- hover 개선 (사용자 정의 함수 시그니처) + 3 tests
- `handle_hover`: 키워드 매치 실패 시 `sh_extract_sig_from_file` 호출
- 코드 블록 형식: ` ```bmb\nfn name(params) -> RetType\n``` `
- **69/69 PASS** (이전: 66/66)

### Cycle 2608 -- 회귀 확인 + HANDOFF
- `cargo nextest run --release`: **6210/6210 PASS**
- HANDOFF 업데이트

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
| S (BMB-rewrite) | ~92% ✅ | LSP ~97% + verify-host ~75% = 전체 ~92% |
| T (External Bindings) | ~95% ✅ | Node.js 5/5, npm-publish.yml |

### Track S 세부 현황

**LSP (bootstrap/lsp.bmb, ~1200 LOC)**:
- ✅ initialize (signatureHelpProvider 포함 9 capabilities)
- ✅ textDocument/hover (키워드 + 사용자 정의 함수 시그니처)
- ✅ textDocument/completion (37 items)
- ✅ textDocument/publishDiagnostics (bmb check 연동)
- ✅ textDocument/documentSymbol
- ✅ textDocument/definition
- ✅ textDocument/references
- ✅ workspace/symbol (BMB_WORKSPACE env, while-loop 스캔)
- ✅ textDocument/signatureHelp (activeParameter, parameter labels)
- ✅ shutdown + exit
- **테스트**: 69/69 PASS

**verify-host (bootstrap/verify_host.bmb, ~510 LOC)**:
- ✅ BMB_FILE env var + stdin fallback
- ✅ `bmb check` 실행 → 에러/경고 파싱 → JSON
- ✅ `bmb verify` 실행 → verify_result 파싱 → JSON
- ✅ Z3 직접 IPC (`run_z3_direct`): pre/post 추출 → SMT-LIB2 → Z3 실행
- ✅ CI type check 추가
- ✅ verify_host_test.py 33-test suite
- ⬜ incremental verification 미구현
- ⬜ proof database 미구현

### 테스트 현황

| 스위트 | 결과 |
|--------|------|
| `cargo nextest run --release` | ✅ 6210 passed |
| `python3 bootstrap/lsp_test.py` | ✅ 69 passed |
| `python3 bootstrap/verify_host_test.py` | ✅ 33 passed |
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

### 3차 -- Track S ~95%+ 달성 (자율)

현재 ~92%. 잔여 (~8%):
- verify-host: incremental verification, proof database → ~85%
- LSP: code actions, completion from current file → ~99%

### Backlog

| 작업 | 추정 | 우선도 |
|------|------|--------|
| Track R `run` 실제 LLM 실험 | API key 필요 | HUMAN |
| LSP completion from current file | 1 cycle | Medium |
| LSP code actions | 2 cycles | Low |
| verify-host incremental verification | 2-3 cycles | Medium |
| CI에 lsp_test.py + verify_host_test.py 추가 | 1 cycle (LLVM 설치 포함) | Low |
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
| `bootstrap/lsp.bmb` | ~1200 LOC, 69-test suite |
| `bootstrap/verify_host.bmb` | ~510 LOC, 33-test suite |
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

- 이전 세션: ✅ push 완료 (`9d1a6cf6 → 2e4cadf7`)
- **이번 세션**: 🔴 push pending (커밋 후 push 필요)

### 다음 세션 시작 체크리스트

- [ ] `git push` 완료 확인
- [ ] `./target/release/bmb build bootstrap/lsp.bmb -o bootstrap/lsp.exe`
- [ ] `./target/release/bmb build bootstrap/verify_host.bmb -o bootstrap/verify_host.exe`
- [ ] `python3 bootstrap/lsp_test.py` → 69/69 PASS 확인
- [ ] `python3 bootstrap/verify_host_test.py` → 33/33 PASS 확인

---

**세션 종료**: 2026-05-10 (Cycles 2603-2608)
