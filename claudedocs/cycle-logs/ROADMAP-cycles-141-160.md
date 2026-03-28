# Roadmap: Cycles 141-160 — 언어스펙/컴파일러 변경

> **원칙**: 성능 한계 발견 시 언어 스펙 변경을 감수한다. Workaround 금지.
> **carry-forward**: floyd_warshall 벡터화, @noinline, surpass 벤치마크 동일화

## Phase A: @noinline 어노테이션 (Cycles 141-144)
- 언어 스펙에 @noinline 추가 (parser→attr→MIR→codegen)
- 계약 벤치마크에서 별도 컴파일 시뮬레이션: @pure + @noinline
- purity_opt noinline 버전 → C 대비 성능 우위 재측정

## Phase B: 루프 벡터화 패턴 개선 (Cycles 145-149)
- floyd_warshall IR 분석: 재귀→루프 변환이 벡터화에 불리한 패턴 특정
- MIR TailRecursiveToLoop 패스 개선: LLVM 벡터화 친화적 패턴 생성
- 벤치마크 재측정으로 검증

## Phase C: 벤치마크 확장 + 성능 검증 (Cycles 150-155)
- surpass 벤치마크 BMB/C 알고리즘 동일화 (공정 비교)
- contract_opt 벤치마크 확장 (@pure + @noinline 시나리오)
- 전체 벤치마크 재측정 + 성능 판정

## Phase D: 문서화 + 버전 범프 (Cycles 156-160)
- ROADMAP 업데이트
- 벤치마크 결과 종합
- 버전 범프 v0.97.2
