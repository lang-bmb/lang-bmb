# Cycle 2803: playground-wasm Phase 1 — wasm32 빌드 성공
Date: 2026-05-13

## Re-plan
HANDOFF 권장 작업: "playground-wasm Phase 1 scoping (1 cycle)". HUMAN 차단 없는 자율 P2 작업.
Scope: wasm32 타겟 빌드 가능성 조사 + 기반 작업.

## Scope & Implementation

**Investigation:**
- `cargo check --target wasm32-unknown-unknown -p bmb` → mio 오류 (tokio::features = ["full"] 차단)
- 블로킹 deps: `lsp/mod.rs`(tower-lsp), `repl/mod.rs`(rustyline), `interp/eval.rs`(std::fs/process/env)
- 핵심 발견: lib 타겟만 빌드하면 됨 (`--lib` flag)

**Implementation:**
1. `bmb/Cargo.toml`: `tokio`, `tower-lsp`, `rustyline`, `notify`, `notify-debouncer-mini`를
   `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`로 이동
2. `bmb/src/lib.rs`: `lsp`, `repl`, `build` 모듈에 `#[cfg(not(target_arch = "wasm32"))]` 추가
3. `bmb/src/interp/eval.rs`:
   - 임포트 (std::env/fs/process) cfg gate
   - builtin 등록 블록 cfg gate
   - 14개 builtin 함수 정의 cfg gate
   - `register_builtin()` public 메서드 추가 (WASM output capture용)
4. `ecosystem/bmb-wasm/` 신규 생성:
   - `Cargo.toml`: wasm-bindgen + js-sys + serde_json
   - `src/lib.rs`: `check()`, `run()`, `version()` wasm-bindgen API
   - thread_local WASM_OUTPUT 버퍼로 println 출력 capture
5. 워크스페이스에 `ecosystem/bmb-wasm` 추가

## Verification & Defect Resolution

| 검증 항목 | 결과 |
|----------|------|
| `cargo check --target wasm32-unknown-unknown -p bmb --lib` | ✅ |
| `cargo check --target wasm32-unknown-unknown -p bmb-wasm` | ✅ |
| `cargo check --release -p bmb -p bmb-wasm` | ✅ |
| `cargo test --release -p bmb --lib` | ✅ 3774 PASS |

**Fix**: `bmb-wasm/Cargo.toml`에 `js-sys = "0.3"` 추가 (web_time_ms에서 js_sys::Date::now() 사용)

## Reflection

- **Scope fit**: Phase 1 scoping 예상을 초과 달성 — 실제 wasm32 빌드 성공까지 완료
- **Philosophy drift**: None. 도그푸딩 활동 (BMB 컴파일러를 BMB 에코시스템에 통합)
- **Latent defects**:
  - `js-sys`가 네이티브 빌드에서도 포함됨 → `[target.'cfg(target_arch = "wasm32")'.dependencies]`로 이동 필요
  - `print()` (줄바꿈 없는 버전), `sb_println()` 등 다른 print builtins 미처리 — capture 불완전
- **Roadmap impact**: Phase 2 (wasm-pack 빌드 + .wasm 파일 생성) 즉시 착수 가능

## Carry-Forward
- Actionable:
  1. `bmb-wasm/Cargo.toml`: `js-sys`를 wasm32 조건부 dep으로 이동
  2. `bmb-wasm/src/lib.rs`: `print`, `sb_println`, `write_stdout` 등 추가 println builtins capture
  3. wasm-pack 설치 및 실제 .wasm 파일 빌드 (Cycle 2804)
- Structural Improvement Proposals:
  - Interpreter에 generic output writer 지원 추가하면 println 캡처가 더 robust해짐
    (현재는 register_builtin으로 개별 교체 필요)
- Pending Human Decisions: None
- Roadmap Revisions: playground-wasm P2 ISSUE 진행 중. Phase 2로 전환.
- Next Recommendation: Cycle 2804 — wasm-pack 설치 + `wasm-pack build` 실행 + WASM 파일 생성
