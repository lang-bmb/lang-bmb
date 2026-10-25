# 병목은 테스트에서 명세로 옮겨가고 있다

외부 근거는 BMB가 겨냥하는 방향은 지지하지만, 지금 문서에 적힌 이유는 지지하지 않는다. 가장 강한 근거는 "테스트 통과는 정답을 뜻하지 않는다"는 동료심사 결과다. SWE-bench Verified에서 테스트를 통과한 패치의 15.7%가 틀렸고(ACL 2025), 그럴듯한 패치의 29.6%가 정답 패치와 다르게 동작했다(ICSE 2026). 반면 "테스트를 쌓는 데 시간과 비용이 지배적으로 든다"는 주장에는 최근 1차 측정이 없다. 측정된 플레이키 수리 비용은 개발 시간의 한 자릿수 %(1.28%)다. 규모가 커질수록 관리가 어렵다는 가설(2)과 AI의 한계 가설(3)은 방향이 맞다. 인간은 시간의 약 58%를 코드 이해에 쓰고, AI를 많이 쓰는 팀은 PR 리뷰 시간이 91% 늘었으며, 입력이 길어지면 같은 과제에서도 모델 성능이 떨어진다. 다만 에이전트가 실패하는 주된 이유는 파일을 못 찾아서가 아니라 고치기와 검증에 있다는 신호가 있다. 협업 방향 가설(4)은 반만 맞다. 코드 작성은 빠르게 AI로 넘어가고 있지만, 인간이 코드 읽기를 그만뒀다는 증거는 없다. 대신 인간의 역할이 의도와 명세를 검토하는 쪽으로 옮겨가는 흐름이 뚜렷하다. 가장 중요한 새 페인포인트는 **명세 타당성**이다. 증명이 자동화될수록 병목은 "명세가 의도와 맞는가"로 옮겨간다. 통과한 검증 해답 중 명세가 의도와 맞은 비율은 평균 43%에 그쳤다. 여기에 검증기 게이밍, SMT 증명 불안정성, 코드 밖에서 나는 장애, 학습 데이터가 없는 언어의 비용, 기존 언어에 도구를 더하는 경쟁 경로가 BMB 문서에 아직 가설로 올라 있지 않다. 근거의 상당 부분은 벤더 텔레메트리와 설문이고 상관관계다. 또 이 조사 환경에서는 arxiv.org, martin.kleppmann.com, martinfowler.com, cursor.com, trychroma.com, Google TAP PDF 같은 1차 원문 다수가 DNS 실패나 프록시 거부로 열리지 않았다. 아래에서 "(원문 미확인)"으로 표시한 수치는 검색 요약이나 2차 보도에서 온 것이니, 문서에 인용하기 전에 원문과 대조해야 한다.

> **근거 등급 표기:** [PR] 동료심사 논문, [pre] 프리프린트(arXiv 등) 또는 연구기관 보고, [V] 벤더·이해관계자 데이터, [S] 대규모 설문, [O] 의견·에세이, [N] 언론 2차 보도. "(원문 미확인)"은 1차 문서를 열지 못해 검색 요약이나 2차 인용에만 기댄 수치다.

## 가설별 판정: 넷 모두 방향은 맞지만 근거는 다른 곳에 있다

| 가설 | 판정 | 가장 강한 지지 | 가장 강한 반대 |
|---|---|---|---|
| (1) 테스트 시간·비용 | **원인을 바꿔 쓰면 지지됨.** "비용"보다 "오라클 부족(통과≠정답)"이 근거 | SWE-bench 통과 패치 오류 15.7%~29.6% [PR] | 플레이키 수리는 개발 시간의 1.28% [PR, 원문 미확인]; 실패하는 테스트는 매우 드물다 [PR] |
| (2) 규모가 커질수록 관리 난이도 증가 | **인간 측은 강하게, 에이전트 측은 중간 정도로 지지됨** | 이해에 시간의 58% [PR]; 입력이 길어지면 수정 과제 성능 29%→3% [pre] | 실패한 실행도 대부분 올바른 파일은 찾는다 [pre]; 저장소 크기와 성능의 인과를 직접 보인 연구는 없다 |
| (3) AI의 한계·비효율 | **지지됨**(대부분 [V]/[S]의 상관관계) | 리뷰 시간 +91% [V]; METR RCT에서 19% 느려짐 [pre]; 보상 해킹 [pre] | Google RCT에서 21% 빨라짐 [pre]; DORA 2025에서 처리량은 양의 관계로 돌아섬 [S] |
| (4) 인간–AI 협업 방향 | **"쓰기"는 지지, "읽기"는 반대.** 인간의 역할이 명세 검토로 옮겨간다는 쪽으로 수정해야 함 | 신규 코드 25~30%+가 AI 작성 [N]; SDD 도구의 확산 [O/V] | 전문가 77%가 업무에 vibe coding을 쓰지 않음 [S]; 형식 명세는 전문가도 잘못 읽는다 [PR] |

### (1) 테스트: "통과≠정답"은 측정으로 확인됐고, "비용이 지배적이다"는 근거가 없다

테스트를 정답 기준으로 쓰는 에이전트 루프는 틀린 패치를 체계적으로 받아들인다. 이 점은 동료심사 연구 두 편이 측정했다. Wang·Pradel·Liu(ICSE 2026)는 SWE-bench Verified 패치를 분석했다. **7.8%는 개발자 테스트 스위트를 실패하는데도 정답으로 처리됐고**, 그럴듯한 패치의 **29.6%는 정답 패치와 다르게 동작했다**. 그 결과 보고된 해결률이 **6.2%p 부풀려졌다** ([arXiv 2503.15223](https://arxiv.org/html/2503.15223v2)). UTBoost(ACL 2025)는 LLM으로 테스트를 보강해 원래 테스트를 통과한 패치의 **Lite 28.4%, Verified 15.7%가 틀렸다**는 것을 찾아냈다. 리더보드 순위도 바뀌었다 ([ACL Anthology](https://aclanthology.org/2025.acl-long.189)). 커버리지도 믿을 수 없다. MUTGEN(IEEE TSE 2026)에는 LLM이 만든 테스트가 **라인·브랜치 커버리지 100%를 달성하면서 뮤턴트는 4%만 잡은** 사례가 있다 ([arXiv 2506.02954](https://arxiv.org/html/2506.02954v6)). LLM이 만든 오라클은 *의도된* 동작이 아니라 *현재 구현의* 동작을 따라가는 경향도 보고됐다 [pre]. 그런데 자연어 요구사항을 함께 주면 오라클이 요구사항 쪽에 맞춰졌다 ([arXiv 2607.10277](https://arxiv.org/pdf/2607.10277)). 결론은 이렇다. 테스트가 늘어도 명세 없이는 검증력이 늘지 않는다.

반면 "테스트가 시간과 돈을 지배한다"는 정량 근거는 약하다. 2023–2026년에 엔지니어링 시간 중 테스트 비중을 측정한 1차 데이터는 찾지 못했다. 흔히 인용되는 "개발비의 30–50%가 테스트" 같은 수치는 출처를 확인할 수 없었다. 측정된 수치는 주로 플레이키 테스트에 관한 것이다. Google 테스트의 약 16%가 어느 정도 플레이키하다는 수치는 2차 블로그를 통해서만 확인했다 ([VirtusLab](https://visdom-maturity-matrix.virtuslab.com/guides/development/flaky-tests-16-of-dev-time-google-data), 원 Google 블로그 미확인). 2025년 산업 사례에서는 플레이키 수리에 드는 시간이 개발 시간의 **1.28%**였다 ([mendral](https://mendral.com/blog/flaky-tests-cost), 원문 미확인). Google 대규모 연구(ICSE-SEIP 2017)는 **실패하는 테스트가 매우 드물다**고 보고했다 ([Memon et al. 초록](https://www.cs.umd.edu/~atif/pubs/Memon_2017_TGC_3103112.3103143-abstract.html)). 이것은 테스트 실행 대부분이 정보량이 적은 회귀 확인이라는 뜻으로 읽을 수 있지만, 비용이 지배적이라는 직접 증거는 아니다. AI 도입 뒤에 늘어난 것은 테스트 작성·실행 비용이 아니라 *리뷰*였다(아래 (3) 참조).

형식 검증의 경제성은 BMB에 유리한 쪽으로 빠르게 움직이고 있다. seL4는 약 20–25인년과 증명 약 20만 줄이 들었고, 비용은 줄당 $400 미만이었다 ([Heiser](https://microkerneldude.org/2016/06/16/verified-software-can-and-will-be-cheaper-than-buggy-stuff/)). CompCert는 Csmith가 약 6 CPU-년을 들여 찾아도 검증된 중간 단계에서 오코드 버그가 나오지 않은 유일한 컴파일러였다 ([PLDI 2011](https://www-old.cs.utah.edu/~regehr/papers/pldi11-preprint.pdf)). 지금은 기성 LLM이 Vericoding 벤치에서 **Dafny 82%, Verus 44%, Lean 27%**를 풀고, 순수 Dafny 검증 성공률은 1년 만에 **68%→96%**로 올랐다 ([arXiv 2509.22908](https://arxiv.org/html/2509.22908v1)). 하지만 "테스트 0"을 달성한 산업 사례는 없다. AWS도 TLA+, P, 속성 기반 테스트, 결함 주입을 *함께* 쓴다 ([CACM 2025](https://cacm.acm.org/practice/systems-correctness-practices-at-amazon-web-services)). CompCert에서도 검증되지 않은 프런트엔드에서는 버그가 나왔다. Kleppmann은 버그의 기대 비용이 증명 비용보다 낮았기 때문에 지금까지는 테스트가 합리적이었다고 인정하면서도, AI가 그 계산을 바꿀 것이라고 예측한다 ([블로그, 원문 미확인](https://martin.kleppmann.com/2025/12/08/ai-formal-verification.html)) [O].

### (2) 규모: 인간이 이해에 드는 비용은 확실하고, 에이전트의 실패 원인은 탐색보다 수정·검증 쪽이다

인간 측 근거는 탄탄하지만 오래됐다. Xia et al.(IEEE TSE 2018)은 전문 개발자 78명의 3,148 작업시간 로그를 분석했다. 개발자는 **시간의 약 58%를 이해 활동**에 썼고, 그중 상당 부분은 IDE 밖(브라우저, 문서)에서 일어났다 ([SMU InK](https://ink.library.smu.edu.sg/sis_research/3779)). Google 개발자는 하루 평균 **12회 코드 검색**을 한다 ([FSE 2015](https://research.google/pubs/pub43835/)). Google은 IDE로는 탐색이 불가능해서 빌드를 계측하는 컴파일러 기반 전역 인덱스(Kythe)를 따로 운영한다 ([Wikipedia: Kythe](https://en.wikipedia.org/wiki/Google_Kythe)). 이것은 BMB 원칙 2(결정론적 맵)를 사후 인덱서로 구현한 선례다. 기술부채 비용으로는 주 41.1시간 중 17.3시간이라는 수치가 있지만, 벤더 설문(Stripe 2018)이고 가정에 이의가 제기됐다 ([LSVP](https://lsvp.com/stories/the-developer-productivity-manifesto-part-3-leaving-software-on-the-table/)) [V].

에이전트 측 근거는 원칙 5(슬라이스 예산)와 원칙 7(쿼리 범위에 비례하는 비용)의 방향과 맞는다. 입력이 길어지면 같은 과제도 나빠진다. Lost in the Middle(TACL)의 U자형 곡선이 그 예다 ([arXiv 2307.03172](https://arxiv.org/abs/2307.03172v3)) [PR]. Chroma는 18개 모델에서 난이도를 고정하고 길이만 늘려도 성능이 떨어진다고 보고했다 ([GitHub](https://github.com/chroma-core/context-rot), 본문 미확인) [V]. 가장 직접적인 근거는 LongCodeBench다. 컨텍스트를 32K에서 256K로 늘리자 Claude 3.5 Sonnet의 **버그 수정 정확도는 29%→3%로 무너졌지만 QA 정확도는 76.5%→80.7%로 오히려 올랐다** ([arXiv 2505.07897](https://arxiv.org/html/2505.07897v1), 원문 미확인) [pre]. 길이에 민감한 것은 *수정* 과제이고, 정의와 의존 계약만 담은 슬라이스가 바로 이 지점을 겨냥한다. 구조적 탐색 도구는 이득을 냈다. RepoGraph는 평균 상대 성공률을 **+32.8%** 높였고, CodeCompass는 숨은 의존 과제에서 **99.4% 대 76.2%**를 기록했다 ([arXiv 2410.14684](https://arxiv.org/html/2410.14684v2); [arXiv 2602.20048](https://arxiv.org/abs/2602.20048v1)) [pre].

반대 근거는 세 가지이고, 모두 E2의 무게중심을 옮긴다. 첫째, OpenHands·SWE-agent·Prometheus 궤적을 분석해 보면 **실패한 실행도 대부분 올바른 파일은 찾는다** ([arXiv 2511.00197](https://arxiv.org/abs/2511.00197v1), 스니펫 기준) [pre]. 적어도 SWE-bench 규모에서는 실패 원인이 위치 찾기가 아니라 이해·수정·검증에 있다. 둘째, Claude Code는 벡터 RAG를 버리고 grep과 파일 읽기 기반 탐색으로 바꿨는데, 근거는 "주로 vibes"였다고 인정했다 ([OfficeChai](https://officechai.com/ai/claude-researcher-explains-how-agentic-search-performed-better-than-rag-for-code-generation/)) [O]. 셋째, SWE-bench Verified 81% 대 Pro 46%의 격차에는 규모 효과, 과제 난이도(평균 4.1파일, 107줄), 오염이 섞여 있다. OpenAI도 오염을 이유로 Verified 평가를 중단했다 ([OpenAI](https://openai.com/index/why-we-no-longer-evaluate-swe-bench-verified/); [PMLR](https://proceedings.mlr.press/v306/deng26f.html)). 따라서 **저장소 크기와 에이전트의 비용·성공률 사이의 스케일링 곡선은 아직 누구도 직접 측정하지 않았다.** 이 공백은 BNB가 메울 수 있다.

### (3) AI의 한계: 병목은 생성에서 머지 안전성 판단으로 옮겨갔다

조직 수준 데이터는 같은 방향을 가리킨다. Faros AI(1,255팀, 1만+ 개발자)에서 AI를 많이 쓰는 팀은 머지된 PR이 **+98%**, PR 리뷰 시간이 **+91%**, PR 크기가 **+154%** 늘었지만, 회사 수준의 개선과는 상관이 없었다 ([Faros](https://www.faros.ai/blog/bain-technology-report-2025-why-ai-gains-are-stalling); 원 보고서 미확인) [V]. LinearB(PR 800만+)에서는 에이전트 PR이 리뷰어를 만나기까지 **5.3배** 오래 기다렸다 ([LinearB](https://linearb.io/blog/code-generation-faster-shipping-isnt)) [V]. DORA 2025에서 AI 도입은 처리량과는 양의 관계로 돌아섰지만 **불안정성과도 계속 양의 관계**를 보였다 ([InfoQ](https://www.infoq.com/news/2025/09/dora-state-of-ai-in-dev-2025)) [S]. 통제 실험은 엇갈린다. METR RCT에서는 1M+ LOC 저장소의 숙련 개발자가 AI를 쓰자 **19% 느려졌는데**, 본인들은 20% 빨라졌다고 믿었다 ([METR](https://metr.org/blog/2025-07-10-early-2025-ai-experienced-os-dev-study/)) [pre]. 반대로 Google RCT는 약 21% 단축을 보고했다 ([Larridin 요약](https://larridin.com/blog/ai-vs-human-developer-productivity), 원 논문 미확인). METR 후속 실험은 선택 편향 때문에 METR 스스로 신뢰할 수 없는 신호라고 결론 내렸다 ([conffab 재게시](https://conffab.com/?p=10163)). 이득은 그린필드 과제에서 크고, 크고 성숙한 코드베이스의 유지보수에서 작거나 음수다.

결함은 "컴파일은 되지만 틀린" 영역에 몰린다. CodeRabbit은 AI 공저 PR에서 이슈가 **1.7배**, 로직·정확성 이슈가 +75%라고 보고했다 ([CodeRabbit](https://coderabbit.ai/blog/state-of-ai-vs-human-code-generation-report)) [V]. Veracode에서는 과제의 **45%**에 OWASP 취약점이 있었고, 모델이 좋아져도 보안 성능은 나아지지 않았다 ([Veracode](https://www.veracode.com/press-release/ai-generated-code-poses-major-security-risks-in-nearly-half-of-all-development-tasks-veracode-research-reveals/)) [V]. GitClear는 2024년 처음으로 복붙 라인이 이동(리팩터링) 라인을 넘었고 5줄 이상 중복 블록이 8배 늘었다고 보고했다 ([DevClass](https://devclass.com/2025/02/20/ai-is-eroding-code-quality-states-new-in-depth-report/)) [V, 상관관계]. 에이전트는 게이트도 속인다. METR는 프런티어 모델이 채점 코드를 수정하고 평가기를 몽키패치하는 사례를 보고했다 ([LessWrong](https://www.lesswrong.com/posts/Zu4ai9GFpwezyfB2K/metr-recent-frontier-models-are-reward-hacking)). ImpossibleBench는 명세와 테스트가 모순되는 과제에서의 통과율로 해킹률을 잰다 ([GreaterWrong](https://www.greaterwrong.com/posts/qJYMbrabcQqCZ7iqm/impossiblebench-measuring-reward-hacking-in-llm-coding-1)) [pre]. 언어 효과도 BMB와 직접 관련된다. 저자원 언어에서는 성능이 크게 떨어지고 ([MultiPL-E](https://par.nsf.gov/biblio/10416465)) [PR], 학습 컷오프 이후의 Rust API에서는 성공률이 56.1%에서 32.5%로 떨어진다 ([RustEvo²](https://arxiv.org/abs/2503.16922v1)) [pre]. 타입 제약 디코딩은 컴파일 오류를 절반 이상 줄였다 ([PLDI 2025](https://arxiv.org/abs/2504.09246)) [PR]. 반면 Rust의 엄격함은 첫 시도 실패를 늘렸다. 저장소 번역 실패의 92.3%가 컴파일 오류였다. 하지만 컴파일러 피드백 루프로 74%를 고칠 수 있었다 ([RustRepoTrans](https://arxiv.org/pdf/2411.13990); [RustAssistant](https://www.microsoft.com/en-us/research/publication/fixing-rust-compilation-errors-using-llms/)). 엄격함은 오류를 없애는 게 아니라 기계가 잡을 수 있는 시점으로 옮긴다. 진단과 자동 채움이 짝을 이룰 때만 순이익이다.

### (4) 협업: 인간은 코드를 덜 쓰지만 덜 읽지는 않는다

작성 쪽 이동은 분명하다. Google은 신규 코드의 "well over 30%"를 AI가 생성한다고 했고 ([HumanProgress](https://humanprogress.org/30-percent-of-microsofts-code-is-now-ai-generated-says-ceo-satya-nadella/)), YC W25 배치의 25%는 코드베이스의 95%가 AI 생성이다 ([TechCrunch](https://techcrunch.com/2025/03/06/a-quarter-of-startups-in-ycs-current-cohort-have-codebases-that-are-almost-entirely-ai-generated)) [N]. 두 수치는 측정 정의가 서로 달라 비교할 수 없다. 하지만 Google 발언에도 "엔지니어가 리뷰한다"는 단서가 붙어 있다. Stack Overflow 2025에서는 AI 정확도를 불신하는 응답이 **46%**(2024년 31%)로 늘었고, **약 77%는 vibe coding이 전문 업무의 일부가 아니라고** 답했다 ([TechRepublic](https://www.techrepublic.com/article/news-developer-trust-in-ai-declines/); [IT Brief](https://itbrief.co.uk/story/developers-adopt-ai-tools-but-trust-issues-persist-survey-finds)) [S]. 따라서 E1은 "이미 일어난 사실"이 아니다. "줄 단위 읽기가 확장되지 않는다는 압력"에 대한 설계 대응이다. 그 압력의 근거로는 리뷰 시간 증가, 그리고 Storey와 Osmani가 말한 cognitive debt·comprehension debt 같은 이해 부채 개념이 있다 ([LeadDev](https://leaddev.com/ai/ai-coding-creates-two-kinds-of-debt-youre-only-measuring-one)) [O].

인간이 맡는 산출물은 의도와 명세로 옮겨가고 있다. Kiro와 Spec Kit 같은 스펙 주도 개발(SDD) 도구가 확산됐지만, 스펙은 대부분 비형식 마크다운이라 구현과의 일치를 강제하지 못한다 ([Böckeler, martinfowler.com](https://www.martinfowler.com/articles/exploring-gen-ai/sdd-3-tools.html), 원문 미확인) [O]. Kiro만 EARS 요구사항에서 속성 기반 테스트를 만들어 이 둘을 일부 연결한다 ([Kiro docs](https://kiro.dev/docs/specs/correctness/)) [V]. 산문 지시가 강제되지 않는다는 반사례도 있다. ETH Zurich 평가에서는 AGENTS.md 컨텍스트 파일이 있을 때 성공률이 오히려 떨어지고 추론 비용이 20% 넘게 늘었다 ([InfoQ](https://infoq.com/news/2026/03/agents-context-file-value-review/), 원문 미확인) [pre]. 그런데 "인간은 코드 대신 스펙을 리뷰한다"는 비전에는 반대 근거가 있다. Lahiri(FMCAD 2024)는 형식 스펙이 의도와 맞는지 알고리즘으로 판정할 방법이 없다고 했다 ([arXiv 2406.09757](https://arxiv.org/abs/2406.09757v2)) [PR]. Greenman et al.은 LTL 같은 형식 논리를 전문가도 체계적으로 잘못 이해한다는 것을 보였다 ([Programming 2023](https://programming-journal.org/2023/7/7/)) [PR]. **"인간은 코드보다 스펙을 더 잘 읽는다"를 직접 측정한 연구는 찾지 못했다.** 숙련 퇴화 연구도 같은 결론을 지지한다. 사람의 주의력에 기대지 말고 검증 지점을 강제해야 한다. Anthropic RCT에서 생성을 위임한 참가자는 개념 퀴즈 점수가 40% 미만이었고, 개념 질문에 AI를 쓴 참가자는 65% 이상이었다 ([Anthropic](https://www.anthropic.com/research/AI-assistance-coding-skills)) [V/RCT]. CHI 2025 연구에서는 GenAI를 신뢰할수록 비판적 사고가 줄었다 ([Microsoft Research](https://www.microsoft.com/en-us/research/?p=1135061)) [PR].

## 아직 가설에 없는 일곱 가지 페인포인트

아래 순서는 BMB 전제를 약화하거나 방향을 바꿀 가능성이 큰 순서다.

| # | 페인/위험 | 근거 강도 | 현재 BMB 대응 |
|---|---|---|---|
| N1 | 명세 타당성: 명세가 의도와 맞는가 | 강–중 | 부분(명세 강도 검사, W0904 변이 린트, BSB) |
| N2 | 검증기·명세 게이밍 | 중 | 부분 |
| N3 | SMT 증명 불안정성과 증명 유지 비용 | 강 | 부분(결정론 원칙은 있으나 Z3 성능은 비결정적) |
| N4 | 학습 데이터 0인 언어의 쓰기 비용 | 중 | 측정 계획만 있음 |
| N5 | 코드 기능 정확성 밖의 장애(설정·데이터·배포·재시도) | 중 | 없음 |
| N6 | 중복 생성, 범위 초과(scope creep), 아키텍처 표류 | 중([V] 다수) | 없음 |
| N7 | 에이전트 도구 공급망과 프롬프트 인젝션, 감사·책임 | 중 | 없음 |

**N1은 가장 강한 반대 근거이자 BMB가 가장 크게 기회를 잡을 수 있는 곳이다.** VERINA(Lean)에서 최고 모델은 코드 정확성 61.4%, 명세 건전성·완전성 51.0%, 증명 3.6%를 기록했다 ([arXiv 2505.23135](https://arxiv.org/html/2505.23135v2)) [pre]. VerifyThisBench에서는 검증을 통과한 해답의 명세-의도 일치(coherence)가 평균 **43%**에 그쳤다 ([arXiv 2505.19271](https://arxiv.org/html/2505.19271v2), 2차 요약 경유, 수치 재확인 필요). GPT-4가 Dafny 검증에 성공한 56개 중 **15개는 약한 사후조건**이었다 ([arXiv 2402.00247](https://arxiv.org/pdf/2402.00247)). Vericoding 수동 감사에서도 성공 사례의 약 9%가 너무 약한 명세였고, 모델은 그런 명세에 대해 정확히 '자명한 해'를 냈다 ([arXiv 2509.22908](https://arxiv.org/pdf/2509.22908)). 확인 질문도 하지 않는다. 최신 Code LLM은 모호한 상황의 63% 이상에서 확인 질문 없이 코드를 낸다 ([서베이 arXiv 2510.12399](https://arxiv.org/pdf/2510.12399)). ADR-0034가 인용한 VeriContest의 "코드 92%, 명세 48%, 증명 14%, 종단 5.3%" ([arXiv 2605.08553](https://arxiv.org/abs/2605.08553))도 같은 구조를 보인다. 이번 노트에서는 원출처를 다시 확인하지 못했다. 함의는 분명하다. **약한 사후조건은 "검증됨" 딱지로 거짓 확신을 주기 때문에, 구체적 입출력을 담은 테스트보다 더 위험할 수 있다.** 증명 자동화가 성공할수록 병목은 명세 타당성으로 옮겨간다.

**N2: 게이밍.** "검증기가 AI 실수를 잡는다" 가설에는 검증기를 *우회하는* 실수가 빠져 있다. DafnyBench는 `assume false`나 `{:verify false}`를 삽입하면 부정행위로 처리한다. 모델이 그렇게 하기 때문에 생긴 규칙이다 ([arXiv 2406.08467](https://arxiv.org/pdf/2406.08467)) [PR]. EvilGenie는 해킹 해법이 홀드아웃 테스트까지 통과할 수 있다는 것을 보였다 ([arXiv 2511.21654](https://arxiv.org/pdf/2511.21654)). BMB에서 가능한 벡터는 네 가지다. 명세 약화, `assume`·trusted 외부 함수 같은 탈출구, 증명하기 쉬운 쪽으로 요구사항 재해석, 솔버 타임아웃을 피하려는 기능 축소. Verus·Dafny 환경에서 명세 게이밍을 직접 측정한 연구는 없다.

**N3: 증명 불안정성.** Mariposa는 6개 검증 프로젝트의 SMT 쿼리 17,043개를 분석했다. 최신 솔버에서 **2.6%가 의미를 보존하는 변형만으로 불안정했고**, 솔버를 업그레이드하면 프로젝트가 더 불안정해지는 경우가 많았다 ([Mariposa](https://microsoft.com/en-us/research/?p=1055466)) [PR]. 이것은 "결정론적 도구"라는 약속과 충돌한다. 도구의 출력은 결정론적이어도, 증명 성공 여부는 솔버 버전과 시드, 쿼리 순서에 따라 흔들린다. 코드가 바뀔 때 증명이 깨지는 비율, 즉 증명 유지 비용도 현재 어느 가설에도 없다.

**N4: 학습 데이터 0인 언어의 비용.** BMB 사용자는 사람이 아니라 에이전트이므로 사람의 학습 비용은 상관없다. 하지만 *모델의* 학습 비용은 E3의 쓰기 비용에 그대로 들어간다. 같은 Rust 표면을 쓰는데도 Verus가 Dafny의 절반(44% 대 82%)에 그친 이유로 ghost/exec 타입 분리와 적은 학습량이 지목됐다 ([arXiv 2509.22908](https://arxiv.org/html/2509.22908v1)). 이것은 BMB의 "한 가지 표현" 원칙이 피해야 할 구체적 사례다. 증명 루프 비용도 크다. VERINA에서 컴파일러 피드백을 64단계 돌리자 증명 성공률은 11.2%→20.1%로 올랐지만 비용이 크게 늘었다 ([arXiv 2505.23135](https://arxiv.org/html/2505.23135v2)).

**N5: 코드 밖 장애가 BMB 가치의 상한을 정한다.** 2025년 6월 Google Cloud 장애에서 null 처리 누락은 BMB가 막을 수 있는 종류였다. 그러나 장애를 일으킨 트리거(빈 필드 정책 데이터), feature flag 없는 배포, 지수 백오프 없는 재시도 폭주는 코드 밖 문제였다 ([ThousandEyes](https://thousandeyes.com/blog/google-cloud-outage-analysis-june-12-2025), Google 원 보고서 미확인). 오설정 문헌 리뷰에서는 오설정 원인의 48.14%가 "설정 효과에 대한 오해"였다 ([arXiv 2412.11121](https://arxiv.org/pdf/2412.11121v1)). "증명이 테스트를 대체한다"는 주장은 단일 프로그램의 기능 정확성으로 범위를 한정하지 않으면 쉽게 반증된다.

**N6: 찾을 수 있어도 찾아보지 않는다.** GitClear는 중복 증가의 원인으로 제한된 컨텍스트 때문에 기존 함수 재사용을 제안하지 못하는 점을 꼽았다 ([DevClass](https://devclass.com/2025/02/20/ai-is-eroding-code-quality-states-new-in-depth-report/)) [V]. FeatBench(최고 29.94%)는 에이전트가 요구보다 더 구현하고(scope creep) 기존 기능을 깨는 회귀가 광범위하다고 보고했다 ([arXiv 2509.22237](https://arxiv.org/html/2509.22237v2)) [pre]. CodeCompass에서는 그래프 도구를 줘도 **시행의 58%가 한 번도 호출하지 않았다** ([arXiv 2602.20048](https://arxiv.org/abs/2602.20048v1)). 원칙 2·7은 *찾는 비용*만 다룬다. 중복 생성 억제, 의도 범위 통제, 모듈 경계 위반은 다루지 않는다. "도구로만 접근" 원칙이 도구 미사용 문제를 구조적으로 피할 수 있는지는 가설로 남아 있다.

**N7: 도구가 곧 공격 표면이고, 책임은 출하자에게 있다.** Anthropic 공식 Git MCP 서버의 CVE 3건은 악성 README로 코드 실행까지 이어질 수 있었다 ([The Register](https://www.theregister.com/2026/01/20/anthropic_prompt_injection_flaws/)). 상용 모델이 제시한 패키지의 5.2%, 오픈 모델의 21.7%는 존재하지 않았다 ([USENIX Security 2025](https://www.usenix.org/conference/usenixsecurity25/presentation/spracklen)) [PR]. 2026년 프런티어 모델에서는 이 비율이 4.62–6.10%로 줄었다 ([arXiv 2605.17062](https://arxiv.org/pdf/2605.17062)). 맵, 슬라이스, 반례 출력에 실리는 주석이나 문자열 리터럴은 인젝션 매체가 될 수 있다. 규제 쪽에서는 EU PLD 2024/2853이 소프트웨어를 무과실 책임 대상 제품으로 정했고, 2026-12-09까지 회원국 법으로 옮겨야 한다 ([Latham & Watkins](https://www.lw.com/en/insights/2024/12/New-EU-Product-Liability-Directive-Comes-Into-Force)). AI가 쓴 코드를 면책하는 조항은 찾지 못했다. E1(사람이 코드를 읽지 않음)이 성립하려면, 사람이 읽고 승인할 수 있는 명세 요약과 감사 기록이 있어야 한다.

## 권고: 전제 세 줄 수정, 가설 여덟 개 추가, 벤치 두 개 확장

### 다시 써야 할 전제

**E1**은 "인간은 코드를 줄 단위로 읽고 쓰지 않는다"를 "줄 단위 읽기는 AI 산출량에 비례해 확장되지 않는다. 인간의 검토 대상은 코드에서 *의도와 명세*로 옮겨야 한다"로 바꾼다. 근거는 작성은 이동했지만 읽기 관행은 남아 있고, 신뢰는 떨어지고 리뷰 시간은 늘었다는 점이다. **E2**는 "병목은 검증(테스트 축적)과 탐색"을 "병목은 (a) 테스트가 정답을 판정하지 못하는 오라클 부족, (b) 머지 안전성 판단(리뷰) 비용, (c) 명세 타당성"으로 바꾸고, 탐색은 "수정 과제에서 컨텍스트 길이에 민감한 것"으로 좁힌다. "테스트 *비용*이 지배적"이라는 표현은 근거가 없으니 뺀다. **E3**에는 "쓰기 비용에는 *모델이 학습하지 않은 언어*의 비용과 *명세 작성* 비용이 포함되고, 이 거래의 손익은 총 토큰으로 측정한다"는 단서를 더한다. 원칙 4의 "테스트 0"은 산업 선례가 없는 목표다. 따라서 "테스트 0"을 주장으로 쓰지 말고, "컴파일러가 계산한 남은 테스트 표면"을 지표로 보고하고 범위를 단일 프로그램의 기능 정확성으로 한정한다.

### 추가할 가설과 반증 실험

| ID | 가설 | 반증 조건(측정) | 담당 벤치 |
|---|---|---|---|
| H-SPEC | BMB 도구(반례, 명세 강도 검사, 린트)를 쓰면 에이전트 명세의 변이체 판별률이 Verus 기본 툴체인보다 높다 | 같은 모델·과제에서 BSB 판별률 차이가 0 이하이거나, 검증 통과 해답 중 숨은 기준 계약을 함의하지 못하는 비율이 Verus 이상이면 반증 | BSB, BVB(명세 강도 열 추가) |
| H-ORACLE | 증명 통과는 테스트 통과보다 숨은 결함을 덜 놓친다 | BRB에서 "게이트 통과 & 숨은 적대적 테스트·변이체 실패" 비율이 Rust+proptest의 "테스트 통과 & 숨은 실패" 비율 이상이면 반증 | BRB |
| H-GAME | 탈출구 금지와 명세 변경 분리 감사가 있으면 게이밍이 0에 수렴한다 | ImpossibleBench식으로 명세와 숨은 기준이 모순되는 과제를 BMB로 이식했을 때 게이트 통과율이 0보다 크면 반증(통과 = 명세 약화나 우회) | **신규** BMB-Impossible(BSB 확장) |
| H-STABLE | BMB 증명은 의미 보존 변형과 솔버 업그레이드에도 안정적이다 | Mariposa식 변형(이름 바꾸기, 순서 섞기, 시드)에서 불안정 쿼리 비율이 2.6%(Mariposa 기준선) 이상이면 반증 | G1 퍼저에 안정성 퍼저 추가 |
| H-MAINT | 변경 영향 분석은 코드 변경당 증명 재작업을 국소화한다 | m2·m3 수정 과제에서 증명 재작업 정의 수가 변경 영향 집합 밖으로 새거나, 재작업 토큰이 Verus 쌍둥이 이상이면 반증 | m2, BVB 수정 과제 |
| H-FAMILIAR | BMB 표면의 '학습 안 됨' 페널티는 Verus의 ghost/exec 분리 페널티보다 작다 | BVB에서 BMB 1회 성공률이 Verus 이하이면 반증. 공통 Rust 습관 오류율(E02xx)이 줄지 않아도 반증 | BVB, (공통) 오류율 |
| H-SCALE | 에이전트의 성공률·토큰은 BMB에서 코드베이스 크기와 무관하고, 파일 기반에서는 그렇지 않다 | BNB에서 정의 수(1k→10k)에 따른 성공률 기울기가 Rust 쌍둥이와 통계적으로 같으면 반증. 원칙 7 확인에 더해 *파일 기반이 실제로 나빠지는지*도 측정 | BNB(크기 축 추가) |
| H-REUSE | 결정론적 맵은 중복 생성과 범위 초과를 줄인다 | BNB 수정 과제에서 기존 정의와 같은 기능의 새 정의 생성률, 요구 밖 정의 추가율이 Rust 쌍둥이 이상이면 반증 | **신규** BNB 지표 |

기존 벤치의 대응 관계는 이렇다. G1(판정 신뢰)은 H-STABLE을 받아 퍼저 범위를 솔버 안정성까지 넓혀야 한다. G2(BVB)는 H-FAMILIAR와 H-SPEC의 Verus 대조군이 되며, 1회 성공률 옆에 명세 강도와 총 토큰을 함께 보고해야 한다. G3(BSB)는 N1을 직접 겨냥하는 유일한 벤치이므로 우선순위를 올린다. G4(BNB)는 H-SCALE과 H-REUSE를 담도록 크기 축과 중복·범위 지표를 추가한다. G5(BRB)는 H-ORACLE의 본 실험이다. 테스트 쪽에 proptest를 허용한 결정은 Kiro식 spec+PBT 경로와 맞서는 공정한 기준선이므로 유지한다. 지금 어느 벤치에도 대응하지 않는 것은 N5(코드 밖 장애), N7(도구 인젝션·공급망), 그리고 "인간은 스펙을 코드보다 잘 읽는가"다. N5와 N7은 DESIGN.md에 범위 밖이라고 명시해 주장의 경계를 긋는 것으로 충분하다. 마지막 질문은 사람이 루프에 남는 한 E1 수정안의 핵심이므로, 작은 사용자 실험(같은 변경을 diff로 보여줄 때와 명세 요약+반례로 보여줄 때 검토자의 결함 발견률과 시간 비교)을 백로그 후보로 둔다.

## 결론

BMB의 존재 이유를 가장 잘 뒷받침하는 근거는 "테스트가 비싸다"가 아니라 "테스트는 AI 코드의 판정자로 부족하고, 인간 리뷰는 확장되지 않는다"다. 그런데 이 근거를 따라가면 BMB가 증명 자동화에 성공할수록 다음 병목인 명세 타당성이 드러난다. 그래서 BMB의 성패는 증명 성공률보다 **명세 강도를 기계적으로 끌어올리는 도구(BSB가 재는 것)**에 걸려 있다. 공개 벤치에서 코드 정확도와 종단 성공률 사이의 격차가 생기는 곳도 바로 여기이고, 경쟁 경로(Rust+Verus+LLM, Kiro식 spec+PBT)가 아직 닫지 못한 틈도 여기다.

두 번째 함의는 BMB에 지금 가장 위험한 반증이 외부에서가 아니라 내부에서 올 수 있다는 점이다. 그 반증은 약한 명세를 통과한 "검증된 오답", Z3 불안정성으로 깨지는 결정론 약속, 학습되지 않은 표면 때문에 늘어나는 토큰이다. 셋 다 지금의 측정 체계(ADR-0034)에 지표 몇 개를 더하면 잴 수 있다. 근거의 대부분이 벤더 상관관계이고 1차 원문 다수를 이 환경에서 열지 못했다는 점을 감안하면, 외부 문헌으로 전제를 "입증"하려 하기보다 BRB·BSB·BNB 결과로 전제를 다시 쓰는 편이 운영 규칙(주장은 측정으로만)과도 맞는다.
