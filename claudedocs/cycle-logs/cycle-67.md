# Cycle 67: CIR Lowering + CIR Output Tests

## 개발 범위
- CIR lowering (lower.rs): +31 tests covering expr_to_proposition, lower_expr, lower_type, references_return_value, lower_binop/unaryop, integration tests
- CIR output (output.rs): +23 tests covering format_proposition, format_expr, format_effects, format_text, format_json, Display traits

## 현재 상태
- 테스트: ✅ 968개 (884+154+23) — +54 from baseline 914

## 미비/결함/개선 도출
| 유형 | 내용 | 심각도 |
|------|------|--------|
| 결함 | Span has no `file_id` field (fixed) | Low |
| 결함 | Type::Tuple elements need Box (fixed) | Low |
| 결함 | Named return `result` not recognized in postconditions (fixed: use `ret`) | Low |
| 결함 | Clippy approx_constant for 3.14/2.718 (fixed: use 1.5) | Low |
