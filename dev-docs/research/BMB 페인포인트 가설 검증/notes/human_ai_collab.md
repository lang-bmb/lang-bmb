# 인간–AI 협업 방향(소프트웨어 개발, 2024–2026): 인간 역할·병목 이동·워크플로

> 조사일 2026-10-25. 표기: [PR]=동료심사 논문, [pre]=arXiv 등 비심사, [V]=벤더/이해관계자 보고서, [S]=대규모 설문, [O]=의견·에세이, [N]=언론 2차 보도.
> 한계: martinfowler.com, martin.kleppmann.com 원문은 DNS 오류로 직접 열지 못했다. 해당 내용은 검색 스니펫과 2차 요약에 기반한다. 대부분 수치는 검색 요약 단계에서 확인했으며 원문 PDF 대조는 하지 않았다.

## Q1. E1("인간은 더 이상 코드를 줄 단위로 읽고 쓰지 않는다")이 실제로 일어나고 있는가?

### Takeaway
"작성"은 빠르게 AI로 넘어가고 있다(대기업 신규 코드 25~30%+, 일부 스타트업·AI 회사는 70~100%). 하지만 "읽기"에서 손을 뗐다는 증거는 없다. 오히려 신뢰가 떨어지고 있고, 전문가 다수는 vibe coding을 업무에 쓰지 않는다. E1의 "쓰기" 절반은 추세가 뒷받침하고, "읽기" 절반은 현재 관행과 어긋난다. 다만 읽기가 병목·부담이 되고 있다는 점(Q2)이 E1의 동기를 뒷받침한다.

### Cited Findings
- 작성 비중(기업 발언, 측정 정의가 서로 다름):
  - Google 순다르 피차이: 2024년 10월 실적 발표에서 신규 코드의 "more than a quarter"를 AI가 생성하고 엔지니어가 리뷰한다고 했고, 2025년 4월에는 "well over 30%"라고 했다. 대상은 *신규* 코드다 [N] — [Entrepreneur](https://entrepreneur.com/business-news/ai-is-taking-over-coding-at-microsoft-google-and-meta/490896), [Business Today/HumanProgress](https://humanprogress.org/30-percent-of-microsofts-code-is-now-ai-generated-says-ceo-satya-nadella/)
  - Microsoft 사티아 나델라(LlamaCon, 2025-04-29): "maybe 20-30% of the code that is inside of our repos today and some of our projects are probably all written by software". 대상은 *저장소 내* 코드라 Google 수치와 직접 비교할 수 없다 [N] — [TechRadar](https://www.techradar.com/pro/a-shockingly-high-amount-of-microsoft-code-is-now-written-by-ai-it-admits), [Business Today](https://www.businesstoday.in/amp/tech-today/news/story/30-of-microsofts-code-is-now-ai-generated-says-ceo-satya-nadella-474167-2025-04-30)
  - Y Combinator W25 배치의 25%는 코드베이스 95%가 AI 생성이다(Jared Friedman, 2025-03). 정의는 "인간이 타이핑한 코드 대비 AI 코드"이고 import 같은 보일러플레이트는 제외했다. 같은 영상에서 Diana Hu는 "코드를 읽고 버그를 찾는 능력"이 여전히 필요하다고 했고, Garry Tan은 규모가 커지면 고전적 코딩 역량이 필요하다고 단서를 달았다. YC는 AI 회사 투자자라 이해관계가 있다 [N] — [TechCrunch](https://techcrunch.com/2025/03/06/a-quarter-of-startups-in-ycs-current-cohort-have-codebases-that-are-almost-entirely-ai-generated)
  - Anthropic: Claude Code 개발자 Boris Cherny가 Claude Code 코드베이스의 80~90%를 Claude가 작성했다고 말했다는 보도가 있다. 회사 전체 70~90%, 일부 엔지니어 100%라는 보도도 있다. 1차 출처는 확인하지 못했고 2차 보도뿐이다 [N] — [implicator.ai](https://www.implicator.ai/the-accidental-architect-how-boris-cherny-built-silicon-valleys-most-talked-about-coding-tool/), [YC Light Cone 요약](https://traces.com/s/jn7e26f5f3jg60zssbjy87c39981a054)
- 채택과 신뢰(설문):
  - Stack Overflow 2025(n=49,009, 166개국): 84%가 AI 도구를 사용 중이거나 사용 예정이다(2024년 76%). AI 정확도를 불신 46% 대 신뢰 33%이고, "highly trust"는 3%다. 불신은 2024년 31%에서 올랐다. 호감도는 60%로 2024년 72%, 2023년 77%에서 내려왔다. 경력 개발자는 "highly trust" 2.6%, "high distrust" 20%다 [S] — [TechRepublic](https://www.techrepublic.com/article/news-developer-trust-in-ai-declines/), [ShiftMag](https://shiftmag.dev/stack-overflow-survey-2025-ai-5653/)
  - 같은 설문에서 약 77%가 vibe coding은 전문 업무의 일부가 아니라고 답했다. AI 에이전트는 "정기 사용" 23%(보도 제목 기준), "매일 사용" 14.1%, "사용 계획 없음" 37.9%다. 집계 방식에 따라 보도 수치가 다르다 [S] — [IT Brief](https://itbrief.co.uk/story/developers-adopt-ai-tools-but-trust-issues-persist-survey-finds), [The New Stack](https://thenewstack.io/23-of-devs-regularly-use-ai-agents-per-stack-overflow-survey/)
  - DORA 2025(Google): 채택률 90%(2024년 76%)다. AI를 "조금/전혀 신뢰하지 않음"은 30%(2024년 39.2%), "적어도 어느 정도 신뢰"는 70%(2024년 87.9%)다. 80% 이상이 생산성이 늘었다고, 59%가 코드 품질이 좋아졌다고 자기보고했다 [S] — [TechTarget](https://www.techtarget.com/searchsoftwarequality/news/366631712/Google-DORA-Software-delivery-caught-up-to-AI-coding-tools)
  - Sonar 설문(2026-01, 1,100명 이상): 96%가 AI 코드의 기능적 정확성을 완전히 신뢰하지는 않는다. 38%는 AI 코드 리뷰가 동료 코드 리뷰보다 힘들다고 답했다 [V] — [HackerNoon 요약](https://hackernoon.com/ai-is-writing-more-codeand-creating-a-review-bottleneck)
- 생산성 체감과 실측의 괴리: METR RCT(2025년 2~6월, 숙련 OSS 개발자 16명, 실제 과제 246건, Cursor 사용)에서 AI를 허용하자 완료 시간이 19% *늘었다*. 개발자 본인은 사후에도 20% 빨라졌다고 믿었다. 원인으로 AI 제안을 검토·수정하는 시간이 지목됐다. 2025년 초 도구 기준이고 표본이 작다 [pre/RCT] — [Cybernews](https://cybernews.com/news/ai-tools-software-developers/), [Let's Data Science](https://letsdatascience.com/blog/developers-thought-ai-made-them-faster-the-data-said-otherwise)
- 이해 부채(comprehension/cognitive debt): Margaret-Anne Storey(UVic)가 2025년 10월 "cognitive debt"(팀 공유 이해의 부채)를 처음 썼다고 LeadDev가 보도했다. "comprehension debt"는 Addy Osmani 등 실무자가 퍼뜨렸다. 기술 부채가 코드의 속성인 데 비해 이 부채는 "팀과 코드의 관계의 속성"이라는 구분이 있다 [O] — [LeadDev](https://leaddev.com/ai/ai-coding-creates-two-kinds-of-debt-youre-only-measuring-one), [Addy Osmani](https://addyosmani.com/blog/comprehension-debt/), [tianpan.co](https://tianpan.co/blog/2026/06/21/comprehension-debt-the-system-no-human-understands)

### Inferences
- E1의 "쓰기" 측면은 추세와 맞는다. "읽기" 측면은 현재 업계가 *유지하려고 애쓰는* 관행이다. 인간 리뷰는 여전히 공식 안전장치로 남아 있다(Google: "엔지니어가 리뷰"). 다만 신뢰 하락, 리뷰 부담 증가, 이해 부채가 겹쳐 "줄 단위 읽기가 지속 불가능해지고 있다"는 논거는 강해지고 있다. BMB는 E1을 "이미 일어난 사실"보다 "줄 단위 읽기가 확장되지 않는다는 압력에 대한 설계 대응"으로 쓰는 편이 증거와 더 잘 맞는다.
- 공개 수치는 측정 정의(신규/전체 코드, 문자/라인/수락률)가 서로 달라 비교할 수 없다. 이 자체가 출처 추적(provenance)이 없다는 증거다.

### Gaps
- JetBrains State of Developer Ecosystem 2025, GitHub Octoverse 2025의 구체 수치(AI 코드 비중, 리뷰 관행)는 이번에 확인하지 못했다.
- "개발자가 AI 코드를 실제로 몇 % 읽는가"를 직접 측정한 대규모 연구는 찾지 못했다.
- Cherny/Anthropic 수치의 1차 출처(팟캐스트 원문)는 확인하지 못했다.

## Q2. 코드 리뷰가 새 병목인가? (PR 볼륨, 리뷰 시간, 피로, AI 리뷰 도구 효과)

### Takeaway
여러 독립·벤더 데이터가 같은 방향을 가리킨다. AI는 코드 생산량과 PR 크기를 늘리지만 리뷰·머지 시간도 늘리고, 출하 처리량이나 안정성은 그만큼 오르지 않는다. AI 리뷰 도구는 코멘트 상당수가 반영되지만 PR 종료 시간을 줄이지는 못했다(단일 산업 사례).

### Cited Findings
- Faros AI(DORA 인용 맥락): AI 고채택 팀은 머지된 PR이 약 98% 늘었지만 PR 리뷰 시간도 약 91% 늘었다. 요약본마다 리뷰 시간 증가 폭이 다르다("4배 이상"이라는 요약도 있다) [V] — [codegen.com](https://codegen.com/?p=22025); 상충: [HackerNoon](https://hackernoon.com/ai-is-writing-more-codeand-creating-a-review-bottleneck)
- LinearB 2026 Benchmarks(PR 800만 건 이상): AI 보조 PR은 인간 PR보다 약 2.6배 크다. 에이전트 PR은 리뷰어가 잡기까지 약 5.3배 더 오래 대기한다 [V] — [LinearB](https://linearb.io/blog/code-generation-faster-shipping-isnt)
- CircleCI 2026 State of Software Delivery: feature 브랜치 처리량은 전년 대비 59% 늘었지만, 중앙값 팀의 main 브랜치 처리량은 줄었다. 제약이 "작성"에서 "머지해도 안전한지 판단"으로 옮겨갔다는 해석이다 [V] — [Codacy 블로그 인용](https://blog.codacy.com/ai-breaking-code-review-how-engineering-teams-survive-pr-bottleneck)
- Harvard 워킹페이퍼(Fiona Chen, James Stratton; TechRadar 보도): AI로 코드 산출은 약 30% 늘었지만 완성 소프트웨어에는 통계적으로 유의한 증가가 없었다. 리뷰·머지 시간은 약 49% 늘었다 [pre/N] — [TechRadar](https://techradar.com/pro/ai-agents-are-great-at-writing-new-code-but-then-the-human-bottleneck-slows-everything-down)
- DORA 2025: AI 채택은 처리량과 양의 상관을 보이게 됐지만 불안정성(변경 실패, 재작업, 해결 시간)과도 양의 상관을 보인다. AI는 조직의 강점과 약점을 모두 증폭하는 "amplifier"라고 했다 [S] — [Splunk 요약](https://www.splunk.com/en-us/blog/learn/state-of-devops.html), [InfoQ](https://www.infoq.com/news/2025/09/dora-state-of-ai-in-dev-2025)
- AI 코드 품질: CodeRabbit(2025-12)은 OSS PR 470건(AI 공동작성 320건, 인간 150건)을 분석했다. 이슈는 AI PR 10.83개 대 인간 PR 6.45개(약 1.7배)이고, critical 1.4배, 로직·정확성 +75%, 보안 1.5~2배, 가독성 3배 이상, 과도한 I/O 약 8배다. CodeRabbit은 리뷰 도구 판매사이고, 라벨 정확성을 보장하지 못한다고 스스로 밝혔다 [V] — [CodeRabbit](https://coderabbit.ai/blog/state-of-ai-vs-human-code-generation-report), [The Register](https://www.theregister.com/2025/12/17/ai_code_bugs/)
- AI 코드 리뷰 도구 효과(동료심사): Cihan et al. "Automated Code Review In Practice"(ICSE 2025 SEIP)는 Beko에서 Qodo PR Agent 기반 LLM 리뷰어를 쓴 사례다. 자동 코멘트의 73.8%가 resolved로 표시됐지만, 평균 PR 종료 시간은 5시간 52분에서 8시간 20분으로 *늘었다*. 오검토, 불필요한 수정, 무관한 코멘트가 단점으로 보고됐다 [PR] — [arXiv 2412.18531](https://arxiv.org/abs/2412.18531v2)
- Atlassian 등(ASE 2025 industry): LLM 리뷰 코멘트 중 가독성·버그·유지보수 코멘트는 반영률이 높고, 설계 관련 코멘트는 반영률이 낮다 [PR-industry] — [alphaXiv 2510.05450](https://www.alphaxiv.org/abs/2510.05450v1)

### Inferences
- 병목은 "생성"에서 "검증·승인(merge-safety 판단)"으로 이동했다. 이 점은 여러 출처가 수렴한다. BMB가 리뷰 대상을 줄 단위 diff에서 결정론적 계약·증명 결과로 옮기려는 동기와 정확히 맞는다.
- AI 리뷰어는 국소적 결함(가독성·버그)에는 유효하지만 설계 판단에는 약하다. 이는 "AI는 판단(의도·설계)만"이라는 BMB의 분업과 긴장 관계에 있다. 설계 판단이 현재 AI가 가장 약한 영역일 수 있다.
- "LGTM 러버스탬핑"을 직접 측정한 연구는 찾지 못했다. 리뷰 시간 증가와 대기 증가는 반대로 리뷰가 *더 무거워졌음*을 시사한다.

### Gaps
- 리뷰어 피로·러버스탬핑의 정량 데이터(리뷰 코멘트 밀도 감소 등)를 담은 동료심사 연구는 찾지 못했다.
- Faros/LinearB/CircleCI 원문 보고서는 직접 확인하지 못했다(벤더 블로그 2차 인용).
- GitHub Copilot code review의 효과를 독립적으로 평가한 연구는 찾지 못했다.

## Q3. 스펙 주도 개발(SDD)과 의도 표현: 인간 산출물이 "의도/요구사항"으로 이동하는가?

### Takeaway
2025년 하반기부터 SDD가 뚜렷한 도구 흐름이 됐다(Kiro, Spec Kit, Tessl, "Specifications are the new code"). 다만 스펙은 대부분 *비형식 마크다운*이다. 스펙이 코드와 동기화되는지, 에이전트가 스펙을 실제로 따르는지는 미해결로 지적된다. 컨텍스트 파일(AGENTS.md)은 표준이 없고, 효과에 반대 증거가 있다.

### Cited Findings
- Sean Grove(OpenAI), "The New Code"/"Specifications are the new code"(AI Engineer World's Fair 2025): 프롬프트는 버리고 생성물만 남기는 관행을 비판했다. 버전 관리되는 스펙을 진실의 원천으로 삼자고 주장했고, "the essence of programming is communication"이라고 했다. OpenAI Model Spec(마크다운 묶음)을 사례로 들었는데, GPT-4o 아첨(sycophancy) 버그 때 스펙의 "아첨하지 말 것" 조항이 롤백의 기준점이 됐다는 것이다. 반론으로 "워터폴의 귀환"이라는 비판이 있고, IEEE Software 칼럼도 의문을 제기했다 [O] — [ai.engineer](https://ai.engineer/talks/specifications-are-the-new-code), [HackerNoon](https://hackernoon.com/forget-prompt-engineering-the-real-skill-is-specification-writing), [36kr 비판](https://www.36kr.com/p/3388182127870345)
- Birgitta Böckeler(Thoughtworks), "Understanding Spec-Driven-Development: Kiro, spec-kit, and Tessl"(martinfowler.com, 2025-10-15): SDD의 정의는 아직 정착하지 않았다. 수준을 spec-first(작업 전 스펙 작성), spec-anchored(작업 후에도 스펙을 유지·진화), 그리고 spec-as-source(Tessl처럼 스펙만 유지하고 코드는 생성물)로 나눴다. Kiro는 requirements→design→tasks 단계를 거치고, spec-kit은 비슷한 3단계에 "constitution"(항상 지킬 원칙)을 더한다. Tessl Framework는 2025-09 비공개 베타였다 [O] — [martinfowler.com](https://www.martinfowler.com/articles/exploring-gen-ai/sdd-3-tools.html), [Thoughtworks Radar](https://www.thoughtworks.com/en-au/radar/techniques/spec-driven-development)
- Kiro(AWS): 스펙은 `.kiro/specs/` 아래 마크다운 3파일이고 요구사항은 EARS 표기(WHEN…SHALL)로 쓴다. EARS 요구사항을 실행 가능한 속성으로 바꿔 property-based testing을 생성한다. 문서는 "PBT는 모든 버그의 부재를 보장하지 않지만 예시 기반 테스트보다 훨씬 강한 정확성 증거"라고 인정한다. 2025-11-17 GA와 함께 PBT가 발표됐다 [V] — [Kiro docs: correctness](https://kiro.dev/docs/specs/correctness/), [Verdent 가이드](https://www.verdent.ai/guides/agents/kiro-spec-driven-development)
- AGENTS.md/컨텍스트 파일: MSR 2026 연구(466개 OSS 프로젝트)는 내용 구조에 정착된 표준이 없고 서술형·규범형·금지형이 섞여 있다고 했다 [PR] — [arXiv 2510.21413](https://arxiv.org/html/2510.21413v2). ETH Zurich "Evaluating AGENTS.md"(AGENTbench, Python 과제 138개)는 컨텍스트 파일이 있을 때 성공률이 오히려 떨어지고 추론 비용이 20% 넘게 늘었다고 보고했다. 2차 보도이고 원문은 확인하지 못했다. 불필요한 요구사항이 과제를 어렵게 만드니 최소 요구만 쓰라고 권고한다 [pre] — [InfoQ](https://infoq.com/news/2026/03/agents-context-file-value-review/)

### Inferences
- 업계는 "인간 산출물=의도/스펙" 방향으로 움직이고 있다. 이는 BMB 원칙 3(의도는 선언으로 강제)과 같은 방향이다. 차이도 있다. 업계 SDD의 스펙은 *비형식 산문*이라 구현과의 일치를 강제하지 못한다(Kiro만 PBT로 일부 연결한다). BMB의 "계약=이름 붙은 술어 + 노드 해시에 묶인 의도 산문"은 이 동기화 문제(spec drift)를 겨냥한 차별점으로 위치시킬 수 있다.
- AGENTS.md의 부정적 결과는 "산문 지시는 강제되지 않는다"는 BMB의 문제의식을 뒷받침하는 반사례로 쓸 수 있다.

### Gaps
- Böckeler 글의 비판 세부(마크다운 리뷰 부담, 에이전트의 스펙 무시, MDD와의 유사성)는 원문을 열지 못해 확인하지 못했다.
- GitHub Spec Kit 공식 문서와 채택 통계는 직접 조사하지 못했다.
- Kiro와 AWS Automated Reasoning(형식 검증)의 연결은 검색으로 확인하지 못했다.

## Q4. 신뢰·책임·감사 가능성·규제

### Takeaway
EU 규제는 "누가 썼든 제조자 책임"으로 굳어지고 있다. 소프트웨어는 무과실 책임 대상 제품이 됐고(PLD), 취약점 보고와 SBOM 같은 추적성 의무가 생겼다(CRA). AI 작성 코드를 특별 취급하는 조항은 찾지 못했다. 즉 AI 코드도 출하자가 전적으로 책임지므로 검증 근거와 출처 추적의 가치가 올라간다.

### Cited Findings
- EU 제조물책임지침(PLD) 2024/2853: 2024년 12월 발효됐고 회원국은 2026-12-09까지 국내법으로 옮겨야 한다. 이후 출시 제품에 적용된다. 소프트웨어(독립형·임베디드·SaaS·AI 시스템)를 "제품"으로 보아 무과실 책임 대상이 된다. 업데이트와 보안 취약점까지 책임이 이어지고, 사이버보안 요구 불이행도 결함이 될 수 있으며, 계약으로 면책할 수 없다. 원고의 입증 부담이 줄고 법원이 증거 공개를 명령할 수 있다 [법률사무소 해설] — [Latham & Watkins](https://www.lw.com/en/insights/2024/12/New-EU-Product-Liability-Directive-Comes-Into-Force), [Reed Smith](https://www.reedsmith.com/en/perspectives/2025/07/eu-product-liability-directive-software-digital-products-cybersecurity), [Wolf Theiss](https://www.wolftheiss.com/insights/software-digital-products-and-ai-stricter-safety-requirements-on-the-horizon/)
- 사이버복원력법(CRA, Regulation (EU) 2024/2847): 2026-09-11부터 악용된 취약점과 중대 사고를 보고해야 한다(24시간 조기경보, 72시간 통지, 최종보고). 이미 출시된 제품도 포함된다. 2027-12-11 전면 적용되며, 기술문서에 기계 판독 가능한 SBOM을 넣어야 한다(벤더 해설 기준) [V/해설] — [Bootlin](https://bootlin.com/blog/cyber-resilience-act-cra-obligations-for-manufacturers/), [Docker](https://www.docker.com/blog/eu-cyber-resilience-act-overview/), [noze](https://www.noze.it/en/insights/cyber-resilience-act-sbom/)
- AI 작성 코드도 엄격 책임 대상이고 증거 미공개 시 결함이 추정될 수 있다는 의견이 있으나, 개인 LinkedIn 글이라 권위는 낮다 [O] — [Jens Hein](https://de.linkedin.com/in/jenshein)

### Inferences
- 규제는 "AI가 썼다"를 면책 사유로 인정하지 않는다. 그러면 인간(조직)이 통제권을 유지하는 데 필요한 것은 (a) 무엇이 어떤 의도로 바뀌었는지 남는 감사 기록, (b) 출처·의존성 추적(SBOM), (c) 결함이 없다는 근거를 법정에서 제시할 수 있는 증거다. BMB의 노드 해시 결합 의도와 결정론적 증명 게이트는 (a)와 (c)에 직접 대응할 수 있다. 다만 이것은 추론이며 법적 효력이 검증된 것은 아니다.

### Gaps
- EU AI Act가 코딩 보조 도구나 생성 코드에 미치는 구체적 영향은 조사하지 못했다.
- AI 코드 출처 표시(provenance, 예: 커밋 트레일러 표준) 관련 업계 표준 동향은 찾지 못했다.

## Q5. 형식 방법 + AI: 인간 역할 = 코드 대신 스펙 리뷰? 인간이 스펙을 코드보다 잘 읽는가?

### Takeaway
"AI가 증명을 쓰고 검사기가 거부권을 가지면 줄 단위 리뷰 없이 신뢰할 수 있다"는 비전이 영향력 있는 저자들 사이에서 수렴하고 있다(Kleppmann, Tao, Lahiri). 그러나 모두 남은 병목으로 *스펙이 의도와 맞는지*를 꼽는다. 인간이 형식 스펙을 정확히 읽고 쓰는 능력은 전문가에게도 취약하다는 동료심사 증거가 있다. "스펙 리뷰가 코드 리뷰보다 쉽다"를 직접 입증한 연구는 찾지 못했다.

### Cited Findings
- Martin Kleppmann, "Prediction: AI will make formal verification go mainstream"(2025-12-08): seL4는 C 약 8,700줄에 약 20인년과 Isabelle 증명 약 20만 줄이 들었다. 증명 검사기가 잘못된 증명을 거부하므로 LLM의 그럴듯한 헛소리가 덜 위험해지고, AI 생성 코드를 줄 단위 리뷰 없이 신뢰할 수 있게 된다는 주장이다. Dafny, Nagini, Verus를 언급했다. 이런 증명을 쓸 수 있는 사람이 "몇백 명"이라는 수치는 본인의 추정이다 [O] — [martin.kleppmann.com](https://martin.kleppmann.com/2025/12/08/ai-formal-verification.html), [Substack](https://martinkl.substack.com/p/ai-will-make-formal-verification), [Simon Willison 요약](https://simonwillison.net/random/predictions)
- Terence Tao: Lean이 AI 출력을 정직하게 붙잡아두는 판정자 역할을 하고, 인간은 문제 선택, 워크플로 설계, 결과 확인을 맡는다(OpenAI Academy, 2026-03). AI가 형식화를 빠르게 해내자 결과를 읽고 정리하는 일이 새 병목이 됐다는 2026-06 보도도 있다. 2차 보도 중심이다 [N/O] — [OpenAI Academy](https://academy.openai.com/en/public/blogs/terence-tao-ai-is-ready-for-primetime-in-math-and-theoretical-physics-2026-03-06), [yage.ai](https://yage.ai/share/tao-lean-formalization-credibility-en-20260623.html), [Scientific American](https://www.scientificamerican.com/article/ai-will-become-mathematicians-co-pilot)
- Shuvendu Lahiri(Microsoft Research), "Evaluating LLM-driven User-Intent Formalization for Verification-Aware Languages"(FMCAD 2024): 형식 스펙이 사용자 의도와 맞는지 알고리즘으로 판정할 방법은 없다. 대안으로 스펙을 기호적으로 테스트하는 지표를 제안했고, Dafny/MBPP 인간 라벨과 높은 일치를 보였다 [PR]. 후속 "Intent Formalization: A Grand Challenge for Reliable Coding in the Age of AI Agents"(2026-03)는 비형식 의도를 검사 가능한 형식 스펙으로 옮기는 일을 핵심 난제로 규정했다. 테스트를 프록시 오라클로 써서 스펙의 건전성·완전성·정밀성을 평가하고, TiCoder처럼 테스트로 의도의 모호성을 해소한다 [O/pre] — [arXiv 2406.09757](https://arxiv.org/abs/2406.09757v2), [Illinois 강연 요약](https://calendars.illinois.edu/detail/7097/33504362)
- 인간의 형식 스펙 이해력(반대 증거): Greenman et al. "Little Tricky Logic: Misconceptions in the Understanding of LTL"(Programming 2023)는 연구자와 학습자를 4라운드 조사했다. 영어→LTL *작성*이 가장 흔한 오류 원천이었고, 미묘한 실수가 결함 시스템으로 이어진다. 이 결과로 Alloy 언어가 바뀌었다. 후속 연구(FM 2024)는 유한 트레이스 LTL에서 *전문가도* 여러 오개념을 보인다고 보고했다 [PR] — [Programming Journal](https://programming-journal.org/2023/7/7/), [Brown FM 2024](https://cs.brown.edu/~sk/Publications/Papers/Published/gpdzdkmnz-miscon-finite-infinite-ltl/)

### Inferences
- 형식 방법 진영의 비전(인간=스펙 검토자, 기계=증명 검사)은 BMB의 분업(AI=의도·설계·스펙 선택 판단, 도구=결정론적 검증)과 거의 같다. 다만 Lahiri와 Greenman 증거는 "스펙이 의도와 맞는가"가 인간에게도 AI에게도 어렵다는 점을 보여준다. 따라서 BMB가 인간 통제를 위해 제공해야 할 것은 원시 형식 술어가 아니다. (a) 이름 붙은 술어와 자연어 의도의 쌍, (b) 스펙에서 생성한 구체적 예시·반례(테스트를 오라클로 쓰는 방식, TiCoder/Kiro PBT 계열), (c) 결정론적 지도에서 만든 사람용 요약이다. 이는 BMB의 "별도 도구가 사람용 문서·도식 생성" 설계를 지지하는 근거가 된다.
- "인간은 코드보다 스펙을 더 잘 읽는다"는 가설은 아직 *측정되지 않았다*. BMB 운영 원칙("차별점 가설은 최소 실험으로 측정")에 따르면 직접 실험할 후보다.

### Gaps
- Harmonic(Aristotle), Axiom Math 등 검증 코드 생성 스타트업의 소프트웨어(수학 외) 적용 실적은 이번에 조사하지 못했다.
- 스펙 리뷰와 코드 리뷰의 정확도·시간을 직접 비교한 사용자 연구는 찾지 못했다.
- JetBrains Research(2025-03)가 Claude 3.5 Sonnet으로 Dafny/Nagini/Verus에서 낸 결과는 Kleppmann 글을 인용한 2차 언급으로만 확인했다.

## Q6. 숙련 퇴화(deskilling)와 비판적 사고

### Takeaway
RCT 1건과 대규모 설문 1건이 일관되게 경고한다. AI에 *생성*을 위임하면 새 개념 습득과 비판적 관여가 줄고, *개념 질문*에 AI를 쓰면 손실이 작다. 인간이 계속 통제권을 가지려면 위임 방식이 중요하다.

### Cited Findings
- Anthropic, "AI assistance and coding skills"(2026-01-29, RCT): Python에는 익숙하지만 Trio(비동기 라이브러리)는 처음인 엔지니어 약 52명(51명이라는 보도도 있다)이 참여했다. AI 그룹은 개념 퀴즈 점수가 17% 낮았다(약 50% 대 67%라는 2차 보도). 시간 단축은 약 2분으로 통계적으로 유의하지 않았다. AI를 개념 질문에 쓴 참가자는 65% 이상, 코드 생성을 위임한 참가자는 40% 미만을 받았다. Anthropic은 표본이 작고 새 라이브러리 학습에 한정된다는 한계를 스스로 밝혔다 [V/RCT, 비심사] — [Anthropic](https://www.anthropic.com/research/AI-assistance-coding-skills), [InfoQ](https://infoq.com/news/2026/02/ai-coding-skill-formation/), [DevClass](https://devclass.com/2026/02/02/anthropic-research-skilled-devs-make-better-use-of-ai-but-using-ai-is-bad-for-learning-skills/)
- Lee, Sarkar 외(Microsoft Research + CMU), "The Impact of Generative AI on Critical Thinking"(CHI 2025, DOI 10.1145/3706598.3713778): 지식노동자 319명, 사례 936건을 분석했다. GenAI를 신뢰할수록 비판적 사고가 줄고, 자기 확신이 높을수록 비판적 사고가 늘었다. 비판적 사고의 성격은 정보 검증, 응답 통합, 과업 관리(task stewardship)로 이동한다. 장기적으로 과의존과 독립적 문제해결 능력 저하를 우려했다 [PR] — [Microsoft Research](https://www.microsoft.com/en-us/research/?p=1135061), [The Register](https://www.theregister.com/2025/02/11/microsoft_study_ai_critical_thinking/)
- METR RCT(위 Q1)의 체감과 실측 괴리(+20% 체감 대 −19% 실측)도 자기평가에 기반한 통제감이 신뢰할 수 없다는 증거다 [pre] — [Cybernews](https://cybernews.com/news/ai-tools-software-developers/)

### Inferences
- CHI 2025가 말하는 "검증·통합·과업 관리로의 이동"은 E1 이후 인간 역할의 경험적 묘사에 가깝다. 그러나 같은 연구는 그 역할 수행 능력(비판적 관여)이 AI 신뢰와 함께 약해진다고 경고한다. 그러므로 인간 통제를 유지하려면 인간의 주의력에 기대는 대신 *강제된* 검증 지점(증명 게이트, 선언된 의도와 구현의 기계적 대조)이 필요하다. 이것은 BMB 설계 방향을 지지하는 논거다.
- Anthropic RCT의 "개념 질문형 사용이 덜 해롭다"는 결과는 인간에게 설명·지도(map)를 주는 도구가 생성 위임보다 이해 유지에 낫다는 간접 근거가 된다.

### Gaps
- 숙련 개발자의 *장기* 기술 퇴화를 추적한 종단 연구는 찾지 못했다.
- Anthropic RCT 원 논문의 통계 세부(효과크기, 신뢰구간)는 원문 대조를 하지 못했다.
