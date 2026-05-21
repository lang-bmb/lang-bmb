# ISSUE-20260521 — MIR level CSE for `and/or` 조건 내 동일 load_u8 subexpression

**Status: OPEN**
**Priority: P2**
**Category: Compiler Optimization / MIR**

---

## 문제 요약

BMB의 `and`/`or` 연산자는 short-circuit 평가를 수행하며 별도 basic block을 생성한다.
이로 인해 동일 메모리 주소에 대한 `load_u8` 호출이 같은 `and` 체인에 여러 번 나타나면 LLVM CSE가 불가능하다.

### 증상

```bmb
-- 이 코드는 ptr + pos 바이트를 2회 load
while pos < len and load_u8(ptr + pos) != 10 and load_u8(ptr + pos) != 13 {
    pos = pos + 1
};
```

LLVM IR:
```llvm
; Basic block 1
%b1 = call i64 @load_u8(i64 %ptr_pos)
%cmp1 = icmp ne i64 %b1, 10
br i1 %cmp1, label %bb2, label %exit

; Basic block 2 (별도 block → CSE 불가)
%b2 = call i64 @load_u8(i64 %ptr_pos)  ; 중복 load
%cmp2 = icmp ne i64 %b2, 13
br i1 %cmp2, label %body, label %exit
```

C에서는 `data[pos] != '\n' && data[pos] != '\r'`를 같은 operand로 인식, 단일 load로 최적화된다.

---

## 영향

### Cycle 3022-3023 실측 영향

| 벤치마크 | 이중-load | 단일-load (break) | 개선 |
|---------|----------|-------------------|------|
| csv_parse | 1.018× | 0.891× | -12.7pp |
| http_parse | 0.938× | 0.909× | -2.9pp |
| brainfuck (해당 없음) | — | — | — |

### 영향 범위

`and/or` 조건에 동일 `load_u8(ptr + x)` 표현이 2회+ 나타나는 패턴 전반.
`byte_at` 도 동일 문제 가능 (MIR lowering 경로에 따라).

---

## 근본 원인

MIR lowering 시 `and`/`or` binary expression의 왼쪽과 오른쪽이 각각 독립 코드 생성 경로를 거친다. 공통 subexpression 감지 없음.

**근본 fix 위치**: `bmb/src/codegen/llvm_text.rs` (text backend) + `bmb/src/codegen/llvm.rs` (inkwell).
- `and/or` 표현 lowering 시 왼쪽 operand의 load를 임시 변수에 저장
- 오른쪽 operand가 동일 load (같은 pointer + offset) 이면 재사용

또는 MIR 수준에서:
- `mir/mod.rs`의 MIR instruction level에서 공통 subexpression 감지 + 임시 binding

---

## Workaround (이미 적용됨)

break-based 단일-load 패턴:
```bmb
while pos < len { {
    let b = load_u8(ptr + pos);
    if b == 10 or b == 13 { break };
    pos = pos + 1; ()
} };
```

brainfuck/csv_parse/http_parse 벤치마크에 적용 완료 (Cycles 3018-3023).

---

## 수용 기준

1. `while cond and load_u8(ptr + x) != a and load_u8(ptr + x) != b` 패턴이 동일 load를 1회만 emit
2. 기존 short-circuit 의미 보존 (load 전에 `cond` 평가)
3. `cargo test --release` 전체 통과
4. csv_parse 벤치마크 re-measurement: break-based 버전과 동등 성능

---

## 측정 stamp

```
measurement_date: 2026-05-21
stale_after: 2027-05-21
measurement_source: Cycles 3022-3023 (Cycle 2727 platform, inproc harness)
env: Windows 11 / LLVM 21.1.8 / -O2
```
