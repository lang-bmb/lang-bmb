#!/usr/bin/env python3
"""Progress report tables (dev-docs/reports/progress.md).

  progress.py                 print the generated block
  progress.py --write FILE    replace the generated block in FILE
  progress.py --write-roadmap replace the phase status block of docs/ROADMAP.md
                              (backlog counts per phase, INF-31)

The report is one file updated in place; earlier versions are its git history.
Every value comes from a file in the repository (the health baseline, the
backlog, the roadmap, bench results, the utility report, generated docs), so
two runs on the same commit give the same tables. The "지난 보고" column is
the value in the committed version of FILE (`git show HEAD:FILE`), so running
--write again before committing does not lose the comparison.

A source whose format no longer parses is an error, not an empty cell: the
gate runs scripts/test_progress.py so the report cannot silently rot.
Standard library only.
"""
import datetime
import json
import re
import subprocess
import sys
from pathlib import Path

import backlog

ROOT = Path(__file__).resolve().parent.parent
BEGIN = "<!-- progress:data begin (scripts/progress.py --write; do not edit by hand) -->"
END = "<!-- progress:data end -->"
REPORT = "dev-docs/reports/progress.md"

ID = r"(?:\d|INF|T\d)[A-Z]?-\d{2}[a-z]?"
ID_RE = re.compile(rf"^~?({ID})~?$")
MARKS = {"✅": "done", "🔄": "doing", "⏳": "todo", "🚫": "blocked"}


def read(rel):
    return (ROOT / rel).read_text(encoding="utf-8")


def cells(line):
    return [c.strip() for c in line.strip().strip("|").split("|")]


def fail(msg):
    raise SystemExit(f"progress.py: {msg}")


def num(v):
    return f"{v:,}" if isinstance(v, int) and not isinstance(v, bool) else str(v)


# ---------------------------------------------------------------- sources


def baseline():
    b = json.loads(read("dev-docs/health/baseline.json"))
    if not b.get("history"):
        fail("health baseline has no history")
    return b["metrics"], b["history"][-1]["date"]


def tasks():
    """Backlog rows: id -> status mark name (done/doing/todo/blocked/cancelled)."""
    out = {}
    for line in backlog.text(ROOT).splitlines():
        if not line.startswith("|"):
            continue
        c = cells(line)
        if len(c) != 5:
            continue
        m = ID_RE.match(c[0])
        if not m:
            continue
        if c[0].startswith("~"):
            out[m.group(1)] = "cancelled"
            continue
        out[m.group(1)] = next((v for k, v in MARKS.items() if c[4].startswith(k)), "other")
    if not out:
        fail("no backlog tasks parsed")
    return out


def phase(tid):
    return tid.split("-")[0]


def phase_counts(ts, prefixes):
    parts = []
    for p in prefixes:
        mine = [s for t, s in ts.items() if phase(t) == p and s != "cancelled"]
        if mine:
            parts.append(f"{p} {sum(s == 'done' for s in mine)}/{len(mine)}")
    return " · ".join(parts) if parts else "태스크 없음"


RM_BEGIN = "<!-- roadmap:status begin (scripts/progress.py --write-roadmap; do not edit by hand) -->"
RM_END = "<!-- roadmap:status end -->"
ROADMAP = "docs/ROADMAP.md"
STAGES = [
    ("0 착수·인프라", ["0", "INF"]),
    ("1 최소 관통 경로", ["1A", "1B", "1C", "1D", "1E", "1F"]),
    ("2 언어 확장", ["2A", "2B", "2C", "2D", "2E", "2F", "2G", "2S", "2V", "2T", "2O", "2K", "2U", "2N", "2M", "2R", "2H", "2P"]),
    ("트랙", ["T0", "T1", "T2", "T3", "T4"]),
    ("3 대형 프로젝트", ["3A", "3P"]),
    ("4 자립", ["4A", "4B", "4C", "4D", "4E", "4F", "4G"]),
    ("5 성숙·생태계", ["5A", "5B", "5C", "5D", "5E"]),
]


def roadmap_block(ts=None):
    """Phase status from the backlog: done/total and the open tasks by mark.
    The roadmap's prose says what a phase is for; this says where it stands."""
    ts = tasks() if ts is None else ts
    known = {p for _, ps in STAGES for p in ps}
    stray = sorted({phase(t) for t in ts} - known)
    if stray:
        fail(f"backlog phases not in progress.STAGES: {', '.join(stray)}")
    lines = [RM_BEGIN, "", "| 단계 | 페이즈 | ✅/전체 | 🔄 진행 | ⏳ 대기 | 🚫 막힘 |", "| --- | --- | --- | --- | --- | --- |"]
    for stage, ps in STAGES:
        for p in ps:
            mine = sorted((t, s) for t, s in ts.items() if phase(t) == p and s != "cancelled")
            if not mine:
                continue
            ids = lambda k: ", ".join(f"`{t}`" for t, s in mine if s == k) or "—"  # noqa: E731
            lines.append(f"| {stage} | {p} | {sum(s == 'done' for _, s in mine)}/{len(mine)} | "
                         f"{ids('doing')} | {ids('todo')} | {ids('blocked')} |")
    return "\n".join(lines + ["", RM_END])


def replace_roadmap(text, new):
    if RM_BEGIN not in text or RM_END not in text:
        fail("ROADMAP.md has no roadmap:status markers")
    return text[: text.index(RM_BEGIN)] + new + text[text.index(RM_END) + len(RM_END):]


def status_of(ts, tid):
    if tid not in ts:
        fail(f"backlog has no task {tid}")
    return {"done": "✅", "doing": "🔄", "todo": "⏳", "blocked": "🚫", "cancelled": "취소"}.get(ts[tid], ts[tid])


def ladder():
    """Self-hosting ladder (HEALTH §4): highest rung reached and the next one's state."""
    text = read("dev-docs/HEALTH.md")
    rows = [cells(l) for l in text.splitlines() if re.match(r"^\| L\d \|", l)]
    if not rows:
        fail("HEALTH.md has no ladder table")
    reached = None
    for r in rows:
        if r[2].startswith("✅"):
            reached = r[0]
        else:
            return reached or "없음", f"{r[0]}: {r[2]}"
    return reached, "—"


def hypotheses():
    text = read("docs/ROADMAP.md")
    order = ["확정", "반증", "결정론 근거", "측정 중", "파일럿", "미측정"]
    counts = dict.fromkeys(order, 0)
    names = {k: [] for k in order}
    for line in text.splitlines():
        if not line.startswith("| H-"):
            continue
        c = cells(line)
        k = next((k for k in order if c[2].startswith(k)), None)
        if k is None:
            fail(f"hypothesis {c[0]} has an unknown state {c[2][:20]!r}")
        counts[k] += 1
        names[k].append(c[0])
    if not sum(counts.values()):
        fail("ROADMAP has no hypothesis registry rows")
    return order, counts, names


def utility_axes():
    rows = []
    for line in read("dev-docs/reports/utility.md").splitlines():
        m = re.match(r"^### (2\.\d+) (.+?) — (.+)$", line)
        if m:
            rows.append((f"{m.group(1)} {m.group(2)}", m.group(3)))
    if not rows:
        fail("utility.md has no axis headings (### 2.N name — verdict)")
    return rows


def latest(dirname):
    files = sorted((ROOT / dirname).glob("20??-??-??.md"))
    if not files:
        fail(f"no dated results in {dirname}")
    return files[-1]


def ratio_rows(path, col):
    """Data rows of the results table and the ratio column `col`."""
    out = []
    header = None
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.startswith("|"):
            continue
        c = cells(line)
        if header is None:
            header = c
            if col not in header:
                fail(f"{path.name}: no column {col!r}")
            continue
        if set(c[0]) <= set("-: "):
            continue
        out.append((c, float(c[header.index(col)])))
    if not out:
        fail(f"{path.name}: no rows")
    return out


def diag_codes():
    n = sum(1 for l in read("docs/diagnostics.md").splitlines() if re.match(r"^\| [EW]\d{4} \|", l))
    if not n:
        fail("docs/diagnostics.md lists no codes")
    return n


def cli_commands():
    m = re.search(r'const USAGE: &str = "(.*?)";', read("compiler/src/main.rs"), re.S)
    text = m.group(1) if m else read("compiler/src/main.rs")
    names = sorted(set(re.findall(r"^\s*bmb ([a-z][a-z-]*)", text, re.M)))
    if not names:
        fail("no `bmb <command>` lines in compiler/src/main.rs")
    return names


def mcp_tools():
    text = read("compiler/src/mcp.rs")
    base, optional = [], []
    for name, cond in re.findall(r'^ {12}"([a-z_]+)"( if [a-z_.]+)? =>', text, re.M):
        if name in ("initialize", "ping"):
            continue
        (optional if cond else base).append(name)
    if not base:
        fail("no tool arms in compiler/src/mcp.rs")
    return base, optional


def library():
    files = sorted((ROOT / "lib" / "std").glob("*.bmb"))
    if not files:
        fail("no lib/std/*.bmb")
    lines = sum(len(f.read_text(encoding="utf-8").splitlines()) for f in files)
    return [f.stem for f in files], lines


def adrs():
    return len(list((ROOT / "dev-docs/adr").glob("[0-9][0-9][0-9][0-9]-*.md")))


def head():
    return subprocess.run(["git", "rev-parse", "--short=8", "HEAD"], cwd=ROOT,
                          capture_output=True, text=True, check=True).stdout.strip()


# ---------------------------------------------------------------- sections


def sections(today):
    m, accepted = baseline()
    ts = tasks()
    hb = f"health 기준선({accepted} accept)"

    def h(key):
        if key not in m:
            fail(f"health baseline has no {key}")
        return num(m[key])

    reached, nxt = ladder()
    boot = [
        ("자립 사다리 도달 칸", reached, "`dev-docs/HEALTH.md` §4"),
        ("다음 칸", nxt, "`dev-docs/HEALTH.md` §4"),
        ("4단계 태스크(✅/전체)", phase_counts(ts, ["4A", "4B", "4C", "4D", "4E", "4F", "4G"]), "백로그"),
        ("S1 외부 크레이트", h("S1_external_crates"), hb),
        ("S2 컴파일러가 실행하는 외부 프로세스", ", ".join(m["S2_external_processes"]), hb),
        ("S3 생성 바이너리의 libc 밖 의존", ", ".join(m["S3_runtime_libs_outside_libc"]) or "없음", hb),
        ("S4 stage0 Rust 줄 수", h("S4_stage0_rust_lines"), hb),
        ("S5 Rust API에 묶인 통합 시험 파일", h("S5_tests_bound_to_rust_api"), hb),
        ("S6 BMB로 쓴 컴파일러 줄 수", h("S6_selfhost_bmb_lines"), hb),
    ]

    counts = {k: sum(s == k for s in ts.values()) for k in ("done", "doing", "todo", "blocked")}
    lang = [
        ("2단계 페이즈 태스크(✅/전체)",
         phase_counts(ts, ["2A", "2B", "2C", "2D", "2E", "2F", "2G", "2S", "2T", "2O", "2K", "2U", "2N", "2M", "2R", "2V", "2H", "2P"]),
         "백로그"),
        ("트랙 태스크(✅/전체)", phase_counts(ts, ["T0", "T1", "T2", "T3", "T4", "INF"]), "백로그"),
        ("백로그 ✅ / 🔄 / ⏳ / 🚫",
         f"{counts['done']} / {counts['doing']} / {counts['todo']} / {counts['blocked']}", "백로그"),
        ("ADR 수", num(adrs()), "`dev-docs/adr/`"),
        ("진단 코드 수", num(diag_codes()), "`docs/diagnostics.md`"),
        ("V3 증명되지 않은 코퍼스·시드 정의", h("V3_unproved_defs"), hb),
        ("V4 코퍼스·시드 증명 비용(rlimit)", h("V4_rlimit_total"), hb),
        ("V5 가장 비싼 의무 / 한도", h("V5_rlimit_headroom"), hb),
        ("V7 판정이 흔들리는 의무 비율", h("V7_unstable_queries"), hb),
        ("V8 벤치 부하 증명 비용(rlimit)", h("V8_bench_rlimit_total"), hb),
        ("V10 라이브러리 증명 비용(rlimit)", h("V10_library_rlimit_total"), hb),
        ("V11 충족 가능성 검사 비용(rlimit)", h("V11_sat_check_rlimit_total"), hb),
        ("V12 충족 가능성 검사 미결정 수", h("V12_sat_check_undecided"), hb),
        ("O2 건전성 퍼저 수", h("O2_soundness_fuzzers"), hb),
        ("P1 기대 출력이 있는 실행 프로그램", h("P1_runnable_programs"), hb),
        ("P2 저장소의 BMB 줄 수", h("P2_bmb_lines"), hb),
        ("P3 고칠 길이 붙은 오류 진단 비율", h("P3_errors_with_help"), hb),
    ]

    base_file = latest("bench/baseline/results")
    base_rows = ratio_rows(base_file, "BMB/safe")
    rs = [r for _, r in base_rows]
    base_date = datetime.date.fromisoformat(base_file.stem)
    within = sum(r <= 1.05 for r in rs)
    lib_file = latest("bench/lib/results")
    lib_rows = ratio_rows(lib_file, "BMB/대상")
    perf = [
        ("기준 벤치 BMB/안전 Rust 시간 비(범위)", f"{min(rs):.3f} ~ {max(rs):.3f}",
         f"`bench/baseline/results/{base_file.name}`"),
        ("기준 벤치 G6(≤ 1.05) 충족", f"{within}/{len(rs)}", f"`bench/baseline/results/{base_file.name}`"),
        ("기준 벤치 마지막 측정", f"{base_date} ({(today - base_date).days}일 전)",
         f"`bench/baseline/results/{base_file.name}`"),
    ]
    for c, r in lib_rows:
        perf.append((f"라이브러리 {c[0]} 대 {c[1]} 시간 비", f"{r:.3f}", f"`bench/lib/results/{lib_file.name}`"))
    lib_date = datetime.date.fromisoformat(lib_file.stem)
    perf += [
        ("라이브러리 벤치 마지막 측정", f"{lib_date} ({(today - lib_date).days}일 전)",
         f"`bench/lib/results/{lib_file.name}`"),
        ("B1 기준 벤치 7개 strip 크기 합(바이트)", h("B1_baseline_binary_bytes"), hb),
        ("Benchmarks Game 공식 제출 비교(T3-01·T3-04)",
         f"{status_of(ts, 'T3-01')} · {status_of(ts, 'T3-04')}", "백로그"),
        ("야간 성능 회귀 추적(INF-12)", status_of(ts, "INF-12"), "백로그"),
        ("빌드 시간 관찰(INF-13)", status_of(ts, "INF-13"), "백로그"),
    ]

    mods, lib_lines = library()
    base_tools, opt_tools = mcp_tools()
    eco = [
        ("5단계 태스크(✅/전체)", phase_counts(ts, ["5A", "5B", "5C", "5D", "5E"]), "백로그"),
        ("표준 라이브러리 모듈", f"{len(mods)} ({', '.join(mods)})", "`lib/`"),
        ("표준 라이브러리 줄 수", num(lib_lines), "`lib/`"),
        ("CLI 명령 수", num(len(cli_commands())), "`compiler/src/main.rs`"),
        ("MCP 도구 수(기본 + 선택)", f"{len(base_tools)} + {len(opt_tools)}", "`compiler/src/mcp.rs`"),
    ]

    order, hc, names = hypotheses()
    mission = [(f"가설: {k}", f"{hc[k]}" + (f" ({', '.join(names[k])})" if names[k] and k != "미측정" else ""),
                "`docs/ROADMAP.md` 가설 등록부") for k in order]
    mission += [(f"효용 {name}", verdict, "`dev-docs/reports/utility.md` §2") for name, verdict in utility_axes()]

    return [("1. 부트스트랩", boot), ("2. 언어 개발 수준", lang), ("3. 성능", perf),
            ("4. 생태계", eco), ("5. 미션 효용", mission)]


# ---------------------------------------------------------------- rendering


def previous_values(text):
    """(section, item) -> value from a report's generated block."""
    m = re.search(re.escape(BEGIN) + r"(.*?)" + re.escape(END), text or "", re.S)
    out = {}
    if not m:
        return out
    sec = None
    for line in m.group(1).splitlines():
        if line.startswith("### "):
            sec = line[4:].strip()
        elif line.startswith("| ") and sec:
            c = cells(line)
            if len(c) == 5 and c[0] not in ("항목",) and not set(c[0]) <= set("-: "):
                out[(sec, c[0])] = c[1]
    return out


def to_number(v):
    s = v.replace(",", "")
    return float(s) if re.fullmatch(r"-?\d+(\.\d+)?", s) else None


def change(prev, cur):
    if prev is None:
        return "—"
    if prev == cur:
        return "같음"
    a, b = to_number(prev), to_number(cur)
    if a is None or b is None:
        return "바뀜"
    d = b - a
    if d == int(d) and "." not in prev and "." not in cur:
        return f"{int(d):+,}"
    return f"{d:+.4g}"


def esc(v):
    return str(v).replace("|", "\\|")


def block(prev_text=None, today=None):
    today = today or datetime.date.today()
    prev = previous_values(prev_text)
    out = [BEGIN, "", f"생성: {today} · 기준 커밋 `{head()}`" +
           (" · 지난 보고: 이 파일의 직전 커밋 판" if prev else " · 지난 보고 없음(첫 판)"), ""]
    for title, rows in sections(today):
        out += [f"### {title}", "", "| 항목 | 현재 | 지난 보고 | 변화 | 출처 |", "| --- | --- | --- | --- | --- |"]
        for item, cur, src in rows:
            cur = esc(cur)
            p = prev.get((title, item))
            out.append(f"| {item} | {cur} | {p if p is not None else '—'} | {change(p, cur)} | {src} |")
        out.append("")
    out.append(END)
    return "\n".join(out)


def committed(rel):
    r = subprocess.run(["git", "show", f"HEAD:{rel}"], cwd=ROOT, capture_output=True, text=True)
    return r.stdout if r.returncode == 0 else None


def replace(text, new):
    pat = re.compile(re.escape(BEGIN) + r".*?" + re.escape(END), re.S)
    if not pat.search(text):
        fail(f"no generated block ({BEGIN!r} ... {END!r})")
    return pat.sub(lambda _: new, text, count=1)


def main(argv=None):
    argv = sys.argv[1:] if argv is None else argv
    if argv[:1] == ["--write"]:
        if len(argv) != 2:
            fail("usage: progress.py --write FILE")
        path = Path(argv[1])
        rel = path.resolve().relative_to(ROOT).as_posix()
        path.write_text(replace(path.read_text(encoding="utf-8"), block(committed(rel))), encoding="utf-8")
        print(f"updated {argv[1]}")
        return 0
    if argv == ["--write-roadmap"]:
        path = ROOT / ROADMAP
        path.write_text(replace_roadmap(path.read_text(encoding="utf-8"), roadmap_block()), encoding="utf-8")
        print(f"updated {ROADMAP}")
        return 0
    if argv:
        fail("usage: progress.py [--write FILE | --write-roadmap]")
    print(block(committed(REPORT)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
