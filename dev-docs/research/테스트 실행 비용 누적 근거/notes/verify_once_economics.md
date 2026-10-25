# "작성 시 1회 검증" vs "빌드·배포마다 테스트 재실행"의 경제학

> 조사일 2026-10-25. **방법 주의:** 이번 세션에서 WebFetch가 blog.google, security.googleblog.com, adacore.com, ucl.ac.uk, sos-vo.org, amazon.science 등에서 모두 DNS 오류(ENOTFOUND)로 실패했다. 아래 인용은 **모두 검색 스니펫 기반(snippet-only)**이다. 원문 대조가 필요한 수치는 [S]로 표시했다. 출처 유형: [PR]=동료심사 논문, [V]=벤더/기업 블로그·발표, [2nd]=2차 보도·요약.

## (a) 정적 보장이 반복 테스트·퍼징·QA·런타임 검사를 실제로 대체·감소시킨 사례

### Takeaway
"증명이 테스트를 대체했다"는 가장 강한 문서화 사례는 항공(Airbus A380 단위 테스트를 Caveat 증명으로 대체, DO-333이 이를 공식 허용)이다. 단 통합 테스트(HW/SW integration)는 여전히 남는다. 메모리 안전 언어(Android Rust)는 취약점 밀도·롤백률을 크게 줄였다는 강한 기업 데이터가 있지만, 퍼징·새니타이저 비용 감소를 직접 수치화한 공개 자료는 찾지 못했다.

### Cited Findings
**항공 / DO-333 (가장 직접적인 "증명 → 테스트 대체" 근거)**
- Airbus는 Caveat(Frama-C 전신)로 A380 저수준 요구사항을 증명해 **단위 테스트를 대체**했다(2002년경, AdaCore 저자 기사 [V][S]). 2006 ERTS 논문(Atos Origin·Airbus 엔지니어)은 "formal verification is used for the Unit Validation and replaces the more traditional method of Unit Testing"라고 기술 [PR-학회, S] — [Military Embedded Systems](https://militaryembedded.com/avionics/safety-certification/formal-program-verification-avionics-certification); [ERTS 2006 Duprat et al.](https://go.fisita.com/store/papers/ERTS06/2A2_S.Duprat_AtosOrigin)
- DO-333(DO-178C 형식기법 보충)은 형식 분석이 특정 요구사항 기반 테스트를 보완하거나 **대체**할 수 있게 한다. 단 "with the exception of software/hardware integration tests showing that the EOC is compatible with the target computer", 나머지 실행 코드 검증 목표는 형식 분석으로 충족 가능 [V][S] — [AdaCore: DO-178C standards booklet](https://learn-latest.adacore.com/booklets/adacore-technologies-for-airborne-software/chapters/standards.html); [AdaCore: What is DO-178C](https://adacore.com/industries/avionics/what-is-do-178c)
- 관련 1차 논문: Moy, Ledinot, Delseny, Wiels, Monate, "Testing or Formal Verification: DO-178C Alternatives and Industrial Experience", IEEE Software 2013 [PR] (본문 페치 실패, 수치 미확인) — [PDF](https://www.adacore.com/uploads/techPapers/IEEE_Software_Formal_Or_Testing.pdf)

**SPARK / Tokeneer (Praxis/Altran)**
- Tokeneer: 엔지니어 3명이 260 인일에 약 10,000 LOC 작성, 이후 조사에서 결함 0건 보고(GCN 보도) [V/2nd][S]; 2013 FMATS 발표는 9,939 LOC, 260 effort-days, "5 defects found to date" [V][S] — [Praxis ISSSE 2006 paper](https://www.adacore.com/uploads/technical-papers/issse2006tokeneer_altran.pdf); [Cambridge FMATS CbyC](https://www.cl.cam.ac.uk/archive/mjcg/FMATS2/papers/CbyCForSecurity.pdf)
- Praxis 2005 슬라이드: "testing alone is too expensive and risk-prone to serve as the primary verification activity" — 정적 검증을 1차 수단으로 [V][S] — [Recent Results with CbyC and SPARK](https://sos-vo.org/sites/sos-vo.org/files/sos_files/Recent_Results_with_Correctness_by_Construction_and_SPARK.pdf)
- SPARK 2014 포팅: 미증명 검사 234개 → 39개(정당화된 미증명), 즉 도구 한계로 사람이 일부 책임을 진다 [V] — [AdaCore blog: Tokeneer fully verified](https://blog.adacore.com/tokeneer-fully-verified-with-spark-2014)

**CompCert vs GCC/LLVM (Csmith)**
- Yang et al., PLDI 2011: Csmith로 3년간 325개 이상 신규 버그 보고, 테스트한 모든 컴파일러가 크래시·오컴파일. 단 2011년 초 개발 중이던 CompCert는 약 **6 CPU-년**의 시도에도 wrong-code 오류가 발견되지 않은 유일한 컴파일러 [PR] — [PLDI'11 preprint](https://www-old.cs.utah.edu/~regehr/papers/pldi11-preprint.pdf)
- 반례/한계: 검증되지 않은 CompCert 프런트엔드에서는 wrong-code 버그가 발견되었고, 이후 검증 범위가 확장됨(HN 댓글이 논문을 인용, 2차 [S]) — [HN](https://news.ycombinator.com/item?id=9130934)

**Android 메모리 안전 (Google)**
- 메모리 안전 취약점 비중: 2019년 76% → 2024년 24%(2024는 연말 외삽치). 신규 코드를 메모리 안전 언어로 쓰는 전략, "vulnerabilities decay exponentially" [V] — [Google Security Blog 2024-09-25](https://security.googleblog.com/2024/09/eliminating-memory-safety-vulnerabilities-Android.html); 원자수 2019년 223건, 2022년 85건 [2nd] — [ThreatDown](https://www.threatdown.com/blog/androids-rusty-new-code-shakes-off-huge-number-of-memory-safe-vulnerabilities/)
- 2025-11-13 "Rust in Android: move fast and fix things"(Jeff Vander Stoep): 2025년 처음으로 메모리 안전 취약점 비중 20% 미만; Rust는 C/C++ 대비 메모리 안전 취약점 밀도 **약 1000배 낮음**; Rust 변경의 **롤백률 4배 낮음**, 코드리뷰 시간 **25% 감소** [V][S] — [Google blog](https://blog.google/security/rust-in-android-move-fast-fix-things/)
- 주의: 대부분 코드는 여전히 메모리 비안전(점점 오래될 뿐) [V] — 위 2024 포스트.

**타입 시스템이 잡는 버그 비율**
- Gao, Bird, Barr, ICSE 2017 "To Type or Not to Type": 400개 공개 JS 버그(398 프로젝트) 중 Flow 0.30·TypeScript 2.0 각각 **약 15%** 탐지. 이미 테스트·리뷰를 통과한 버그라 보수적 추정 [PR] — [UCL Discovery](https://discovery-pp.ucl.ac.uk/id/eprint/10064729); [the morning paper 요약](https://blog.acolyer.org/2017/09/19/to-type-or-not-to-type-quantifying-detectable-bugs-in-javascript)
- Airbnb: 약 6개월 포스트모템 조사에서 버그의 **38%가 TypeScript로 예방 가능**(Brie Bunge, JSConf 2019 발표) — **벤더/발표, 비동료심사, 2차 요약만 확인** [V/2nd] — [InfoQ 2020](https://www.infoq.com/news/2020/08/airbnb-typescript-migration)

**AWS**
- s2n-tls: SAW로 Cryptol 명세와 C 코드 동등성 증명, CI에서 코드 변경마다 증명 자동 재확립("proofs are automatically re-established with little to no interaction from the developers"), Chudnov et al. CAV 2018 [PR] — [Springer](https://link.springer.com/chapter/10.1007/978-3-319-96142-2_26); [Galois](https://galois.com/project/amazon-s2n/). HMAC 103줄 C를 3줄 Cryptol 추론으로 축소 [V].
- Brooker & Desai, "Systems Correctness Practices at AWS", CACM 2025-06: 과거엔 빌드 시 단위 테스트(목 다수)+제한적 배포 통합 테스트에 의존, 이후 TLA+, P, PBT, fault injection 등 포트폴리오 추가; "force multiplier… improving developer velocity and delivering more cost-effective services" — 정량 비용 수치 없음 [PR-실무지][S] — [CACM](https://cacm.acm.org/practice/systems-correctness-practices-at-amazon-web-services)
- Zelkova: 하루 약 1천 → 10억 SMT 호출로 5년간 성장, 정책 질의 99%가 160ms 이내 [PR/V] — [A billion SMT queries a day (CAV 2022)](https://www.amazon.science/publications/a-billion-smt-queries-a-day)

**Microsoft/Everest (HACL*)**
- HACL* Curve25519가 Firefox 57(2017)에, ChaCha20/Poly1305가 Firefox 60에 배포; Firefox 77에서 Curve25519 약 27% 속도 향상 [V] — [Mozilla 2017](https://blog.mozilla.org/security/2017/09/13/verified-cryptography-firefox-57/); [Mozilla 2020](https://blog.mozilla.org/security/2020/07/06/performance-improvements-via-formally-verified-cryptography-in-firefox/). **테스트·퍼징 감소에 대한 수치는 찾지 못함.**

### Inferences
- "증명이 테스트를 대체"는 규제 산업(DO-333)에서 **단위 테스트 수준**에 한해 공식화되어 있고, **타깃 HW와의 통합 테스트는 남는다**. BMB 주장도 "단위 수준 테스트 최소화, 통합/시스템 테스트는 유지"로 범위를 제한해야 근거와 부합한다.
- CompCert 사례는 "검증된 부분에선 수 CPU-년의 퍼징이 아무것도 못 찾음 → 그 부분에 대한 반복 퍼징 가치가 낮다"는 논거가 되지만, 동시에 미검증 경계(프런트엔드)에서 버그가 나왔다는 점이 "신뢰 경계 밖은 계속 테스트해야 한다"는 반론이다.
- Android 데이터는 "정적 보장 → 결함 감소 → 롤백·리뷰 비용 감소"라는 재발 비용 절감의 가장 큰 규모 증거다. 다만 원인은 언어 보장이 아니라 신규 코드 비율 효과도 섞여 있다(Google 자신의 "취약점 지수 감쇠" 설명).
- 타입 시스템 15%(동료심사) vs 38%(벤더) 격차는 측정 방법 차이로, BMB 문서에서는 15%를 기준값으로 쓰는 편이 안전하다.

### Gaps
- Google이 Rust 도입으로 퍼징·새니타이저 CPU 비용을 얼마나 줄였는지에 대한 공개 수치 미발견.
- Airbus/Dassault의 단위 테스트 대체로 인한 비용 절감(%)은 IEEE Software 2013 논문 원문에 있을 것으로 보이나 페치 실패로 미확인.
- iFACTS(NATS) 관련 테스트 감소 수치 미발견. Cloudflare/Discord Rust 사례는 시간 제약으로 조사하지 못함.
- HACL*/EverCrypt 배포 후 해당 코드의 퍼징·테스트 축소 여부 자료 없음.

## (b) 검증 자체의 반복 비용 (빌드마다 증명 재검사, 코드 변경 시 증명 유지, 솔버 시간)

### Takeaway
검증은 "1회"가 아니다. 증명은 CI에서 매 변경마다 재검사되며(s2n), 코드 변경 시 유지비가 크게 발생하고(s2n 핸드셰이크 증명, seL4 10년 유지), SMT 기반 증명은 솔버 업그레이드·무관한 변경에도 불안정해질 수 있다(Mariposa: 최신 Z3에서 질의 2.6% 불안정). 대규모 재증명은 수십 분~수 일의 머신 시간을 쓴다.

### Cited Findings
- s2n: 이후 AWS 발표에 따르면 검증 코드가 약 3,000줄로 늘었고 모든 증명이 CI에 있으며, "One effect has been a huge increase in code change and proof maintenance cost. One proof in particular, our TLS handshake proof, sees particularly high development." [V][S] — [Verification is Engineering (sos-vo)](https://sos-vo.org/node/69935)
- Mariposa (Zhou et al., FMCAD 2023): 6개 검증 프로젝트(Dafny, F*, Verus)의 SMT 질의 17,043개에 이름 변경·assert 순서 변경·시드 변경 같은 의미 보존 변형 적용. **솔버 업그레이드가 프로젝트 안정성을 자주 떨어뜨리며**, 최신 솔버에서 질의의 **2.6%가 불안정** [PR] — [Mariposa PDF](https://www.andrew.cmu.edu/user/bparno/papers/mariposa.pdf); [MSR](https://microsoft.com/en-us/research/?p=1055466)
- Verus 프로젝트 질의의 약 1%가 불안정(후속 연구); 불안정성은 "substantially lengthens their code-prove-debug cycle" [PR][S] — [Cazamariposas](https://www.andrew.cmu.edu/user/bparno/papers/cazamariposas.pdf); [Towards Proof Stability (Dafny 2025 workshop)](https://popl25.sigplan.org/details/dafny-2025-papers/5/Towards-Proof-Stability-in-SMT-based-Program-Verification)
- seL4: 100만 줄 이상 Isabelle/HOL 증명, 약 10년간 이식·유지·진화. "the seL4 proof"는 HW 플랫폼·기능 구성별 여러 증명의 집합이며 구현 변경에 따라 지속 유지가 필요 [PR-초청강연] — [Andronick, ITP 2019 "A Million Lines of Proof About a Moving Target"](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ITP.2019.1)
- 증명 노력 모델: Staples et al. 2014 — 9개 L4.verified 프로젝트에서 노력(person-week)과 증명 크기 사이 강한 선형 관계 [PR] — [Trustworthy Systems](https://www.trustworthy.systems/publications/nictaabstracts/Staples_JAMKK_14.abstract); Matichuk et al. ICSE 2015 — 명세 크기와 최종 증명 크기 사이 **이차(quadratic)** 관계 [PR] — [Trustworthy Systems](https://trustworthy.systems/publications/nictaabstracts/Matichuk_MAJKS_15.abstract)
- 회귀 증명 비용: 대형 프로젝트는 "from tens of minutes and hours up to several days" 머신 시간 [PR-서베이] — [QED at Large (Ringer et al. 2019)](https://arxiv.org/pdf/2003.06458). 예: cogent 검증 빌드 약 120 CPU-시간, 32GB 권장 [repo][S] — [cogent repo](https://github.com/emmet-m/cogent). Isabelle 전체 테스트 수 시간(2016 메일링 리스트, 일화) — [isabelle-dev](https://mailman46.in.tum.de/pipermail/isabelle-dev/2016-April/015271.html)
- 증명 취약성: Coq 증명은 "brittle, breaking easily in response to changes"; 증명 공학은 유지보수에서 소프트웨어 공학보다 수십 년 뒤처짐; 증명 수리 도구는 동작 변경 같은 많은 변경을 아직 다루지 못함 [PR/학위논문] — [Ringer dissertation](https://homes.cs.washington.edu/~djg/theses/ringer_dissertation.pdf); [arXiv 2310.06959](https://arxiv.org/abs/2310.06959v5)
- 도구 한계로 인한 사람 비용: SPARK Tokeneer에서 39개 검사는 증명되지 않고 사람이 정당화해야 함 [V] — [AdaCore](https://blog.adacore.com/tokeneer-fully-verified-with-spark-2014)
- 증명 ≠ 실제 구현: "a proof is always done on a formal representation which is an abstraction from certain aspects of the real implementation"(Rockwell Collins), 컴파일러 정확성 격차도 인정됨 [V/PR][S] — [AdaCore booklet](https://learn-latest.adacore.com/booklets/adacore-technologies-for-airborne-software/chapters/standards.html)

**완화책: 증분·캐시 검증**
- Leino & Wüstholz, "Fine-grained Caching of Verification Results"(CAV 2015): 호출 그래프·CFG로 변경 영향 부분만 재검증(Boogie 구현) [PR] — [MSR](https://www.microsoft.com/en-us/research/?p=342581)
- Dafny IDE(F-IDE 2014): 수정된 5개 변형의 검증 총 시간이 1회 검증보다 "only marginally above" — 캐시가 병렬화보다 반응성에 더 중요 [PR] — [arXiv 1404.6602](https://arxiv.org/pdf/1404.6602)
- Isabelle 빌드는 변경 영향 증명만 재검사하는 선택 기능 지원 [PR-서베이] — [QED at Large](https://arxiv.org/pdf/2003.06458)

### Inferences
- BMB의 정의 단위 증명 캐시는 문헌의 표준 완화책(Dafny/Boogie 캐시)과 같은 방향이며, 캐시가 있으면 "증명의 반복 비용 ≈ 변경된 정의 수 × 정의당 솔버 시간"으로 축소된다는 설계 근거가 있다. 반면 테스트는 보통 영향 분석 없이 전체를 돌리는 경우가 많다(이 비교 자체는 측정 필요).
- 그러나 Mariposa 결과는 캐시 키에 **솔버 버전**을 넣어야 하며, Z3 업그레이드 시 전체 재증명과 불안정 질의 대응 비용이 반복적으로 발생함을 뜻한다. 이는 "1회 지불"이라는 주장의 핵심 반례다.
- s2n·seL4는 "증명 유지비가 코드 변경 빈도에 비례해 계속 발생"함을 보여준다. 따라서 상각 이점은 **자주 실행되지만 드물게 변경되는 코드**(암호, 커널, 컴파일러 중간단, 라이브러리 핵심)에서 가장 크다.
- Matichuk의 이차 관계는 명세가 커지면 증명 비용이 초선형으로 증가함을 시사 → 에이전트가 작성하는 계약을 작게 유지해야 할 이유.

### Gaps
- seL4의 변경당 증명 유지 인시(person-hours)와 현재 CI 전체 재검사 시간은 찾지 못함(Andronick ICSE 2012/ITP 2019 원문 필요).
- s2n CI의 증명 실행 시간(분)·월간 비용 수치 미발견(CAV 2018 원문 페치 실패).
- Verus 증분 검증 및 Lean(Mathlib) 빌드 캐시의 정량 효과 미조사.
- Zelkova의 솔버 업그레이드 회귀 관리 비용(CAV 2022 논문에 언급될 가능성) 원문 미확인.

## (c) 1회 검증 vs 반복 테스트의 상각(amortization) 모델·논거

### Takeaway
"검증 1회 비용 vs 테스트 N회 비용"을 직접 비교한 정량 상각 모델은 이번 조사에서 찾지 못했다. 문헌은 대체로 정성적이며(Woodcock 2009조차 비용-편익 증거 부족을 인정), 상각 논거는 사례 수준 데이터(Tokeneer 생산성, Android 롤백률, Csmith 6 CPU-년)로 간접 지지될 뿐이다.

### Cited Findings
- Woodcock, Larsen, Bicarregui, Fitzgerald, "Formal methods: Practice and experience", ACM Computing Surveys 41(4), 2009: 62개 산업 프로젝트 조사. 초안은 "a lack of a substantial body of technical and cost-benefit evidence from applications of formal methods"를 지적 [PR] — [DOI 10.1145/1592434.1592436](https://api.crossref.org/works/10.1145%2F1592434.1592436); [draft PDF](https://homepage.divms.uiowa.edu/~tinelli/classes/181/Fall20/Papers/Wood09.pdf). (시간·비용·품질 효과 비율 수치는 원문 미확인)
- Praxis: 가장 중요한 시스템에서 테스트만으로는 충분한 증거를 만들 수 없다는 논거(정성) [V] — [CbyC and SPARK slides](https://sos-vo.org/sites/sos-vo.org/files/sos_files/Recent_Results_with_Correctness_by_Construction_and_SPARK.pdf)
- 실무자 의견(비공식): SPARK의 V&V 비용이 다른 언어에서 같은 신뢰 수준을 얻는 테스트 비용보다 작을 수 있으나, "proof of partial correctness doesn't negate the need for testing" [블로그 댓글, 낮은 신뢰] — [dev.to](https://dev.to/danlebrero/comment/9k8)
- 증명 노력 추정 모델(선형·이차)은 존재하나 테스트 비용과 대비한 상각 모델은 아님 — Staples 2014, Matichuk 2015 (위 (b) 참조).
- AWS: "force multiplier", "more cost-effective services"라는 경영적 주장만 존재(수치 없음) [S] — [CACM 2025](https://cacm.acm.org/practice/systems-correctness-practices-at-amazon-web-services)

### Inferences
- BMB 소유자의 "1회 지불 → 수십~수천 번의 미래 테스트 실행 회피" 논제에 대해 외부 문헌은 **방향은 지지하나 정량 모델은 없다**. 따라서 BMB가 이를 주장하려면 자체 측정(예: 정의당 증명 시간 vs 그 정의를 덮는 테스트 실행 시간 × 빌드 횟수, 변경 빈도별 유지비)을 커밋해야 한다(CLAUDE.md "주장은 측정으로").
- 간단한 손익분기 틀(추론, 출처 아님): 증명이 이득이려면 `C_proof_write + Σ_changes C_proof_maint + Σ_builds C_recheck(캐시 적중 시 ≈0) + C_solver_upgrade < C_test_write + Σ_changes C_test_maint + Σ_builds C_test_run + C_escaped_defects`. 문헌 근거상 C_test_run은 실제로 빌드마다 발생하지만 보통 저렴하고, 큰 항목은 C_escaped_defects(Android 롤백·취약점)와 C_proof_maint(s2n, seL4)다. 즉 상각 효과의 주된 원천은 "테스트 실행 횟수 절감"보다 "새어나가는 결함 감소"일 가능성이 높다 — 이 점은 소유자 논제의 프레이밍을 수정할 근거가 된다.
- 반례 요약: (1) 증명은 통합/HW 테스트를 대체하지 못함(DO-333), (2) 신뢰 경계 밖(프런트엔드, 명세 오류, 컴파일러)은 계속 테스트 필요(CompCert, Rockwell Collins), (3) 증명 유지비는 변경마다 반복(s2n, seL4), (4) 솔버 불안정성은 무관한 변경·업그레이드에도 비용 발생(Mariposa).

### Gaps
- Boehm식 cost-of-quality 곡선을 형식 검증에 적용한 정량 연구 미발견.
- "증명된 함수당 삭제된 테스트 수" 같은 직접 지표를 보고한 산업 데이터 미발견(Airbus 사례가 가장 근접하나 수치 미확인).
- 런타임 검사 제거(bounds check elision)로 인한 성능 이득의 상각 측면은 이번 범위에서 다루지 못함.
