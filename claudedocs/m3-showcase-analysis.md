# M3 Showcase Library 선정 분석

> 작성: Cycle 2590 (2026-05-09). HUMAN decision 지원용.
> 메인테이너의 최종 선택이 필요합니다.

---

## 후보 라이브러리 (5개)

| 라이브러리 | 함수 수 | 특징 | 성능 클레임 |
|-----------|--------|------|-----------|
| **bmb-algo** | ~64 | 알고리즘 (knapsack, LCS, floyd) | 90x faster than Python, 6.8x faster than C (knapsack) |
| **bmb-compute** | ~35 | 수치 계산 | 미확인 |
| **bmb-crypto** | ~111 | 해싱/인코딩/체크섬 | "Standards-compliant" |
| **bmb-text** | ~48 | 텍스트 처리 (KMP, substring) | 미확인 |
| **bmb-json** | ~64 | JSON 파서/직렬화기 | 벤치마크: ≤1.05x vs C (M1 달성) |

---

## 선정 기준

| 기준 | 가중치 | 설명 |
|------|--------|------|
| BMB 성능 스토리 | ★★★ | "Performance > Everything" 철학 직접 증명 |
| AI/LLM 도메인 정합 | ★★★ | 1차 사용자 = 인간+AI 협업 |
| 계약 증명 가시성 | ★★ | BMB 차별점 — contract가 최적화를 가능케 함 |
| API 품질 | ★★ | C ABI 안정성, Python/Node 바인딩 완성도 |
| 실용성 | ★ | 외부 개발자가 실제 사용할 가능성 |

---

## 후보별 분석

### 1. bmb-algo (추천 #1)
**장점**:
- 성능 스토리 최강: knapsack 6.8x > C, 90x > Python
- 알고리즘은 BMB 계약의 자연스러운 적용 도메인 (pre: 입력 범위, post: 최적성)
- 언어 마케팅에 직관적: "동일 알고리즘, BMB가 더 빠르다"

**단점**:
- DP/graph 알고리즘은 실제 사용 케이스가 좁음

### 2. bmb-json (추천 #2)
**장점**:
- JSON은 AI/LLM 워크플로우의 기본 언어 — 도메인 정합 최고
- M1에서 json_parse ≤1.05x vs C 달성 — 성능 증명 가능
- "zero-copy" + 계약 증명 스토리 강력
- API가 실용적: validate/stringify/get/type/array_len/keys

**단점**:
- json_parse가 C와 동등(≤1.05x)이지, 초월하는 건 아님

### 3. bmb-crypto (미추천)
**장점**: 111개 함수로 가장 포괄적

**단점**:
- 보안 라이브러리는 formal verification 없이 showcase 위험
- 성능보다 정확성이 중요한 도메인 — BMB 철학과 덜 정합
- 111개 함수 유지 비용 높음

### 4. bmb-text (중립)
**장점**: 텍스트 처리는 넓은 사용 케이스

**단점**: KMP search 등이 BMB 계약/성능 차별점을 잘 드러내지 못함

### 5. bmb-compute (미추천)
**장점**: 작아서 유지 비용 낮음

**단점**: 35개 함수로 규모가 작아 showcase 임팩트 약함

---

## 권장사항

**1순위: bmb-algo**
- "BMB가 C보다 빠르다" 직접 증명 가능
- 알고리즘 도메인에서 contract로 bounds 제거 → 최적화 가시적
- npm/pypi 패키지로 외부 배포 후 벤치마크 수치 그대로 마케팅 가능

**2순위: bmb-json**
- AI 워크플로우 연관성
- json_parse 벤치마크로 M1 성과 직접 연결 가능

---

## 다음 단계 (HUMAN 결정 후)

선정되면:
1. 해당 라이브러리를 M3 공식 showcase로 `docs/ROADMAP.md`에 명시
2. 벤치마크 공식 측정 및 README 업데이트
3. npm/pypi 공식 배포 (Track T npm publish 연동)
4. LSP 재작성으로 Track S 90% 진행

---

*이 문서는 Cycle 2590에서 작성됨. 메인테이너 결정 후 `docs/ROADMAP.md`에 반영.*
