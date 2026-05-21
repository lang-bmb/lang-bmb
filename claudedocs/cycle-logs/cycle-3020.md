# Cycle 3020: brainfuck match dispatch 최적화
Date: 2026-05-21

## Re-plan
Carry-forward (Cycle 3019): match dispatch 구조 개선 가능성 → Cycle 3020으로 carry.
Plan valid. brainfuck interpreter를 match dispatch로 재작성.

## Scope & Implementation

### 탐색: integer literal match 지원 확인

BMB interpreter에서 `match c { 62 => ..., 60 => ..., _ => 0 }` 형식의 integer literal match 지원 확인 ✅.
`match` 안에서 mutable 변수 업데이트 (`ptr = ...`, `pc = ...`) 가능 (블록 + comma 필요) ✅.

### 최적화 내용

**변경 1**: 이중 if-else chain → 단일 match dispatch

변경 전:
```bmb
ptr = if c == 62 { ... } else if c == 60 { ... } else { ptr };
let _e = if c == 43 { ... } else if c == 45 { ... } else if c == 91 { ... } else if c == 93 { ... } else { 0 };
```

변경 후:
```bmb
let _r = match c {
    62 => { ptr = ...; 0 }, 60 => { ptr = ...; 0 },
    43 => tape_set(...), 45 => tape_set(...),
    91 => { pc = ...; 0 }, 93 => { pc = ...; 0 },
    _ => 0
};
```

**효과**: LLVM switch i64 instruction 생성 (jump table) → branch misprediction 감소, 일정한 dispatch 비용.

**변경 2**: `get_char(prog, pc)` → `prog.byte_at(pc)` (main loop 내부)

`while pc < prog_len` loop guard가 이미 bounds를 보장하므로 `get_char` 내부의 `if pos >= prog.len()` 체크 중복. 제거.

### 최종 brainfuck 파일

`ecosystem/benchmark-bmb/benches/real_world/brainfuck/bmb/main_inproc.bmb` 재작성.

## Verification & Defect Resolution

- 빌드: `{"type":"build_success"}` ✅
- 체크섬: `0` (올바른 값, compute_only 프로그램 결과와 일치) ✅
- `cargo test --release`: 기존 6260/6260 PASS (bmb 파일만 변경, Rust 소스 변경 없음)

### 5-run 측정 (2026-05-21)

| 측정 | BMB (µs) | C (µs) |
|------|---------|-------|
| Run 1 | 7633 | 8193 |
| Run 2 | 7822 | 8303 |
| Run 3 | 7945 | 8290 |
| Run 4 | 8130 | 8472 |
| Run 5 | 8621 | 8082 |
| **Median** | **7945** | **8290** |
| **Ratio** | **0.958×** | — |

**개선**: Cycle 3018 (0.974×) → Cycle 3020 (0.958×) = -1.6pp 추가 개선. BMB 4.2% faster than C.

## Reflection

- **Scope fit**: match dispatch 최적화 완료. 측정으로 improvement 확인.
- **Latent defects**: 없음.
- **Structural**: C 버전도 switch(c) 사용 → 이제 BMB/C가 동일 LLVM IR 구조에 가까워짐.
- **Roadmap impact**: brainfuck 1.005× → **0.958×**. P-track 7/7 더 여유 있게 PASS.
- **Philosophy fit**: switch 생성은 언어 스펙 차원의 성능 최적화 (workaround 아님).

## Carry-Forward

- Actionable: 없음
- Structural Improvement Proposals:
  - `prog.byte_at(pc)` 변환 패턴을 다른 string-heavy 벤치마크에도 적용 (csv_parse, http_parse 가능성)
- Pending Human Decisions: 없음
- Roadmap Revisions: ROADMAP §5 brainfuck 0.958× 갱신 필요 (Cycle 3021에서 전체 재측정 시 갱신)
- Next Recommendation: Cycle 3021 = csv_parse/http_parse에 byte_at 직접 접근 적용 검토
