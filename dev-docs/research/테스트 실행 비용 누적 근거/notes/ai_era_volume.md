# AI 코딩 시대(2023–2026)의 코드·커밋·PR·CI·배포 볼륨 변화와 테스트/CI 실행 부하

> 조사 방법 메모: 이 세션에서 WebFetch는 모든 호스트에서 DNS 실패(ENOTFOUND)했고, curl도 프록시 403으로 막혔다. 원문 페이지를 열지 못했다. 아래 수치는 모두 **WebSearch 결과 스니펫/요약에서 얻은 것(snippet-only)**이다. 1차 출처 URL이 검색 결과에 나온 경우에는 그 URL을 달았지만 본문 대조는 하지 못했다. 인용하기 전에 원문을 확인해야 한다.
> 출처 유형 표기: [벤더] = 플랫폼/도구 회사 자체 데이터, [2차] = 언론·블로그 재인용, [동료심사] = peer-reviewed. **이 주제에서 동료심사 출처는 찾지 못했다.** 모든 수치가 벤더 텔레메트리이거나 그 2차 인용이다.

## 1. GitHub 플랫폼 볼륨(Octoverse 2024/2025, Actions 분, 에이전트 PR)

### Takeaway
GitHub 전체 커밋·PR·Actions 사용량은 2024→2025에 연 25–35% 늘었다. 2026년 상반기에는 에이전트 때문에 폭증했다고 GitHub 경영진이 직접 말했다. 커밋은 주당 약 2.75억 건으로 2025년 연간의 약 14배 페이스이고, Actions는 주당 21억 분으로 2023년의 4배 이상이다. 소유자 주장("코드량·커밋 빈도 급증 → 빌드당 CI 실행 증가")을 가장 직접 뒷받침하는 외부 근거다.

### Cited Findings
- [벤더, snippet-only] Octoverse 2025: 2025년 푸시된 커밋 약 10억 건(+25.1% YoY). 8월 한 달에만 약 1억 건. 월평균 PR 머지 4,320만 건(+23% YoY). 신규 저장소 1.21억 개 이상. 개발자 1.8억 명 이상 — [Octoverse: A new developer joins GitHub every second as AI leads TypeScript to #1 (GitHub blog, 2025-10-28)](https://github.blog/news-insights/octoverse/octoverse-a-new-developer-joins-github-every-second-as-ai-leads-typescript-to-1/). 제3자 표는 커밋을 986M+로 적는다 — [What 986 million code pushes say about the developer workflow in 2025 (GitHub blog tag page)](https://github.blog/tag/developer-workflows)
- [2차] Octoverse 2025 연간 머지 PR 합계 5.187억 건(월 4,320만 건) — [humai.blog](https://www.humai.blog/github-hits-nearly-1-billion-commits-in-a-year-ai-is-rewriting-the-essence-of-software-development/). 1차 출처로 대조하지 못했다.
- [벤더→2차, snippet-only] GitHub Actions: 2025년 사용량 115억 분(2024년 대비 +35%). 백엔드 재설계 후 하루 7,100만 잡을 처리한다 — GitHub 블로그 "Let's talk about GitHub Actions"(Ben De St Paer-Gotch) 요약: [plushcap](https://www.plushcap.com/content/github/blog/github-lets-talk-about-github-actions), [Blockchain.News (2025-12)](https://blockchain.news/zh/news/github-actions-revamped-performance-flexibility-upgrades). Tenki 블로그는 이 수치를 "Octoverse 2025, 공개 프로젝트 Actions 분"으로 인용한다 — [Tenki: The Hidden CI Tax of AI Coding Agents](https://tenki.cloud/blog/hidden-ci-tax-ai-coding-agents). **출처 범위 불일치**: 공개 프로젝트만인지 전체인지 확인하지 못했다.
- [2차, snippet-only] 비교 기준: Octoverse 2023의 공개 프로젝트 호스티드 Actions는 73억 분(하루 2,000만 분 이상) — [FOSS United forum: Opinionated comments on the Octoverse 2023 report](https://forum.fossunited.org/t/opinionated-comments-on-the-octoverse-2023-report/2612). 이 기준을 쓰면 2023→2025 증가율은 약 +58%다(73억→115억). 단, 두 수치의 측정 범위가 같은지는 확인하지 못했다.
- [벤더 경영진 발언→2차] GitHub COO Kyle Daigle(2026-04-03~04): 커밋 주당 약 2.75억 건(2025년 연간 약 10억 건). Actions는 2023년 주당 약 5억 분 → 2025년 주당 약 10억 분 → "이번 주 지금까지" 21억 분 — [Simon Willison, 2026-04-04](https://simonwillison.net/2026/Apr/4/kyle-daigle/); [HN 재게시](https://news.ycombinator.com/threads?id=eddyg). 21억 분은 주중 집계라 전체 주 수치와 바로 비교할 수 없다.
- [2차] 2026년 6월 Latent Space 팟캐스트 보도는 같은 추세를 "커밋 약 14배 증가, 코딩 에이전트가 원인"으로 요약했다 — [drss.io (Latent Space: GitHub's plan for Agents — Kyle Daigle)](https://drss.io/reader/npub1ntf7j0dy6uep8u0dxl0je8jatxqj59f4srv8gt9vcujaxxga6xaqfzhv2c/GitHub%27s-plan-for-Agents-%E2%80%94-Kyle-Daigle%2C-GitHub). "2026년 140억 커밋 전망"이라는 헤드라인도 있다 — [quasa.io](https://quasa.io/media/github-s-ai-agent-tsunami-275-million-commits-a-week-14-billion-projected-for-2026-and-the-platform-is-starting-to-crack).
- [2차, The Information 보도 기반] 에이전트가 연 PR: 2025-09 약 400만 건 → 2026-03 1,700만 건 이상(6개월에 약 4배) — [36Kr](https://eu.36kr.com/en/p/3838634999417353), [AI2Work: GitHub Agentic Pull Requests Surge 28-Fold in Ten Months](https://ai2.work/blog/github-agentic-pull-requests-surge-28-fold-in-ten-months). Daigle도 팟캐스트에서 "3월에 에이전트가 만든 PR 1,700만 건"이라고 말했다 — [AI & I podcast: GitHub's COO Explains Why AI Hasn't Replaced Developers](https://www.podscan.fm/podcasts/ai-and-i/episodes/githubs-coo-explains-why-ai-hasnt-replaced-developers-1). 400만과 1,700만 수치는 같은 계열 출처라 서로 독립된 확인이 아니다.
- [벤더→2차] GitHub CTO Vlad Fedorov의 가용성 업데이트(2026년 4월 말): 2025년 10월에 용량 10배 증설 계획을 시작했으나 2026년 2월에 "현재 규모의 약 30배"를 설계 목표로 바꿨다. 원인은 AI 에이전트 트래픽이다 — [Neowin](https://www.neowin.net/news/github-pivots-to-availability-first-as-ai-agent-surge-triggers-reliability-crisis/), [Datasaur: GitHub's 30X wake-up call](https://datasaur.ai/blog/githubs-30x-wake-up-call-the-agent-compute-tsunami-is-coming)
- [2차, 미확인] The Register 집계: 2026년 4월 장애 10건, 5월 9건. 6월 가용성 약 88.4%. Microsoft가 GitHub 오버플로를 위해 AWS 용량을 추가했다는 Business Insider 보도가 있으나 확인되지 않았다 — [aiweekly](https://aiweekly.co/alerts/microsoft-taps-aws-to-keep-github-running-amid-ai-surge), [letsdatascience](https://letsdatascience.com/news/github-capacity-surge-pushes-microsoft-to-aws-13a2ffa4)
- [3자 추적기, 2025-08 스냅샷] Copilot coding agent: PR 93,263건 중 머지 56,706건(머지율 61%) — [Tembo AI Coding Agents leaderboard](https://tembo.io/leaderboard). 에이전트별 PR 추적은 [PRarena](https://github.com/aavetis/PRarena)와 [Logicstar Agents in the Wild](https://insights.logicstar.ai)에도 있다(스니펫 표가 깨져 수치를 읽지 못했다).

### Inferences
- 커밋 연 10억 건(2025) → 주 2.75억 건(2026-04) 페이스는 연환산 약 140억 건이다. Actions 분 증가(2023 주 5억 → 2026 주 21억, 약 4배)는 커밋 증가(약 14배 페이스)보다 훨씬 느리다. 따라서 "커밋당 CI 분"은 오히려 줄었을 가능성이 있다. 에이전트 커밋이 잘게 쪼개졌거나, 공개 저장소·CI 없는 저장소에 몰렸거나, 경로 필터·캐시를 썼을 수 있다. 소유자 주장은 "총 CI 부하는 수 배 늘었다"로 쓰는 편이 안전하다. "커밋 수에 비례해 늘었다"고 쓰면 과장이다.
- GitHub이 용량 목표를 10배에서 30배로 올린 것은 에이전트 트래픽이 플랫폼 계획을 넘어섰다는 정성적 근거다. 다만 그 안에서 CI(Actions)가 차지하는 몫은 공개되지 않았다.

### Gaps
- Octoverse 2024의 커밋·PR·Actions 분 공식 수치는 확인하지 못했다(octoverse.github.com 본문을 열지 못함). "2024년 52억 기여" 수치는 저품질 블로그 단일 출처라 제외했다.
- Copilot coding agent의 공식 누적 PR 수와 PR당 CI 실행 횟수는 GitHub 1차 출처에서 찾지 못했다.
- Devin, OpenAI Codex cloud, Claude Code GitHub Action의 PR 수와 그 PR이 유발한 CI 실행 수: 신뢰할 만한 수치를 찾지 못했다.

## 2. CI 벤더 리포트(CircleCI 등): 파이프라인 볼륨·성공률·소요 시간

### Takeaway
CircleCI 2026 리포트(2,800만 개 이상 워크플로)는 일일 워크플로 실행이 평균 +59% YoY라고 보고한다. 이는 "CI 실행량 급증"의 가장 강한 벤더 근거다. 그러나 증가분은 대부분 기능 브랜치에서 나왔고, 중앙값 팀의 main 브랜치 처리량은 -7%였다. main 성공률은 70.8%로 5년 만의 최저다. 즉 CI 실행은 늘었는데 배포로 이어지지 않았다. 실패와 재실행이 늘어난 것과도 맞아떨어진다.

### Cited Findings
- [벤더] CircleCI 2026 State of Software Delivery: 2,800만 개 이상 CI 워크플로를 분석했다. 평균 처리량(하루 워크플로 실행 수)은 +59% YoY — [CircleCI 2026 report PDF](https://circleci.com/landing-pages/assets/2026-state-of-software-delivery-report.pdf); [CB Insights 보도자료 발췌](https://www.cbinsights.com/investor/circleci); [Thoughtworks perspective](https://www.thoughtworks.com/insights/blog/generative-ai/a-thoughtworks-perspective-on-circleci-s-2026-state-of-software-)
- [벤더→2차] 분포 편중: 상위 5% 팀은 처리량 +97%, 중앙값 팀 +4%, 하위 사분위는 증가 없음 — [quashbugs CI/CD Statistics (2026)](https://quashbugs.com/blog/ci-cd-statistics)
- [벤더] 브랜치별: 중앙값 팀은 기능 브랜치 처리량 +15%, main -7%. 상위 10%는 기능 브랜치 약 +50%, main은 정체(CircleCI 블로그는 +1%). p95는 main +26%, 기능 브랜치 +85%. CircleCI의 결론은 "AI 가속을 파이프라인 둔화 없이 흡수하는 팀은 20개 중 1개 미만" — [CircleCI blog: Five takeaways (2026-02-18)](https://circleci.com/blog/five-takeaways-2026-software-delivery-report)
- [벤더] main 브랜치 성공률 70.8%, 5년 이상 만의 최저(권장 기준은 90%) — 같은 출처 및 [Rob Bowley 블로그 (2026-04-02)](https://blog.robbowley.net/2026/04/02/more-code-less-delivery-but-does-the-circleci-2026-report-really-show-1-in-20-teams-are-benefiting/)
- [2차] 실패 후 green 복구에 걸리는 시간은 보통 팀 기준 72분 — [quashbugs](https://quashbugs.com/blog/ci-cd-statistics). 2026년 워크플로 소요 시간(duration) 수치는 찾지 못했다.
- [벤더] CircleCI 2026 Q2 리포트: p95 팀의 처리량은 중앙값의 8배(Q1)에서 9배(Q2)로 벌어졌다 — [CircleCI 2026 Q2 report PDF](https://circleci.com/landing-pages/assets/2026-q2-state-of-software-delivery-report.pdf)
- [벤더] CircleCI 2025 리포트(기준선): 2024년 9월 첫 28일 동안 2.2만 개 이상 조직의 워크플로 약 1,500만 개를 분석했다. 상위 25% 팀은 하위 사분위보다 3배 빨리 배포하고 핵심 워크플로를 5배 빨리 끝낸다 — [CircleCI newsroom (2025-03)](https://circleci.com/newsroom/circleci-publishes-2025-state-of-software-delivery-report), [BusinessWire](https://businesswire.com/news/home/20250318720666/en/CircleCI-Publishes-2025-State-of-Software-Delivery-Report)

### Inferences
- 표본 크기만 봐도 약 1,500만 워크플로(2025)에서 2,800만 이상(2026)으로 커졌다. 다만 이것은 고객 기반 변화일 수 있어 볼륨 증가의 근거로 쓰면 안 된다. 근거로 쓸 수 있는 것은 "팀당 일일 워크플로 +59%"다.
- main 성공률 하락(70.8%)은 실패한 빌드를 다시 돌려야 한다는 뜻이다. "실행당 테스트 비용 × 재실행 횟수"가 함께 늘어난다. BMB의 "빌드마다 반복되는 테스트 비용" 논거와 맞는다.

### Gaps
- Buildkite, Harness, GitLab DevSecOps 설문, Datadog CI Visibility에서 2024–2026 파이프라인 볼륨·소요 시간 추세 수치를 찾지 못했다(검색 예산 소진, 원문 접근 불가).
- CircleCI 2026의 워크플로 소요 시간 중앙값 추세는 확인하지 못했다.

## 3. 에이전트 PR이 유발하는 CI 부하와 비용, 플랫폼의 가격·제한 대응

### Takeaway
대규모 에이전트 운영 사례(Stripe Minions, 주당 1,300개 이상 PR)는 CI 비용 때문에 테스트를 선택 실행하고 CI 라운드 수를 제한한다. GitHub은 2025–2026년에 Actions 과금 구조를 바꿨다. 셀프호스티드 러너 과금은 추진했다가 연기했고, Copilot 코드 리뷰에는 Actions 분 소비를 붙였다. "에이전트 → CI 비용 증가"가 실제 운영·가격 문제가 되었다는 정황 근거다.

### Cited Findings
- [2차, Stripe 1차 원문 미확인] Stripe "Minions": 주당 머지 PR이 1,000건에서 1,300건 이상으로 늘었다. 모든 PR은 사람이 리뷰하고 코드는 전부 에이전트가 썼다 — [ByteByteGo: How Stripe's Minions Ship 1,300 PRs a Week](https://blog.bytebytego.com/p/how-stripes-minions-ship-1300-prs), [Port: Inside the Software Factory](https://software-factories.port.io/stripe)
- [2차] Stripe는 300만 개 이상 테스트 중 변경 파일과 관련된 부분집합만 CI에서 선택 실행한다. 작업당 CI 라운드는 최대 2회이고, 그 뒤에는 사람에게 넘긴다. 인용된 이유는 "CI runs cost tokens, compute, and time"이다. 로컬 린트는 5초 미만 — [dotzlaw: Stripe minions deterministic rails](https://dotzlaw.com/insights/ai-04-stripe-minions-deterministic-rails/), [mintmcp: Stripe minions explained](https://www.mintmcp.com/blog/stripe-minions-explained), [jangwook.net](https://jangwook.net/en/blog/en/stripe-minions-autonomous-coding-agents-1300-prs/)
- [벤더 블로그, CI 러너 판매사라 이해상충 있음] Tenki는 에이전트 PR 규모에서는 병목이 에이전트 주변의 CI 러너, 테스트 큐, 머지 충돌, 리뷰 대역폭에 있다고 주장한다 — [Tenki: What Stripe's 1,300 Agent PRs Per Week Reveal About CI at Scale](https://tenki.cloud/blog/agent-pr-volume-ci-scale). 같은 회사의 "The Hidden CI Tax of AI Coding Agents"(2026-04)는 본문을 확보하지 못했다 — [Tenki](https://tenki.cloud/blog/hidden-ci-tax-ai-coding-agents)
- [벤더 공지→2차] GitHub Actions 2026 가격 변경(2025-12 발표): 2026-01-01부터 호스티드 러너 순가격을 최대 39% 인하했다. 2026-03-01부터는 셀프호스티드 러너에 분당 $0.002 "클라우드 플랫폼 요금"을 부과할 예정이었다(공개 저장소는 무료). GitHub이 밝힌 이유는 Actions 컨트롤 플레인 운영 비용, 그리고 "aligns pricing to match consumption and our cost patterns … as usage grows"(Jared Palmer)다. 반발이 커지자 셀프호스티드 과금은 연기됐다 — [GitHub Resources: Pricing changes for GitHub Actions](https://resources.github.com/actions/2026-pricing-changes-for-github-actions/), [DevClass (2025-12-17)](https://devclass.com/2025/12/17/github-to-charge-for-self-hosted-runners-from-march-2026/), [Techzine: GitHub bends to criticism and delays](https://www.techzine.eu/news/devops/137396/github-bends-to-criticism-and-delays-paid-self-hosting-of-runners/), [IT Brief UK](https://itbrief.co.uk/story/github-cuts-actions-runner-prices-adds-new-usage-fee). 발표문에 AI 에이전트를 직접 원인으로 든 문구는 스니펫에서 확인되지 않았다. 셀프호스티드 과금의 현재 상태(재추진 여부)도 확인하지 못했다.
- [벤더, 1차 changelog] 2026-06-01부터 비공개 저장소의 Copilot 코드 리뷰가 GitHub Actions 분을 소비한다(플랜 포함분에서 차감, 초과분은 표준 Actions 요금). AI 크레딧도 따로 과금된다(두 개의 미터) — [GitHub changelog 2026-04-27](https://github.blog/changelog/2026-04-27-github-copilot-code-review-will-start-consuming-github-actions-minutes-on-june-1-2026/). Copilot은 2026-06-01에 사용량 기반 과금으로 바뀌었다 — [Startup Fortune](https://startupfortune.com/?p=14486)
- [2차, 미확인] 2026년 8월 changelog: 리뷰 2만 줄 제한이 없어지고 봇이 만든 PR도 리뷰할 수 있게 됐다. 2026-09-28부터 기본 노력 수준이 Lite에서 Balanced로 바뀐다 — [startdebugging.net](https://startdebugging.net/es/2026/08/copilot-code-review-defaults-to-balanced-on-september-28/)

### Inferences
- Stripe 사례는 BMB 가설과 직접 이어진다. 테스트 300만 개를 에이전트 PR마다 전부 돌릴 수 없어서 (a) 선택 실행과 (b) CI 라운드 상한이라는 공학적 우회책을 만들었다. "반복 테스트 실행 비용이 에이전트 볼륨에 비례해 병목이 된다"는 주장의 실무 사례로 쓸 수 있다. 다만 원문이 아닌 2차 요약이다.
- 셀프호스티드 과금 추진 자체가 "사용량 증가 → 컨트롤 플레인 비용 증가"를 GitHub이 인정한 것이다. 그러나 AI를 원인으로 명시했는지는 확인하지 못했으므로 그렇게 쓰면 안 된다.

### Gaps
- "에이전트 CI 사용을 제한하는 회사" 사례로는 Stripe의 라운드 상한 외에 공개된 정량 사례를 찾지 못했다.
- Copilot coding agent가 GitHub Actions 위에서 실행되며 Actions 분을 소비한다는 점은 GitHub 문서로 알려져 있다. 그러나 이번 검색에서 1차 출처로 확인하지 못했다. 인용 전에 docs.github.com을 확인해야 한다.
- 에이전트 PR 1건이 유발하는 평균 CI 실행 수나 분: 공개 수치를 찾지 못했다.

## 4. 텔레메트리 연구(Faros, LinearB, DORA 등): PR 수·크기와 배포 빈도

### Takeaway
AI 도입 팀에서는 PR 수(+98%)와 PR 크기(+154%)가 크게 늘었다. 그러나 조직 수준의 배포·전달 성과는 거의 늘지 않았다(Faros, DORA). 테스트하고 리뷰할 변경량은 늘었지만 배포 빈도는 그만큼 늘지 않았다는 뜻이다.

### Cited Findings
- [벤더, 2차 재인용만 확보] Faros AI "AI Productivity Paradox"(2025, 1,255개 팀, 개발자 1만 명 이상): AI 도입률이 높은 팀은 머지 PR +98%, 완료 태스크 +21%, PR 리뷰 시간 +91%, PR 크기 +154%, 개발자당 버그 +9%. 회사 수준 전달 성과와의 상관은 거의 0 — [getunblocked: AI productivity paradox](https://getunblocked.com/blog/ai-productivity-paradox/), [DEV: AI code review is the new bottleneck](https://dev.to/alexcloudstar/ai-code-review-is-the-new-bottleneck-why-faster-code-is-not-reaching-production-faster-416a)
- [벤더, 2차, 미확인] Faros 2026 "Acceleration Whiplash"(2년 텔레메트리, 2.2만 명, 4,000개 이상 팀): 중앙값 PR 리뷰 시간 5배, 리뷰 없이 머지된 PR +31% — [DEV Community](https://dev.to/kunal_d6a8fea2309e1571ee7/ai-coding-assistant-team-adoption-what-breaks-after-everyone-gets-a-license-2026-179i). 수치가 2025 리포트와 달라 1차 대조가 필요하다.
- [벤더, 2차] LinearB 2026 벤치마크(PR 810만 건, 4,800개 팀, 42개국): AI 보조 PR이 비보조 PR보다 훨씬 크다. 머지율은 수동 PR 84.5%, AI 생성 PR 32.7% — 같은 출처(getunblocked/DEV 요약)
- [설문, Google DORA 2025 — 동료심사 아님, 2차 요약만 확보] AI 도입은 2024년과 달리 처리량(throughput) 저하와 더는 상관이 없다. 오히려 처리량 개선과 연관된다. 그러나 전달 불안정성(instability)은 남아 있다. AI는 "증폭기"로 작동한다 — [TechTarget: Google DORA: Software delivery caught up to AI coding tools](https://www.techtarget.com/searchsoftwarequality/news/366631712/Google-DORA-Software-delivery-caught-up-to-AI-coding-tools), [InfoQ (2025-09)](https://www.infoq.com/news/2025/09/dora-state-of-ai-in-dev-2025), [RDEL #112](https://rdel.substack.com/p/rdel-112-whats-ais-impact-on-software)
- [설문, DORA 2024 기준선] AI 도입 25% 증가당 전달 처리량 -1.5%, 안정성 -7.2% — [RDEL #112](https://rdel.substack.com/p/rdel-112-whats-ais-impact-on-software), [cusy.io](https://cusy.io/en/blog/dora-report-2025.html)
- [2차] "개인은 PR을 98% 더 머지했으나 조직 전달 성과는 거의 평평했다"는 요약이 DORA에 귀속되어 돌지만, 실제로는 Faros 수치로 보인다. 출처가 혼동된 것이다 — [Typo](https://typoapp.io/blog/ai-impact-dora-metrics)

### Inferences
- PR 크기 +154% × PR 수 +98%면 리뷰·테스트 대상 변경량은 대략 몇 배 수준으로 늘었을 수 있다. 다만 단순히 곱하는 것은 추정일 뿐이다. 배포 빈도가 정체된 상태에서 PR 단위 CI(머지 전 검증)는 PR 수에 비례해 늘어난다. 따라서 CI 부하는 "배포 횟수"가 아니라 "PR·푸시 횟수"를 따라간다. BMB 논거는 "deploy마다"보다 "PR·push마다 반복되는 테스트"로 쓰는 편이 증거와 더 잘 맞는다.

### Gaps
- Jellyfish, DX의 PR 볼륨 수치는 이번 조사에서 확보하지 못했다.
- DORA 2025의 배포 빈도 분포 변화 수치(예: 일일 배포 비율)는 확보하지 못했다.

## 5. 코드량·churn(GitClear)과 반대 증거 정리

### Takeaway
GitClear(2억 1,100만 변경 라인, 2020–2024)는 AI 시대에 복제 코드와 churn이 늘고 리팩터링이 줄었다고 보고한다. "테스트할 코드가 더 많아지고, 같은 코드를 더 자주 다시 고친다"는 주장의 간접 근거다. 반대 증거는 일관된다. 볼륨(커밋, PR, 기능 브랜치 CI)은 늘었으나 main 브랜치 처리량과 배포·조직 성과는 정체하거나 감소했다.

### Cited Findings
- [벤더 연구, 2차] GitClear "AI Copilot Code Quality"(2025): 2020-01~2024-12의 비자동생성 변경 라인 2억 1,100만 개를 분석했다(2/3 기업, 1/3 오픈소스). 2024년에 인접 코드와 중복되는 5줄 이상 블록의 빈도가 8배로 늘었다. 2024년은 반복 코드 도입이 리팩터링을 처음 앞선 해다. "moved" 라인은 15.88%에서 3.10%로, "copy/pasted"는 10.64%에서 13.90%로 바뀌었다 — [LeadDev: How AI-generated code accelerates technical debt](https://leaddev.com/software-quality/how-ai-generated-code-accelerates-technical-debt), [i-programmer](https://www.i-programmer.info/news/105-artificial-intelligence/17871-gitclear-reveals-ais-negative-impact-on-code-quality.html)
- [2차, 단일 출처] churn(2주 안에 다시 고친 라인)이 2021년 3.31%에서 2024년 6.87%로 늘었다 — [waynesfuture substack](https://waynesfuture.substack.com/p/ai-and-code-quality-faster-more-churn). 2024년 판(2020–2023, 1억 5,300만 라인) 수치와 섞지 말 것 — [AnalyticsVidhya (2024-01)](https://www.analyticsvidhya.com/blog/2024/01/ai-coding-assistants-produce-bad-quality-code-study/)
- [반대 증거, 벤더] CircleCI 2026: 중앙값 팀의 main 처리량 -7%, main 성공률 70.8%. 증가분은 대부분 기능 브랜치에서 나왔다 — [CircleCI blog](https://circleci.com/blog/five-takeaways-2026-software-delivery-report)
- [반대 증거, 2차 비평] Rob Bowley: 기능 브랜치 처리량은 사실상 의미 없는 지표이고 main 기준만 의미 있다. 59% 헤드라인은 기능 브랜치가 섞여 부풀었다 — [Rob Bowley (2026-04-02)](https://blog.robbowley.net/2026/04/02/more-code-less-delivery-but-does-the-circleci-2026-report-really-show-1-in-20-teams-are-benefiting/)
- [반대 증거, 벤더] Faros: PR +98%인데 회사 수준 성과 상관은 0에 가깝다. DORA 2025: 불안정성이 지속된다 — 4절 출처 참조
- [반대 증거, 벤더] CircleCI 분포: 중앙값 팀 처리량은 +4%에 그쳤다. 하위 사분위는 증가가 없었다 — [quashbugs](https://quashbugs.com/blog/ci-cd-statistics)

### Inferences
- 반대 증거는 "배포가 많아진다"는 주장을 약하게 만들지만, "CI 실행이 많아진다"는 주장을 약하게 만들지는 않는다. 기능 브랜치 CI 증가, 실패율 상승에 따른 재실행, 리뷰 대기 중 rebase와 재실행은 모두 배포 없이도 테스트 실행 수를 늘린다. 소유자 주장은 "배포 빈도"보다 "검증 실행 빈도(PR·push·재시도)"를 근거로 삼아야 증거와 맞는다.
- 증가 폭은 상위 팀과 대형 플랫폼(GitHub 전체, Stripe)에 몰려 있다. 중앙값 팀은 소폭(+4%)이다. 따라서 "모든 팀에서 곱으로 늘어난다"는 일반화는 근거가 약하다. "에이전트를 많이 쓰는 조직에서 급증한다"가 근거에 맞는 표현이다.

### Gaps
- 동료심사 연구 중 AI 도입과 CI 실행량·테스트 실행 비용을 직접 잇는 것은 찾지 못했다.
- GitClear 2025 이후(2025–2026) 판의 churn 수치는 확인하지 못했다.
