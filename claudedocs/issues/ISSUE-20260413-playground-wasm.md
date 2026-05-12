# ISSUE-20260413 — Playground WASM 통합

**우선순위**: P2
**영역**: ecosystem, codegen
**상태**: Open

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
