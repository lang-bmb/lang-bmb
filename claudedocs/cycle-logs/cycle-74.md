# Cycle 74: CIR SMT + to_mir_facts Tests

## 개발 범위
- smt/translator.rs: +21 tests (type_to_sort extended, float/todo/break/continue/return/it/sizeof/block/let, wrapping arithmetic, TranslateError Display, unsupported features)
- pir/to_mir_facts.rs: +18 tests (cir_op_to_mir_op, flip_cmp_op, proposition_to_facts all variants, extract_all_pir_facts)

## 현재 상태
- 테스트: ✅ 1137개 — +41 (total +76 from Cycle 73)

## 미비/결함/개선 도출
| 유형 | 내용 | 심각도 |
|------|------|--------|
| 결함 | Proposition::Forall/Exists require `ty` field (fixed) | Low |
| 결함 | Proposition::Old takes (Box<CirExpr>, Box<Proposition>) not (String, Box<CirExpr>) (fixed) | Low |
| 결함 | Type::Refined uses `constraints: Vec<Spanned<Expr>>` not `predicate` (fixed) | Low |
| 결함 | PirProgram requires `proof_db` and `type_invariants` fields (fixed) | Low |
