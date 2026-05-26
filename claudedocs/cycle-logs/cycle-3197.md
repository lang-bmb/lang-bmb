# Cycle 3197: chained_comparison 91→0 (수동 변환 완료)
Date: 2026-05-26

## Re-plan
Plan valid. Inherited scope: 잔여 91개 chained_comparison 수동 변환 — compound or 조건, fn-call first arm, mixed match+else-if.

## Scope & Implementation

**수동 변환 7건 (91→0):**

### 1. fn_name (line ~15363): 91→26 (−65)
- 157-arm 수동 매핑: 3-level nested match + 74개 else-if 체인을 단일 flat match로 통합
- `match fn_name { "@bmb_append_file" => "pp", ...(157 arms)..., _ => "" }`
- compound or 포함 암들은 개별 arm으로 분리

### 2. k@5953 (parse_type_params): 26→22 (−4)
- TK_EQ/TK_EOF/TK_LBRACE 3개 조건 → match 변환
- `match k { 2000000302 => tok_end(t), 2000000900 => pos, 2000000307 => pos, _ => skip_where_clause(...) }`

### 3. type_info@6517, param_type@27123: 22→22 (−0 직접, 최적화)
- type_info: 5-arm match (`1=>"~d"`, `2=>"~s"`, `3=>"~a"`, `4=>"~af"`, `_=>fn분기`)
- param_type: 4-arm match (`"f64"=>"d"`, `"ptr"=>"p"`, `"i32"=>"s"`, `_=>"i"`)

### 4. kind@3197 (parse_atom): 22→11 (−11)
- 첫 arm `is_int_literal(kind)` fn-call 유지, 나머지 else-if 전체를 `else { match kind { ... } }` 로 감싸기
- 16-arm match (TK_FLOAT, TK_TRUE/FALSE, TK_IDENT, TK_STRING, TK_CHAR, TK_NOT, TK_BNOT, TK_MINUS, TK_LPAREN, TK_LBRACE, TK_SET, TK_NIL, TK_LBRACKET, TK_AMP, TK_FN)

### 5. extract_pre_asts@32591, extract_post_asts@32649: 11→9 (−2)
- TK_EQ/TK_EOF/TK_LBRACE: `""`, TK_PRE/TK_WHERE 처리 → flat match

### 6. k@2757 (parse_block_let), k@4173 (parse_let_skip_type): 9→7 (−2)
- 타입 키워드 체인 (TK_I64/I32/F64/BOOL/STRING/STAR/QUESTION 등 14-arm) → match 변환

### 7. ntype@7613 (get_exit_label): 7→0 (−7)
- 문제 발생: `content.find('    if ntype == "if" {')` 가 `shallow_blocks`를 먼저 찾아 양쪽 함수 body 삭제
- 복구: git HEAD에서 `shallow_blocks` 원본 복원 + `get_exit_label` flat match (22-arm, `"for"` arm 추가) 삽입
- 22-arm: if/while/loop/for/for_incl/for_step/for_step_incl/break/continue/return/block/seq/let/let_mut/tuple/array/call/method/struct_init/binop/unary/assign/_ 

**CRLF 버그 발생 및 해소:**
- fn_name flatten 스크립트가 `\n` split + `\r\n` join → `\r\r\n` 생성
- 해소: `re.sub(b'\r+\n', b'\r\n', data)` 정규화 적용

## Verification & Defect Resolution
- `bmb check`: 1,227 warnings (chained_comparison: **0** ✅, non_snake_case: 108, semantic_duplication: 1,119)
- Stage 1 bootstrap: ✅ `{"type":"build_success","output":"bootstrap/compiler.exe"}`

## Reflection
- **Scope fit**: 수동 변환 7건으로 chained_comparison 91→0 완전 달성. M10 Track A ✅ COMPLETE.
- **총 진행**: Cycle 3192 시작 기준 ~757 → 0 (−757 chained_comparison)
- **non_snake_case 108개**: 전부 TK_*() 함수 의도적 대문자 명명. Human Decision 미결.
- **semantic_duplication 1119개**: 구조적 장기 작업.
- **get_exit_label 버그**: 검색 앵커가 충분히 고유하지 않으면 잘못된 함수 body 삭제 가능. 함수 시작 시그니처를 앵커로 사용해야 함.

## Carry-Forward
- Actionable: non_snake_case 108개 처리 방향 Human Decision
  - Option A: TK_FN → tk_fn (대규모 리네이밍, 가독성 변화)
  - Option B: BMB `@allow(non_snake_case)` 어노테이션 지원 추가
- Actionable: semantic_duplication 1119개 — M10 잔여 사이클(3198-3199)에서 가능 범위 분석
- Structural Improvement Proposals: None
- Pending Human Decisions: TK_* non_snake_case 처리 방향
- Roadmap Revisions: chained_comparison 완전 달성으로 M10 Track A 완료
- Next Recommendation: Cycle 3198 — semantic_duplication 1119개 분석 + non_snake_case Human Decision 대기
