# 대규모 코드베이스의 관리 난이도: 인간과 AI 에이전트 (2023–2026 근거)

> 조사 방법 메모: 이 세션에서는 WebFetch/curl이 arxiv.org, trychroma.com, cursor.com, alphaxiv.org에 도달하지 못했다(프록시 거부/DNS 실패). 따라서 아래 수치 대부분은 **검색 결과 스니펫과 2차 요약**에서 온 것이며, 1차 문서 본문으로 대조하지 못한 항목은 "[1차 미대조]"로 표시했다. 동료심사(peer-reviewed) / 프리프린트 / 벤더 주장을 항목마다 구분했다.

## Q1. 인간: 큰 코드베이스의 비용 (이해 시간, 기술부채, 탐색)

### Takeaway
개발자 시간의 과반(~58%)이 코드 이해에 쓰이고, 주당 ~17시간이 기술부채·나쁜 코드 처리에 쓰인다는 측정/설문이 있다. 대규모 조직(Google)은 IDE로는 탐색이 불가능해 컴파일러 기반 전역 인덱스(Kythe, Code Search)를 별도 인프라로 운영하며, 개발자는 하루 평균 12회 코드 검색을 한다. "이해·탐색이 병목"이라는 E2의 인간 측 전제는 근거가 탄탄하다(다만 핵심 수치는 2015–2018년 데이터).

### Cited Findings
- **프로그램 이해 ~58%** (동료심사, IEEE TSE 2018): Xia et al. "Measuring Program Comprehension: A Large-Scale Field Study with Professionals". 7개 실제 프로젝트, 전문 개발자 78명, 3,148 작업시간의 애플리케이션 상호작용 로그. 활동을 navigation/editing/comprehension/other로 분류(Minelli et al. 방식). 개발자는 시간의 약 58%를 이해 활동에 쓰고, 웹 브라우저·문서 편집기를 이해에 자주 사용. 시니어가 주니어보다 이해 비율이 유의하게 낮음 — [SMU InK](https://ink.library.smu.edu.sg/sis_research/3779); [Monash](https://research.monash.edu/en/publications/measuring-program-comprehension-a-large-scale-field-study-with-pr/)
  - 2차 요약은 이해 중 IDE 내 ~20%, 브라우저 ~27%, 문서 편집기 ~10%라고 하나 원문 대조 못함 [1차 미대조] — [GT book 요약](https://book.gtoolkit.com/developers-spend-most-of-their-time-figuri-7d4vgu7gbxcqjcvymru1vmyvt)
  - 주의: 58%는 평균이고 프로젝트별 편차 있음. "navigation"은 별도 카테고리로 집계되어 이해와 합치면 비편집 시간이 더 커짐(정확한 navigation 비율은 미확인).
- **Google 코드 검색 빈도** (동료심사, FSE 2015): Sadowski, Stolee, Elbaum "How Developers Search for Code: A Case Study". 개발자 1인당 근무일 평균 5개 검색 세션, 총 12개 쿼리(중앙값 6). 쿼리는 주로 "이미 어느 정도 아는 코드의 특정 위치"를 찾는 용도이며, 목적은 API 사용법, 코드가 하는 일, 실패 원인, 코드 위치 — [Google Research](https://research.google/pubs/pub43835/); [AcaWiki 요약(설문 396명 + 27명 2주 로그)](https://acawiki.org/How_Developers_Search_for_Code:_A_Case_Study)
  - 2026년 후속 연구(검색 스니펫): AI 보조 도구 도입 후에도 Google 내부 Code Search 사용 빈도는 변하지 않았고, 예제 찾기는 줄고 "코드 학습·탐색" 용도는 증가 [1차 미대조, 논문 제목 미확인] — [위 검색 결과 요약](https://research.google/pubs/pub43835/)
- **Google의 인덱싱 인프라**: Kythe는 빌드를 계측해 컴파일 정보를 뽑아 그래프로 변환, 언어 간 병합, 하루 여러 번 갱신. Google 엔지니어: "그 정도 코드를 탐색하려면 공통 인덱서가 필요하다. Eclipse 같은 일반 IDE로는 탐색할 수 없다" — [Wikipedia: Google Kythe](https://en.wikipedia.org/wiki/Google_Kythe); [SE Daily SED835 Monoliths transcript](https://softwareengineeringdaily.com/wp-content/uploads/2019/05/SED835-Monoliths.pdf)
- Code Search는 파일 내 대부분 토큰을 클릭 가능하게 해 정의·사용처로 이동(컴파일러 기반 cross-reference) — [Software Engineering at Google, Ch.17 Code Search](https://abseil.io/resources/swe-book/html/ch17.html); 2020년 일부 OSS(TensorFlow, Go, Bazel 등)에 공개 — [Google Open Source Blog 2020](https://opensource.googleblog.com/2020/04/code-search-for-google-open-source.html)
- **기술부채 시간 비용** (벤더 설문, Stripe Developer Coefficient 2018): 개발자는 주당 13.5시간을 기술부채 처리, 3.8시간을 나쁜 코드(디버깅·리팩터링)에 써서 합계 17.3시간 / 주 41.1시간 중. LSVP는 Stripe의 GDP 영향 추정 가정에 이의 제기 — [LSVP: Developer Productivity Manifesto pt.3](https://lsvp.com/stories/the-developer-productivity-manifesto-part-3-leaving-software-on-the-table/)
- **경제 규모** (업계 단체 보고서, CISQ 2022, Synopsys 후원): 미국 저품질 소프트웨어 비용 약 $2.41조(2022), 누적 기술부채 약 $1.52조, 운영 실패 $1.8조(2020년 $1.56조에서 증가) — [CISQ 2022 report](https://www.it-cisq.org/the-cost-of-poor-quality-software-in-the-us-a-2022-report/); [Synopsys 보도자료](https://news.synopsys.com/2022-12-06-Software-Quality-Issues-in-the-U-S-Cost-an-Estimated-2-41-Trillion-in-2022)
- **경험 많은 개발자 + 큰 저장소 + AI = 오히려 느려짐** (METR RCT, 2025-07, 프리프린트/연구기관): 평균 22k+ 스타, 1M+ LOC 대형 OSS 저장소의 숙련 개발자 16명, 실제 과제 246개를 AI 허용/금지로 무작위 배정. AI 허용 시 완료 시간 19% 증가. 개발자 사전 예측은 24% 단축, 사후 인식은 20% 단축. 한계: 표본 작음, 56%가 Cursor 처음 사용, 2025년 초 도구(Cursor + Claude 3.5/3.7) 스냅샷 — [METR blog](https://metr.org/blog/2025-07-10-early-2025-ai-experienced-os-dev-study/); [Simon Willison 요약](https://feeds.simonwillison.net/2025/Jul/12/ai-open-source-productivity/)

### Inferences
- E2(병목은 검증과 탐색)의 "탐색/이해" 측면은 인간 데이터로 강하게 뒷받침된다. 단 근거 수치는 2015–2018년 인간 연구다. 에이전트로 일반화하려면 Q2의 근거가 필요하다.
- Google 사례는 원칙 2(코드에서 결정론적으로 유도한 맵)와 같은 방향이다. 대규모에서는 IDE 아닌 **컴파일러 유도 전역 인덱스**가 필요했다. 다만 Kythe는 사후 인덱서이고 BMB는 언어 설계로 내장하려 한다는 차이가 있다.
- METR 결과는 익숙한 대형 저장소에서 AI 도구의 이득이 사라질 수 있음을 시사한다. 원인 분해(컨텍스트 부족, 검토 비용 등)는 이 노트에서 확인하지 못했다.

### Gaps
- Microsoft의 시간 사용 연구(예: 코딩 비중 수치)를 1차 출처로 확인하지 못했다.
- 결합도(coupling)·아키텍처 침식이 유지보수 비용에 주는 정량 효과(2023–2026)는 이번에 출처를 찾지 못했다.
- Google 모노레포 규모 수치("20억 줄", Potvin & Levenberg, CACM 2016)는 검색으로 확인하지 못했다. 모델 지식으로만 알고 있으니 인용 전에 대조가 필요하다.
- Meta의 Glean 등 코드 인덱싱 경험은 검색 결과가 없었다.
- onboarding 시간에 대한 2023–2026 정량 연구는 미확보.

## Q2. AI 에이전트: 저장소 규모·컨텍스트 길이에 따른 성능 저하

### Takeaway
함수/단일 이슈 수준에서 다파일·장기 과제로 가면 점수가 크게 떨어진다(SWE-bench Verified ~81% → Pro ~46%, FeatBench ~30%, NL2Repo <40%). 입력 길이가 늘면 같은 과제여도 성능이 떨어진다(Lost in the Middle, Chroma context rot, LongCodeBench 32K→256K에서 Claude 3.5 Sonnet 29%→3%). 다만 Verified–Pro 격차에는 오염(contamination)과 과제 난이도가 섞여 있어 "규모 자체" 효과로 분리되지 않는다.

### Cited Findings
- **SWE-Bench Pro** (Scale AI, arXiv 2509.16941, ICML 2026 채택 — PMLR v306): 41개 유지 중인 저장소의 1,865개 문제(public / held-out / commercial). 전문가가 수 시간~수 일 걸릴 과제. public 세트 참조 해법 평균 107줄, 4.1파일 변경. v1 기준 통일 scaffold에서 Pass@1 45% 미만. Scale 페이지(초기)는 상위가 GPT-5 23.3%(<25%)라고 표시해 버전·세트(public vs commercial)마다 수치가 다르다 — [arXiv](https://arxiv.org/pdf/2509.16941); [PMLR](https://proceedings.mlr.press/v306/deng26f.html); [Scale leaderboard](https://labs.scale.com/leaderboard/swe_bench_pro_public)
- **Verified vs Pro 격차** (2차 블로그, 2026): Claude Opus 4.5는 Verified 80.9%, Pro 45.9%(~35pt 차) [1차 미대조] — [Morph: SWE-Bench Pro leaderboard](https://www.morphllm.com/swe-bench-pro)
- **오염 요인** (OpenAI 자체 발표, 2026-02): SWE-bench Verified는 점점 오염되어 더 이상 평가에 쓰지 않고 Pro를 권장. 감사 결과 GPT-5.2, Claude Opus 4.5, Gemini 3 Flash가 일부 Verified 해법을 재현할 수 있었고, 어려운 과제 138개 중 59.4%가 테스트 설계 결함 [2차 요약 경유] — [OpenAI](https://openai.com/index/why-we-no-longer-evaluate-swe-bench-verified/); [AI Weekly 요약](https://aiweekly.co/alerts/openai-drops-swe-bench-verified-backs-pro-after-flaw-audit)
  - 반대 주의: 2026-05 Datacurve 감사가 Pro 채점기가 시행의 약 1/3을 잘못 채점했다고 주장한다는 2차 보도가 있다 [검증 못함] — [Morph](https://www.morphllm.com/swe-bench-pro)
- **FeatBench** (arXiv 2509.22237, 프리프린트): 27개 저장소에서 기능 구현 과제 157개, 코드 힌트 없는 자연어 요구만 입력. GPT-5 + Trae-agent 최고 Resolved 29.94%. 에이전트가 과잉 구현("scope creep")하고 기존 기능을 깨는 회귀가 광범위. 자율 에이전트(Trae)가 파이프라인(Agentless)보다 정확하지만 토큰 소비가 상당히 많다 — [arXiv](https://arxiv.org/html/2509.22237v2)
- **NL2Repo-Bench** (arXiv 2512.12730, 프리프린트): 저장소 전체 생성 과제에서 최강 에이전트도 평균 테스트 통과율 40% 미만, 저장소 전체를 올바르게 완성하는 경우는 드묾 — [arXiv](https://arxiv.org/html/2512.12730v1)
- **LongCodeBench** (Rando et al., arXiv 2505.07897, 프리프린트): 실제 GitHub 이슈 기반 LongCodeQA와 LongSWE-Bench. Claude 3.5 Sonnet의 LongSWE-Bench 정확도는 컨텍스트 32K→256K에서 29%→3%. Qwen2.5는 70.2%→40%. 반면 QA에서 Claude 3.5 Sonnet은 32K 76.5% → 256K 80.7%로 떨어지지 않음. 즉 **수정(bug fix) 과제는 길이에 민감, QA는 덜 민감** — [arXiv](https://arxiv.org/html/2505.07897v1)
- **Lost in the Middle** (Liu et al., TACL 2023/24, 동료심사): 다중 문서 QA와 key-value 검색에서 관련 정보가 입력 처음/끝에 있을 때 성능이 가장 높고 중간이면 떨어지는 U자형 곡선. 명시적 장문맥 모델도 입력이 길어지면 성능이 크게 하락. 20문서 설정에서 ~75%→~55%→~72% 수치는 2차 블로그 [1차 미대조] — [arXiv 2307.03172](https://arxiv.org/abs/2307.03172v3); [Liner 요약](https://liner.com/review/lost-in-middle-how-language-models-use-long-contexts)
- **Chroma "Context Rot"** (벤더 기술 보고서, 2025-07-14, Hong·Troynikov·Huber): GPT-4.1, Claude 4, Gemini 2.5, Qwen3 등 18개 모델. 과제 난이도를 고정하고 입력 길이만 늘려도 신뢰도가 하락했다(단순 검색·텍스트 복제 과제 포함). 질문과 needle의 의미적 유사도가 낮을수록, distractor가 있을수록(1개만 있어도) 저하가 빨라짐. 반복 단어 복사 과제도 길이에 따라 저하. 저자 결론은 컨텍스트를 최대화하지 말고 선별·압축하라는 "context engineering" [본문 미접근, 2차 요약] — [GitHub chroma-core/context-rot](https://github.com/chroma-core/context-rot); [ZenML 요약](https://www.zenml.io/llmops-database/context-rot-evaluating-llm-performance-degradation-with-increasing-input-tokens)
  - 2차 출처의 "10k→100k+에서 20–50% 하락" 같은 수치는 서로 달라 인용 비권장.
- **에이전트 궤적의 낭비** (프리프린트들):
  - Trae Agent의 SWE-bench Verified 궤적 100개를 수동 검토한 결과 거의 모든 궤적에 낭비 컨텍스트가 있었고, 첫 범주는 과제와 무관한 정보 — [arXiv 2509.23586 "Reducing Cost of LLM Agents with Trajectory Reduction"](https://arxiv.org/pdf/2509.23586)
  - OpenHands, SWE-agent, Prometheus 분석: 실패 실행이 성공보다 길고 변동이 크며, **실패 실행도 대부분 올바른 파일은 찾는다** [스니펫] — [arXiv 2511.00197 "Understanding Code Agent Behaviour"](https://arxiv.org/abs/2511.00197v1)
  - SWE-agent의 탐색 명령(search, find, open, goto, scroll, ls)을 행동 범주로 묶어 성공당 토큰 비용 ~313K(거버너 적용 전) [스니펫, 오프라인 재현] — [arXiv 2606.15579](https://arxiv.org/pdf/2606.15579)
  - SWE-agent·OpenHands 궤적 4,000개를 그래프로 인코딩: 강한 모델과 풍부한 프롬프트일수록 탐색·검증 패턴이 더 복잡 — [arXiv 2512.02393 (Graphectory)](https://arxiv.org/pdf/2512.02393v2)

### Inferences
- 원칙 5(슬라이스 예산)와 7(쿼리 범위에 비례하는 탐색 비용)은 "입력이 길수록 같은 과제도 나빠진다"(Liu 2023, Chroma 2025, LongCodeBench)는 근거와 부합한다. 특히 LongCodeBench에서 **수정 과제가 길이에 가장 민감**했다는 점이 BMB의 "정의 + 의존 계약만 담은 슬라이스" 방향을 지지한다.
- 그러나 2511.00197의 "실패 실행도 대부분 올바른 파일을 찾는다"는, SWE-bench 규모에서는 **파일 위치 찾기(navigation)보다 이해·수정·검증이 실패 원인**일 수 있음을 시사한다. E2의 두 축 중 "검증"이 더 무거울 수 있다. BMB 문서에서는 두 축을 분리해 측정해야 한다.
- FeatBench의 scope creep과 회귀는 "탐색"보다 **의도 범위 통제와 회귀 검증** 문제다. BMB의 증명 게이트가 회귀를 일부 잡을 수는 있지만, 과잉 구현 자체를 막지는 못한다.
- Verified–Pro 격차는 규모 효과, 과제 난이도(4.1파일, 107줄), 오염, scaffold가 섞여 있다. "규모가 커져서 떨어진다"는 인과로 쓰면 안 된다.

### Gaps
- 에이전트 궤적에서 "탐색/검색 단계 비율(%)"을 직접 보고한 1차 수치를 본문으로 확인하지 못했다(fetch 차단). 2511.00197, 2509.23586 본문 확인이 필요하다.
- RepoBench, SWE-bench Multimodal, SWE-Lancer 등 추가 벤치는 이번에 다루지 못했다.
- 저장소 크기(LOC)와 해결률의 상관을 직접 측정한 연구를 찾지 못했다. 이것이 원칙 7에 가장 직접적인 근거인데 공백이다.

## Q3. 효과가 있는 탐색·검색 접근 (repo map, 코드 그래프, LSP, RAG, 에이전트형 grep)

### Takeaway
코드 그래프/구조적 탐색 도구는 SWE-bench류에서 일관된 상대 개선을 보고하고(RepoGraph 평균 +32.8% 상대, LocAgent 파일 위치 92.7%, CodeCompass 숨은 의존 과제 99.4% vs 76.2%), Cursor는 semantic search + grep 조합이 grep 단독보다 질의응답 정확도를 평균 12.5% 높인다고 보고했다(벤더). 반대로 Claude Code는 벡터 RAG를 버리고 grep/파일 읽기 기반 에이전트형 검색을 택했고 근거는 "주로 vibes"라고 인정했다. 공통 교훈은 **도구가 있어도 에이전트가 쓰지 않으면 효과가 없다**는 것이다(CodeCompass 그래프 접근 시행의 58%가 도구 호출 0회).

### Cited Findings
- **RepoGraph** (arXiv 2410.14684, ICLR 2025 채택으로 알려짐 — 채택 여부 미대조): 줄 단위 저장소 코드 그래프 플러그인. SWE-bench에서 4개 방법(에이전트·절차형)에 붙여 평균 상대 성공률 +32.8%. SWE-agent에서는 `search_repo` 행동으로 추가 — [arXiv](https://arxiv.org/html/2410.14684v2)
- **LocAgent** (arXiv 2503.09089, 프리프린트/ACL 2025로 알려짐 — 미대조): 파일·클래스·함수를 import/호출/상속으로 연결한 이종 방향 그래프에서 멀티홉 탐색. 미세조정 Qwen-2.5-Coder-32B로 SWE-Bench-Lite 파일 단위 위치 정확도 최대 92.70%, 하류 이슈 해결 Pass@10 +12%. Python 한정 — [arXiv](https://arxiv.org/pdf/2503.09089v2); [Emergent Mind](https://www.emergentmind.com/topics/locagent)
- **CodeCompass** (arXiv 2602.20048, 2026 프리프린트): Claude Code에 의존 그래프를 노출하는 MCP 서버. 숨은 의존 과제에서 완료율 99.4% vs 기본 에이전트 76.2%, BM25 검색 78.2%. 그래프 접근이 있는 시행의 58%는 도구를 한 번도 호출하지 않아 명시적 프롬프트가 필요했다. 단일 FastAPI 저장소, 소규모 — [arXiv](https://arxiv.org/abs/2602.20048v1); [HF papers](https://huggingface.co/papers/2602.20048)
- **Aider repo map** (오픈소스 도구 설계, 정량 평가 아님): tree-sitter로 심볼 정의·참조 그래프를 만들고 PageRank류로 순위화한 뒤 토큰 예산(기본 `--map-tokens` 1k)에 맞게 이진 탐색으로 자름 [2차 출처, Aider 공식 문서 미대조] — [agentpatterns.ai: Repository Map Pattern](https://www.agentpatterns.ai/context-engineering/repository-map-pattern/); [fast.io Aider context](https://fast.io/resources/aider-context-window.md)
- **Cursor semantic search** (벤더 블로그, 2025): 자체 Cursor Context Bench에서 semantic search 추가 시 질의응답 정확도 평균 +12.5%(모델별 6.5–23.5%), 테스트한 모든 frontier 모델에서 이득. 온라인 A/B에서 코드 유지(retention) +0.3%, 대형 코드베이스(1,000+ 파일)에서 +2.6%(2차 요약 간 해석 차이 있음). 결론은 grep과 semantic search 조합이 최선 [본문 미대조] — [Cursor blog](https://cursor.com/blog/semsearch); [ZenML 요약](https://www.zenml.io/llmops-database/enhancing-ai-coding-agent-performance-with-custom-semantic-search)
- **Claude Code의 에이전트형 검색** (개발자 발언, 비공식): 초기 버전은 기성 임베딩 + 로컬 벡터 DB RAG를 썼으나 여러 방식을 시도한 뒤 agentic search(grep, 파일 읽기)로 전환. 근거는 "내부 벤치 일부, 주로 vibes"라고 인정. 2차 필자들은 단순성, 인덱스 신선도(staleness), 보안·프라이버시를 이유로 든다(Cherny 본인 발언으로 확인 안 됨) — [OfficeChai](https://officechai.com/ai/claude-researcher-explains-how-agentic-search-performed-better-than-rag-for-code-generation/); [SmartScope 정리](https://smartscope.blog/en/ai-development/practices/rag-debate-agentic-search-code-exploration/)
- 저장소 수준 RAG 서베이(arXiv 2510.04905)가 이 분야 접근을 정리한다(이번에 본문 미확인) — [arXiv](https://arxiv.org/pdf/2510.04905)
- Morph(semantic search 판매 벤더)는 grep + semantic 조합이 성공률 평균 +31%, 대형 코드베이스 +56%라고 주장한다. 이해상충이 있으므로 마케팅으로 취급 — [Morph blog](https://morphllm.com/blog/semantic-search)

### Inferences
- 구조적(그래프/심볼) 맵은 SWE-bench 수준에서도 측정 가능한 이득을 준다. BMB 원칙 2(L0–L2 결정론적 맵)는 이 계열과 같은 방향이며, Kythe처럼 **컴파일러가 보장하는 정확한 cross-reference**라는 점이 tree-sitter 휴리스틱(Aider, LocAgent)보다 강할 수 있다. 다만 아직 측정은 없다.
- CodeCompass의 "도구 미사용 58%"는 BMB에 중요한 교훈이다. 맵/슬라이스 도구를 제공하는 것과 **에이전트가 실제로 쓰게 만드는 것**은 별개이고, BMB가 "결정론적 도구로만 접근"을 강제하는 설계는 이 문제를 구조적으로 회피할 수 있다(가설, 측정 필요).
- 에이전트형 grep은 "인덱스 신선도·단순성"에서 장점이 있다. BMB 맵은 코드에서 결정론적으로 유도되므로 staleness를 빌드 시점에 제거할 수 있다는 점이 차별점이 될 수 있다.

### Gaps
- Sourcegraph Cody의 컨텍스트 검색 평가 수치는 검색 결과가 없었다.
- LSP 기반 도구(예: Serena 등 MCP LSP 서버)의 정량 평가는 찾지 못했다.
- 같은 과제에서 "grep 전용 vs 구조적 맵"의 토큰 비용을 직접 비교한 독립 연구는 미확보다. 원칙 7 검증에 필요한 실험이므로 BMB 자체 측정이 필요하다.

## Q4. AI 작성 코드가 대규모 유지보수성에 미치는 영향 (중복, churn, 아키텍처 표류)

### Takeaway
GitClear 2025 데이터(2억 1,100만 변경 라인)는 AI 보조 확산기에 중복 블록이 급증하고 이동(refactoring) 라인이 급감했다고 보고한다. 2024년은 복사·붙여넣기 라인이 이동 라인을 처음 넘은 해다. 원인으로 "제한된 컨텍스트 때문에 기존 함수 재사용을 제안하지 못함"이 지목된다. 이는 탐색 실패가 곧 아키텍처 침식으로 이어진다는 경로를 시사한다. 단 벤더 데이터이고 상관관계다.

### Cited Findings
- **GitClear "AI Copilot Code Quality 2025"** (벤더 연구): 2020-01~2024-12 비자동생성 파일의 2억 1,100만 변경 라인, 2/3는 GitClear 고객 기업의 비공개 코드이고 나머지는 OSS. 5줄 이상 중복 블록이 2024년에 8배 증가(2년 전보다 중복 빈도 10배). 이동 라인 39.9% 감소. 2024년 처음으로 복사·붙여넣기 라인이 이동 라인을 초과 — [DevClass 2025-02-20](https://devclass.com/2025/02/20/ai-is-eroding-code-quality-states-new-in-depth-report/); [LeadDev](https://leaddev.com/software-quality/how-ai-generated-code-accelerates-technical-debt); [원문](https://www.gitclear.com/ai_assistant_code_quality_2025_research)
  - 수치 불일치: 이동 비율을 2021→2024년 "25% → 10% 미만"이라고 하는 출처와 "15.88% → 3.10%"라고 하는 출처가 있다 — [I-Programmer](https://www.i-programmer.info/news/105-artificial-intelligence/17871-gitclear-reveals-ais-negative-impact-on-code-quality.html); [Wayne's Future](https://waynesfuture.substack.com/p/ai-and-code-quality-faster-more-churn)
  - GitClear의 해석: tab 키로 새 블록을 쉽게 삽입하게 만들고, 제한된 컨텍스트 창 때문에 기존 함수 재사용을 거의 제안하지 못한다 — [DevClass](https://devclass.com/2025/02/20/ai-is-eroding-code-quality-states-new-in-depth-report/)
- FeatBench: 에이전트의 scope creep과 기존 기능 파괴 회귀가 광범위 — [arXiv 2509.22237](https://arxiv.org/html/2509.22237v2)

### Inferences
- "재사용 실패 → 중복 → 수정 누락 결함"은 탐색 비용 문제가 유지보수 비용으로 전환되는 경로다. BMB가 이를 다루려면 맵 질의에 "기존 유사 정의 찾기"(중복 탐지, 계약 기반 검색)가 포함되어야 한다. 현재 원칙 2·7은 "찾기 비용"만 다루고 **중복 생성 억제**는 명시하지 않는다(BMB가 다루지 않는 페인포인트 후보).
- 아키텍처 표류(레이어 위반, 의존 방향 위반)는 슬라이스 예산(원칙 5)으로 정의 단위 크기를 묶어도 막히지 않는다. 모듈 간 의존 규칙을 증명 게이트처럼 강제하는 장치가 없으면 남는 페인포인트다.

### Gaps
- GitClear 이외의 독립(학술) 연구로 AI 코드 중복·churn을 측정한 2024–2026 결과는 이번에 확보하지 못했다.
- AI 작성 코드의 아키텍처 침식을 직접 측정한 연구는 찾지 못했다.

## Q5. 반대 근거와 BMB가 다루지 않는 페인포인트

### Takeaway
반대 근거는 세 가지다. (1) 장문맥 QA에서는 길이가 늘어도 성능이 유지되는 경우가 있다(LongCodeBench Claude QA 76.5%→80.7%). (2) Claude Code는 인덱스 없는 grep 기반 탐색으로 실무에서 성공적이다. (3) 실패 궤적도 대부분 올바른 파일을 찾는다. 따라서 "탐색이 병목"은 과제 유형(수정 vs 질의)과 규모에 따라 달라지며, 아직 저장소 크기와 성능의 인과를 직접 보인 연구는 없다.

### Cited Findings
- LongCodeBench QA: Claude 3.5 Sonnet 32K 76.5% → 256K 80.7%(하락 없음). 같은 모델의 버그 수정은 29%→3% — [arXiv 2505.07897](https://arxiv.org/html/2505.07897v1)
- 1M 토큰 MRCR v2에서 Opus 4.6 76%, Sonnet 4.5 18.5%라는 블로그 인용(세대 간 장문맥 개선 시사) [Anthropic 1차 출처 미대조] — [Unblocked blog](https://getunblocked.com/blog/context-rot-claude-code/)
- Claude Code의 agentic grep 채택과 "vibes" 근거 — [OfficeChai](https://officechai.com/ai/claude-researcher-explains-how-agentic-search-performed-better-than-rag-for-code-generation/)
- GrepRAG 벤치에서 grep이 구조적 검색에서 semantic 방식을 이긴다는 언급(2차, 원 논문 미확인) — [Morph semantic search 페이지 내 언급](https://morphllm.com/blog/semantic-search)
- 실패 실행도 대부분 올바른 파일을 찾음 — [arXiv 2511.00197](https://arxiv.org/abs/2511.00197v1)
- Google Code Search 사용 빈도가 AI 도구 도입 후에도 불변(2026 후속 연구 스니펫): 인간에게도 탐색 수요가 AI로 사라지지 않음 [1차 미대조]

### Inferences
- **BMB 가설과의 정합성 요약**:
  - E2(검증·탐색 병목): 인간 데이터(58% 이해, 하루 12회 검색)와 에이전트 데이터(장과제 점수 급락, 길이 민감도)가 지지한다. 그러나 에이전트 실패의 주원인이 "파일 찾기"보다 "이해·수정·검증"일 수 있어(2511.00197) **검증 축 비중이 더 클 가능성**이 있다.
  - 원칙 7(쿼리 범위 비례 비용): 방향은 context rot·lost-in-the-middle과 부합하지만, 저장소 크기와 에이전트 비용/성공률의 스케일링 곡선을 직접 측정한 연구는 없다. BMB가 자체 측정으로 채워야 할 공백이다.
  - 원칙 5(슬라이스 예산): 수정 과제가 길이에 가장 민감하다는 LongCodeBench 결과가 가장 직접적인 지지 근거다.
  - 원칙 2(결정론적 맵): RepoGraph/LocAgent/CodeCompass/Kythe가 방향을 지지한다. 단 "도구 미사용" 문제(CodeCompass 58%)와 grep 대비 우위의 불확실성(Claude Code)이 남는다.
- **BMB가 다루지 않는 대규모 페인포인트 후보**:
  1. 중복 생성·재사용 실패(GitClear): 탐색이 싸도 "찾아보지 않고 새로 쓰는" 행동은 별도 억제가 필요하다.
  2. scope creep·회귀(FeatBench): 의도 범위 통제, 기존 동작 보존 검증.
  3. 아키텍처 표류·모듈 경계 위반: 정의 단위 예산으로는 막히지 않는다.
  4. 벤치·평가 오염(Verified 오염): BMB 벤치 동결 원칙과 관련된 측정 위험.
  5. 사람의 검토 비용과 인식 착오(METR: 느려졌는데 빨라졌다고 인식): 인간 검토자가 루프에 있을 때의 비용.
  6. 비코드 지식(이슈, 설계 문서, 런타임 동작): Xia et al.에서 이해 활동의 상당 부분이 IDE 밖(브라우저, 문서)에서 일어났다. 코드 유도 맵만으로는 덮이지 않는다.

### Gaps
- 장문맥 모델 개선(2026년 1M 컨텍스트 모델)이 저장소 수준 수정 과제에서 탐색 필요를 실제로 줄이는지 보여주는 독립 측정은 찾지 못했다.
- "agentic grep으로 충분하다"를 정량적으로 입증한 독립 연구도, 반증한 독립 연구도 확보하지 못했다(벤더 주장만 양쪽에 있음).
