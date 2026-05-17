# Cycle 2902: inkwell/text 백엔드 정합성 검사 스크립트
Date: 2026-05-17

## Re-plan
Carry-Forward 없음. HANDOFF "Structural Improvement 3: inkwell/text 백엔드 함수 등록 정합성 테스트 (Rule 7 위반 방지)" 항목 선택.

## Scope & Implementation

**목표**: Cycle 2891에서 발생한 inkwell 백엔드 40+ 함수 누락 패턴을 방지하는 CI 스크립트 작성.

**파일 생성**: `scripts/check_backend_parity.py`

### 스크립트 설계

**추출 방식**:
- inkwell (`bmb/src/codegen/llvm.rs`): `"bmb_xxx"` 문자열 리터럴 전수 추출
  - `add_function("bmb_xxx", ...)` 직접 호출
  - `reg_str_fn!("bmb_name", "bmb_xxx", ...)` 매크로 2번째 인수 (C ABI 이름)
- text IR (`bmb/src/codegen/llvm_text.rs`): `declare ... @bmb_xxx(...)` 선언문

**비교 결과** (첫 실행):
- 146개 공통 함수 (SHARED)
- 34개 inkwell-only → 대부분 text IR에서 short name (`vec_new`, `println_f64`) 또는 다른 메커니즘 사용
- 20개 text-only → 파일시스템 ops, arc 참조 카운팅 등 inkwell 미구현 기능

**기존 차이 분류** (`INKWELL_ONLY_EXPECTED`, `TEXT_ONLY_EXPECTED`):
- 두 리스트에 현재 알려진 의도적 차이를 등록
- 새로운 함수 추가 시 새 mismatch만 보고

**최종 결과**: PARITY OK — 0 unexpected mismatches

**사용법**:
```bash
python3 scripts/check_backend_parity.py          # 보고서 출력
python3 scripts/check_backend_parity.py --verbose # shared 함수 전체 포함
python3 scripts/check_backend_parity.py --ci      # exit 1 on new mismatch
```

## Verification & Defect Resolution
```
BMB Backend Parity Report
  inkwell registered : 180
  text declared      : 166
  shared             : 146
  inkwell-only (WARN): 0
  text-only    (WARN): 0

PARITY OK: Both backends match on 146 bmb_* functions.
```

`cargo test --release`: 2388/2388 PASS (Cycle 2901에서 확인됨, 스크립트 추가만이라 변화 없음).

## Reflection
- **Scope fit**: Rule 7 위반 방지 CI 도구 완성.
- **Limitation**: 두 백엔드의 naming inconsistency(inkwell: `bmb_vec_new` / text: `vec_new`) 가 exclusion 목록으로 처리됨. 근본 수정(이름 통일)은 별도 cycle.
- **효과**: 앞으로 새 빌트인 추가 시 `python3 scripts/check_backend_parity.py --ci` 한 줄로 Rule 7 준수 확인 가능.
- **Philosophy drift**: 없음.
- **Roadmap impact**: HANDOFF Structural Improvement 3 완료.

## Carry-Forward
- Actionable: None
- Structural Improvement Proposals:
  - 두 백엔드 naming convention 통일 (bmb_ prefix 일관 적용) — exclusion 목록 제거 가능
  - `scripts/check_backend_parity.py --ci`를 CI 파이프라인에 통합 (`quick-check.sh` 또는 `full-cycle.sh`)
- Pending Human Decisions:
  - B축 재측정 실행
  - Java 바인딩 계속 개발 여부
- Roadmap Revisions: None
- Next Recommendation: Cycle 2903 — Java scaffold batch (bmb-json/compute/crypto/text) 또는 bmb_runtime.c CI 자동 rebuild 스크립트
