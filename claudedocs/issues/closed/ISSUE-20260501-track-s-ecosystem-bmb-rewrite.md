# ISSUE: Track S — Ecosystem BMB-rewrite

> **트랙**: S (Ecosystem BMB-rewrite, orthogonal)
> **마일스톤**: M1 (현재 1/5) → M2 → M3 (90% 목표) → M4 (100%)
> **현 상태**: 1/5 — 부트스트랩 컴파일러만 BMB
> **만든 사이클**: 2508
> **앵커**: `docs/ROADMAP.md` § "Vision v1.0 Framework", spec § 5

## 현 상태 (Cycle 2589 업데이트)

| 도구 | 현재 언어 | BMB 재작성 진척 |
|------|---------|--------------|
| 부트스트랩 컴파일러 | BMB ✅ | `bootstrap/compiler.bmb` 32K LOC, Stage 1-3 ✅ |
| fmt | **BMB ✅** | `tools/bmb-fmt/main.bmb` (234 LOC) — CI에서 사용 중 (format-check step) |
| lint | **BMB ✅** | `tools/bmb-lint/main.bmb` (301 LOC) + `bootstrap/lint/lint.bmb` (10 checks) — CI 사용 중 |
| bench runner | **BMB ✅** | `tools/bmb-bench/main.bmb` (315+748 LOC) — CI 사용 중 |
| check | **BMB ✅** | `tools/bmb-check/main.bmb` (235 LOC) — CI 사용 중 |
| test | **BMB ✅** | `tools/bmb-test/main.bmb` (274 LOC) — CI 사용 중 |
| doc | **BMB ✅** | `tools/bmb-doc/main.bmb` (doc gen) |
| LSP (메인스트림) | Rust + BMB | `bmb/src/lsp/mod.rs` + `bootstrap/lsp.bmb` (496 LOC) 시작점 |
| verify | Rust (+ Z3 외부) | `bmb/src/verify/`. Z3 IPC 의존은 영구 — 호스트만 BMB 필요 |
| gotgan (패키지 매니저) | Rust | `ecosystem/gotgan/`. 0% — M4 |
| vscode-bmb | TypeScript | extension UI — BMB 재작성 비현실 |
| tree-sitter-bmb | C/JS | grammar — BMB 재작성 비현실 |

**중요**: Track S 이슈 작성 시점(2026-05-01)에 `tools/*.bmb` 도구들이 누락되었음.
실제로 fmt/lint/bench/check/test는 이미 BMB로 재작성되어 CI에서 검증 중.

## M3 목표 (90%)

LSP, fmt, lint, verify, bench BMB 재작성 = **5/5 = 100% of M3 set**, ROADMAP에서 M3 트랙 S = 90%로 명시 (verify Z3 IPC 호스트만 BMB).

## 작업 우선순위 (1차안)

1. **fmt** (가장 작은 도구 추정, BMB만으로 자급)
2. **lint** (fmt 위에 구축 가능)
3. **bench runner** (compile + run + measure — BMB process_spawn API 의존)
4. **LSP** (가장 복잡 — JSON-RPC stdio, async, incremental)
5. **verify** (Z3 IPC은 외부 — 호스트 BMB)

## 의존성

- BMB 언어 기능 충족도 점검 필요:
  - 트레이트 (LSP의 다양한 메시지 핸들러)
  - 비동기 또는 스레드 (LSP)
  - JSON-RPC (manual implementation)
- `bootstrap/lsp.bmb` 현재 상태 점검 (이미 시작점 있음)

## 작업 단계 (1차안 — fmt 우선)

1. **Phase 1 — fmt PoC** (Cycle 2520+)
   - `bootstrap/fmt.bmb` 신규
   - 파서 재사용 (`bootstrap/parser.bmb`) → AST → pretty printer
   - 기존 Rust fmt 결과와 diff 0 검증

2. **Phase 2 — lint PoC** (Cycle 2522+)
   - 파서 + AST → lint rules (트레이트 또는 enum dispatch)

3. **Phase 3 — bench runner** (Cycle 2525+)
   - process_spawn + stdout 파싱 + 통계 (BMB 자체 통계 함수)

4. **Phase 4 — LSP** (장기, M3 진입 후)

## 완료 조건

- [ ] M3 트랙 S 90%: fmt/lint/bench/LSP/verify-host BMB 재작성
- [ ] 옛 Rust 도구는 회귀 비교 baseline으로만 유지

## 추정 사이클

장기 (15-20 cycles 분산). M1/M2 작업 중 부분 진척 가능.
