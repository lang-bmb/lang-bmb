# Cycle 2908: bmb-algo C 바인딩 scaffold
Date: 2026-05-18

## Re-plan
HANDOFF Carry-Forward 없음. ROADMAP M4 ④ 바인딩 축: C ❌ 남음.
Python/Node/C#/Java ✅ — C만 누락. gen_headers.py로 헤더 자동 생성 완료 상태이므로
`bindings/c/` 디렉토리(Makefile + example.c + test.c + README.md) 생성이 적합한 autonomous 작업.

## Scope & Implementation

**목표**: `ecosystem/bmb-algo/bindings/c/` 생성 (4파일).

**생성 파일**:
- `Makefile` — Windows/Linux/macOS 크로스 플랫폼 빌드 규칙
- `example.c` — 전 범주(math/bit/array/sort/string) 사용 예제
- `test.c` — 76개 단위 테스트 (전 55 함수 커버)
- `README.md` — API 컨벤션 + 빌드 가이드

**주요 발견**: `bmb_unique_count`는 정렬된 입력을 요구 (인접 비교 방식).
테스트 초기값 수정: `{3,1,4,1,5,9,2,6}` → `{1,1,2,3,4,5,6,9}` (sorted_dup).

**C 바인딩 특이점**:
- Array 인자는 `int64_t arr` — caller가 `(int64_t)ptr` 캐스트
- String 인자는 `void *` — `bmb_ffi_cstr_to_string` + `bmb_ffi_free_string` 사용
- 모든 호출은 `bmb_ffi_begin()` / `bmb_ffi_end()` 래핑 필수

## Verification & Defect Resolution

```
gcc -O2 -I../../include test.c -L../.. -l:bmb_algo.dll → 컴파일 성공
./test.exe → 76 passed, 0 failed
./example.exe → All examples passed
```

**수정된 결함**: `unique_count` 테스트 기댓값 오류 (정렬 전제 누락) → 즉시 수정.

## Reflection

- **Scope fit**: M4 ④ 바인딩 축 C 항목 완성. 55 함수 전체 커버.
- **Latent defects**: 없음. 컴파일+런타임 76/76 통과.
- **Philosophy drift**: 없음 — 기존 바인딩 패턴(Python/Node/C#/Java)과 일관.
- **Roadmap impact**: M4 ④ 바인딩 완성도 C 추가. 나머지 4개 라이브러리(compute/crypto/text/json) 동일 패턴으로 진행 가능.
- **Rule 9 검토**: 아직 4개 라이브러리 남음 → 조기 종료 조건 미충족.

## Carry-Forward
- Actionable: bmb-compute C 바인딩 scaffold (Cycle 2909)
- Structural Improvement Proposals: None
- Pending Human Decisions:
  - B축 재측정 (API key + 환경 준비 후)
  - tier3-spawn-overhead ISSUE-20260512 Option A/B/C 선택
- Roadmap Revisions: M4 ④ C 항목 ✅ (bmb-algo)
- Next Recommendation: Cycle 2909 — bmb-compute C 바인딩
