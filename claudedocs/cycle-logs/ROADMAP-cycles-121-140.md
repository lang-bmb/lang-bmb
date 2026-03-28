# Roadmap: Cycles 121-140 — Performance Validation & Language Completeness

> **원칙**: 모든 작업은 C/Rust 추월 성능 검증을 위한 것. 한계 발견 시 언어 스펙/컴파일러 변경 포함.
> **핵심 문제**: "계약이 성능에 기여한 벤치마크는 0개" — 이것을 해결하지 않으면 BMB의 존재 이유가 증명되지 않음.

## Phase A: Contract→Performance 실전화 (Cycles 121-125)

계약 최적화가 실제 벤치마크에서 성능 향상을 보여야 함.

- Cycle 121: purity_opt 벤치마크 분석 — @pure CSE가 왜 0% 이득인지 IR 분석
- Cycle 122: Pure function CSE 최적화 패스 수정/강화
- Cycle 123: Contract-driven bounds elimination이 실제 벤치마크에 적용되도록 개선
- Cycle 124: 새 계약 벤치마크 — 별도 컴파일 단위 시나리오 (cross-module)
- Cycle 125: Contract 성능 재측정 + IR 분석 문서화

## Phase B: Benchmark 완전성 + 성능 분석 (Cycles 126-130)

벤치마크 커버리지 확대 및 성능 병목 분석.

- Cycle 126: 기존 38 벤치마크 전수 재측정 + 최신 컴파일러 검증
- Cycle 127: WARN/regression 분석 — 성능 저하 원인 추적
- Cycle 128: LLVM IR 품질 분석 — BMB vs C IR 비교 (FASTER 벤치마크)
- Cycle 129: 새 벤치마크 추가 (contract 가치 증명 시나리오)
- Cycle 130: 벤치마크 결과 종합 + ROADMAP 업데이트

## Phase C: 컴파일러 최적화 품질 (Cycles 131-135)

LLVM 속성/최적화 패스 강화로 성능 향상.

- Cycle 131: range() 속성 bootstrap 구현
- Cycle 132: nonnull contract→속성 자동 변환 강화
- Cycle 133: stdlib f64 math intrinsics 노출 (sqrt, sin, cos)
- Cycle 134: 인라이닝 힌트 개선 (tak 등 재귀 벤치마크)
- Cycle 135: 3-Stage bootstrap 검증 + 성능 회귀 테스트

## Phase D: Ecosystem & Documentation (Cycles 136-140)

에코시스템 성숙 + 결과 문서화.

- Cycle 136: stdlib 네이티브 빌드 검증 강화
- Cycle 137: gotgan 빌드 파이프라인 개선
- Cycle 138: 벤치마크 공정성 재검증 + Docker 재현성
- Cycle 139: ROADMAP + dev-docs 전체 업데이트
- Cycle 140: 버전 범프 + 종합 평가
