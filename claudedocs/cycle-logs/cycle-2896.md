# Cycle 2896: B축 재측정 준비 — problem.md 버그 수정 + bmb_reference int-key 패턴
Date: 2026-05-15

## Re-plan
Carry-Forward 없음 (Cycle 2895). ROADMAP 우선순위 ② B축 재측정 준비. API key는 .env.local에 설정되어 있으나 모델명이 "claude-text"로 표시됨 — 실측 시 claude-sonnet-4-6 확인 필요. API key 없이 가능한 준비 작업을 수행.

## Scope & Implementation

**Files changed**: `ecosystem/bmb-ai-bench/problems/69_overflow_detect/problem.md`, `ecosystem/bmb-ai-bench/problems/83_pipeline/problem.md`, `ecosystem/bmb-ai-bench/protocol/bmb_reference.md`

### 6개 실패 케이스 분석 (2026-05-13 98.0% 기준)
| 문제 | 실패 원인 | 언어 갭 관련 | 수정 가능 |
|------|----------|------------|---------|
| 49_roman_to_int | 코드 잘림 (795 chars, 10 시도) | ❌ | 간접적 (패턴 추가) |
| 69_overflow_detect | problem.md 버그 (예제 출력 ≠ 설명) | ❌ | ✅ 수정 |
| 72_alternating | AI가 다중 TC 루프 누락 | ❌ | 어려움 |
| 75_longest_plateau | 코드 잘림 (1221 chars, 10 시도) | ❌ | 간접적 |
| 83_pipeline | problem.md 불명확 (m을 op로 오독) | ❌ | ✅ 수정 |
| 85_registry_pattern | int→int 맵 없음 → 병렬 vec 사용 | △ | ✅ 패턴 추가 |

### 변경 내용

**69_overflow_detect/problem.md**:
- 예제 3번 케이스를 ambiguous boundary(b=2147483648, 제약 위반)에서 명확한 overflow 케이스로 교체
  - `3 1 2 3 100000 100000 1 -1 2147483648` → `3 1 2 3 100000 100000 1 -3 1000000000`
  - 예제 출력: `1 -2147483648` → `1 -3000000000` (설명과 일치)
  - 설명: "fits → 0 -2147483648" (모순) → "-3×10^9=-3×10^9 < -2147483648 → overflow"
  - Note: 실제 tests.json은 correct — problem.md 예제만 버그였음

**83_pipeline/problem.md**:
- `## Parse Order (explicit)` 섹션 추가: pseudo-code로 `n → array → m → m개 op loop` 구조 명시
- `**m is the count of operations**` 강조 — AI가 m을 op type으로 오독하는 것 방지

**bmb_reference.md**:
- `## Pattern: Integer-keyed registry/map using str_hashmap` 추가 (line 578 이후)
  - `to_string(key)` 로 int→str 변환 후 str_hashmap 사용 패턴
  - set/get/overwrite/count 전부 예시
  - "Key rule: always convert integer keys to strings via to_string()" 명시

### B축 인프라 점검
- `.env.local`: `OPENAI_COMPATIBLE_ENDPOINT`, `OPENAI_COMPATIBLE_API_KEY`, `OPENAI_COMPATIBLE_MODEL` 설정됨 ✅
- `run_experiment.py --dry-run --pilot` 정상 실행: BMB binary 존재, reference 34553 chars ✅
- 현재 모델: "claude-text" (local proxy alias) — 재측정 시 claude-sonnet-4-6 확인 필요
- 실제 tests.json: 12개 모두 올바름 (problem.md 예제 버그와 무관)

## Verification & Defect Resolution
- 코드 변경 없음 — `cargo test --release` 불필요
- `run_experiment.py --dry-run --pilot` → 정상 ✅
- 69_overflow_detect tests.json 확인: 12개 테스트 모두 correct overflow logic 적용 ✅
- bmb_reference.md int-key pattern: 문법 검증 (to_string + str_hashmap_insert/get/contains/len, 모두 기존 테스트에서 검증됨) ✅

## Reflection
- **Scope fit**: B축 재측정 준비 완료. API key 없이 가능한 모든 준비 작업 수행.
- **Latent defects**: 72_alternating (다중 TC 루프 누락)과 75/49 (코드 잘림)은 problem.md 수정으로 해결되지 않음. 이는 LLM 응답 길이 제한 + AI reasoning 문제로 언어/문서 수준 수정 범위 외.
- **Roadmap impact**: 재측정 시 69_overflow_detect 실패 1건은 problem.md 수정으로 해소 가능. 85_registry_pattern은 int-key 패턴 추가로 개선 가능. 예상 개선: 98.0% → 98.0-98.5% (1-2건 추가 통과 가능성).
- **Pending HUMAN**: 재측정 실행은 API key + 모델 설정 확인 후 HUMAN이 실행.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  - 72_alternating: problem.md에 "multiple test cases" 구조를 더 명시적으로 강조하는 예제 추가
  - 49/75 truncation: bmb_reference.md에 더 concise한 vec 반복 패턴 추가 고려
  - run_experiment.py에 anthropic SDK 직접 지원 추가 검토 (현재 OpenAI-compatible endpoint만 지원)
- Pending Human Decisions:
  - B축 재측정 실행 (API key + claude-sonnet-4-6 설정 확인 후)
- Roadmap Revisions: None
- Next Recommendation: Cycle 2897 — C# 바인딩 scaffold 시작 (M4-6, 3-5 cycles 예상) 또는 P-track 벤치마크 실측
