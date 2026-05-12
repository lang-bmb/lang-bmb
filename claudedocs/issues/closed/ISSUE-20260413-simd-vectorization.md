# ISSUE-20260413 — SIMD 자동 벡터화

**우선순위**: ~~P1~~ → **Superseded** by `ISSUE-20260413-simd-codegen.md` (Cycle 2220, Apr 13)
**영역**: codegen, language-spec
**상태**: **Superseded** — 방안 B(SIMD 1급 타입)가 채택되었고 스캐폴딩이 Cycles 2215-2219에 완료됨. codegen은 simd-codegen 이슈에서 추적.
**관련 벤치마크**: n_body (95% Apr 13 재측정), mandelbrot (99%), spectral_norm (99%)

## 결론 (2026-04-13)

사용자 합의(옵션 B: SIMD 1급 타입)에 따라 방안 B 채택. 후속은 `ISSUE-20260413-simd-codegen.md` 참조.

- 방안 A (LLVM loop vectorizer 메타데이터): 보조 옵션으로 잔존 가능
- 방안 B (1급 타입): **채택** — Type::Vector 스캐폴딩 완료
- 방안 C (pure + contract 기반): 장기 연구 과제로 보류

원문 보존용 원래 분석은 아래:

---


## 문제

BMB는 수치 연산 벤치마크에서 GCC와 비교 시 5-26% 뒤처짐. 원인: LLVM 자동 벡터화가 BMB IR에서 GCC 대비 약하게 동작. AVX2/NEON 활용 부재.

## 해결 방안

### 방안 A — LLVM Loop Vectorizer 활성화 최적화
- 현재 `set_loop_vectorization(true)` 설정됨 (llvm.rs)
- 그러나 BMB IR이 vectorizer가 인식 가능한 형태가 아닐 수 있음
- 조치: `@llvm.loop.vectorize.enable !true` 메타데이터 방출

### 방안 B — SIMD 인트린식 스펙 추가
- `@simd` 속성 또는 `simd<N>` 타입 도입
- 예: `simd<f64, 4>` = 4-wide f64 벡터
- LLVM `<4 x double>` 타입으로 직접 매핑
- 언어 스펙 확장 필요

### 방안 C — pure + contract 기반 벡터화
- `pure fn`이고 루프 데이터 의존성 없음이 계약으로 증명되면 자동 벡터화
- BMB 철학과 일치 (Contract → Performance)

## 구현 순서

1. [ ] 현재 n_body BMB vs C LLVM IR diff (vectorize 여부 확인)
2. [ ] Loop vectorize hint 메타데이터 방출 시도
3. [ ] 효과 없으면 방안 B: simd<T, N> 타입 RFC 작성
4. [ ] 벤치마크: n_body, mandelbrot, spectral_norm 목표 ≤ 100%

## 완료 기준

- n_body ≤ 100%, mandelbrot ≤ 100% (vs Clang)
- matrix_multiply AVX2 활용 확인
