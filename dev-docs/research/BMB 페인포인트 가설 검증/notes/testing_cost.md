# 소프트웨어 테스트·검증의 시간·비용 부담과 AI 보조 코딩에서의 변화 (2023–2026 근거)

> 범위: BMB 전제 E2(병목은 생성이 아니라 검증·탐색), E3(작성 비용↑로 검증·탐색 비용↓), 원칙 4(증명이 테스트를 대체, "테스트 0"이 목표)의 **테스트 비용** 부분이 실증적으로 지지되는지 확인.
> 표기: [PR] = 동료심사 논문(학회/저널), [pre] = arXiv 프리프린트, [IND] = 업계/벤더 보고서·블로그, [2차] = 2차 보도(원문 미확인). 이번 조사에서 일부 원문(Kleppmann 블로그, Google TAP PDF)은 네트워크 오류(DNS 실패)로 직접 열지 못했고 검색 요약으로만 확인했다. 해당 항목은 표시해 둔다.

## 1. 테스트의 정량적 비용: 엔지니어링 시간, CI 비용, 플레이키 테스트, 유지보수

### Takeaway
테스트 비용은 "크고 측정 가능하다"는 근거는 많지만, **개발 시간 중 테스트 비중을 직접 측정한 최근(2023–2026) 1차 데이터는 찾지 못했다**. 정량 근거는 주로 플레이키 테스트(Google 계열 수치: 테스트의 약 16%가 어느 정도 플레이키, 컴퓨트의 2–16%가 재실행)와 산업 사례(개발 시간의 약 1.3%가 플레이키 수리)에 집중되어 있고, 측정 대상(테스트 비율/실패 비율/빌드 비율/컴퓨트/시간)이 서로 달라 직접 비교가 불가능하다.

### Cited Findings
- [2차, 원 출처 Google 2016 Micco] Google의 약 420만 개 테스트 중 약 16%가 어느 정도 플레이키함을 보였다는 수치가 가장 널리 인용됨. 또한 Google 플랫폼 개발자는 전체 컴퓨트 자원의 2–16%를 플레이키 테스트 재실행에 쓴다고 보고됨 — [VirtusLab: Flaky tests: 16% of dev time (Google data)](https://visdom-maturity-matrix.virtuslab.com/guides/development/flaky-tests-16-of-dev-time-google-data); [getpanto flaky test statistics](https://www.getpanto.ai/blog/flaky-test-statistics) (둘 다 2차 블로그, 원문 Google Testing Blog 미확인)
- [2차] Microsoft: 대규모 CI에서 표본 빌드의 26%에 플레이키 실패가 나타났다는 수치, "테스트 실패의 13%가 플레이키"라는 수치, "연 114만 달러 개발자 시간 비용"이라는 수치가 블로그마다 다르게 인용됨. 마지막 수치는 1차 출처를 찾지 못함 → 인용 주의 — [getpanto](https://www.getpanto.ai/blog/flaky-test-statistics); [testdino Flaky Test Benchmark Report 2026](https://testdino.com/blog/flaky-test-benchmark-report-2026) [IND]
- [PR/학술 인용] 한 학술 논문이 Microsoft·Google을 함께 인용하여 "테스트의 4–16%가 플레이키, CI 테스트 실행의 1.5%가 플레이키"라고 정리 — [TUM 플레이키 테스트 비용 모델 산업 사례연구(PDF)](https://mediatum.ub.tum.de/doc/1730194/1730194.pdf) (검색 요약 기준)
- [PR, 2025 산업 사례연구, Parry et al.] 개발자가 시간의 **1.28%**를 플레이키 테스트 수리에만 사용 (블로그 인용 경유, 원문 미확인) — [mendral: flaky tests cost](https://mendral.com/blog/flaky-tests-cost) [IND]
- [PR, TUM+CQSE] 산업 CI에서 플레이키 테스트가 유발하는 비용 요인을 정리하고 비용 모델을 도출한 연구 존재 — [mediatum.ub.tum.de PDF](https://mediatum.ub.tum.de/doc/1730194/1730194.pdf)
- [PR, ICSE-SEIP 2017, Memon et al. "Taming Google-scale Continuous Testing"] Google에서 테스트 워크로드를 제어해 체크인→피드백 대기 시간을 줄이려는 작업. 초록 요지: **실패하는 테스트는 매우 적고**, 실패하는 테스트는 대상 코드에 가까이 있으며, 최근 3명 초과 개발자가 수정한 코드가 더 자주 깨진다 — [UMD 초록 페이지](https://www.cs.umd.edu/~atif/pubs/Memon_2017_TGC_3103112.3103143-abstract.html)
- [pre/PR, Ghaleb et al. 2021] CI 개선 전략 비교(100개 실제 프로젝트, 14개 선택·우선순위 기법 변형). "워크로드가 커지면 피드백 시간과 빌드 컴퓨트 비용이 매우 커지며, Google·Microsoft에서 CI 비용은 수백만 달러에 달한다"고 기술 — [arXiv 2102.06666](https://ar5iv.labs.arxiv.org/html/2102.06666)
- [TU Delft, 2026] 8개 오픈소스 Java 프로젝트에서 배치 테스트로 매 커밋 전체 테스트 대비 CPU 에너지 평균 80–85% 절감(BatchStop4 평균 80.3%, linear-4 평균 85.2%), 에너지 절감이 시간 절감과 밀접히 비례 — [TU Delft repository](https://repository.tudelft.nl/file/File_de4f214a-66fa-4c64-84de-08ec7488409c?preview=1)
- [IND/2차, JetBrains State of Developer Ecosystem 2024] 테스트를 하는 개발자 비율 85%(2023) → 95%(2024); 단위·통합·E2E 테스트 비율 모두 상승; 테스트 도구에서 AI를 쓰는 비율은 18%. 시간 배분 수치는 "65%가 근무시간 절반 이상을 코딩에 씀(2023년 57%)"뿐이고 테스트 시간 비중은 없음 — [The New Stack](https://thenewstack.io/developers-testing-more-jetbrains-study-finds/)

### Inferences
- "테스트가 비싸다"를 수치로 뒷받침하는 가장 탄탄한 축은 **플레이키 테스트와 CI 컴퓨트**다. 그러나 개별 조직 수준에서 시간 비중은 한 자릿수 %(예: 1.28%) 수준으로 보고되는 경우가 있어, "테스트가 *주된* 병목"이라는 강한 주장을 직접 지지하지는 않는다.
- Memon 2017의 "실패하는 테스트는 매우 적다"는 관찰은 거꾸로 해석하면, 대부분의 테스트 실행은 정보량이 낮은 회귀 확인이며 비용의 대부분이 *실행·유지*에 있다는 뜻이다. 이는 "증명으로 대체 가능한 회귀 테스트" 비중이 크다는 BMB 측 논거로 쓸 수 있으나 직접 측정은 아니다.

### Gaps
- 2023–2026년 기준 "엔지니어링 시간 중 테스트/QA 비중"을 측정한 1차 데이터(시간 기록·텔레메트리)를 찾지 못했다. 흔히 인용되는 "개발 비용의 30–50%가 테스트" 류 수치는 고전 교과서/업계 추정이며 이번 조사에서 1차 출처 확인 불가.
- Google TAP 관련 "pass→fail 전이의 84%가 플레이키", "일 1.5억 테스트 실행" 등의 수치는 검색 결과에서 확인되지 않았다(원 PDF는 DNS 오류로 열지 못함). 인용하려면 원문 확인 필요.
- 테스트 스위트 성장률(시간에 따른 테스트 LOC/실행 시간 증가)과 테스트 유지보수 비용(테스트 수정 커밋 비율)에 대한 최근 정량 연구는 이번 조사 범위에서 찾지 못함.
- Meta의 플레이키 테스트 관련 1차 수치(예: Probabilistic flakiness score)도 이번 조사에서 확인하지 못함.

## 2. AI 코딩 도구가 테스트·검증 부담을 어떻게 바꾸는가 + AI 생성 테스트의 품질

### Takeaway
AI 도입은 산출량(PR 수·크기)을 늘리고 **리뷰 시간과 배포 불안정성을 함께 늘린다**는 업계 데이터가 일관된다(Faros: 리뷰 시간 +91%, DORA 2025: AI 도입↔불안정성 증가). 숙련 개발자 RCT(METR 2025)에서는 오히려 19% 느려졌다. AI 생성 테스트는 커버리지는 높아도 결함 검출력(뮤테이션 점수)이 낮을 수 있고, 오라클이 *의도된* 동작이 아니라 *현재(버그 포함)* 동작을 따라가는 경향이 보고된다.

### Cited Findings
- [IND, Faros AI 2025, 1,255팀·1만+ 개발자 텔레메트리] AI 고사용 팀: 완료 태스크 +21%, 머지 PR +98%, **PR 리뷰 시간 +91%**, 회사 수준 개선과는 유의한 상관 없음. 동일 데이터 요약에서 평균 PR 크기 +154%, 개발자당 버그 +9% — [Faros AI blog](https://www.faros.ai/blog/bain-technology-report-2025-why-ai-gains-are-stalling); [Augment 요약](https://www.augmentcode.com/guides/ai-productivity-paradox-engineering-delivery) (원 보고서 미확인, 상관관계일 뿐)
- [IND, Google DORA 2025 "State of AI-assisted Software Development"] AI 도입은 처리량(throughput)과 양(+)의 관계로 전환되었으나 **배포 불안정성은 계속 증가**; 비슷한 조건에서 AI 도입이 높은 사람이 더 높은 불안정성을 보고 — [InfoQ](https://www.infoq.com/news/2025/09/dora-state-of-ai-in-dev-2025); [RedMonk](https://redmonk.com/rstephens/2025/12/18/dora2025/) (2차 보도)
- [pre/RCT, METR 2025] 숙련 OSS 개발자 16명, 246개 태스크(2025.2–6), Cursor Pro + Claude 3.5/3.7 Sonnet. AI 허용 시 **19% 더 오래 걸림**. 사전 예측 24% 단축, 사후에도 20% 빨라졌다고 믿음 — [eWeek](https://eweek.com/news/news-ai-tools-slow-developer-productivity-study) (2차; 표본 작고 특정 조건)
- [IND, Stack Overflow Developer Survey 2025, 약 4.9만 명] AI 정확도 신뢰 하락(블로그 기준 40%→29%; 불신 46%, 전년 31%). "거의 맞지만 틀린(almost right)" AI 결과가 최대 불만(66%), "AI 생성 코드 디버깅이 더 오래 걸린다" 45% (두 수치가 매체별로 뒤바뀌어 보도됨 → 원문 확인 필요) — [Stack Overflow blog](https://stackoverflow.blog/2025/07/29/developers-remain-willing-but-reluctant-to-use-ai-the-2025-developer-survey-results-are-here/); [InfoWorld](https://infoworld.com/article/4031673/ai-use-among-software-developers-grows-but-trust-remains-an-issue-stack-overflow-survey.html); [VentureBeat](https://venturebeat.com/ai/stack-overflow-data-reveals-the-hidden-productivity-tax-of-almost-right-ai-code)
- [IND, GitClear 2025, 2.11억 변경 라인 2020–2024] 이동/리팩터 라인 비율 ~25%(2021) → <10%(2024); 2024년 처음으로 복붙 라인이 이동 라인을 초과; 5줄+ 중복 블록 8배 증가; 한 달 이상 된 코드 수정 비율 30%(2020)→20%(2024) — [LeadDev](https://leaddev.com/technical-direction/how-ai-generated-code-accelerates-technical-debt); [i-programmer](https://www.i-programmer.info/news/105-artificial-intelligence/17871-gitclear-reveals-ais-negative-impact-on-code-quality.html) (상관관계, 인과 아님)
- [PR, FSE 2024 Industry, Meta TestGen-LLM] Instagram Reels/Stories 평가에서 생성 테스트의 **75%가 빌드, 57%가 안정적으로 통과, 25%가 커버리지 증가**. 테스트어톤 전체에서 적용 클래스의 11.5% 개선, 권고의 73%가 엔지니어에게 수락 — [arXiv 2402.09171](https://arxiv.org/pdf/2402.09171); [FSE 2024 페이지](https://2024.esec-fse.org/details/fse-2024-industry/18/Automated-Unit-Test-Improvement-using-Large-Language-Models-at-Meta)
- [PR, IEEE TSE 52(5) 2026 / arXiv 2506.02954, MUTGEN] 커버리지는 결함 검출의 약한 대리지표: HumanEval-Java 한 대상에서 LLM 생성 테스트가 **라인·브랜치 커버리지 100%이면서 뮤턴트는 4%만 사멸**. 뮤테이션 피드백을 프롬프트에 넣은 MUTGEN이 204개 대상에서 EvoSuite·일반 프롬프트보다 뮤테이션 점수 유의하게 높음 — [arXiv 2506.02954](https://arxiv.org/html/2506.02954v6)
- [pre] LLM이 생성한 오라클은 *기대 동작*보다 *실제 구현 동작*에 맞추는 경향; 코드에 버그가 있을 때 올바른 단언을 분류하는 정확도가 특히 낮음 — ["Do LLMs generate test oracles that capture the actual or the expected program behaviour?" 리뷰](https://www.themoonlight.io/en/review/do-llms-generate-test-oracles-that-capture-the-actual-or-the-expected-program-behaviour)
- [pre] GHRB 실제 Java 버그 36개, GPT-4o·StarCoder: 클래스 수준 컨텍스트에서 오라클 정확도 53.64%(메서드 수준 40.74%, 테스트 프리픽스만 40.38%) — [arXiv 2601.05542](https://www.arxiv.org/pdf/2601.05542)
- [pre, 2026.7] 자연어 요구사항에서 생성한 오라클이 SUT(구현)보다 요구사항(REQ)에 더 가깝게 정렬됨(Defects4J Lang 실제 버그 10개) → 명세가 주어지면 "현재 동작 고정" 문제가 완화 — [arXiv 2607.10277](https://arxiv.org/pdf/2607.10277)

### Inferences
- 업계 데이터는 "생성이 빨라질수록 병목이 리뷰·검증으로 이동한다"는 E2와 **방향이 일치**한다. 다만 늘어난 것은 주로 *코드 리뷰* 시간·불안정성이며, *테스트 작성·실행* 비용 자체가 늘었다는 직접 데이터는 아니다.
- AI 생성 테스트의 핵심 결함은 "구현을 오라클로 삼는 것"(현재 동작 고정)이다. 즉 테스트가 늘어도 검증력은 명세 없이는 늘지 않는다. 이는 "명세(계약) 우선"이라는 BMB 방향과 맞지만, 동시에 *명세 작성*이 실제 병목임을 시사한다(4절).
- METR RCT는 "AI가 검증·이해 비용 때문에 숙련자를 느리게 한다"는 해석과 양립하지만, 원인 분해(테스트 vs 리뷰 vs 프롬프팅)는 하지 않았다.

### Gaps
- AI 도입 전후 *테스트 코드 양·CI 실행 시간·플레이키 비율* 변화를 측정한 연구를 찾지 못함.
- Faros·DORA 원 보고서 PDF는 직접 확인하지 않았다(2차 보도 기반).
- 에이전트가 쓴 테스트의 대규모 뮤테이션 점수 산업 데이터는 찾지 못함(MUTGEN이 주요 학술 근거).

## 3. 테스트는 AI가 도입한 버그를 실제로 잡는가 — "테스트 통과 ≠ 정답" 현상

### Takeaway
에이전트 벤치마크에서 "테스트를 통과한 패치"의 상당 부분이 실제로는 틀리다는 근거가 강하고 동료심사를 거쳤다. SWE-bench Verified에서 통과 패치의 약 15.7%(UTBoost), 29.6%가 정답 패치와 다른 동작(PatchDiff), 해결률이 약 6.2%p 부풀려짐. 즉 **테스트는 AI 코드의 검증 오라클로서 불충분**하다는 점이 측정으로 확인된다.

### Cited Findings
- [PR, ICSE 2026, Wang·Pradel·Liu "Are 'Solved Issues' in SWE-bench Really Solved Correctly?"] CodeStory·LearnByInteract·OpenHands 패치(SWE-bench Verified) 분석. 검증 약점으로 **7.8%의 패치가 개발자 테스트 스위트를 실패하는데도 정답 처리**; 그럴듯한(plausible) 패치의 **29.6%가 정답 패치와 다른 동작**; 동작 차이 패치 중 수동 검사로 **28.6%가 확실히 틀림**; 보고된 해결률이 **6.2%p 부풀려짐** — [arXiv 2503.15223](https://arxiv.org/html/2503.15223v2); [ICSE 2026 페이지](https://conf.researchr.org/details/icse-2026/icse-2026-research-track/59/Are-Solved-Issues-in-SWE-bench-Really-Solved-Correctly-An-Empirical-Study)
- [PR, ACL 2025, UTBoost] LLM 생성 테스트로 보강: 테스트 불충분 인스턴스 36개, 잘못 통과 처리된 패치 345개. 원 테스트 통과 패치 중 **SWE-Bench Lite 28.4%(170/599), Verified 15.7%(92/584)가 오류**. 리더보드 항목의 40.9%(Lite)·24.4%(Verified)에 영향, 순위 변동 18·11건. 오류 패치의 80%+가 django·sympy에 집중. 파서 결함으로 인한 오표기도 별도 발견 — [arXiv 2506.09289](https://arxiv.org/html/2506.09289v1); [ACL Anthology](https://aclanthology.org/2025.acl-long.189); [저자 블로그](https://ddkang.substack.com/p/swe-bench-verified-is-flawed-despite) (버전별 수치 상이 → 인용 시 버전 명시)
- [pre, SWE-Bench+ 2024] 원 SWE-bench의 해법 누출(solution leakage)과 약한 테스트 문제 지적 — [arXiv 2410.06992](https://arxiv.org/pdf/2410.06992)
- [PR, TSE 2026, MUTGEN] 커버리지 100%·뮤턴트 사멸 4% 사례(위 2절) — 커버리지 기반 적정성은 결함 검출을 보장하지 않음 — [arXiv 2506.02954](https://arxiv.org/html/2506.02954v6)

### Inferences
- "테스트 통과"를 정답 기준으로 쓰는 에이전트 루프는 체계적으로 과적합 패치를 받아들인다. 이는 BMB 원칙 4(증명이 테스트를 대체)의 *동기*를 강하게 지지한다.
- 다만 두 연구 모두 문제를 **더 많은/더 좋은 테스트(LLM 생성 차분 테스트)**로 드러냈다. 즉 반대 해석도 가능: "테스트가 무력한 게 아니라 테스트가 부족했다." 증명 기반 대안은 정답 패치와의 동작 동등성 같은 *명세*가 있어야 같은 효과를 낸다.

### Gaps
- 실제 산업 코드베이스(벤치마크 외)에서 AI 패치가 테스트를 통과하고도 프로덕션 결함을 낸 비율에 대한 데이터는 찾지 못함.
- 테스트 적정성 지표(커버리지/뮤테이션)와 실제 결함 검출 간 관계의 최신 대규모 메타분석은 이번 조사에서 확인하지 못함(고전: Inozemtseva & Holmes ICSE 2014 "Coverage is not strongly correlated with test suite effectiveness" — 이번 세션에서 원문 미확인).

## 4. 형식 검증의 경제성 — 역사적 비용, LLM 이후 변화, 산업 사례, 반론

### Takeaway
역사적으로 형식 검증은 매우 비쌌다(seL4: 약 20–25 인년, C 약 1만 줄 미만, 증명 20만 줄, 줄당 <$400). 그러나 검증된 부분의 결함 부재는 측정으로 확인됐다(CompCert 검증 중간단계에서 Csmith가 오코드 버그를 찾지 못함). LLM으로 증명 생성 비용이 급락 중이며(Dafny 벤치 68%→96%, vericoding Dafny 82%/Verus 44%/Lean 27%), Kleppmann(2025.12)은 AI가 형식 검증을 주류화할 것이라 예측. 반론의 핵심은 **명세 작성이 진짜 병목**이라는 점이며, Verus는 Dafny보다 LLM 성공률이 현저히 낮다.

### Cited Findings
**역사적 비용·성공 사례**
- [PR/1차 블로그] seL4 증명 약 20 인년(이 중 seL4 코드베이스 11 인년), 탐색 작업 포함 시 약 25 인년; 증명 스크립트 약 20만 줄 대 C 약 6천~1만 줄; 일회성 도구 투자 제외 총비용 **줄당 $400 미만**(Heiser 블로그) — [microkerneldude: verified software can and will be cheaper than buggy stuff (2016)](https://microkerneldude.org/2016/06/16/verified-software-can-and-will-be-cheaper-than-buggy-stuff/); [Trustworthy Systems PDF](https://www.trustworthy.systems/publications/nicta_full_text/4180.pdf); [NICTA 2014 Productivity for proof engineering](https://www.trustworthy.systems/publications/nictaabstracts/Staples_JAMKK_14.abstract) (노력은 증명 스크립트 줄 수와 강한 선형관계)
- [PR, PLDI 2011, Yang·Chen·Eide·Regehr "Finding and Understanding Bugs in C Compilers"] Csmith로 3년간 325개+ 미지 컴파일러 버그 보고(GCC 79, LLVM 202). **CompCert는 Csmith가 오코드(wrong-code) 오류를 찾지 못한 유일한 컴파일러**(약 6 CPU-년 투입); 다른 컴파일러에서 발견된 중간단계 버그가 CompCert에는 부재. 단, 검증되지 않은 프런트엔드 등에서 몇 개 버그는 발견 — [PLDI 2011 preprint](https://www-old.cs.utah.edu/~regehr/papers/pldi11-preprint.pdf); [QED at Large 서베이 arXiv 2003.06458](https://arxiv.org/pdf/2003.06458)
- [PR/실무, CACM 2025.6, Brooker·Desai "Systems Correctness Practices at Amazon Web Services"] AWS는 TLA+, P(상태기계 언어), 속성 기반 테스트, 결함 주입, 이벤트 기반 시뮬레이션을 혼합. TLA+가 초기 미묘한 버그 발견과 정확성을 유지한 성능 최적화에 가치 — **형식+반형식 혼합이지 테스트 대체가 아님** — [CACM](https://cacm.acm.org/practice/systems-correctness-practices-at-amazon-web-services); [ACM Queue](https://queue.acm.org/detail.cfm?id=3712057)
- [IND, AWS] SAT/SMT 솔버를 하루 10억 회+ 호출(실시간 고객 보안 검사 포함); Nitro Isolation Engine·암호 구현 정확성 증명 — [Amazon Science: A decade of mathematical certainty](https://amazon.science/blog/a-decade-of-mathematical-certainty-reflections-on-the-automated-reasoning-group); [UMN 콜로키엄 초록](https://cse.umn.edu/node/156896)

**LLM 이후 변화**
- [pre, 2025.9, Bursuc et al. "A benchmark for vericoding"] 12,504개 명세(Dafny 3,029 / Verus 2,334 / Lean 7,141). 기성 LLM 성공률 **Dafny 82%, Verus 44%, Lean 27%**. 자연어 설명 추가는 유의한 개선 없음. 순수 Dafny 검증은 1년 사이 **68%→96%**. Verus가 낮은 이유로 ghost/실행 타입 분리와 저자원 언어임을 지목 — [arXiv 2509.22908](https://arxiv.org/html/2509.22908v1)
- [pre, 2026.2, AlgoVeri] 고전 알고리즘 77개 정렬 벤치: 최고 Dafny 약 40%, Verus 약 25%, Lean <8% → 비자명 알고리즘에서는 성공률 급락 — [arXiv 2602.09464](https://arxiv.org/html/2602.09464v1)
- [블로그 의견, Kleppmann 2025.12.8 "Prediction: AI will make formal verification go mainstream"] 현재 증명은 어렵고(PhD급) 노동집약적(seL4 예). LLM으로 형식화가 훨씬 싸질 것; 증명 검사기는 속일 수 없으므로 LLM 코드 신뢰에 유리. 대부분 시스템에서 버그의 기대 비용 < 증명 비용이었으나 그 계산이 바뀜. 병목은 기술보다 **문화 변화**일 것 — [martin.kleppmann.com](https://martin.kleppmann.com/2025/12/08/ai-formal-verification.html) (원문 직접 열람 실패, 검색 요약 기준)

**반론·한계**
- [pre, 다수] 명세 작성이 핵심 장애물: "올바른 명세를 구성하는 어려움 때문에 실용적 검증이 여전히 어렵다"; LLM 생성 코드 *양*이 수작업 명세 작성 능력을 크게 초과; 숙련자도 분리 논리로 메모리 안전 요구를 정확히 쓰기 어렵다; 2026 서베이는 "번역 병목, verifier tax, 확장성 격차"가 미해결이라 정리; 벤치 데이터 오염 가능성 경고 — [arXiv 2410.14835 Towards Automated Verification of LLM-Synthesized C Programs](https://arxiv.org/pdf/2410.14835); [arXiv 2608.14590 Toward Safe LLM Agents survey](https://arxiv.org/pdf/2608.14590); [arXiv 2603.13414](https://www.alphaxiv.org/abs/2603.13414.md); [LiveFMBench arXiv 2605.01394](https://arxiv.org/pdf/2605.01394)

### Inferences
- seL4(인년 단위)→vericoding(Dafny 82%, 자동) 사이의 격차는 "작성 비용(증명)"이 LLM으로 급락하고 있음을 보여준다. 이는 E3(작성 비용을 올려 검증 비용을 낮추는 거래)가 **과거보다 훨씬 유리해졌다**는 근거다.
- 그러나 Verus 계열(BMB와 가장 유사한 표면) 성공률이 Dafny의 약 절반(44% vs 82%), 알고리즘 벤치에서는 25% 수준이다. BMB가 Verus식 ghost/exec 분리를 택하면 같은 페널티를 받을 수 있다 — 표면 설계에서 측정해야 할 지점.
- AWS·CompCert 모두 증명은 *테스트와 병행*된다. "테스트 0"을 달성한 산업 사례는 확인되지 않았다(CompCert도 미검증 부분에서 버그 발견). 원칙 4의 "컴파일러가 남은 테스트 표면을 계산"은 이 현실(검증 경계 바깥은 테스트 필요)과 부합하는 설계다.

### Gaps
- LLM 시대 "검증 코드 줄당 비용"의 최신 정량치(인시/줄, $/줄)는 찾지 못함. vericoding은 성공률만 보고.
- Dafny/Verus/Lean의 *산업* 도입 사례(예: AWS Cedar의 Dafny/Lean 사용, Microsoft의 Verus 기반 시스템)에 대한 비용 수치는 이번 조사에서 확보하지 못함.
- Hillel Wayne 등 실무자의 구체적 반론 원문은 검색되지 않음.

## 5. 전제에 대한 반대 근거 (테스트는 싸다/병목은 다른 곳)

### Takeaway
반대 근거는 "테스트 비용이 작다"기보다 **"병목은 테스트가 아니라 코드 리뷰·이해·요구사항/명세"**라는 형태로 나타난다. 이는 E2의 "검증(테스트 축적)" 부분은 약화하지만 "탐색(이해)" 부분과 명세 중심 설계는 오히려 지지한다.

### Cited Findings
- 플레이키 수리에 쓰는 시간이 개발 시간의 1.28% 수준이라는 산업 사례(2025, Parry et al., 2차 인용) — 테스트 유지 비용이 조직 시간의 작은 부분일 수 있음 — [mendral](https://mendral.com/blog/flaky-tests-cost) [IND]
- Google 대규모 연구: 실패하는 테스트는 매우 적다 — 대부분 테스트 실행은 싸고 자동화된 회귀 확인 — [Memon et al. ICSE-SEIP 2017 초록](https://www.cs.umd.edu/~atif/pubs/Memon_2017_TGC_3103112.3103143-abstract.html)
- AI 도입으로 늘어난 것은 *리뷰 시간*(+91%)과 PR 크기(+154%) — 병목이 사람의 리뷰·이해에 있음 — [Faros AI](https://www.faros.ai/blog/bain-technology-report-2025-why-ai-gains-are-stalling) [IND]
- 개발자 최대 불만은 "거의 맞는" 코드의 디버깅 — 검증의 어려움이 테스트 작성보다 *이해·디버깅*에 있음 — [Stack Overflow 2025](https://stackoverflow.blog/2025/07/29/developers-remain-willing-but-reluctant-to-use-ai-the-2025-developer-survey-results-are-here/) [IND]
- 명세 작성이 형식 검증의 실제 병목이며 LLM 코드 양이 명세 작성 능력을 초과 — 증명으로 테스트를 대체해도 비용은 명세로 이동 — [arXiv 2410.14835](https://arxiv.org/pdf/2410.14835); [arXiv 2608.14590](https://arxiv.org/pdf/2608.14590)
- Kleppmann 자신도 대부분 시스템에서 버그 기대비용이 증명 비용보다 낮았다고 인정(즉 지금까지는 테스트가 경제적으로 합리적 선택) — [Kleppmann 2025](https://martin.kleppmann.com/2025/12/08/ai-formal-verification.html)
- 요구사항에서 생성한 오라클은 구현보다 요구사항에 정렬 → 명세(요구사항)가 있으면 LLM 테스트 품질 문제의 상당 부분 완화 가능, 즉 "증명" 없이 "명세+테스트"로도 개선 여지 — [arXiv 2607.10277](https://arxiv.org/pdf/2607.10277)

### Inferences
- 종합: (a) "테스트 통과가 정답을 보장하지 못한다"는 강하게 지지됨(3절, 동료심사). (b) "AI로 리뷰·검증 부하 증가"는 업계 데이터로 일관되게 지지됨(상관관계). (c) "테스트 *비용*이 지배적 병목"이라는 정량 근거는 약함 — 시간 비중 직접 측정이 부족하고, 측정된 플레이키 비용은 한 자릿수 %. (d) 증명 대체는 작성 비용이 급락 중이나, 비용이 **명세 작성**으로 이동하며 Verus형 표면은 LLM에게 아직 어렵다.
- 따라서 BMB 문서에서 E2를 서술할 때 "테스트 축적 비용"보다 "테스트의 *오라클 부족*(통과≠정답)과 리뷰·이해 비용"을 근거로 삼는 편이 증거와 더 잘 맞는다. 원칙 4의 "테스트 0"은 산업 선례가 없는 목표이므로, "남은 테스트 표면 최소화"를 측정 가능한 지표로 두는 것이 운영 규칙(주장은 측정으로)과 부합.

### Gaps
- "병목이 요구사항/명세"임을 정량화한 2023–2026 1차 연구(예: 결함 원인 중 요구사항 비중)는 이번 조사에서 확보하지 못함.
- 테스트 비용이 작음을 *직접* 주장하는 동료심사 연구는 찾지 못함 — 반대 근거는 대부분 간접적(병목 이동).
