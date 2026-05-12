# ISSUE-20260512 — store_u8 with `addr_expr + 0` pattern: null-base inttoptr drops store

## 핵심 메타

**우선순위**: **P0** (silent correctness bug — output divergence without compile/run error)
**영역**: codegen / llvm_text.rs (or .rs) / store_u8 lowering
**상태**: **CLOSED** — Cycle 2777 fix 완료 (llvm_text.rs param_set heuristic 제거, 6 intrinsics)
**estimated_cycles**: 3-5 (hypothesis — compiler operand selection fix + verify)

## 측정 stamp

| 필드 | 값 |
|------|----|
| `measurement_date` | 2026-05-12 (Cycle 2772) |
| `stale_after` | 2026-08-12 |
| `measurement_source` | `ecosystem/benchmark-bmb/benches/real_world/json_serialize/bmb/main.bmb` + minimal repro test_buf2.bmb |
| `observed_rate` | json_serialize: `Array: {1,2,3,4,5]` (BMB) vs `Array: [1,2,3,4,5]` (C). buf_char 호출이 store 누락 (pos=0 시) |
| `scope` | `store_u8(buf + pos, c)` 패턴 + pos=0 + 함수 인자 buf 케이스 |
| `env_hash` | win32 / LLVM 21.1.8 / MSYS2 UCRT64 |

## 문제

`store_u8(buf + pos, c)` 패턴에서 buf와 pos가 둘 다 함수 인자 i64일 때, BMB 컴파일러는 IR로 다음과 같이 lowering:

```llvm
%sgep_base.0 = inttoptr i64 %pos to ptr           <- pos를 base ptr로 선택 (잘못)
%sgep_elem.0 = getelementptr inbounds i8, ptr %sgep_base.0, i64 %buf
store i8 %c, ptr %sgep_elem.0
```

`pos = 0` 시 `inttoptr 0` → null ptr → GEP from null = UB → **LLVM이 store 제거**.
`pos > 0` 시 `inttoptr pos` → 비-null but invalid ptr → 그러나 GEP + buf로 valid addr 결과 → store 동작.

→ **store 동작이 pos 값에 의존**. correctness 불일치.

## 핵심 증거

### Minimal repro

```bmb
@inline fn buf_char_a(buf: i64, pos: i64, c: i64) -> i64 = {
    store_u8(buf + pos, c);  -- pos를 base로 선택 → pos=0 시 null base
    pos + 1
};

fn main() -> i64 = {
    let buf = calloc(20, 1);
    let _p1 = buf_char_a(buf, 0, 91);   -- write '[' at pos 0
    let _p2 = buf_char_a(buf, 1, 49);   -- write '1' at pos 1
    let _p3 = buf_char_a(buf, 2, 93);   -- write ']' at pos 2
    store_u8(buf + 3, 0);                -- null terminator
    let _u = puts_cstr(buf);              -- expected "[1]", got "" (empty)
    0
};
```

출력: empty string. pos=0 시 store 누락, pos=1/2 시도 누락 (왜? — null base GEP의 LLVM 더 일반화된 optimize-out 가능성).

### IR (buf_char_a):
```
%sgep_base.0 = inttoptr i64 %pos to ptr        <- buggy: pos as base
%sgep_elem.0 = getelementptr inbounds i8, ptr %sgep_base.0, i64 %buf
%store_u8_trunc.0 = trunc i64 %c to i8
store i8 %store_u8_trunc.0, ptr %sgep_elem.0
```

### Compare: 직접 `store_u8(buf + 0, 91)` (인자 아닌 local buf)

```bmb
fn main() -> i64 = {
    let buf = calloc(100, 1);
    store_u8(buf + 0, 91);  -- 정상 작동
    ...
};
```

여기서는 buf가 함수 외부 local이므로 codegen이 buf를 base로 정확히 인식. 회귀 없음.

## 추정 root cause

BMB compiler `store_u8(addr_expr, value)` codegen에서 `addr_expr = a + b` 시 base 선택 휴리스틱이 잘못된 operand 선택. 함수 인자가 둘 다 i64일 때 (heap pointer vs offset 구분 불가) 잘못된 선택 가능성.

후보 위치:
- `bmb/src/codegen/llvm.rs` (inkwell backend) `store_u8` 인라인 codegen
- `bmb/src/codegen/llvm_text.rs` (text backend) 동일

## 영향 평가

| 영역 | 영향 |
|------|------|
| **correctness** | 🚨 silent — compile success + run success + 잘못된 출력 |
| json_serialize bench | ✅ confirmed (`Array: {1,2,3,4,5]`) |
| 다른 bench | ⚠️ 비슷한 패턴 잠재 (csv_parse 진단 시 동일 가능성 후보) |
| 부트스트랩 | ⚠️ store_u8 사용 위치 점검 필요 |
| measurement | 🚨 verify_bench_outputs 도구가 catch 했음 — 이런 silent bug 사례가 더 있을 수 있음 |

## 해결 방안 (Decision Framework)

### Level 1 — 언어 스펙
- `store_u8(addr, value)` 의미는 명확. 변경 불필요.

### Level 2 — 컴파일러 구조 (proper fix)
- `addr_expr = a + b` 시 base/offset 선택 휴리스틱 점검
- 첫 operand을 항상 base로 선택 (source order 보존)
- 또는: 둘 다 inttoptr 후 add로 lowering (UB 회피)
- 또는: BMB type system에 `*u8` / `*i64` 명시 구분 강화 (이미 일부 지원 — typed pointer arr[idx*3])

### Level 3 — 최적화 패스
- N/A (lowering 자체 문제)

### Level 4 — 코드 생성
- text + inkwell 양쪽 fix 필요 (Rule 7)

### Option A: 컴파일러 fix (proper)
- `estimated_cycles`: 3-5 **(hypothesis)**
- 절차: store_u8 codegen identify → 첫 operand을 base로 → text + inkwell 동시 fix → verify_bench_outputs PASS
- 리스크: 다른 store 패턴 회귀 가능성
- Rule 6: Rust 동결, bootstrap에 동일 lowering 있으면 거기에 fix

### Option B: bench source workaround
- `estimated_cycles`: 1-2
- 절차: `store_u8(buf + pos, c)` → 명시적 `let addr = buf + pos; store_u8(addr, c)` (test 결과 동일 bug — 이 workaround 작동 안 함)
- → workaround 단순한 솔루션 부재 (Principle 2 위반 회피)

### Option C: BMB source 분해
- `estimated_cycles`: 1
- 절차: buf_char 함수 제거, 인라인 store_u8 (let-free 패턴 시도)
- 리스크: source readability 손실, bench fairness 미보장

## HUMAN 결정 필요

- Rule 6 (Rust 동결) vs P0 correctness bug — 부트스트래핑 차단 아닌 silent correctness 회귀
- bootstrap에 동일 패턴 있는지 점검 후 결정

## 종결 기준

- [ ] minimal repro test_buf2.bmb 정상 출력 ("[1]")
- [ ] json_serialize bench `Array: [1,2,3,4,5]` 출력
- [ ] verify_bench_outputs.py json_serialize PASS
- [ ] 다른 bench 회귀 없음 (full Tier 1/3 verify PASS)
- [ ] 골든 테스트 추가 (`store_u8(arg_a + 0, c)` 패턴)

## 메타

- 관련 ISSUE:
  - `ISSUE-20260511-golden-flakiness-inttoptr.md` (동일 family — `inttoptr` codegen)
  - `ISSUE-20260512-bench-output-fairness-survey.md` (parent — detection mechanism)
- 인용 cycle: cycle-2772.md (진단)
- 외부 참조: `bmb/src/codegen/{llvm,llvm_text}.rs` store_u8 lowering
