# Cycle 2709: 가설 분리 — Token packing overflow + O(n²) AST OOM 공존 확정
Date: 2026-05-11

## Re-plan
인계받은 actionable (Cycle 2708 Carry-Forward): 2K↔3K threshold + 메모리 곡선.
실제 진행 중 2K-3K 비단조성 발견 → arena 한도 다양화로 가설 재검증 → **byte-size threshold 분리**로 결정적 데이터 확보.
Trigger ⚪ NONE.

## Scope & Implementation

### Phase 1: 2K-3K binary search (실패)

| Lines | 결과 | 해석 |
|-------|------|------|
| 2200 | OOM 16G (13.3s) | |
| 2400 | OOM 16G (13.7s) | |
| 2500 | OK (1.1s) | **truncation 위치 우연히 valid** |
| 2600 | OOM 16G (13.2s) | |
| 2700 | OOM 16G (13.2s) | |
| 2800 | OOM 16G (12.9s) | |
| 2900 | OOM 16G (12.8s) | |

비단조 결론: line count는 의미 있는 메트릭 아님. truncation valid syntax 여부가 좌우.

### Phase 2: 동일 full source × 다양한 arena 한도

| Limit | 결과 | 시간 |
|-------|------|------|
| 4G | parse-1:3 | 283ms |
| 8G | parse-1:3 | 248ms |
| 16G | parse-1:3 | 252ms |
| 24G | parse-1:3 | 265ms |
| 32G | parse-1:3 | 336ms |
| 48G | parse-1:3 | 254ms |

**즉 full source는 OOM 후행 효과가 아니라 별도 결함** (어떤 한도에서도 동일 0.2~0.3초 fail).

### Phase 3: byte size threshold (smoking gun)

| Lines | Bytes | 결과 | 해석 |
|-------|-------|------|------|
| 19000 | 956,464 | parse-trunc | <1M, valid OK boundary |
| 19500 | 979,950 | OOM 16G | <1M, 컴파일 시도 → 메모리 폭발 |
| **19850** | **995,850** | **OOM 16G** | **<1M, 컴파일 path** |
| **19900** | **997,956** | **parse-trunc** | **<1M (artifact)** |
| **19950** | **1,000,126** | **parse-1:3** | **>1M, token overflow 시작** |
| 19980 | 1,001,235 | parse-1:3 | >1M |
| 20000 | 1,002,115 | parse-1:3 | >1M |
| 20500 | 1,021,762 | parse-1:3 | >1M |
| 20802 (full) | 1,036,359 | parse-1:3 | >1M |

**1,000,000 byte 경계에서 명확하게 두 모드가 갈림**.

### Root cause 분석

`bootstrap/compiler.bmb` line 206:
```bmb
fn TK_FN() -> i64 = 2000000000 + 100;
fn TK_LET() -> i64 = 2000000000 + 101;
...
```

`fn pack_int_tok` (line 398):
```bmb
fn pack_int_tok(acc: i64, pos: i64) -> i64 =
    if acc >= 9000000000000 { 9200000000000 * 1000000 + pos }
    else { acc * 1000000 + pos };
```

토큰 인코딩: `kind * 1_000_000 + pos`.
**Pos가 1,000,000 byte 초과 시 kind와 충돌** → 토큰을 잘못된 kind로 디코딩 → line 1:3 위치에서 잘못된 if-context 진입.

source line 1 위치 ~0–24:
- byte 0: '/'
- byte 1: '/'
- byte 2: ' '
- byte 3: 'B'

토큰 디코딩이 pos 정보를 잃어 source 처음으로 되돌아간 듯한 효과. 따라서 parse error가 "line 1:3"로 보고됨.

## Verification & Defect Resolution

| 검증 | 결과 |
|------|------|
| byte size threshold (정확히 1MB) | ✅ 19850/19900 OOM/trunc vs 19950+ parse-1:3 |
| 모든 arena limit에서 동일 parse-1:3 | ✅ 4G-48G 일관 |
| Token packing overflow 가설 정확 | ✅ source code line 398 pack_int_tok |
| O(n²) 메모리 성장도 살아있음 | ✅ <1M source에서 OOM 발생 (8000 bytes/line 비율, 거의 8x 증폭) |

**두 결함 공존 확정**:
1. **Token packing overflow** at source byte > 1,000,000 — 즉시 fail
2. **O(n²) AST 메모리 성장** at source byte < 1,000,000 — 16GB OOM

결함: 없음 (이 사이클은 진단만).

## Reflection

### 외부 관찰자 관점

1. **Cycle 2708 결론 정정**: "OOM 우세, parse error는 부작용" → **틀림**. 두 결함이 별개 path로 공존. 4G~48G 모든 arena 한도에서 동일 parse-1:3 발생이 분리의 결정적 증거.

2. **Cycle 2237 시점 fixed point PASS의 의미**: 그 시점 compiler.bmb는 < 1MB. 그 후 compiler가 자라 1MB 임계 초과 → 토큰 packing이 더 이상 unique하지 않음 → 부트스트랩 회귀.

3. **OOM 8x 증폭 비율**: 985KB source (19850 lines) → 16GB OOM. 즉 **AST 메모리가 source 크기의 ~16,000배**. O(n²) 이상의 worse-case 성장.

4. **두 결함 수정 비용 차이**:
   - **Token packing**: 단일 상수 변경 (1M → 10M) + TK_*() 함수의 base (2_000_000_000 → 적절한 base). 위험 — TK_INT 영역 (정수 literal과의 충돌)과 충돌하지 않게 재설계 필요. 영향 — 모든 토큰 함수.
   - **O(n²) AST**: 문자열 기반 AST 자체의 sharing 또는 binary 인코딩 전환 — **대규모 재작성** (수개월).

### Roadmap impact

- ROADMAP M5-1 표 행 "arena OOM (pre-existing, 32G+ 초과, O(n²) 문자열 AST 성장)" → **정정 필요**:
  - **두 결함 공존**: ① token packing 1MB overflow ② O(n²) AST 메모리
  - **bypass 경로**: ① 만 fix하면 (compiler ≤ 32M source가 가능해지지만) ② 가 다시 차단
  - **순서**: ① fix 먼저 (저비용 + Cycle 2237 회귀 명확), ② 는 별도 장기 트랙

## Carry-Forward

- Actionable (Cycle 3 = 2710): **Token packing fix 범위 추정**
  - TK_* 함수의 영향 범위 grep
  - 1M → 10M scale-up의 의존성 (TK_INT 영역과 충돌 여부, pack_int_tok의 9_000_000_000_000 임계와의 관계)
  - 변경량 (LOC 단위) + fix 시도 가능 여부 결정
- Structural Improvement Proposals:
  - **Cycle 10에서 ROADMAP/HANDOFF 정정**: M5-1 OOM 행 + memory note
  - **Token packing redesign**: pos 필드를 i64의 별도 비트로 분리 (예: `kind << 32 | pos`) — proper fix 권고
- Pending Human Decisions: 변경 없음
- Roadmap Revisions: 없음 (정정은 Cycle 10)
- Next Recommendation: Cycle 3 = Token packing fix 범위 분석 + go/defer 결정
