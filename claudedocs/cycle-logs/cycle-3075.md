# Cycle 3075: 상태 진단 — M7 Contract Pipeline 전제 검증
Date: 2026-05-23

## Re-plan
Carry-Forward from Cycle 3074: 없음.
ROADMAP: M7 착수. 진단 후 M7-1 contract 부착 시작.
이번 사이클 범위: M7 착수 전 5개 전제 조건 검증.

## Scope & Implementation

### 진단 1: for-in range `for i in 0..n`
**결론: ✅ 이미 지원됨 — 언어 갭 없음**

- Rust grammar.lalrpop 1603/1767행: `"for" var "in" SpannedRangeExpr "{" ... "}"` → `Expr::For`
- bootstrap/compiler.bmb 2069행: `if tok_kind(t3) == TK_DOTDOT() { parse_for_end(...) }`
- TK_DOTDOT (283행), TK_DOTDOTEQ (285행) 토큰 존재
- 실제 사용 예: compiler.bmb 9037행 `for _iter in 0..5 {`

### 진단 2: Bootstrap pre/post 파싱 + assume 주입
**결론: ✅ 완전 구현됨**

- TK_PRE (219행), TK_POST (220행) 토큰 정의
- Lexer: 541행 `else if word == "pre" { make_tok(TK_PRE(), endpos) }`
- `extract_pre_asts` (16277행), `extract_post_asts` (16305행) — 소스에서 AST 추출
- `build_contracts_map` (16333행), `build_post_contracts_map` (16384행) — 함수별 contract 맵 빌드
- 컴파일 파이프라인 17304-17311행에서 자동 활성:
  ```
  let contracts_map = build_contracts_map(source);
  let assumed_ir = inject_contract_assumes_all(cleaned_ir, contracts_map);
  let post_map = build_post_contracts_map(source);
  let post_assumed_ir = inject_post_assumes_all(assumed_ir, post_map);
  ```
- `annotate_return_ranges` (17311행) — post-condition에서 return range 속성 생성

### 진단 3: LLVM assume 주입 표현력 한계
**결론: ⚠️ 정수 비교만 지원**

`contract_ast_to_assumes` (16461행) 지원 패턴:
- `(binop and (...) (...))` — 재귀 분리
- `(binop CMPOP (var <x>) (int N))` — 파라미터 vs 정수 비교

**미지원**: 메서드 호출 (`method.len()` → `(method len (var))`)은 lhs_op = "" → 빈 IR 반환.

M7-1 HANDOFF 타겟 (`method_to_runtime_fn`, `get_call_return_type`, `is_string_returning_fn`)은 모두 String/bool 반환 → assume 미발동.
→ 정수 파라미터 함수 병행 타겟으로 **즉시 LLVM 혜택**도 달성 가능.

### 진단 4: Z3 검증 bootstrap 지원
**결론: ❌ 미구현 — M7-2 범위**

bootstrap/compiler.bmb에 Z3/SMT 관련 코드 없음.
현재 contracts는 `llvm.assume` (성능 최적화)에만 사용됨.
M7-2 = bootstrap에서 Z3 IPC 호출 파이프라인 추가 필요.

### 진단 5: Rule 6 이중 구현 (Rust + bootstrap) 필요 여부
**결론: ✅ M7-1에는 불필요**

M7-1 = 기존 함수에 pre/post 조항 추가. 새 AST 노드 없음 → 이중 lowering 수정 불필요.
pre/post 파싱은 이미 양쪽(Rust + bootstrap)에 구현됨.

## Verification & Defect Resolution

### 스모크 테스트 3종 (진단 직후 실행)

**테스트 1: Z3 채널 생존 확인**
```
./target/release/bmb verify tests/examples/valid/010_simple_contract.bmb
{"type":"verify_result","total":2,"verified":2,"failed":0}
```
→ ✅ Z3 파이프라인 정상

**테스트 2: llvm.assume 주입 확인 (find_char)**
- `bootstrap/compiler.bmb` 7972행 `find_char`에 `pre pos >= 0 and ch >= 0` 추가
- Stage 1 빌드: `build_success` ✅
- `--emit-ir` 85558행: `%_assume_0 = icmp sge i64 %pos, 0; call void @llvm.assume(i1 %_assume_0)` ✅
- **발견**: 다중 `pre` 절은 `pre A\n  pre B` 형식 미지원 → `pre A and B` 단일 절로 결합 필수

**테스트 3: bootstrap/compiler.bmb 전체 Z3 검증**
```
./target/release/bmb verify bootstrap/compiler.bmb
{"type":"verify_result","total":1513,"verified":1513,"failed":0}
```
→ ✅ 1513개 함수 전부 검증 완료 (계약 없는 함수는 vacuously true)

## Reflection
- **Scope fit**: 100% — 5개 진단 + 3개 스모크 테스트 완료
- **핵심 발견**: for-in range 언어 갭 이미 해소 + Z3/llvm.assume 파이프라인 완전 동작 확인
- **아키텍처 인사이트**: `bmb verify compiler.bmb` = 1513/1513 통과. 즉시 계약 작성 가능
- **한계 발견**: assume 주입이 정수 비교만 지원 → HANDOFF 3개 타겟 (String/bool 반환)은 assume 미발동, Z3 전용
- **문법 발견**: 다중 pre 절은 `pre A and B` 형식만 지원 (줄 분리 불가)

## Carry-Forward
- **Actionable**: M7-1 착수 — compiler.bmb 함수들에 pre/post contract 부착
  - **Track A** (정수 파라미터 → llvm.assume 즉시 혜택): `find_char` (완료 ✅), `find_pattern_noa`, `skip_ws_comments` 등
  - **Track B** (HANDOFF 타겟 → Z3 spec only): `method_to_runtime_fn`, `get_call_return_type`, `is_string_returning_fn`
- **Structural Improvement Proposals**:
  - assume 주입 범위 확장 (String len, method call → `ptrtoint + icmp`) — M7 선택 과제
- **Pending Human Decisions**: 없음
- **Roadmap Revisions**: 없음 (모든 전제 조건 스모크 테스트로 확인)
- **Next Recommendation**: Cycle 3076 — M7-1 Track A 대량 contract 부착 (정수 파라미터 함수 10+종)
