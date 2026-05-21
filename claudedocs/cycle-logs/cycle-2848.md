# Cycle 2848: {expr} Complex String Interpolation
Date: 2026-05-14

## Re-plan
Carry-Forward (2847): `{expr}` 복잡 표현식 보간, str_hashmap keys iterator, 필드 복합 할당 native 지원.
`{expr}` 보간 우선 처리 — AI 코드 생성 품질에 직접적 영향.

## Scope & Implementation

**접근법**: `ast/expr.rs`의 `desugar_string_interp`에 미니 재귀 하강 파서(`InterpMini`) 내장.

지원 문법:
- 기존: `{ident}`
- 신규: `{a + b}`, `{a * b}`, `{a - b}`, `{a / b}`, `{a % b}` — 산술 binary
- 신규: `{p.x}`, `{p.x.y}` — 필드 접근 체인
- 신규: `{-n}` — 단항 부호
- 신규: `{(a + b)}` — 괄호 서브식
- 패스스루: `{0}`, `{1}` 등 순수 숫자 — format-arg 자리표시자 유지
- 미지원: 함수 호출 (복잡도 대비 효과 낮음; let 바인딩으로 회피)

변경 파일:
- `bmb/src/ast/expr.rs`:
  - `InterpMini` 구조체 + `parse_interp_expr` 함수 신규 (약 120줄)
  - `desugar_string_interp`: brace-depth 추적으로 matching `}` 검색, 내용을 `parse_interp_expr`로 파싱
  - 기존 ident-only 경로는 `parse_interp_expr`가 동일 결과 반환하므로 호환 유지
- `bmb/tests/integration.rs`: `test_interp_string_interp_expr` 5개 케이스 추가
- `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`:
  - "Pattern: String expression interpolation (v0.98.5+)" 신규
  - Pitfalls에 `{expr}` 제한 사항 명시

**수정**: `UnaryOp::Neg` → `UnOp::Neg` (BMB의 단항 연산자 타입명 수정)

## Verification & Defect Resolution
- test_interp_string_interp_expr: 1/1 (5 케이스) 통과 ✅
- cargo test --release 전체: **2372 passed; 0 failed** ✅ (EXIT:0)

## Reflection
- ✅ `{expr}` 보간 구현 완성 — `{n + 1}`, `{p.x}`, `{a * b}` 등 대부분의 AI 생성 패턴 지원
- **설계 결정**: 함수 호출은 미지원으로 스코프 제한. `{to_string(n)}` → `let s = to_string(n); "{s}"` 워크어라운드 문서화
- **인사이트**: `desugar_string_interp`이 파싱 타임에 호출되므로 서브-파서를 내장해야 함. `InterpMini`가 깔끔하게 분리됨.
- 함수 호출 지원은 추후 별도 사이클에서 가능 (args 파싱이 재귀적으로 복잡)

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  * str_hashmap keys iterator (현재 key 나열 방법 없음)
  * `{fn(args)}` 함수 호출 보간 지원 (args 재귀 파싱 필요)
  * 필드 복합 할당 native 지원 (codegen 확장)
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: str_hashmap keys iterator 또는 enum 기반 variant 패턴 추가
