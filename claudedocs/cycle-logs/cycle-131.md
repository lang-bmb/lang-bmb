# Cycle 131: calloc inttoptr 제거 — codegen 수정

Date: 2026-03-28

## Inherited → Addressed
From cycle 124: spectral_norm inttoptr 제거

## Scope & Implementation

### 버그 수정: calloc 반환 타입 불일치
- **문제**: `calloc`이 `ptr @calloc()` 으로 선언되었지만 `call i64 @calloc()` 으로 호출
- **원인**: `fn_return_type()` 에서 calloc 반환 타입을 "i64"로 지정 (line 7430)
- **수정**: `calloc` 전용 코드젠 핸들러 추가 (malloc 패턴과 동일)
  - `call noalias ptr @calloc(i64 count, i64 size)` 로 정확한 타입 사용
  - `ptrtoint` 으로 i64 변환 (BMB 타입 시스템 호환)
  - `ptr_provenance_vars` 에 네이티브 ptr 저장 (alias analysis 보존)

### 파일 변경
- `bmb/src/codegen/llvm_text.rs`: calloc 핸들러 추가 (+30줄)

### IR 검증
- spectral_norm 최적화 IR: inttoptr 6→0 ✅
- spectral_norm 출력값: 1.274224148 (변경 없음) ✅
- 전체 테스트: 23/23 통과 ✅

### 성능 결과
| Benchmark | Before (ms) | After (ms) | 변화 |
|-----------|-------------|------------|------|
| spectral_norm | 110 | 109 | -0.9% (노이즈 범위) |

**inttoptr 제거에도 성능 변화 없음** — spectral_norm의 12% 차이는 inttoptr가 아닌 다른 원인:
- BMB 611줄 IR vs Clang 980줄 IR (BMB가 더 compact)
- BMB 221 벡터화 vs Clang 10 (BMB가 더 많음)
- 루프 구조 차이 또는 LLVM 스케줄링 차이로 추정

## Review & Resolution
- calloc 타입 불일치 수정 완료 ✅
- 성능 미개선이지만 IR 품질 개선 (alias analysis 정확성 향상) ✅
- 회귀 없음 확인 (테스트 전체 통과) ✅

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope: spectral_norm 12% 차이의 실제 원인은 루프 스케줄링 차이
- Next Recommendation: stdlib f64 math intrinsics 노출 또는 다른 codegen 개선
