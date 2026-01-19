# BMB Research: Zero-Cost Safety

> 이 디렉토리는 BMB의 "Zero-Cost Safety" 비전을 위한 연구 자료를 포함합니다.

## 문서 목록

| 문서 | 설명 |
|------|------|
| `compiler-performance-analysis.md` | C/Rust 컴파일러가 의도적으로 포기하는 1~10% 성능 분석 |
| `zero-cost-theory.md` | 런타임 비용 0을 위한 이론적 문제와 해결방안 |
| `ideal-language-spec.md` | 이상적 언어 스펙 목표 (언어 X 기준서) |
| `bmb-gap-analysis.md` | BMB 현재 구현 vs 이론적 목표 갭 분석 |
| `bmb-improvement-plan.md` | BMB 언어 스펙/컴파일러 개선 계획 (v0.52~v0.56) |

## 핵심 인사이트

### 제거 가능한 비용 (BMB 목표)

| 영역 | 현재 비용 | 해결 방법 | 목표 |
|------|----------|----------|------|
| Bounds check | 0.5~3% | 의존 타입 (Fin[N]) | 0% |
| Overflow check | ~1% | 범위 산술 추론 | 0% |
| Aliasing | 1~5% | disjoint + noalias | 0% |
| Virtual dispatch | ~0.5% | Sealed + Defunctionalization | 0% |

### 제거 불가능한 본질적 한계

| 영역 | 이유 | 손실 |
|------|------|------|
| 분기 예측 | 물리적 한계 | 1~2% |
| 레지스터 할당 | NP-Complete | 2~3% |
| 명령어 스케줄링 | NP-Hard | 1~2% |

**이론적 최소 손실**: ~3~7%

## BMB 차별화

```
Rust: 컴파일 타임 안전성 + 런타임 비용 0.5~3%
BMB:  컴파일 타임 안전성 + 런타임 비용 0% (증명됨)

핵심: Contract + Refinement Types + SMT → 증명된 최적화
```

## 관련 문서

- `docs/ROADMAP.md` - 메인 로드맵 (v0.52+ 섹션)
- `docs/BENCHMARK_ROADMAP.md` - 벤치마크 계획
- `docs/CONTRACT_ANALYSIS.md` - 계약 기반 최적화 분석

---

*작성일: 2026-01-19*
