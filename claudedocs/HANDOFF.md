# BMB Session Handoff — 2026-05-10 (Cycles 2609-2617 + 철학 정렬)

> **이전 HEAD**: `196a02e9` (Track S LSP 69-test + verify-host Z3 IPC)
> **새 HEAD**: `d9a4fad9` (Track S LSP rename + CI Z3 + 100-test — S~99%)
> **Push 상태**: ⏳ **push 대기** — 다음 세션 시작 전 먼저 실행
> **실무 앵커**: `claudedocs/ROADMAP.md` (이번 세션 신규 작성)

---

## 0. 철학 정렬 (이번 세션 추가 작업)

이번 세션에서 4가지 drift를 진단하고 로드맵을 재정렬했다.

| Drift | 내용 | 처방 |
|-------|------|------|
| A | 도그푸딩 프레임 미명시 | 모든 활동 = 도그푸딩, low-level 게이트 항상 열림 |
| B | B(Failure Rate) #1인데 미측정 | M4 첫 액션 = LLM 1-shot baseline 측정 |
| C | AI-native 선언 vs. 언어 갭 | 미지원 패턴 목록화 → 스펙 백로그 |
| D | 앵커 문서 분산 | `claudedocs/ROADMAP.md` = 유일한 실무 앵커 |

**신규 작성**: `claudedocs/ROADMAP.md` — 다음 세션부터 이 파일이 1차 참조점.  
**설계 스펙**: `docs/superpowers/specs/2026-05-10-bmb-philosophy-roadmap-design.md`

---

## 1. 이번 세션 산출물 (Cycles 2609-2617)

### Cycle 2609 — LSP completion (파일 심볼)
- `handle_completion`: URI → `read_file` → `scan_for_completion` → 파일 심볼 추가
- `scan_for_completion`: while-loop 기반 fn/struct/enum 스캔
- **79/79 PASS**

### Cycle 2610 — verify-host incremental (파일 레벨 캐시)
- `file_hash(content)`: djb2 while-loop (`hash:length` 형식)
- `cache_path` / `read_cache` / `write_cache`: `<file>.vh_cache` 캐시
- **42/42 PASS**

### Cycle 2611 — verify-host proof database + .gitignore
- `proof_db_path`: `<file>.vh_proofdb`
- `run_z3_direct`: pre/post hash 기반 Z3 결과 캐시
- `.gitignore`: `*.vh_cache`, `*.vh_proofdb` 추가
- **48/48 PASS**

### Cycle 2612 — LSP code actions (pre/post condition stubs)
- `codeActionProvider: true` (10 capabilities)
- `build_code_actions`: fn 라인 감지 → "Add pre/post condition"
- **88/88 PASS**

### Cycle 2613 — CI 통합 + 플랫폼 독립
- `lsp_test.py` / `verify_host_test.py`: `_EXE` 플랫폼 변수
- `.github/workflows/ci.yml`: `track-s-tests` job (ubuntu-latest, llvm+z3)

### Cycle 2614 — LSP completion 함수 시그니처 detail
- `extract_fn_sig_at`: fn 위치에서 직접 시그니처 추출
- **92/92 PASS**

### Cycle 2615 — LSP rename
- `find_rename_edits` / `handle_rename`: WorkspaceEdit 반환
- `renameProvider: true` (11 capabilities)
- **100/100 PASS**

### Cycle 2616 — CI Z3 설치
- `track-s-tests` job: `sudo apt-get install -y llvm z3`
- `which z3 && z3 --version` 검증 step

### Cycle 2617 — 철학 정렬 + HANDOFF
- `claudedocs/ROADMAP.md` 신규 작성
- `claudedocs/HANDOFF.md` 업데이트

---

## 2. 현재 상태

### Track 스냅샷

| Track | % | 상태 |
|-------|---|------|
| M (Machine-First) | ~100% ✅ | 완료 |
| N (MCP Server) | ~99% ✅ | 13 tools / 4 resources / 3 prompts |
| O (Context Pack) | ~95% ✅ | `uses` 의존성 그래프 포함 |
| Q (Ambiguity Audit) | ~92% ✅ | 10 checks, CI gate |
| R (LLM Bench) | ~95% ✅ | run+analyze 파이프라인 |
| S (BMB-rewrite) | ~99% ✅ | LSP 100-test + verify-host 48-test + CI Z3 |
| T (External Bindings) | ~95% ✅ | Node.js 5/5, npm-publish.yml 준비 |

### Track S 세부

**LSP (`bootstrap/lsp.bmb`, ~1450 LOC)**: 11 capabilities, 100/100 PASS
- initialize, hover, completion (시그니처 detail), diagnostics, documentSymbol,
  definition, references, workspace/symbol, signatureHelp, codeAction, rename, shutdown

**verify-host (`bootstrap/verify_host.bmb`, ~575 LOC)**: 48/48 PASS
- bmb check/verify 파싱, Z3 직접 IPC, incremental cache (파일+proof DB)
- 미구현: 함수 레벨 incremental (M4 범위)

### 테스트 현황

| 스위트 | 결과 |
|--------|------|
| `cargo nextest run --release` | ✅ 6210 passed |
| `python3 bootstrap/lsp_test.py` | ✅ 100 passed |
| `python3 bootstrap/verify_host_test.py` | ✅ 48 passed |
| `bmb-mcp pytest` | ✅ 90 passed |
| `bmb-ai-bench pytest` | ✅ 30 passed |

---

## 3. 다음 세션 우선순위

### 즉시 실행 (세션 시작 전)

```bash
git push   # d9a4fad9 push
```

### 1순위 — M3 완료 (HUMAN 결정 필요)

1. **M3 showcase 선정**: bmb-algo (1순위) / bmb-json (2순위) 중 결정
2. **공식 벤치마크 측정**: 선정된 라이브러리 성능 측정 (1-2 cycles)
3. **npm publish**: `npm-publish.yml` → `workflow_dispatch` → `dry_run: false`
4. **PyPI publish**: `pypi-publish.yml` → `workflow_dispatch`

### 2순위 — M3→M4 전환 준비 (자율)

5. **B baseline 측정**: Track R에서 LLM 1-shot 성공률 최초 측정 (M4 첫 액션)
6. **언어 갭 이슈 등록**: LLM 미지원 패턴 → `claudedocs/issues/` 백로그

### 3순위 — 선택적

7. **verify-host 함수 레벨 캐시**: 2-3 cycles (low priority)
8. **v0.100 버전 선언**: M3 완료 후 메인테이너 결정

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
- **lsp.exe 재빌드**: `./target/release/bmb build bootstrap/lsp.bmb -o bootstrap/lsp.exe`
- **verify_host.exe 재빌드**: `./target/release/bmb build bootstrap/verify_host.bmb -o bootstrap/verify_host.exe`
- **BMB 소스 em-dash 금지**: U+2014 → ASCII 하이픈
- **캐시 파일**: `*.vh_cache`, `*.vh_proofdb` → `.gitignore` 등록됨

---

## 5. HUMAN 결정 대기

| 항목 | 현황 |
|------|------|
| git push (d9a4fad9) | ⏳ 세션 시작 전 필수 |
| M3 showcase 선정 | ⏳ bmb-algo(1순위) / bmb-json(2순위) |
| npm publish | ⏳ `workflow_dispatch` dry_run=false |
| PyPI publish | ⏳ `workflow_dispatch` |
| v0.100 버전 선언 | ⏳ M3 완료 후 |
| Track R LLM 실험 | ⏳ API key + `BMB_BENCH_API_KEY` 필요 |

---

## 6. 다음 세션 시작 체크리스트

- [ ] `git push` 완료 확인
- [ ] `claudedocs/ROADMAP.md` 읽기 (실무 앵커)
- [ ] `./target/release/bmb build bootstrap/lsp.bmb -o bootstrap/lsp.exe`
- [ ] `./target/release/bmb build bootstrap/verify_host.bmb -o bootstrap/verify_host.exe`
- [ ] `python3 bootstrap/lsp_test.py` → 100/100 확인
- [ ] `python3 bootstrap/verify_host_test.py` → 48/48 확인

---

**세션 종료**: 2026-05-10 (Cycles 2609-2617 + 철학 정렬)
