# Cycle 2664: M5-5b ✅ 작동 — `mark_str_ptr_if` 새 MIR 명령어로 lower-time type 부재 우회
Date: 2026-05-11

## Re-plan
Cycle 2663 carry-forward: 옵션 A (type-checker → AST attach) 가능성 조사.
**조사 결과**: bootstrap compiler.bmb에는 별도 type-checker 모듈 부재 — type 정보가 AST 노드 type / str_sb / struct field type 등에 분산.
SCOPE ADJUST: 옵션 A 큰 변경 → 옵션 D (실용적 hybrid) 채택 — lowering 단계는 conditional 발행, 코드젠 단계에서 결정.

## Scope & Implementation

### 1. 새 MIR 명령어 `mark_str_ptr_if %dest, %src`
- **lowering 단계**: var 케이스에 conditional 발행 (val_type == "var")
- **코드젠 단계**: src var name을 `str_sb`에서 lookup, string이면 dest에 str_ptr_marker push

### 2. 코드젠 디스패치 (`compiler.bmb`)
```bmb
// line 14400 옆에 추가:
else if low_starts_with_at(line, p, "mark_str_ptr_if") { llvm_handle_mark_str_ptr_if(line, p, str_sb) }

// 신규 함수:
fn llvm_handle_mark_str_ptr_if(line: String, p: i64, str_sb: i64) -> String =
    let rest = line.slice(p + 16, line.len());
    let comma = find_comma(rest, 0);
    let dest = trim_end(rest.slice(0, comma));
    let src_start = low_skip_ws(rest, comma + 1);
    let src_clean = trim_end(rest.slice(src_start, rest.len()));
    let src_is_str = is_string_var_sb(src_clean, str_sb);
    let w = if src_is_str { push_str_ptr_marker(str_sb, dest) } else { 0 };
    same_mapping("");
```

### 3. Lowering 발행 (두 경로 모두)
- `lower_array_repeat_sb` (recursive, line 5119-5121): val_type=="var" 케이스 추가
- `step_array_repeat` (iterative, line 5395-5397): val_type=="var" 케이스 추가
- → CLAUDE.md Rule 3 준수

### 4. M5-5b 검증
```bmb
let s = "hello";
let arr = [s; 3];
println(arr[0]);  // "hello"
println(arr[1]);  // "hello"
println(arr[2]);  // "hello"
```
**결과**: ✅ 정상 출력 (이전: 포인터 정수 출력)

## Verification & Defect Resolution

**테스트 결과**:
- `cargo test --release` ✅ 6210 passed (3773+47+13+2354+23+0=6210)
- 골든 테스트 4/4 통과 (`arr_str_println`, `arr_str_alias`, `arr_str_for_loop`, `arr_str_mut_set`)
- M5-5b 신규 케이스 ✅ 작동

**부트스트랩**:
- Stage 1 정상 빌드
- Stage 2/3 미수행 (arena OOM pre-existing)

## Reflection

**Scope fit**:
- 의도 = M5-5b 진단 + fix → 달성 ✅
- 추가 발견 = `mark_str_ptr_if` 패턴이 일반화 가능 — M5-5c, M5-5d도 동일 접근 가능 (다음 사이클)

**Latent defects**:
- 없음 — 회귀 0건, 신규 기능 작동
- M5-5c/d는 미검증 — 같은 인프라로 처리 가능한지 검증 필요 (Cycle 2665+)

**Structural improvement opportunities**:
- 골든 테스트 추가: `test_golden_arr_str_var_repeat.bmb` (Cycle 2665)
- 동일 패턴을 fn-return / struct-field에 확장 (M5-5c, M5-5d)
- 향후 type-checker 분리 시 옵션 A로 더 깨끗하게 통합 가능 (장기)

**Philosophy drift 점검**:
- "복잡도는 기피 사유 아니다" — 옵션 D 채택은 hybrid 실용 (workaround 아님)
- 새 MIR 명령어 추가 = 깔끔한 인프라 확장 (lowering ↔ codegen 책임 분리 유지)
- 근본 해결 = lowering 단계 type 부재를 codegen 단계 lookup으로 우회 (정확한 layer에서 처리)

**Roadmap impact**:
- M5-5 매트릭스 4/7 → 5/7 ✅ (var-repeat 추가)
- M6 type registry 설계 변경 — 즉시 큰 인프라 불필요, `mark_str_ptr_if` 패턴 일반화로 진행
- M3-2 잔여 작업 = HUMAN 결정 (publish, README) 대기

**User-facing quality**:
- BMB 사용자 관점: `let s = "x"; let arr = [s; N]; println(arr[i])` 자연스럽게 작동
- 이전: 포인터 정수 출력 (혼란) → 현재: 정확한 string 출력
- AI 친화 = LLM이 작성하는 자연 패턴 지원 확대

## Carry-Forward
- Actionable:
  - Cycle 2665: M5-5b 골든 테스트 추가 + M5-5c (`fn() -> Array<String>`) 시도
  - Cycle 2666+: M5-5d (`p.field[i]`) 시도, 동일 패턴 일반화
- Structural Improvement Proposals:
  - `mark_str_ptr_if` 일반화 — fn-return-type, struct-field-type 처리에 동일 적용 가능
  - 장기: type-checker 분리 + AST type attach (장기 architectural improvement)
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: M5-5 매트릭스 5/7 ✅ (var-repeat 추가) — ROADMAP 갱신 예정
- Next Recommendation: Cycle 2665 — 골든 테스트 추가 + M5-5c 시도
