# 계약 완전성·등가 프로브 (Phase 1a) — 구현 계획

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** BMB 자기 코드베이스(`compiler.bmb` 48K)에서 계약(`pre`/`post`)의 완전성률·등가 dedup 기회·폴백률·Z3 레이턴시를 측정하는 `bmb contract-probe` 명령을 만든다. (설계 문서 §1~4의 go/no-go 측정 슬라이스)

**Architecture:** compiler.bmb에 이미 존재하는 3개 패턴을 확장한다 — ① `eff_z3_*`의 SMT2 방출 + `exec_with_stdin("z3","-smt2 -in",smt)` z3 호출 (46942-46954), ② `semdp_*`의 시그니처 버킷 쌍별 순회, ③ `semdp_build_json`의 JSON 출력 + 명령 디스패치(48933). 산술/bool 단편만 SMT2로 직렬화하고 나머지는 "폴백"으로 분류한다 (설계 §1.5 경계).

**Tech Stack:** BMB (bootstrap/compiler.bmb), z3 (-smt2 -in, 기존 호출 경로), Rust 참조 컴파일러(Stage 1 빌드용), 골든 테스트.

**범위 밖 (Phase 1b, 측정 게이팅 후 별도 계획):** `bmb dedup-check` 프로덕션 명령, `.bmb-registry` 영속화, MCP `bmb_dedup_check` 툴, 컴파일 시점 정규화, B 합성.

---

## 파일 구조

| 파일 | 책임 | 변경 |
|------|------|------|
| `bootstrap/compiler.bmb` | 프로브 로직 (SMT2 빌더, 완전성/등가 질의, 버킷팅, 측정 집계, 명령 디스패치) | Modify |
| `tests/golden/test_golden_contract_probe.bmb` | 골든 입력 (완전/부분/등가 계약 케이스) | Create |
| `claudedocs/measurements/contract_probe_self_2026-06-02.md` | compiler.bmb 측정 결과 (로컬 전용) | Create |

> compiler.bmb 변경이므로 Rule 3 (3-Stage 검증) 적용: 각 커밋 후 최소 Stage 1, 최종 Fixed Point 확인.

---

## Task 1: 계약 추출 인덱스 확인 (Discovery)

**목표:** SMT2 직렬화의 입력이 될 계약 표현이 컴파일 중 어떻게 인덱싱되는지 확정한다. `contracts-check`(48677 근처)와 `cc_*` 함수가 이미 pre/post를 다루므로 그 인덱스 포맷을 재사용한다.

**Files:**
- Read: `bootstrap/compiler.bmb` (contracts-check 경로)

- [ ] **Step 1: 계약 인덱스 함수 찾기**

Run:
```
grep -n "fn cc_\|contracts-check\|fn .*post\|fn .*precond\|post_of\|pre_of" bootstrap/compiler.bmb | head -40
```
Expected: `cc_*` 또는 contracts-check가 함수별 `(name, pre_text, post_text, signature)`를 담은 엔트리 인덱스(예: `"name\tpre\tpost\tsig\n"`)를 빌드하는 위치 발견.

- [ ] **Step 2: 인덱스 포맷 1줄 샘플 확인**

Run:
```
grep -n "fn fn_index\|fn build_fn_index\|fn collect_fns\|\\\\t.*\\\\t.*post\|index_get_field\|callers_get_field" bootstrap/compiler.bmb | head -20
```
Expected: `semdp_outer`가 쓰는 entries 인덱스(`callers_get_field`로 필드 추출)의 빌더 위치. 계약 텍스트가 이 인덱스에 있는지, 별도 인덱스인지 확정.

- [ ] **Step 3: 발견 사항 기록**

`claudedocs/measurements/contract_probe_self_2026-06-02.md`에 다음을 기록 (이후 Task들이 이 필드명을 참조):
```
## 계약 인덱스 포맷 (Task 1 발견)
- 인덱스 빌더 함수: <함수명> (line <N>)
- 엔트리 라인 포맷: "<field0>\t<field1>\t...\n"
- 필드 추출: <callers_get_field 등> 사용
- pre 텍스트 필드 인덱스: <i>
- post 텍스트 필드 인덱스: <j>
- 시그니처(입력타입,출력타입) 필드 인덱스: <k> (없으면 "별도 추출 필요")
```

- [ ] **Step 4: 커밋**

```
git add claudedocs/measurements/contract_probe_self_2026-06-02.md
```
> 주의: `claudedocs/*`는 로컬 전용(.gitignore). 실제로는 추적되지 않으므로 `git add`가 무시될 수 있음 — 이 경우 파일만 남기고 커밋하지 않는다. compiler.bmb 변경이 없으므로 이 Task는 코드 커밋 없음.

---

## Task 2: 산술/bool 단편 → SMT2 직렬화

**목표:** 계약 표현식 텍스트(`it == (c == 32 or c == 9)`, `it >= 0` 등)를 SMT-LIB2 식으로 변환. 단편(`+ - * == != < > <= >= and or not` + i64 리터럴 + 식별자 + `it`)만 처리, 그 외 토큰(문자열, 함수 호출, `.method()`)이 나오면 `""`(번역 불가) 반환.

**Files:**
- Modify: `bootstrap/compiler.bmb` (eff_z3_* 근처, 46940 이후에 추가)
- Test: `tests/golden/test_golden_contract_probe.bmb` (Task 5에서 통합)

- [ ] **Step 1: 실패 테스트 작성 (단위 — 임시 main 프로브)**

`tests/golden/test_golden_contract_probe.bmb` 생성:
```bmb
// 계약 완전성/등가 프로브 골든 테스트
// is_ws: 완전 계약 (post가 출력 유일 결정)
fn is_ws(c: i64) -> bool
  pre c >= 0
  post it == (c == 32 or c == 9 or c == 10 or c == 13)
= c == 32 or c == 9 or c == 10 or c == 13;

// is_ws_dup: is_ws와 등가 (구현 다름, 계약 같음)
fn is_ws_dup(c: i64) -> bool
  pre c >= 0
  post it == (c == 32 or c == 9 or c == 10 or c == 13)
= { let r = c == 32; r or c == 9 or c == 10 or c == 13 };

// abs_part: 부분 계약 (post가 출력 결정 못 함)
fn abs_part(x: i64) -> i64
  post it >= 0
= if x >= 0 { x } else { 0 - x };

fn main() -> i64 = 0;
```

- [ ] **Step 2: SMT2 식 변환 함수 작성**

compiler.bmb에 추가 (`eff_z3_gen_call_edge` 인접):
```bmb
// 계약 표현식 텍스트를 SMT-LIB2 식으로 변환.
// 산술/bool 단편만. 변환 불가 토큰 만나면 "" 반환 (폴백 신호).
// 토큰화는 기존 lexer 재사용 불가(텍스트 단편) → 단순 재귀하강.
// 반환: SMT2 prefix 식 문자열, 또는 "" (불가)
fn cp_expr_to_smt(expr: String) -> String
  post it.len() >= 0
= {
    let t = str_trim(expr);
    // 'it' 와 식별자, 정수 리터럴, 괄호식, 이항 연산만 처리.
    // 구현: 최상위 연산자 분해 (or > and > 비교 > +- > */ 우선순위).
    // 분해 불가(문자열 리터럴 '"', '.', 함수호출 '(' 뒤 식별자)면 "".
    cp_parse_or(t)
};
```
> Step 2의 `cp_parse_or`/`cp_parse_and`/`cp_parse_cmp`/`cp_parse_add`/`cp_parse_atom` 5개 하강 함수는 Task 1에서 확인한 계약 텍스트 포맷에 맞춰 작성한다. 각 함수는 우선순위 레벨의 연산자를 좌측부터 분리하고, 양변을 SMT2 prefix(`(or A B)`, `(>= A B)`, `(+ A B)`)로 조립한다. atom은 정수 리터럴 → 그대로, 식별자/`it` → 그대로(심볼), `(` → 괄호 매칭 후 재귀, 그 외 → `""`.

- [ ] **Step 3: 단편 외 토큰 폴백 확인**

`cp_parse_atom`에서 `"` (문자열), `.` (메서드), `str_`/known-fn-call 패턴을 만나면 `""` 반환하도록 보장. 상위 함수는 하위가 `""`면 즉시 `""` 전파.

- [ ] **Step 4: Stage 1 빌드 + 골든 컴파일 통과 확인**

Run:
```
cargo build --release
./target/release/bmb check tests/golden/test_golden_contract_probe.bmb
```
Expected: check PASS (파서/타입 통과). 아직 프로브 로직은 호출 안 됨.

- [ ] **Step 5: 커밋**

```
git add bootstrap/compiler.bmb tests/golden/test_golden_contract_probe.bmb
git commit -m "feat(probe): arithmetic/bool contract expr -> SMT2 serialization"
```

---

## Task 3: 완전성 질의 (∃!it)

**목표:** post를 SMT2로 변환 가능할 때, `∀x,it1,it2. pre ∧ post(x,it1) ∧ post(x,it2) → it1==it2` 의 부정이 UNSAT인지 z3로 확인. UNSAT = 완전. 변환 불가 = 폴백.

**Files:**
- Modify: `bootstrap/compiler.bmb`

- [ ] **Step 1: 완전성 SMT2 빌더 작성**

`eff_z3` z3 호출 패턴(46942-46954) 복제:
```bmb
// 완전성: pre, post 텍스트와 입력 심볼들로 SMT2 구성.
// it1,it2 두 출력이 같은 post를 만족하면서 다르면 SAT(=불완전).
// UNSAT => 완전. 변환 불가 => "fallback".
fn cp_check_complete(pre_txt: String, post_txt: String, sig_decls: String) -> String
  post it.len() >= 1
= {
    let post_smt1 = cp_expr_to_smt(cp_subst(post_txt, "it", "it1"));
    let post_smt2 = cp_expr_to_smt(cp_subst(post_txt, "it", "it2"));
    if post_smt1 == "" or post_smt2 == "" { "fallback" }
    else {
      let pre_smt = if pre_txt == "" { "true" } else { cp_expr_to_smt(pre_txt) };
      if pre_smt == "" { "fallback" }
      else {
        let sb = sb_new();
        let _d = sb_push(sb, sig_decls);          // (declare-const x Int) 등 + it1,it2
        let _p = sb_push(sb, "(assert " + pre_smt + ")\n");
        let _a1 = sb_push(sb, "(assert " + post_smt1 + ")\n");
        let _a2 = sb_push(sb, "(assert " + post_smt2 + ")\n");
        let _ne = sb_push(sb, "(assert (not (= it1 it2)))\n");
        let _c = sb_push(sb, "(check-sat)\n");
        let raw = exec_with_stdin("z3", "-smt2 -in", sb_build(sb));
        if str_contains_s(raw, "unsat") == 1 { "complete" } else { "partial" }
      }
    }
};
```
> `sig_decls`는 입력 파라미터와 `it1`,`it2`의 `(declare-const ...)`. bool 출력이면 `Bool`, i64면 `Int`. `cp_subst`는 단순 토큰 치환(`it` → `it1`). `str_contains_s`는 기존 문자열 contains 빌트인 (Task 1에서 정확한 이름 확인 — `str_contains`는 i64 반환이므로 `== 1` 가드).

- [ ] **Step 2: 시그니처 선언 빌더 작성**

```bmb
// 입력 타입과 출력 타입으로 (declare-const ...) 생성.
// 출력은 it1, it2 둘 다 선언.
fn cp_sig_decls(params: String, ret_type: String) -> String
  post it.len() >= 0
= {
    let it_sort = if ret_type == "bool" { "Bool" } else { "Int" };
    let p = cp_param_decls(params);  // 각 파라미터 "(declare-const <name> Int)\n"
    p + "(declare-const it1 " + it_sort + ")\n" + "(declare-const it2 " + it_sort + ")\n"
};
```
> `cp_param_decls`는 Task 1의 시그니처 필드 포맷에 맞춰 파라미터명·타입을 추출해 선언. String/기타 타입 파라미터가 있으면 `cp_expr_to_smt`가 어차피 폴백되므로 Int로 선언해도 무해(질의가 폴백됨). 단순화: 모든 파라미터 Int 선언, bool 파라미터만 Bool.

- [ ] **Step 3: z3 가용성 가드**

z3 미설치 시 `exec_with_stdin`이 빈/에러 반환. `raw`가 "sat"/"unsat" 모두 없으면 "fallback" 반환하도록 Step 1 마지막 분기 수정:
```bmb
if str_contains_s(raw, "unsat") == 1 { "complete" }
else if str_contains_s(raw, "sat") == 1 { "partial" }
else { "fallback" }
```

- [ ] **Step 4: Stage 1 빌드 확인**

Run: `cargo build --release && ./target/release/bmb check tests/golden/test_golden_contract_probe.bmb`
Expected: PASS

- [ ] **Step 5: 커밋**

```
git add bootstrap/compiler.bmb
git commit -m "feat(probe): completeness query (exists-unique it) via z3"
```

---

## Task 4: 등가 질의 (⟺) + 시그니처 버킷

**목표:** 같은 시그니처 버킷 내 완전 계약 쌍에 대해 `∀x. pre → (post_a ⟺ post_b)` 의 부정이 UNSAT인지 확인. UNSAT = 등가(dedup 기회).

**Files:**
- Modify: `bootstrap/compiler.bmb`

- [ ] **Step 1: 등가 SMT2 빌더 작성**

```bmb
// 두 완전 계약의 등가 검사. UNSAT(부정) => 등가.
fn cp_check_equiv(pre_txt: String, post_a: String, post_b: String, sig_decls1: String) -> String
  post it.len() >= 1
= {
    let sa = cp_expr_to_smt(post_a);
    let sb_ = cp_expr_to_smt(post_b);
    if sa == "" or sb_ == "" { "fallback" }
    else {
      let pre_smt = if pre_txt == "" { "true" } else { cp_expr_to_smt(pre_txt) };
      if pre_smt == "" { "fallback" }
      else {
        let q = sb_new();
        let _d = sb_push(q, sig_decls1);   // 파라미터 + it (단일) 선언
        let _p = sb_push(q, "(assert " + pre_smt + ")\n");
        // 부정: pre 하에 post_a XOR post_b 가 성립하면 SAT(=비등가)
        let _x = sb_push(q, "(assert (not (= " + sa + " " + sb_ + ")))\n");
        let _c = sb_push(q, "(check-sat)\n");
        let raw = exec_with_stdin("z3", "-smt2 -in", sb_build(q));
        if str_contains_s(raw, "unsat") == 1 { "equiv" }
        else if str_contains_s(raw, "sat") == 1 { "distinct" }
        else { "fallback" }
      }
    }
};
```
> 여기 `sig_decls1`은 `it`을 단일 선언(it1/it2 아님). `cp_sig_decls`에 변종 추가 또는 파라미터화.

- [ ] **Step 2: 시그니처 버킷 키 함수**

```bmb
// 버킷 키 = 입력 타입 시퀀스 + 출력 타입. 같은 키끼리만 등가 비교.
fn cp_bucket_key(params: String, ret_type: String) -> String
  post it.len() >= 0
= cp_param_types(params) + "->" + ret_type;
```
> `cp_param_types`는 파라미터 타입만 추출(`i64,i64` 등). Task 1 인덱스 포맷 사용.

- [ ] **Step 3: 구문 정규형 선통과 (Z3 전 자명 동일 제거)**

```bmb
// post 텍스트를 공백 정규화. 정규형이 같으면 Z3 없이 등가.
fn cp_canon(txt: String) -> String
  post it.len() >= 0
= str_replace_all(str_trim(txt), "  ", " ");
```
> 등가 비교 루프에서 `cp_canon(post_a) == cp_canon(post_b)`면 Z3 호출 생략(레이턴시 절감). 단 heap string `==` 주의(CLAUDE.md): 정규형 비교는 `cp_str_eq`(byte_at 루프) 헬퍼로. Task 1에서 기존 문자열 동등 헬퍼 확인 후 재사용.

- [ ] **Step 4: Stage 1 빌드 확인**

Run: `cargo build --release && ./target/release/bmb check tests/golden/test_golden_contract_probe.bmb`
Expected: PASS

- [ ] **Step 5: 커밋**

```
git add bootstrap/compiler.bmb
git commit -m "feat(probe): equivalence query + signature bucketing + syntactic canon prepass"
```

---

## Task 5: 측정 집계 + `contract-probe` 명령

**목표:** 파일을 받아 모든 계약 함수를 순회하며 완전성/등가/폴백을 집계하고 JSON으로 출력하는 `bmb contract-probe <file>` 명령. `semdp_build_json` + 디스패치(48933) 패턴 복제.

**Files:**
- Modify: `bootstrap/compiler.bmb`

- [ ] **Step 1: 집계 JSON 빌더 작성**

```bmb
// 4지표 집계: 완전 N, 부분 N, 폴백 N, 등가 클러스터 쌍 N, z3 호출 수.
fn cp_build_json(input: String, entries: String) -> String
  post it.len() >= 1
= {
    // 1패스: 각 함수 완전성 판정 → "name\tcomplete|partial|fallback\tpost\tbucket\n"
    let classified = cp_classify_all(entries, 0, sb_new());
    // 2패스: 버킷별 완전 계약 쌍 등가 검사 → 등가 쌍 수
    let equiv_pairs = cp_count_equiv(classified, 0, 0);
    let n_complete = cp_count_tag(classified, "complete");
    let n_partial = cp_count_tag(classified, "partial");
    let n_fallback = cp_count_tag(classified, "fallback");
    let out = sb_new();
    let _h = sb_push(out, "{\"type\":\"contract_probe\",\"file\":\"" + json_esc(input) + "\"");
    let _c = sb_push(out, ",\"complete\":" + int_to_string(n_complete));
    let _pp = sb_push(out, ",\"partial\":" + int_to_string(n_partial));
    let _fb = sb_push(out, ",\"fallback\":" + int_to_string(n_fallback));
    let _eq = sb_push(out, ",\"equiv_pairs\":" + int_to_string(equiv_pairs) + "}");
    sb_build(out)
};
```
> `cp_classify_all`, `cp_count_equiv`, `cp_count_tag`는 `semdp_outer`/`semdp_inner` 순회 패턴을 그대로 복제. `cp_count_equiv`는 버킷 키가 같고 둘 다 complete인 쌍만 `cp_check_equiv` 호출.

- [ ] **Step 2: 명령 디스패치 추가**

48933의 `semantic-duplicate` 분기 옆에 추가:
```bmb
else if cmd == "contract-probe" and argc >= 3 { contract_probe_file(get_arg(2)) }
```
그리고 `semantic_duplicate_file` 패턴(48226) 복제한 `contract_probe_file`:
```bmb
fn contract_probe_file(input: String) -> i64
  post it >= 0
= {
    let src = read_file(input);
    if src.len() == 0 { println_str("{\"type\":\"contract_probe\",\"error\":\"empty_or_missing\"}"); 1 }
    else {
        let entries = build_fn_index(src);  // Task 1에서 확인한 인덱스 빌더 (정확한 이름 사용)
        println_str(cp_build_json(input, entries));
        0
    }
};
```

- [ ] **Step 3: help 텍스트 추가**

49119 근처 명령 목록에:
```bmb
println_str("  contract-probe <file>         Contract completeness/equivalence metrics (JSON)");
```

- [ ] **Step 4: Stage 1 빌드 + 골든 실행**

Run:
```
cargo build --release
./target/release/bmb run bootstrap/compiler.bmb -- contract-probe tests/golden/test_golden_contract_probe.bmb
```
Expected (z3 설치 시): `{"type":"contract_probe","file":"...","complete":2,"partial":1,"fallback":0,"equiv_pairs":1}`
(is_ws + is_ws_dup = complete & equiv 1쌍, abs_part = partial)

- [ ] **Step 5: 골든 테스트 등록 + 커밋**

기존 골든 테스트 하니스에 expected 출력 등록 (Task 1에서 골든 등록 방식 확인 — 통상 `tests/golden/` + 기대값 파일). 그 후:
```
git add bootstrap/compiler.bmb tests/golden/test_golden_contract_probe.bmb
git commit -m "feat(probe): contract-probe command + 4-metric JSON aggregation"
```

---

## Task 6: BMB 자기 측정 + 3-Stage 검증

**목표:** `compiler.bmb`(48K) 전체에 프로브를 돌려 4지표를 얻고, 부트스트랩 Fixed Point를 확인한다.

**Files:**
- Modify: `claudedocs/measurements/contract_probe_self_2026-06-02.md` (로컬 전용)

- [ ] **Step 1: compiler.bmb 자기 측정 (레이턴시 포함)**

Run (PowerShell):
```
$t = Measure-Command { ./target/release/bmb run bootstrap/compiler.bmb -- contract-probe bootstrap/compiler.bmb | Tee-Object -Variable out }
$out; "elapsed: $($t.TotalSeconds)s"
```
Expected: JSON 4지표 + wall-clock. 기록.

- [ ] **Step 2: 완전성률·폴백률·dedup기회 계산 + 결과 시나리오 판정**

측정 노트에 기록:
```
## compiler.bmb 자기 측정 (2026-06-02)
- complete: <N>, partial: <N>, fallback: <N>, equiv_pairs: <N>
- 완전성률 = complete / (complete+partial+fallback) = <%>
- 폴백률 = fallback / total = <%>
- z3 wall-clock (48K): <초>
- 판정: [dedup기회多+레이턴시수용→Phase2 / 완전성률低→완전계약 선행과제 / 레이턴시高→캐싱필요] 중 택1 + 근거
```

- [ ] **Step 3: 3-Stage Fixed Point 검증 (Rule 3)**

Run:
```
./target/release/bmb build bootstrap/compiler.bmb --emit-ir -o s3.ll
# Stage 1 바이너리로 self-compile (rebuild-bootstrap-exe.sh 또는 기존 부트스트랩 스크립트)
./scripts/bootstrap.sh
```
Expected: Stage 2 IR == Stage 3 IR (Fixed Point). golden 테스트 회귀 0.

- [ ] **Step 4: cargo 회귀 확인**

Run: `cargo test --release`
Expected: 기존 PASS 수 유지 (6,282 기준), 0 FAIL.

- [ ] **Step 5: 커밋 (compiler.bmb 변경 있으면)**

```
git add bootstrap/compiler.bmb
git commit -m "chore(probe): finalize contract-probe after self-measurement + fixed-point verify"
```
> 측정 노트(`claudedocs/*`)는 로컬 전용이라 커밋하지 않음.

---

## Self-Review (작성자 체크)

- **Spec 커버리지**: §1 완전성/등가 정의 → Task 3/4. §2 dedup-check 흐름 → Phase 1b(범위 밖, 의도적). §3 옵션 B SMT2 → Task 2/3/4. §4 측정 하니스 4지표+버킷+정규형 → Task 4/5/6. §5 그래프 스키마 → Phase 1b. ✅ Phase 1a는 측정(§4)에 집중, 프로덕션(§2 명령·MCP·레지스트리)은 게이팅 후로 정당하게 분리.
- **Placeholder 스캔**: Task 1은 명시적 discovery(실제 grep 명령+산출물)이며 "TBD" 아님. Task 2의 5개 하강 함수와 Task 1 의존 필드명은 "발견값 사용"으로 시퀀싱 — 탐색적 부트스트랩 기능 특성상 정당. z3 식 빌더 코드는 기존 eff_z3 + 46954 패턴에 grounded.
- **타입 일관성**: `cp_expr_to_smt`/`cp_check_complete`/`cp_check_equiv`/`cp_sig_decls`/`cp_bucket_key`/`cp_build_json`/`contract_probe_file` 시그니처 Task 간 일치. `str_contains` i64 반환 → `== 1` 가드 일관 적용. heap string `==` 주의 → `cp_str_eq` byte_at 패턴 명시.

## 미해결 위험 (실행 중 모니터)

1. **계약 인덱스에 시그니처/타입이 없을 수 있음** (Task 1 결과 의존) — 없으면 파라미터 타입 추출 헬퍼를 추가 작성. 버킷팅 단순화: 타입 추출 실패 시 출력 타입만으로 버킷.
2. **z3 레이턴시 폭발** — 정규형 선통과 + 버킷팅으로 1차 방어. 48K에서 분 단위면 Task 6 Step 2에서 "레이턴시高" 판정 → Phase 1b 전에 proof 캐싱(incremental.rs 연계) 설계 필요.
3. **bootstrap 재귀 깊이** — `cp_classify_all`/`cp_count_equiv`가 semdp 패턴(재귀 순회)이므로 대형 입력에서 스택. semdp가 48K에서 동작하므로 동일 패턴이면 안전하나, 중첩 쌍 루프는 O(n²) 재귀 → 버킷 분할이 깊이도 줄임.
