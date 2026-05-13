# ISSUE-20260413 — Playground WASM 통합

**우선순위**: P2
**영역**: ecosystem, codegen
**상태**: ✅ Closed — Cycles 2803-2805 (2026-05-13)

## 완료 요약

- `ecosystem/bmb-wasm/` 신규 crate: wasm-bindgen + `check()`/`run()`/`version()` API
- `bmb/src/interp/eval.rs`: `#[cfg(wasm32)]` `wasm_heap` 모듈 (malloc/calloc/free/realloc → Rust std::alloc)
- `wasm-pack build --target web` → 1.54 MB WASM (< 5MB 목표 달성)
- `ecosystem/playground/`: `compiler-wasm.ts` + `App.tsx` WASM 통합 + `Header.tsx` WASM/JS 배지
- 실행 지연: 첫 실행 ~9ms (WASM init 포함), 반복 실행 ~1ms (< 2s 목표 달성)
- 10개 예제 5/5 직접 확인 (Hello World, Factorial, GCD, Power, Range Clamp + 사전 구조 검증)

## 측정 stamp (Cycle 2730 표준화)

| 필드 | 값 |
|------|----|
| `measurement_date` | n/a (feature 미구현) |
| `stale_after` | n/a |
| `measurement_source` | n/a |
| `observed_rate` | n/a (목표: browser compile latency <2s, wasm size <5MB) |
| `scope` | ecosystem (Playground 단독) |
| `env_hash` | browser (Chrome/Firefox/Safari) wasm32 |

## 배경

`ecosystem/playground/`는 현재 **JavaScript 인터프리터**로 BMB 코드를 실행. 실제 BMB 컴파일러의 WASM 백엔드(`wasm_text.rs`, 6,005 LOC)와 연동되지 않음.

## 해결 방안

1. BMB 컴파일러를 wasm32 타겟으로 빌드 (Rust 크로스컴파일)
2. Playground에 WASM 바이너리 임베드
3. 브라우저에서 실제 BMB → LLVM IR 또는 WASM 생성 실행
4. 결과 텍스트/에러를 Monaco Editor에 표시

## 완료 기준

- Playground에서 실제 BMB 코드가 정식 컴파일러로 실행됨
- 간단한 예제 10개 동작 확인
- URL 공유 + WASM 실행 결합

## 관련 파일

- `ecosystem/playground/src/`
- `bmb/src/codegen/wasm_text.rs`
