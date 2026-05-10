# BMB Session Handoff — 2026-05-10 (Cycle 2618 — 전체 작업 검토 + 로드맵 갱신)

> **HEAD**: `3747c35e` (docs: 철학 정렬 + claudedocs 재구성)
> **Push 상태**: ✅ pushed (origin/main == local main)
> **실무 앵커**: `claudedocs/ROADMAP.md`

---

## 0. 이번 세션 작업 (Cycle 2618)

### 전면 검토 결과 요약

ROADMAP.md §1~§5 기준으로 전체 작업 상태를 검토하고 갱신했다.

**주요 발견**:

| 발견 | 내용 |
|------|------|
| B 상태 정정 | "미측정"→ 비공식 결과 존재 (90.9%, 2026-03-26, ~100문제). 인프라 완성, 공식 선언만 없음 |
| 언어 갭 실제 | CLAUDE.md 목록 중 실제 파서 갭: let-tuple / static-method-call / Option expr. 나머지는 지원됨 |
| M3 진도 | ~75% → ~90%로 상향. Track S 99%, 바인딩 5/5 ✅. 잔여: showcase 선정+벤치+publish |
| bmb-mcp 미커밋 | `ecosystem/bmb-mcp` 변경사항 커밋 필요 |
| benchmark-bmb | v0.34(2026-01-09) 결과만 존재. v0.98 공식 벤치 없음 |

---

## 1. 현재 상태

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

### 마일스톤 상태

| 마일스톤 | 상태 |
|---------|------|
| M1 Self-Validated | ✅ COMPLETE |
| M2 AI-Ready Infra | ✅ COMPLETE |
| M3 External Bindings | 🔄 ~90% (showcase 선정+벤치+publish 잔여) |
| M4 Adopted | ⬜ 대기 |

### 테스트 현황

| 스위트 | 결과 |
|--------|------|
| `cargo nextest run --release` | ✅ 6210 passed |
| `python3 bootstrap/lsp_test.py` | ✅ 100 passed |
| `python3 bootstrap/verify_host_test.py` | ✅ 48 passed |
| `bmb-mcp pytest` | ✅ 90 passed |
| `bmb-ai-bench pytest` | ✅ 30 passed |

---

## 2. 태스크 목록

### M3 완료 태스크

| # | 태스크 | 성격 | 소요 |
|---|--------|------|------|
| M3-1 | **[HUMAN]** showcase 선정: bmb-algo vs bmb-json | 결정 | 즉시 |
| M3-2 | showcase 공식 벤치마크 측정 (v0.98 기준) | 자율 | 1-2 cycles |
| M3-3 | **[HUMAN]** npm publish: `workflow_dispatch` → `dry_run: false` | 실행 | 즉시 |
| M3-4 | **[HUMAN]** PyPI publish: `workflow_dispatch` → `publish: true, repository: pypi` | 실행 | 즉시 |
| M3-5 | bmb-mcp 미커밋 변경사항 커밋+push | 위생 | 즉시 |

### M4 준비 태스크 (선행 가능)

| # | 태스크 | 성격 | 소요 |
|---|--------|------|------|
| M4-1 | **[HUMAN+KEY]** B 공식 측정: `BMB_BENCH_API_KEY` + `bmb-ai-bench run` | B축 | 1 cycle |
| M4-2 | 언어 갭 이슈 등록 (let-tuple / static-method / Option-expr) | Drift C | 즉시 |
| M4-3 | `let (a, b) = expr` — tuple destructuring 파서 추가 | 언어 | 2-3 cycles |
| M4-4 | `Type::method()` — static method call expression 파서 추가 | 언어 | 2-3 cycles |
| M4-5 | `Option::Some(x)` 표현식 위치 지원 | 언어 | 1-2 cycles |
| M4-6 | C# 바인딩 scaffold | 바인딩 | 3-5 cycles |

---

## 3. 다음 세션 우선순위

### 즉시 (세션 시작 전)

```
git push  # 이미 완료 (3747c35e pushed)
```

### 1순위 — M3 완료 (HUMAN 결정 필요)

1. **M3-1** showcase 선정 → **M3-2** 벤치마크 측정 (자율)
2. **M3-3** npm publish + **M3-4** PyPI publish
3. **M3-5** bmb-mcp 미커밋 커밋 (자율, 즉시 가능)

### 2순위 — M4 준비 (자율 선행 가능)

4. **M4-2** 언어 갭 이슈 등록 (즉시)
5. **M4-1** B 공식 측정 (API key 필요)

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
| M3 showcase 선정 | ⏳ bmb-algo(1순위) / bmb-json(2순위) |
| npm publish | ⏳ `workflow_dispatch` dry_run=false |
| PyPI publish | ⏳ `workflow_dispatch` publish=true, repository=pypi |
| v0.100 버전 선언 | ⏳ M3 완료 후 |
| B 공식 측정 | ⏳ `BMB_BENCH_API_KEY` 설정 필요 |

---

## 6. 다음 세션 시작 체크리스트

- [ ] `claudedocs/ROADMAP.md` 읽기 (실무 앵커)
- [ ] `./target/release/bmb build bootstrap/lsp.bmb -o bootstrap/lsp.exe`
- [ ] `./target/release/bmb build bootstrap/verify_host.bmb -o bootstrap/verify_host.exe`
- [ ] `python3 bootstrap/lsp_test.py` → 100/100 확인
- [ ] `python3 bootstrap/verify_host_test.py` → 48/48 확인

---

**세션 종료**: 2026-05-10 (Cycle 2618 — 전체 작업 검토 + ROADMAP 갱신)
