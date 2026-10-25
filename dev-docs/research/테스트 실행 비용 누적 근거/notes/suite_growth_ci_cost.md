# 테스트 스위트 성장과 CI 반복 실행 비용 (외부 근거)

> 수집 방법 주의: 이 환경에서 WebFetch가 research.google.com, shopify.engineering, uber.com, export.arxiv.org, cs.ubc.ca, vanderbilt.edu 등 거의 모든 원문 호스트에서 DNS 실패(ENOTFOUND)했다. **아래 수치는 전부 검색 스니펫(검색 도구가 요약한 원문 발췌)으로만 확인했다.** 원문 PDF를 직접 열어 대조하지 못했으므로, 최종 보고서에 인용하기 전 핵심 수치(특히 Google·Meta·Shopify·Uber)는 원문 재확인을 권한다. [PR]=동료심사 논문, [IND]=산업 트랙 논문/사내 발표, [BLOG]=회사 엔지니어링 블로그, [VENDOR]=벤더 보고서/마케팅.

## Q1. 테스트 스위트는 시간에 따라 어떻게 성장하는가

### Takeaway
대형 조직의 1차 데이터는 "테스트 수와 커밋 빈도가 각각 선형 성장 → 전수 실행 비용은 이차(quadratic) 성장"이라는 구조를 직접 진술한다(Google 2016). Shopify는 모놀리스 테스트가 연 20–30% 증가한다고 보고했다. 반면 오픈소스의 테스트/코드 비율 추이에 대한 학술 데이터는 패턴 분류 수준이며, 단일한 "성장 곡선" 수치는 찾지 못했다.

### Cited Findings
- [IND, Google 2016] John Micco 발표 "Continuous Integration at Google Scale"(2016-04-25): "submit rate – grows linearly", "test pool size – grows linearly" → 둘의 곱으로 실행 시간이 **이차적으로(quadratic) 성장**한다고 명시. 대응책인 just-in-time 스케줄링으로 일 1.5억+ 테스트를 선형 시간 복잡도로 처리한다고 주장 — [SlideShare: 2016 04-25 continuous integration at google scale](https://www.slideshare.net/JohnMicco1/2016-0425-continuous-integration-at-google-scale) (스니펫 확인)
- [PR, ICSE-SEIP 2017] Memon et al.: 이전 방식(모든 변경마다 모든 테스트 실행)에서 "compute resources were growing quadratically"가 프로젝트 동기였다는 취지 — [Taming Google-Scale Continuous Testing](https://research.google.com/pubs/archive/45861.pdf) (스니펫)
- [IND, Google 2017] Micco "The State of Continuous Integration Testing @Google": 개별 테스트 **420만 개**가 상시 실행 — [research.google/pubs/archive/45880.pdf](https://research.google.com/pubs/archive/45880.pdf) (스니펫)
- [BLOG, Shopify] 코어 모놀리스 테스트 **170,000개 이상**, 스위트가 **매년 20–30% 증가**, 한 대 머신에서 순차 실행 시 **41시간 이상** — [Keeping Developers Happy with a Fast CI (Shopify Engineering)](https://shopify.engineering/faster-shopify-ci) (스니펫; 게시연도 원문 미확인, 대략 2019–2021로 추정됨). 2차 요약에는 "모놀리스 빌드마다 40만+ 단위 테스트" 수치가 있으나 출처 신뢰도 낮음 — [ByteByteGo: Shopify Tech Stack](https://blog.bytebytego.com/p/shopify-tech-stack)
- [IND, Spotify] iOS pre-merge 스위트 **48,000개 테스트, 약 30분** — 검색 요약 기반(원 발표 미확인) — [qubika: testing at scale](https://qubika.com/blog/testing-at-scale/) (2차 출처)
- [PR, Zaidman et al. 2011, EMSE] 오픈소스·산업 시스템의 버전 이력 마이닝: change history / growth history / coverage evolution 세 관점 제시. 동기적 성장, 시차 성장, "test backlog"(테스트가 뒤처지다 몰아서 작성) 패턴 식별 — [ResearchGate](https://www.researchgate.net/publication/220277674_Studying_the_co-evolution_of_production_and_test_code_in_open_source_and_industrial_developer_test_processes_through_repository_mining) (스니펫; 구체 비율 수치 미확인)
- [PR, Marsavina et al. SCAM 2014] 5개 OSS에서 세밀한 공진화 패턴 6종 — [PDF](https://azaidman.github.io/publications/marsavinaSCAM2014.pdf)
- [PR, Miranda/Avelino/Santos Neto 2025, JSEP 37(7)] GitHub 526개 저장소·6개 언어. 테스트 공진화 패턴 5종, 높은 공진화는 작은 팀·적은 수정 유지보수와 상관 — [Wiley](https://onlinelibrary.wiley.com/doi/full/10.1002/smr.70035), 데이터 [Zenodo](https://zenodo.org/records/14705473) (테스트 LOC 비율 수치는 스니펫에 없음)
- [PR, Bailey & Cadar ICST 2025] 성숙한 C/C++ 9개 프로젝트, 누적 78년 개발 기간: 테스트 강조와 전체 커버리지는 대부분 증가, 그러나 많은 패치가 여전히 테스트 부족, **flaky 테스트 비율은 증가** — [srg.doc.ic.ac.uk](https://srg.doc.ic.ac.uk/publications/25-testing-evol-icst.html)
- [PR, Wang et al. TOSEM 2023] 975개 Java 프로젝트 대상 production-test 공진화 식별 재검토 — [ACM DL](https://dl.acm.org/doi/10.1145/3607183)

### Inferences
- "테스트 수 선형 증가 × 변경 빈도 선형 증가 = 전수 실행 비용 이차 증가"는 Google 1차 자료가 명시한 모델이며, BMB 주장의 핵심 구조("반복 실행 비용 누적")와 정확히 일치한다. 연 20–30% 증가(Shopify)는 복리로 약 3년에 2배.
- 학술 공진화 연구는 "테스트 코드는 대체로 프로덕션 코드와 함께 늘고 비중이 유지·상승한다"는 정성적 결론이지, 실행 비용을 직접 다루지 않는다.

### Gaps
- Google/Microsoft의 연도별 테스트 수 시계열(예: 2010→2020)은 찾지 못함. 2016–2017 시점 스냅숏만 있음.
- 오픈소스 테스트/코드 LOC 비율의 정량 추세(예: 중앙값 변화)는 원문 접근 실패로 확보 못함.

## Q2. 대기업 CI 실행 규모와 비용

### Takeaway
Google TAP은 2016년 기준 하루 80만 빌드·1.5억 테스트 실행을 "막대한 컴퓨트 비용"으로 수행했고, Meta·Uber·Microsoft 모두 CI 컴퓨트 절감을 별도 시스템 투자로 정당화했다. 다만 달러 단위 CI 총비용이나 "전체 컴퓨트 중 테스트 비중" 같은 수치는 공개 자료에서 찾지 못했다.

### Cited Findings
- [PR, Google, ICSE-SEIP 2017] "In an average day, TAP integrates and tests — at enormous compute cost — more than 13K code projects, requiring 800K builds and 150 Million test runs." 분석 데이터: 2016-02-11~03-11, 50만+ CL, 550만 고유 테스트 타깃, 40억+ 테스트 결과 — [Taming Google-Scale Continuous Testing](https://research.google.com/pubs/archive/45861.pdf) (스니펫)
- [IND, Google 2017] 하루 1.5억 테스트 실행, **테스트당 하루 평균 35회 실행**, 420만 테스트 — [The State of CI Testing @Google](https://research.google.com/pubs/archive/45880.pdf) (스니펫)
- [BLOG/2차, Google ~2025] 현재 TAP의 전체 사이클은 최대 시간당 1회, 이후 ML 기반 "Speculative" 사이클 도입 — [hackthology: Speculative Testing at Google](https://hackthology.com/speculative-testing-at-google-with-transition-prediction.html) (2016 이후 시스템이 바뀌었음을 보여줌)
- [PR, Microsoft, ICSE-SEIP 2016] CloudBuild: 수천 명 엔지니어가 수백 제품을 하루 여러 번 빌드·테스트; 빌드·테스트 속도 1.3x–10x 향상 — [ACM DL: CloudBuild](https://dl.acm.org/doi/10.1145/2889160.2889222). BuildXL은 Microsoft 내부에서 **하루 15만+ 빌드** — [GitHub microsoft/BuildXL](https://github.com/microsoft/BuildXL)
- [BLOG+PR, Uber 2024–2025] SubmitQueue 개선(2024-07 Go/iOS/Android 모노레포 적용): CI 자원 사용 **53% 감소**, 주간 CPU 시간 **44% 감소**, p95 대기 37% 감소 — [Slashing CI Costs at Uber](https://www.uber.com/us/en/blog/slashing-ci-costs-at-uber/); [arXiv 2501.03440 "CI at Scale: Lean, Green, and Fast"](https://arxiv.org/pdf/2501.03440) (스니펫). 주의: 이 절감은 테스트 선택이 아니라 머지 큐 스케줄링/투기 실행 개선에서 나옴.
- [PR, Meta] 기존 빌드 의존성 기반 선택도 변경당 전체 테스트의 최대 25%를 고르며, Meta 규모에서 "지속 불가능한 인프라 비용"을 초래 — [ai-best-practices 케이스 요약](https://ai-best-practices.com/case-studies/meta-s-predictive-test-selection-captures-99-9-of-regressions-while-runn) (2차; 원문 [arXiv 1810.05286](https://arxiv.org/pdf/1810.05286) 확인 필요)
- [PR, Meta] 위험 인지 배치 테스트(RAPB-la): 테스트 실행 32.4% 감소, 연 **약 $491K** 절감 추정 — 검색 요약에서만 확인, 논문명/연도 원문 미확인(Gaps 참조)
- [VENDOR/공식, GitHub Octoverse] 공개 프로젝트에서 GitHub Actions **115억 분**(2025) 무료 사용, 셀프호스티드 포함 135억 분; 2024 대비 35% 증가(2024: 85억 분, 공개 기준). 2024 보고서는 105.4억 분으로 기준이 달라 직접 비교 주의 — [Octoverse 2025](https://github.blog/news-insights/octoverse/octoverse-a-new-developer-joins-github-every-second-as-ai-leads-typescript-to-1/); [Octoverse 2024](https://github.blog/news-insights/octoverse/octoverse-2024/)

### Inferences
- Google 수치로 계산하면 테스트 하나가 하루 35회, 연간 약 1.3만 회 재실행된다(35×365, 단순 산술). 대부분 통과(Q4)하므로 "한 번 증명하면 끝"인 검증과 대비되는 구조적 근거가 된다.
- Octoverse의 Actions 분 사용량 연 35% 증가는 CI 실행량의 생태계 수준 성장 지표지만, 그중 테스트 비중은 공개되지 않았다.

### Gaps
- Google/Meta/Microsoft의 CI 달러 비용, 전체 데이터센터 컴퓨트 중 CI·테스트 비중: 공개 1차 자료 못 찾음.
- Slack·Dropbox·LinkedIn 자체 블로그의 CI 비용 수치: 검색으로 찾지 못함. cicdpipelinecost.com이 Dropbox CI 컴퓨트 ~50% 감소, Spotify CI 벽시계 50% 감소를 주장하나 원 출처 추적 불가 — [cicdpipelinecost.com/case-studies](https://cicdpipelinecost.com/case-studies) (신뢰도 낮음)
- RAPB-la($491K) 논문 정확한 서지 미확인.

## Q3. 일반 팀의 CI 경제학 (빌드 시간, 사용량, 탄소)

### Takeaway
일반 팀의 CI 워크플로는 중앙값 몇 분으로 짧지만 꼬리가 길고(p95 25분+), 학술 연구에서는 오픈소스 빌드의 84%가 10분을 넘는다. 빌드 시간이 시간에 따라 "선형으로" 늘지는 않는다는 반증성 결과가 있다. CI 탄소 발자국 추정은 생태계 전체로 수백 tCO2e 규모다.

### Cited Findings
- [PR, Ghaleb/da Costa/Zou, EMSE 2019] Travis CI 사용 67개 GitHub 프로젝트, 104,442 빌드: **84%가 허용 기준 10분 초과, 40%가 30분 초과**. **빌드 시간은 시간에 따라 선형으로 증가하지 않음**(반증 근거). 빌드 시간이 길수록 CI 에너지 비용 증가 — [ResearchGate](https://www.researchgate.net/publication/331385025_An_Empirical_Study_of_the_Long_Duration_of_Continuous_Integration_Builds); [PDF](https://seal-queensu.github.io/publications/pdf/EMSE-Taher-2019.pdf)
- [VENDOR, CircleCI 2025 State of Software Delivery] 워크플로 시간 중앙값 **2분 43초**, 하위 25% 38초 미만, 75%가 약 8분 이하, **p95 25분 초과, 평균 11분** — [PDF](https://circleci.com/landing-pages/assets/2025-state-of-software-delivery-report.pdf). 2024 보고서: 평균 2분 49초(전년 대비 13% 빠름; 2025는 중앙값이라 비교 불가) — [CircleCI 2024 newsroom](https://circleci.com/newsroom/2024-state-of-software-delivery-report/)
- [PR/preprint 2026] GitHub Actions 워크플로 진화 연구: 워크플로가 진화하며 빌드 시간 증가가 관찰되고 긴 빌드가 반복적 우려 — [arXiv 2602.14572](https://arxiv.org/pdf/2602.14572) (스니펫, 정량치 미확인)
- [PR, 2024 "Software Frugality ... the Case of CI"] 396개 Java 저장소의 838개 GitHub Actions 워크플로를 재현해 에너지 측정 — [ar5iv 2410.15816](https://ar5iv.labs.arxiv.org/html/2410.15816)
- [preprint 2025 "Environmental Impact of CI/CD Pipelines"] 18,000+ 저장소, 220만+ 워크플로 실행 기반 GitHub Actions 생태계 추정: 가장 가능성 높은 시나리오 **456.9 tCO2e**, 물 5,738.2 kL; 지역에 따라 150.5(노르웨이)~994.9(인도) tCO2e. Actions 실행 데이터 보존 400일로 장기 분석 제한 — [arXiv 2510.26413](https://arxiv.org/html/2510.26413v1)
- [실무 글] 샤딩은 벽시계를 줄이지만 셋업 오버헤드가 샤드 수만큼 늘어 총 과금 분은 오히려 증가 — [DEV Community: cut monorepo CI time in half](https://dev.to/jimmyyeung/journey-of-systematically-cut-our-monorepo-ci-time-in-half-ec8) (개인 사례)

### Inferences
- 일반 팀 다수(중앙값 수 분)에게 CI 실행 비용은 대기업만큼 크지 않을 수 있다 — BMB 주장을 "모든 팀에 큰 비용"으로 일반화하면 CircleCI 분포가 반증이 된다. 비용 누적 주장은 스위트가 커진 장수 프로젝트·모노레포·꼬리(p95) 쪽에서 강하다.
- 456.9 tCO2e는 (무료 공개 저장소 대상) 절대량으로는 작다. 탄소 논거는 약한 보조 근거로만 쓰는 편이 안전하다.

### Gaps
- 개발자 1인당 CI 비용($/dev/월)의 신뢰할 만한 업계 통계는 찾지 못함(벤더 마케팅만 존재).
- 같은 프로젝트에서 테스트 수 증가와 CI 분 사용량 증가를 장기 추적한 연구는 찾지 못함.

## Q4. 낭비: 결과가 바뀌지 않는 실행, flaky 재실행, 중복

### Takeaway
Google에서 테스트 실행의 약 99%는 통과하고, 실제 고장을 잡은 실행은 1.23%에 불과했다(2016 데이터). Meta는 의존 테스트의 1/3만 돌려도 결함 변경의 99.9%를 잡았다. 즉 반복 실행의 대다수는 정보량이 거의 없는 실행이라는 직접 근거가 있다. 다만 오픈소스 CI에서는 테스트 실패의 74%가 실제 버그를 잡았다는 "테스트가 가치를 낸다"는 근거도 함께 있다.

### Cited Findings
- [PR, Google ICSE-SEIP 2017] "of all test executions we examined, only a tiny fraction (**1.23%**) actually found a test breakage (or a code fix)". 550만 영향 테스트 중 한 번이라도 실패한 것은 **6.3만 개**(≈1.1%, 산술). CL당 PASSED:FAILED 테스트 타깃 비율 **99:1**. 커밋당 평균 FAILED < 0.5% — [Taming Google-Scale Continuous Testing](https://research.google.com/pubs/archive/45861.pdf) (스니펫)
- [IND, Google 2017] "99% of all test executions pass" — [45880.pdf](https://research.google.com/pubs/archive/45880.pdf) (스니펫)
- [PR, Google ICSE-SEIP 2019] 전이(pass↔fail transition)를 일으키는 커밋은 전체 커밋의 **8.3%** — [Assessing Transition-based Test Selection Algorithms at Google](https://mpapad.github.io/publications/pdfs/ICSE-SEIP2019.pdf) (스니펫, 맥락 미확인)
- [BLOG, Google Testing Blog 2016] 테스트 실행의 약 **1.5%가 flaky 결과**, 테스트의 약 **16%가 어느 정도 flaky** (단위가 다름: 실행 vs 테스트) — 2차 요약에서 일관되게 인용됨 — [Flaky Test Statistics (Quash)](https://quashbugs.com/blog/flaky-test-statistics) (2차; 원 게시물 "Flaky Tests at Google and How We Mitigate Them" 직접 확인 못함)
- [2차, 출처 불명확] Google이 테스트 예산의 약 2–16%를 재실행에 쓴다는 수치가 George Mason대 논문을 경유해 인용됨 — 원출처 추적 불가, **사용 비권장** — [Flaky Test Statistics (Quash)](https://quashbugs.com/blog/flaky-test-statistics)
- [PR, Labuschagne/Inozemtseva/Holmes, FSE 2017] Travis CI 61개 Java 프로젝트: 테스트 스위트 실행의 **18%가 실패**, 그 실패의 **13%가 flaky**; 비-flaky 실패 중 **74%는 실제 버그**, 나머지(약 26%)는 잘못되거나 낡은 테스트(유지보수 비용). 실패한 빌드에서도 테스트 케이스 실행의 **0.38%**만 실패 — [Measuring the Cost of Regression Testing in Practice](https://www.cs.ubc.ca/~rtholmes/papers/fse_2017_labuschange.pdf) (스니펫)
- [Thesis, Labuschagne, Waterloo] 빌드 실패의 19.4%가 테스트 코드만 고쳐서 해결, 12.8%가 비결정적 테스트 — [UWSpace](https://uwspace.uwaterloo.ca/handle/10012/10683)
- [PR, ICST 2024 industry, TUM] 산업 사례: flaky 테스트 처리 시간이 생산적 개발자 시간의 **최소 2.5%**; flaky 테스트 1회 재실행 비용 약 **0.02센트**, 수동 조사가 필요한 실패 파이프라인 1건 약 **$5.67** — [Cost of Flaky Tests in CI: An Industrial Case Study](https://conf.researchr.org/details/icst-2024/icst-2024-industry/1/Cost-of-Flaky-Tests-in-CI-An-Industrial-Case-Study); [PDF](https://mediatum.ub.tum.de/doc/1730194/1730194.pdf)
- [2차] Microsoft: 알려진 flaky 테스트를 로컬에서 100회 재실행 시 86%는 CI에서만 flaky; Microsoft 실패의 ~13%가 flaky, 연 $1.14M 비용 등의 수치는 원출처 확인 불가 — [Flaky Test Statistics (Quash)](https://quashbugs.com/blog/flaky-test-statistics)
- [PR, Bailey & Cadar ICST 2025] 성숙한 C/C++ 프로젝트에서 flaky 테스트 비율이 지난 10년간 증가 — [srg.doc.ic.ac.uk](https://srg.doc.ic.ac.uk/publications/25-testing-evol-icst.html)

### Inferences
- "반복 실행의 ~99%는 결과가 같다"는 Google 수치가 BMB의 낭비 주장에 가장 강한 외부 근거다. 단 이것은 "테스트가 무가치하다"가 아니라 "대부분의 실행이 정보량이 적다"는 뜻이다. 회귀를 잡는 1%가 존재 이유이며, Labuschagne의 74% 실버그 수치가 이를 뒷받침하는 반대 방향 근거다.
- 정적 증명은 flaky 재실행 비용(비결정성)을 구조적으로 제거한다는 점에서 별도의 논거가 될 수 있다. 다만 flaky는 주로 통합/E2E 테스트(네트워크·동시성·환경)에서 나오는데, 이는 SMT 계약으로 대체하기 가장 어려운 영역이기도 하다(추론).

### Gaps
- "의존성 라이브러리의 테스트가 재실행되는 비용"을 측정한 연구는 찾지 못함. Bazel은 기본적으로 통과한 테스트 결과를 캐시해 변경 없는 의존성 테스트는 재실행하지 않는다 — [VirtusLab Bazel book: Test Caching](https://bazel.virtuslab.com/book/1~2~1/) — 이는 "라이브러리마다 테스트를 매번 재실행"이라는 전제가 캐시 빌드 시스템에서는 부분적으로 해소됨을 뜻한다(반증 방향).
- 중복(redundant) 테스트 비율의 대규모 산업 수치 미확보.

## Q5. 완화책과 그 절감률이 드러내는 낭비 규모

### Takeaway
선택·스케줄링 기법이 큰 폭(약 2배, 40–55%)의 절감을 내면서 결함 검출을 거의 유지한다는 사실 자체가 "전수 반복 실행"의 대부분이 낭비임을 정량적으로 보여준다. 동시에 이는 BMB에 대한 핵심 반론이기도 하다: 반복 실행 비용은 증명 없이도 기존 도구(선택·캐시)로 상당 부분 줄일 수 있다.

### Cited Findings
- [PR+BLOG, Meta/Facebook, ICSE-SEIP 2019; 블로그 2018-11-21] 예측적 테스트 선택: 1년 이상 운영, 수정 코드에 의존하는 테스트의 **약 1/3만 실행**하고도 회귀의 **99.9% 이상** 포착, 테스트 인프라 **효율 2배**(총 테스트 인프라 비용 약 절반). 개별 테스트 실패의 95% 이상 보고 — [Predictive test selection (Engineering at Meta)](https://engineering.fb.com/2018/11/21/developer-tools/predictive-test-selection/); [arXiv 1810.05286](https://arxiv.org/pdf/1810.05286); [ACM DL](https://dl.acm.org/doi/10.1109/ICSE-SEIP.2019.00018). 2025 재현 연구(TU Wien 석사논문) 존재 — [repositum.tuwien.at](https://repositum.tuwien.at/bitstream/20.500.12708/215633/1/Aichmann%20Stefan%20-%202025%20-%20Predictive%20Test%20Selection%20A%20Replication%20Study.pdf)
- [IND, Google 2016] just-in-time 스케줄링으로 이차 → 선형 시간 복잡도 — [Micco 2016 slides](https://www.slideshare.net/JohnMicco1/2016-0425-continuous-integration-at-google-scale)
- [BLOG+PR, Uber 2024] CI 자원 53%, 주간 CPU 시간 44% 감소 — [Uber blog](https://www.uber.com/us/en/blog/slashing-ci-costs-at-uber/); [arXiv 2501.03440](https://arxiv.org/pdf/2501.03440)
- [BLOG, Shopify] 코어 모놀리스 CI p95를 **45분 → 18분**; 테스트 예산 실험에서 테스트 선택 스위트의 70%만 실행해도 실패 발견 — [Keeping Developers Happy with a Fast CI](https://shopify.engineering/faster-shopify-ci); [Test Budget](https://shopify.engineering/test-budget-time-constrained-ci-feedback); 관련 [Spark Joy by Running Fewer Tests](https://shopify.engineering/spark-joy-by-running-fewer-tests) (스니펫)
- [PR, Microsoft ICSE-SEIP 2016] CloudBuild 캐싱·분산으로 빌드·테스트 1.3x–10x 가속 — [ACM DL](https://dl.acm.org/doi/10.1145/2889160.2889222)
- [IND, Spotify] Bazel로 변경 세트에 필요한 테스트만 실행 → CI 시작 시간 66% 감소(총 컴퓨트 아님) — [qubika (2차)](https://qubika.com/blog/testing-at-scale/)
- [VENDOR] Nx affected / Bazel 선택 테스트로 PR당 CI 분 70–95% 감소, 모노레포 CI 비용 60–80% 절감 주장 — [IAN Blog](https://iancloud.ai/blog/ci-cost-optimization-monorepos-test-sharding-caching-2026); [cicdcost.com](https://cicdcost.com/monorepo-ci-cost) (마케팅성, 최선 사례로 취급)
- [2차] Microsoft·Google·Datadog이 TIA로 CI 테스트 시간 50–80% 절감을 공개했다는 주장 — [Drizz](https://www.drizz.dev/post/test-impact-analysis) (원출처 미확인); TIA 개괄 — [Martin Fowler, The Rise of Test Impact Analysis (2017)](https://martinfowler.com/articles/rise-test-impact-analysis.html)

### Inferences
- 절감률 수렴대: 대기업 1차 자료 기준 **약 40–67%의 실행/컴퓨트 절감**(Meta 2/3 실행 생략, Uber 44–53%), 결함 검출 손실은 0.1% 수준(Meta). 이것이 "전수 반복 실행에 포함된 낭비"의 하한 추정치로 쓰일 수 있다.
- BMB 관점의 반론(counter-evidence)으로 명시해야 할 것: (1) 낭비의 상당 부분은 테스트 선택·캐싱·스케줄링으로 이미 회수 가능하다, (2) Google 2016 자료는 이 완화책들로 비용 곡선을 이차에서 선형으로 낮췄다고 주장한다, (3) 빌드 시간은 선형으로 증가하지 않는다(Ghaleb 2019), (4) 일반 팀의 워크플로 중앙값은 3분 미만(CircleCI 2025). 따라서 BMB의 차별 가치는 "실행 비용 절감" 단독보다 "실행으로는 원리적으로 못 얻는 보장 + 남은 선형 비용 제거"로 프레이밍하는 편이 증거에 부합한다.

### Gaps
- Bazel 원격 캐시의 1차 측정 절감률(Google/기업 공개)은 찾지 못함 — 벤더의 가상 예시(10분 → 90초)뿐 — [VirtusLab/EngFlow guide](https://visdom-maturity-matrix.virtuslab.com/guides/delivery/bazel-remote-caching-engflow)
- 테스트 선택 도입 후의 비용이 시간이 지나며 다시 증가하는지(재성장) 추적한 연구 없음.
