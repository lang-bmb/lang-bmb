# Cycle 3219: M11-C Phase 1 — ifs_flex_check_goto Z3 Fix + Stack Array 구조 매핑
Date: 2026-05-27

## Re-plan

**Inherited scope**: Cycle 3218 Carry-Forward — M11-A 종료 선언, M11-C 전환.
**확정 작업**:
1. (a) `ifs_flex_check_goto` `pre next_p >= 0` 추가 → Z3 141/141 달성 (5분 quick win)
2. (b) grammar.lalrpop 위치 확정 + `[T; N]` 기존 지원 수준 파악
3. (c) bootstrap의 현 type/parser/codegen 구조 매핑 → Cycle 3220+ 구현 계획

**Trigger**: ⚪ NONE — plan valid.

## Scope & Implementation

### (a) ifs_flex_check_goto Z3 Fix

`bootstrap/compiler.bmb:25605` 수정:

**Before**: `pre n_pure >= 0 and max_pure >= 0`  
**After**: `pre n_pure >= 0 and max_pure >= 0 and next_p >= 0`

**원인**: 함수가 `next_p`를 반환하는 경로가 2개이고 (`goto_target == ""`, `goto_tgt == goto_target` 모두),  
`post it >= 0`을 보장하려면 `next_p >= 0`이 precondition으로 필요.

**결과**: Z3 141/141 ✅ (이전: 140/141, `ifs_flex_check_goto` pre-existing FAIL)

### (b) grammar.lalrpop 위치 + [T; N] 현황

- **위치**: `bmb/src/grammar.lalrpop` (NOT `bmb/src/parser/grammar.lalrpop`)
- **[T; N] 타입 지원** (Rust grammar line 1144):
  ```lalrpop
  "[" <t:Type> ";" <n:"int"> "]" => Type::Array(Box::new(t), n as usize)
  ```
- **[val; N] 리터럴 지원** (line 2483):
  ```lalrpop
  "[" <value:SpannedExpr> ";" <count:"int"> "]" => Expr::ArrayRepeat { ... }
  ```
- Rust 컴파일러는 완전 지원. Bootstrap 컴파일러는 부분 지원 (타입 스킵).

### (c) Bootstrap 구조 매핑

#### 파서 레벨 (현재 상태)

| 함수 | 역할 | `[T; N]` 처리 |
|------|------|---------------|
| `parse_block_let_skip_array_type` | `let x: [T; N] =` 파싱 | **타입 스킵** → `parse_block_let_value` |
| `parse_let_skip_array_type` | expr 컨텍스트 동일 | **타입 스킵** |
| `parse_param_array_type` | `fn(x: [T; N])` 파싱 | **i64로 처리** |
| `skip_array_type_tokens` | return type `[T; N]` | **i64로 처리** |
| `parse_expr` → array_repeat | `[val; N]` 리터럴 | `(array_repeat val count)` AST |

#### MIR 레벨 (현재 상태)

- `(array_repeat val N)` → `lower_array_repeat_sb`:
  - `calloc(N+2, 8)` (heap, i64 elements)
  - Header: cap@[0], len@[1]
  - Elements: [2..N+1] at 8-byte intervals
  - **Stack array와 완전히 다른 구조**

- Alloca: `llvm_gen_alloca` → 항상 `name = alloca i64`

#### Codegen 레벨 (현재 상태)

- `gep` MIR → `getelementptr i64` (8-byte 단위)
- `alloca %name` → `%name = alloca i64`
- **i8 단위 alloca 지원 없음**

#### 필요한 변경 (Cycle 3220+ 구현 계획)

**전략**: 새 AST 노드 `stack_bytes_new` 추가 방식 vs 타입 시스템 확장 방식

| 접근 | 장점 | 단점 |
|------|------|------|
| `stack_bytes(N)` 빌트인 | 최소 변경, 즉시 구현 가능 | BMB 언어 설계 관점에서 저수준 |
| `let x: [u8; N] = [0u8; N]` 타입 기능 | 언어 설계 proper, 일반화 | 파서+타입마커+codegen 전체 변경 필요 |

**결론**: `[u8; N]` stack array 타입을 제대로 지원하되, 첫 단계로 **i8 배열 타입**에 한정.

구현 계획:
1. **Cycle 3220**: 파서 확장 — `parse_block_let_skip_array_type` → `parse_block_let_array_type_aware`
   - `[u8; N]` 타입 annotation 감지 + N 캡처
   - 타입마커 `stack_bytes_N` 전달
2. **Cycle 3221**: `[0u8; N]` 리터럴 처리 — `(array_repeat 0 N)` + 타입 annotation 조합
   - `lower_let_sb`: 타입 annotation이 `[u8; N]`이면 `lower_stack_array_sb(N)` 분기
   - `lower_stack_array_sb(N)`: emit `  alloca_bytes %_tX, N` MIR
3. **Cycle 3222**: codegen — `alloca_bytes %name, N` → `%name = alloca [N x i8], align 16` + memset
   - `llvm_gen_alloca_bytes` 신규 함수
4. **Cycle 3223**: 타입마커 전파 — `push_stack_bytes_marker` + GEP i8 분기
5. **Cycle 3224-3225**: 테스트 + brainfuck 벤치마크 포팅
6. **Cycle 3226**: Stage 1 빌드 + Fixed Point 검증

**단순화 가능**: brainfuck의 경우 `tape` 변수가 `i64`로 사용되고 `load_u8`/`store_u8` + raw pointer arithmetic을 사용하므로, GEP 변경 없이 기존 코드가 동작함. 핵심은 `tape = calloc(30000, 1)` → `tape = alloca_bytes(30000)` 전환뿐.

## Verification & Defect Resolution

```json
{"type":"lint","file":"bootstrap/compiler.bmb","warnings":0}
{"type":"verify_result","total":141,"verified":141,"failed":0}
```

2390 tests passed ✅ (이전 대비 변화 없음, ifs_flex_check_goto 계약 추가만)

## Reflection

**Scope fit**: 3가지 목표 모두 달성.
- Z3 141/141 ✅ (pre-existing FAIL 해소)
- grammar.lalrpop 위치 확정 + 지원 수준 파악
- bootstrap 파서/MIR/codegen 구조 완전 매핑

**Latent defects**: 없음. `ifs_flex_check_goto` 수정은 계약 강화 (더 제한적 pre). 호출 사이트에서 `next_p < 0`을 전달하는 경우 없음 (사용 패턴 확인).

**Structural improvement opportunities**:
- bootstrap에서 `[T; N]` 타입을 전반적으로 i64로 처리하는 부분 → M11-C에서 단계적 개선
- `lower_array_repeat_lit_sb`의 heap calloc → 향후 type annotation 기반 stack/heap 선택 가능

**Philosophy drift**: 없음. Z3 fix는 명백한 버그 수정.

**Roadmap impact**: Z3 141/141 달성 → 기술 상태 스냅샷 업데이트 필요. M11-C Phase 1 구현 계획 수립 완료.

## Carry-Forward

- **Actionable**: Cycle 3220 — bootstrap 파서에 `[u8; N]` type annotation 인식 + N 캡처
- **Structural Improvement Proposals**: 없음
- **Pending Human Decisions**: 없음
- **Roadmap Revisions**: Z3 FAIL 0개 (141/141) → HANDOFF/ROADMAP 업데이트 필요
- **Next Recommendation**: Cycle 3220 — `parse_block_let_array_type_aware` + `lower_stack_array_sb` 구현
