# Cycle 3328: bootstrap P-track 회귀 분석
Date: 2026-05-30

## Re-plan
HANDOFF에서 bootstrap P-track 회귀(csv 1.134×❌, lexer 1.459×❌)를 P2로 분류.
원인 가설: tuple calloc 오버헤드.

## Scope & Implementation
- IR 비교 분석 수행:
  - bootstrap compiler_s1.exe와 Rust compiler로 각 벤치마크 IR 생성
  - opt.ll 파일 비교: **IDENTICAL** (byte-for-byte 동일!)
  - tuple calloc 오버헤드 가설 반증 (opt 이후 calloc 0회)
- 실제 P-track 재측정:
  - **Rust P-track: 7/7 ✅** (모두 C보다 빠름)
  - **Bootstrap P-track: 6/7** (csv 1.048× 경계선)
    - brainfuck: 0.841× ✅, csv: 1.048× ⚠️, http: 0.891× ✅
    - json_parse: 0.531× ✅, json_serialize: 0.861× ✅
    - lexer: 0.522× ✅, sorting: 0.181× ✅

## Verification & Defect Resolution
- **핵심 발견**: Cycle 3234 측정값(csv 1.134×, lexer 1.459×)은 STALE.
  - 현재 bootstrap은 identical IR 생성 → 레거시 회귀 해소됨
  - lexer: 1.459× → 0.522× (BMB가 C보다 2× 빠름!)
  - csv: 1.134× → 1.048× (여전히 경계선이나 대폭 개선)
- **빌드 파이프라인 차이** (bootstrap vs Rust):
  - Bootstrap: `opt --mcpu=native scalarizer` + `clang -O3 opt.ll runtime.c` (combined)
  - Rust: `clang -O3 -ffunction-sections -fdata-sections -c ir.ll -o obj` + 별도 링크 + `--gc-sections`
  - Bootstrap binary / Rust binary ≈ 1.1-1.2× (빌드 파이프라인 차이)

## Reflection
- 스코프 적합: 원인 분석 완료. tuple calloc 가설 반증. IR 동일성 확인.
- 철학 적합: 정확한 측정으로 잘못된 가설 수정 (Verification Principle).
- 로드맵 영향: bootstrap P-track 회귀는 실질적으로 해소됨. csv 1.048× 마지막 개선 기회.
- **개선 기회**: bootstrap의 `build_link`를 Rust 스타일로 변경하면 csv 1.048× → ~0.9× 예상:
  1. IR 별도 컴파일 (runtime과 분리)
  2. `--gc-sections` 추가
  3. `-ffunction-sections -fdata-sections` 추가

## Carry-Forward
- Actionable: bootstrap 빌드 파이프라인 개선 (build_link → 별도 컴파일 + gc-sections)
- Structural Improvement Proposals: Cycle 3234 stale 측정값을 ROADMAP에서 갱신
- Pending Human Decisions: None
- Roadmap Revisions: bootstrap P-track 회귀 해소 기록 필요
- Next Recommendation: Cycle 3329 — bootstrap build_link 개선 구현
