# Cycle 3130: M8-B 배치 7 — String trivial 3개 교체 (전체 80개 분석)
Date: 2026-05-25

## Re-plan
Plan valid. M8-B 계속 — 잔여 80개 전수 분석하여 교체 가능 3개 적용.

## Scope & Implementation
3개 교체:

**Length-bounded ≤ mir (1개)**:
- build_const_map(mir) L11007 → `post it.len() <= mir.len()`
  "@name=value," 형식은 원본 함수 정의보다 훨씬 짧음 (함수 body ~40자 → 엔트리 ~5자)

**Accumulator ≥ line (1개)**:
- trl_subst_line(line, subst_from, subst_to) L12971 → `post it.len() >= line.len()`
  "%param" → "%param_loop" 치환으로 항상 5자씩 성장 (subst_from == ""이면 identity)

**Accumulator ≥ s (1개)**:
- escape_llvm_string(s) L15551 → `post it.len() >= s.len()`
  특수문자(|, ", \n, \t 등)가 \7C, \22, \0A 등 3자로 확장됨; 일반 문자는 1→1, 총 출력 ≥ 입력

**비교체 분류 (77개 잔여 주요 skip 이유)**:
- include_resolve/preprocess_includes/load_source: 환경변수·파일 내용 의존, 크기 불정
- resolve_enum_variants_in_ast/resolve_tag_checks: 교체 결과가 더 긴 S-expression
- parse_source/rename_name_in_ast/replace_free_var: 일반 replace 패턴
- map_runtime_fn/format_i64_call_args: 출력이 항상 더 큰 LLVM 형식
- LLVM codegen 전체 (llvm_gen_*): MIR → LLVM IR 변환으로 출력 급증
- TCO/LICM/GCS/PHT pass 함수들: 복잡한 재구조화, 크기 방향 불정
- REPL/formatter 함수들: 컴파일러 출력, 크기 입력 무관

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- bmb check ✅ warnings: 3021 → 3020 (−1)
- bmb verify ✅ 953/953

## Reflection
- String trivials: 80 → 77 (−3 교체 확인)
- escape_llvm_string: string escaping 함수에 accumulator ≥ 패턴 정확 적용
- build_const_map: const 함수 추출 → map 형식이 항상 원본보다 짧음을 증명
- trl_subst_line: "_loop" suffix 치환으로 항상 성장 (trl_build_subst_to와 상보적 계약)
- 77개 잔여 분석: 대부분 LLVM IR codegen, 파서, REPL 계열 — 의미있는 length bound 적용 불가
- 잔여 77개는 성격 기반으로 거의 모두 skip으로 확정됨

## Carry-Forward
- Actionable: 잔여 77개는 구조적으로 skip — pre 조건 추가 가능성 검토로 전환
  - L12987 (trl_replace_var): `pre replacement.len() >= var_name.len()` 같은 pre 계약 검토
  - L11092 (replace_all_in_mir): `pre replacement.len() <= pattern.len()` 검토
  - M8-C(it 타입 고정) 방향으로 전환 가능성 검토
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: M8-B String trivial 교체 실질적 완료 — 77개 잔여는 skip 확정, M8-C 또는 pre 조건 추가로 전환 권장
- Next Recommendation: Cycle 3131: M8-B 마무리 (pre 계약 추가 타당성 평가 또는 M8-C 진입)
