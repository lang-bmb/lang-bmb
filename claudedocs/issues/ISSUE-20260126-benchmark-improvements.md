# BMB 벤치마크 분석 및 개선 방향

**Date**: 2026-01-26
**Version**: v0.51.31

## 벤치마크 결과 요약

| 벤치마크 | C (ms) | BMB (ms) | 비율 | 상태 |
|---------|--------|----------|------|------|
| mandelbrot | 5.9 | 4.4 | 75% | FAST |
| fannkuch | 78.7 | 63.9 | 81% | FAST |
| fasta | 6.1 | 5.3 | 87% | FAST |
| spectral_norm | 5.1 | 5.2 | 102% | OK |
| fibonacci | 21.7 | 22.6 | 104% | OK |
| n_body | 19.8 | 20.6 | 104% | OK |
| binary_trees | 80.1 | 84.6 | 106% | SLOW |

---

## v0.51.31 실험 결과: n_body

### 가설 검증: "재귀 → 루프 변환으로 벡터화 개선"

**실험**: `main_loop.bmb` - 모든 재귀를 while 루프로 변환

**결과**:
| 버전 | 평균 시간 | 비율 |
|------|----------|------|
| C | 19.8 ms | 100% |
| BMB (recursive) | 20.6 ms | 104% |
| BMB (loop) | 21.1 ms | 107% |

**결론**: 루프 변환이 오히려 더 느림. 재귀 버전이 더 좋은 성능.

### 근본 원인 분석

루프 vs 재귀가 문제가 아님. **진짜 원인은 `inttoptr` 패턴**:

```llvm
; BMB 생성 IR - LLVM이 pointer aliasing 분석 불가
%_t2.i39 = add nsw i64 %_t1.i38, %bodies
%load_f64_ptr.0.i40 = inttoptr i64 %_t2.i39 to ptr
```

```c
// C 버전 - LLVM이 struct 접근으로 인식
bodies[i].vx = ...;
```

**C 버전의 장점**:
1. 정적 전역 배열 → LLVM이 크기와 레이아웃 알고 있음
2. 구조체 멤버 접근 → 깔끔한 GEP 인스트럭션
3. 포인터 에일리어싱 분석 가능

**BMB 버전의 한계**:
1. `malloc` → 불투명 포인터
2. 수동 오프셋 계산 → `inttoptr` 필요
3. LLVM이 에일리어싱 증명 불가 → 벡터화 실패

### 해결 방안

**P1 (언어 기능 추가 필요)**: BMB에 구조체 지원 추가
```bmb
struct Body {
    x: f64, y: f64, z: f64,
    vx: f64, vy: f64, vz: f64,
    mass: f64
};

fn advance(bodies: &mut [Body; 5], dt: f64) = {
    // 구조체 필드 접근 → 깔끔한 LLVM IR
    bodies[i].vx -= dx * bodies[j].mass * mag;
};
```

**현재 상태**: 104%는 허용 범위. 구조체 없이는 C와 동등하게 만들기 어려움.

---

## 식별된 개선 영역

### 1. binary_trees: 메모리 할당 오버헤드 (106%)

**문제점**:
- 개별 malloc/free 호출이 많음
- C와 동일한 패턴이지만 약간 느림

**원인**:
- BMB의 함수 호출 오버헤드 (`node_new`, `node_get_left` 등)
- 인라인이 안 되는 작은 함수들

**해결 방안**:
1. **AggressiveInlining 임계값 조정**: 현재 15 인스트럭션 → 더 높게
2. **인라인 힌트 속성 추가**: `@inline` 또는 `alwaysinline` 속성

---

### 2. fibonacci: 해결됨 (104%)

**v0.51.30 개선 내용**:
- i32 narrowing 최적화로 sext/trunc 체인 제거
- Before: 177% → After: 104%

**남은 4% 차이**:
- 측정 노이즈 범위 내
- 호출 규약 또는 스택 프레임 차이

---

## 우선순위별 개선 작업

### P0 (필수) - 언어 기능 추가
- [ ] **구조체 타입 시스템**: n_body, binary_trees 모두에 필요
- [ ] 구조체 리터럴 및 필드 접근 문법
- [ ] 구조체 배열 지원

### P1 (높음) - binary_trees 인라인
- [ ] 작은 접근자 함수 인라인 최적화
- [ ] AggressiveInlining 임계값 검토

### P2 (중간) - 코드 생성 개선
- [ ] `noalias` 속성 적극 활용
- [ ] 구조체 레이아웃 힌트 (packed, aligned)

---

## 긍정적 결과

BMB가 C보다 빠른 케이스들:
- **mandelbrot (75%)**: 부동소수점 집약 계산에서 BMB 컴파일러 최적화 우수
- **fannkuch (81%)**: 정수 연산 및 배열 접근 최적화 우수
- **fasta (87%)**: 문자열/I/O 처리 효율적

이는 BMB 컴파일러의 MIR 최적화 패스가 특정 패턴에서 우수함을 보여줌.

---

## 결론

v0.51.31 분석 결과:
1. **fibonacci**: 해결됨 (177% → 104%)
2. **n_body**: 재귀 버전이 최적 (104%), 루프 변환 불필요
3. **binary_trees**: 인라인 최적화 필요 (106%)

**근본 해결책**: BMB에 **구조체 타입 시스템** 추가가 필요.
수동 메모리 레이아웃은 LLVM 최적화를 방해함.
