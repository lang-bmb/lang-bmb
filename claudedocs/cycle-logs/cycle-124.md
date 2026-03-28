# Cycle 124: inttoptr 분석 + 성능 병목 특정

Date: 2026-03-28

## Inherited → Addressed
From cycle 123: spectral_norm inttoptr 제거 — 분석 완료, 수정은 codegen 변경 필요

## Scope & Implementation

### inttoptr 근본 원인 분석

**spectral_norm**: calloc 반환 → i64 저장 → inttoptr로 ptr 변환
- codegen에 ptr_provenance_vars 시스템 존재 (v0.96.36)
- 그러나 spectral_norm의 calloc 결과가 provenance 추적에서 누락 (6개 inttoptr 잔존)
- **원인**: 벤치마크가 `*f64` 포인터 변수를 사용 — provenance가 f64 포인터를 통해 전파되지 않음

### 성능 영향 정량화

| 요인 | spectral_norm | floyd_warshall |
|------|-------------|----------------|
| inttoptr 수 | 6 | 0 |
| 벡터화 비율 (BMB/C) | N/A | 83/165 (50%) |
| 성능 차이 | -12% | -14% |

### Decision Framework 적용

| 문제 | 수준 | 검토 |
|------|------|------|
| inttoptr 잔존 | **코드 생성** (Level 4) | ptr_provenance 추적 범위 확대 필요 |
| 벡터화 차이 | **최적화 패스** (Level 3) | 재귀→루프 변환 패턴이 벡터화 불리 |
| 메모리 접근 패턴 | **언어 스펙** (Level 1) | while 루프 + 포인터 접근 패턴은 이미 지원 |

## Review & Resolution
- inttoptr 원인 특정 완료 ✅
- 벡터화 차이 원인 특정 완료 ✅
- 수정은 codegen 레벨 변경 필요 → Phase C (Cycle 131+)로 이관

## Carry-Forward
- Pending Human Decisions: None
- Discovered out-of-scope:
  - ptr_provenance f64 포인터 확장 (codegen/llvm_text.rs)
  - MIR→IR 루프 변환 벡터화 친화적 패턴 생성
- Next Recommendation: Phase A 마무리 — 계약 최적화 가치 종합 문서화
