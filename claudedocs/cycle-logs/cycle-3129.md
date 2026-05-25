# Cycle 3129: M8-B 배치 6 — String trivial 9개 교체 (전 범위 분석 완료)
Date: 2026-05-25

## Re-plan
Plan valid. M8-B 계속 — 89개 잔여 trivial 전수 분석 완료 후 교체 가능 9개 적용.

## Scope & Implementation
9개 교체, 5개 패턴:

**Length-bounded ≤ src (2개)**:
- build_struct_registry(src) L3459 → `post it.len() <= src.len()` (compact "Name:fields;" 형식)
- build_enum_registry(src) L3581 → `post it.len() <= src.len()` (compact "Name:variants;" 형식)

**Length-bounded ≤ body (1개)**:
- find_free_vars_in_mir(body, params) L5744 → `post it.len() <= body.len()` (body에서 자유변수명만 추출)

**Accumulator ≥ params/new_mapping (3개)**:
- trl_build_subst_to(params) L13029 → `post it.len() >= params.len()` ("%p_loop;" 형식으로 항상 성장)
- changed_mapping(new_mapping, llvm_line) L17367 → `post it.len() >= new_mapping.len()` ("!" + len + ":" + mapping 접두사)
- changed_mapping_empty(new_mapping) L17370 → `post it.len() >= new_mapping.len()` (동일 패턴)

**Exact constant bound ≤ 54 (2개)**:
- format_range_attr(has_lo, lo, has_hi, hi_inclusive) L18368 → `post it.len() <= 54` (최대 i64 수 2개 포함 54자)
- compute_ret_range(asts) L18382 → `post it.len() <= 54` (format_range_attr 위임 또는 "")

**Exact constant bound ≤ 7 (1개)**:
- classify_error(msg) L18929 → `post it.len() <= 7` ("parse"/"resolve"/"type"/"compile", 최대 7자)

**비교체 분류 (80개 잔여 주요 skip 이유)**:
- LLVM IR codegen 함수들: 출력이 입력보다 훨씬 큰 IR 생성
- Parser/AST 변환 함수들: S-expression 변환으로 크기 예측 불가
- fix_ret_line/fix_typed_ret_placeholders_ir: 들여쓰기 변동으로 방향 불확정
- rpe_simplify_phi: 상수값이 길면 입력 초과 가능
- replace_all_str 계열: new_s > old_s이면 성장 가능

## Verification & Defect Resolution
- cargo test --release ✅ (6278 tests, 0 failed)
- bmb check ✅ warnings: 3028 → 3021 (−7)
- bmb verify ✅ 953/953

## Reflection
- String trivials: 89 → 80 (−9 교체 확인)
- classify_error 신규 "constant bound ≤ 7" 패턴 적용 (카테고리 분류자 고정 반환)
- format_range_attr/compute_ret_range: i64 최대값 20자 × 2 + 고정 포맷 = 54자 상한 정확도 확인
- 전체 89개 분석 완료: 나머지 80개는 IR codegen/파서/포맷터로 의미있는 length 계약 적용 불가
- warnings -7 (vs 편집 9개): semantic_duplication 쌍 관계로 일부만 감소

## Carry-Forward
- Actionable: 80개 잔여 중 deeper 분석 필요한 구간 — 초기 라인(177, 231, 239) + IR gen 함수들
  - L177/231/239: 파일 상단 core 함수, 다음 배치에서 검토 필요
  - IR gen 함수들(grow 패턴): pre 조건 추가 가능성 검토
- Structural Improvement Proposals: None
- Pending Human Decisions: None
- Roadmap Revisions: None
- Next Recommendation: Cycle 3130: L177/L231/L239 + 저번호 대역 분석 + pre 조건 추가 가능성 탐색
