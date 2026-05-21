# Cycle 2697: set_cover 회귀 단일-질문 IR 진단 + fix
Date: 2026-05-11

## Re-plan
Carry-Forward (Cycle 2696): 회귀 3개 분석. Advisor 권고: 단일 질문 IR diff (set_cover).

## Scope & Implementation

### 단일 질문
> `%_t133 = or i64 %_t125, %_t131, %ne` — `or` opcode 3번째 인자 `%ne` 어디서 발행?

### 진단

`bootstrap/compiler.bmb:7142` builtin method intrinsic 분기:
```bmb
} else if fn_name == "@bit_or" {
    let args = line.slice(paren_pos + 1, close_pos);
    let comma = find_char(args, 0, 44);
    let left = trim_end(args.slice(0, comma));
    let right = trim(args.slice(comma + 1, args.len()));
    "  " + dest + " = or i64 " + left + ", " + right
}
```

이 분기는 **arity 체크 없음**. fn_name이 `@bit_or`이면 무조건 LLVM `or i64`로 발행. 3-arg 호출 시 `right`에 콤마가 남아 IR이 `or i64 a, b, c` 잘못.

### 원인

`test_golden_set_cover.bmb` 35라인 `fn bit_or(a, b, n) -> i64` — user 정의 3-arg 함수. lowering 시 망글링으로 `@bit_or`이 되면서 builtin `@bit_or` (2-arg) 분기와 충돌.

### Fix (즉시)

source rename: `bit_or` → `bits_or_n` (4 occurrences). 검증:
- IR opt -O2 ✅
- 실행: stdout "2" (expected "2" 일치) ✅

### Fix (장기) — 권장

`compiler.bmb:7142` 분기에 arity 체크 추가:
```bmb
} else if fn_name == "@bit_or" {
    let args = line.slice(paren_pos + 1, close_pos);
    let first_comma = find_char(args, 0, 44);
    let after = first_comma + 1;
    let second_comma = find_char(args, after, 44);
    if second_comma < args.len() {
        // user-defined function with same name — fall through to normal call
        ""  // or signal "not builtin"
    } else {
        // 2-arg builtin path (기존 로직)
        ...
    }
}
```

또는 더 깔끔하게: `@bit_or` builtin은 method 호출 (`x.bit_or(y)`)만 매칭하도록 namespacing. 별도 cycle 권장.

### token_scan / tokenizer (회귀 2)

직접 검증: token_scan 실행 → segfault (exit 139). 본 사이클 단일 질문 범위 외 — 이슈에 segfault fact 기록, 별도 cycle 분석.

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| set_cover IR opt -O2 | ✅ (rename 후) |
| set_cover 실행 stdout | ✅ "2" 매니페스트 일치 |
| token_scan 회귀 (별개 분석) | ❌ segfault 확인 — 이슈 갱신 |

결함: set_cover ✅ fix. token_scan/tokenizer 잔재 (별도 cycle).

## Reflection

**핵심 통찰**:
- builtin method intrinsic 분기에 arity 체크 누락 — Cycle 2384 시점 도입 시 가정한 "method 호출만 들어옴"이 깨짐
- user fn 이름이 builtin과 충돌 시 silent IR corruption — 위험한 패턴 (lint 후보)

**도그푸딩 가치**:
- 골든 스위트가 회귀 감지 (advisor 권고대로 end-to-end 실행)
- 단일 질문 집중으로 5분 내 원인 진단

**Roadmap impact**:
- M4-9 (clang knapsack outlier) 분석 외에 builtin arity 체크 정정 후보 추가
- token_scan segfault — 별도 자율 cycle

## Carry-Forward
- Actionable:
  - Cycle 2698 — 골든 스위트 재실행 (12 FAIL → 1-2 잔존 예상) + HANDOFF 갱신
  - Cycle 2699 — 통합 commit
- Structural Improvement Proposals:
  - **컴파일러**: `@bit_*` builtin 분기에 arity 체크 (장기, 위험도 medium)
  - **Track Q lint**: user fn 이름 = builtin 이름 충돌 감지 규칙
  - **테스트 컨벤션**: golden test에서 builtin 예약 이름 회피 가이드 추가
- Pending Human Decisions: 없음
- Roadmap Revisions: 없음
- Next Recommendation: Cycle 2698 — 골든 스위트 신규 manifest 적용 후 재실행 (백그라운드)
