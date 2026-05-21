# Cycle 3023: http_parse single-load 최적화 + P-track 전체 재측정
Date: 2026-05-21

## Re-plan
Carry-forward (Cycle 3022): http_parse `and` chain 이중-load 점검.
http_parse에 동일 패턴 4곳 발견. Cycle 3022와 동일 기법 적용.

## Scope & Implementation

### http_parse 이중-load 제거 (4곳)

적용 패턴 (Cycle 3022와 동일):

1. **request-line 스킵** (line 34): `!= 10 and != 13` → 단일 load break
2. **header EOL 스캔** (line 49): `!= 10 and != 13` → 단일 load break  
3. **콜론 이후 공백 스킵** (line 67): `== 32 or == 9` → 단일 load break
4. **정수 파싱** (line 73): `>= 48 and <= 57` + body `load_u8` 3번 → 단일 load 재사용

정수 파싱은 기존에 load를 3회 했음 (조건 2회 + 본문 1회):
```bmb
while p < eol and load_u8(ptr + p) >= 48 and load_u8(ptr + p) <= 57 {
    acc = acc * 10 + load_u8(ptr + p) - 48;  -- 3번째 load
```
→ 단일 load로 통합:
```bmb
let db = load_u8(ptr + p);
if db < 48 or db > 57 { break };
acc = acc * 10 + db - 48;  -- load 재사용
```

**정확성**: BMB 체크섬 160002980000, C 체크섬 160002980000 → 완전 일치 ✅

## Verification & Defect Resolution

- 빌드: `{"type":"build_success"}` ✅
- 체크섬 일치 ✅

### P-track 전체 5-run median (2026-05-21, Cycles 3017-3023 최적화 후)

| 벤치마크 | BMB (µs) | C (µs) | 비율 | 이전 (Cycle 3017) | 개선 |
|---------|---------|-------|------|-----------------|------|
| brainfuck | 7632 | 7984 | **0.956×** | 1.037× | -8.1pp |
| csv_parse | 2557 | 2869 | **0.891×** | 1.018× | -12.7pp |
| http_parse | 2151 | 2366 | **0.909×** | 0.938× | -2.9pp |
| lexer | 1388 | 8198 | **0.169×** | 0.175× | -0.6pp |
| json_parse | 2605 | 3168 | **0.822×** | 0.815× | +0.7pp |
| json_serialize | 464 | 695 | **0.668×** | 0.701× | -3.3pp |
| sorting | 461875 | 2996508 | **0.154×** | 0.154× | stable |

**7/7 PASS. 전부 BMB faster.**

## Reflection

- **Scope fit**: http_parse 최적화 + 전체 P-track 재측정 완료.
- **Latent defects**: 없음.
- **Root cause insight**: BMB `and` operator의 short-circuit semantics가 LLVM CSE 최적화를 방해함. 동일 `load_u8` 표현이 별도 basic block에 있으면 LLVM이 CSE 불가. C에서는 `&&` 연산의 operand가 같은 변수면 자동 CSE.
- **Structural improvement**: MIR 수준에서 `and/or` 조건 내 동일 load subexpression CSE 패스 추가 가능 → 사용자가 break-based 패턴 사용 강요 없이 자동 최적화. (ISSUE 가치 있음)
- **Roadmap impact**: P-track 7/7 전부 BMB faster — brainfuck/csv_parse/http_parse 모두 1.0× 이하로 진입.

## Carry-Forward

- Actionable: 없음
- Structural Improvement Proposals:
  - BMB MIR CSE 패스: `and/or` 체인 내 동일 `load_u8(ptr + x)` 표현 자동 CSE → ISSUE 등록 검토
- Pending Human Decisions: 없음
- Roadmap Revisions: ROADMAP §5 전체 갱신 (brainfuck 0.956×, csv 0.891×, http 0.909×)
- Next Recommendation: Cycle 3024 = MIR CSE ISSUE 등록 + commit + 다음 최적화 탐색
