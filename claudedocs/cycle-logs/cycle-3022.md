# Cycle 3022: csv_parse 단일-load 최적화 (1.018× → 0.846×)
Date: 2026-05-21

## Re-plan
Carry-forward: csv_parse byte_at 단일-load 최적화 탐색.
csv_parse 이미 str_data + load_u8 패턴 사용. byte_at 적용 불가.

**발견**: `while cond and load_u8() != 10 and load_u8() != 13` 패턴에서 동일 바이트를 2회 load. break-based 단일-load 구조로 전환하면 메모리 액세스 50% 감소.

## Scope & Implementation

### 발견: 이중 load 패턴

BMB `and` 단락 평가로 인해 다음 코드는 `ptr + line_end` 바이트를 2회 load:
```bmb
while line_end < len and load_u8(ptr + line_end) != 10 and load_u8(ptr + line_end) != 13 {
```

C에서는 `data[line_end] != '\n' && data[line_end] != '\r'` — LLVM이 단일 load로 최적화.
BMB에서는 `and` 체인이 별도 basic block을 생성 → CSE 최적화 불가.

### 수정: break-based 단일-load

3곳의 이중-load while 조건 → 단일-load break 패턴으로 전환:

1. **line_end 스캔**:
```bmb
while line_end < len { {
    let b = load_u8(ptr + line_end);
    if b == 10 or b == 13 { break };
    line_end = line_end + 1; ()
} };
```

2. **앞 공백 스킵** (2곳):
```bmb
while p < line_end { {
    let wb = load_u8(ptr + p);
    if wb != 32 and wb != 9 { break };
    p = p + 1; ()
} };
```

3. **비인용 필드 스캔**:
```bmb
while p < line_end { {
    let fb = load_u8(ptr + p);
    if fb == 44 { break };
    total_chars = total_chars + 1;
    p = p + 1; ()
} };
```

**정확성**: 체크섬 55003850000 (원본 = 신규) 동일 ✅

## Verification & Defect Resolution

- 빌드: `{"type":"build_success"}` ✅
- 체크섬 일치: 55003850000 ✅

### 7-run median 측정 (2026-05-21)

| 측정 | BMB (µs) | C (µs) | 비율 |
|------|---------|-------|------|
| 이전 (Cycle 3019) | ~3088 | ~2721 | ~1.135× |
| 이전 (Cycle 3017 3-run) | 3103 | 3049 | 1.018× |
| **신규 7-run median** | **2443** | **2889** | **0.846×** |

**개선**: 1.018× → **0.846×** (-17.2pp). BMB 15.4% faster than C.

**근본 원인**: and-chain 조건에서 동일 바이트 2회 load → break-based 단일 load로 CSE 달성.

## Reflection

- **Scope fit**: csv_parse 단일-load 최적화 완료.
- **Latent defects**: 없음.
- **Philosophy**: C LLVM이 자동으로 CSE하는 것을 BMB에서 `and`-chain 구조 때문에 못 함 → 언어 수준에서 single-load 패턴 강제. 근본 fix는 BMB의 `and` 연산이 동일 subexpression을 CSE하도록 컴파일러 개선 (MIR 수준).
- **Roadmap impact**: csv_parse 0.846× — P-track 7/7 PASS + 전체 BMB faster.
- **Structural improvement**: BMB MIR에서 `and/or` 조건의 동일 load CSE 최적화 패스 추가 가능 (별도 ISSUE 가치 있음).

## Carry-Forward

- Actionable: 없음
- Structural Improvement Proposals:
  - BMB MIR level CSE for `and`/`or` chains with identical subexpressions (별도 ISSUE 검토)
  - http_parse `and` chain 패턴 점검 (이미 str_data + load_u8, 이중 load 없는지 확인)
- Pending Human Decisions: 없음
- Roadmap Revisions: ROADMAP §5 csv_parse 0.846× 갱신
- Next Recommendation: Cycle 3023 = ROADMAP 갱신 + commit + http_parse 패턴 점검
