// T2-10 measurement driver: copy to compiler/tests/zz_cost.rs (see README.md).
// The programs are those of `proofs_are_not_vacuous` (tests/verify.rs) plus bench/game.
use std::fs;
use std::path::{Path, PathBuf};
fn root() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("tests") }
fn bmb_files(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(dir).unwrap().map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "bmb")).collect();
    v.sort(); v
}
fn name(p: &Path) -> String {
    let parent = p.parent().unwrap().file_name().unwrap().to_str().unwrap();
    format!("{parent}/{}", p.file_name().unwrap().to_str().unwrap())
}
fn programs(fail: bool) -> Vec<(String, Vec<(String, String)>)> {
    let mut programs: Vec<(String, Vec<(String, String)>)> = Vec::new();
    let one = |p: &Path| (name(p), vec![(name(p), fs::read_to_string(p).unwrap())]);
    if fail {
        programs.extend(bmb_files(&root().join("verify").join("fail")).iter().map(|p| one(p)));
        return programs;
    }
    for dir in [root().join("verify").join("pass"), root().join("run")] {
        programs.extend(bmb_files(&dir).iter().map(|p| one(p)));
    }
    let bench = root().join("..").join("bench");
    programs.extend(bmb_files(&bench.join("baseline")).iter().map(|p| one(p)));
    for set in ["m1", "m1b", "bvb"] {
        let src = bench.join(set).join("seed").join("src");
        let files = bmb_files(&src).iter().map(|p| (name(p), fs::read_to_string(p).unwrap())).collect();
        programs.push((format!("{set}/seed"), files));
    }
    let libs: Vec<(String, String)> = bmb_files(&bench.join("..").join("lib")).iter()
        .map(|p| (name(p), fs::read_to_string(p).unwrap())).collect();
    for lib in &libs { programs.push((lib.0.clone(), vec![lib.clone()])); }
    for dir in [root().join("lib_run"), bench.join("game")] {
        for p in bmb_files(&dir) {
            let mut files = vec![(name(&p), fs::read_to_string(&p).unwrap())];
            files.extend(libs.iter().cloned());
            programs.push((name(&p), files));
        }
    }
    programs
}
fn frontend(files: &[(String, String)]) -> Option<(bmb::ast::Program, bmb::verify::Tables)> {
    let mut sm = bmb::source::SourceMap::default();
    let mut diags = bmb::diag::Diags::default();
    let mut program = bmb::ast::Program { mods: Vec::new() };
    for (name, text) in files {
        let id = sm.add(name.clone(), text.clone());
        let toks = bmb::lexer::lex(id, text, &mut diags);
        let mut p = bmb::parser::Parser::new(toks, &mut diags, text);
        program.mods.extend(p.parse_program().mods);
    }
    let mut c = bmb::check::Checker::new(&mut diags);
    c.check_program(&program);
    let tables = c.into_tables();
    if diags.has_errors() { return None; }
    Some((program, tables))
}
#[test]
#[ignore]
fn cost() {
    let which = std::env::var("COST_CHECK").unwrap();
    let fail = std::env::var("COST_FAIL").is_ok();
    let only = std::env::var("COST_ONLY").ok();
    for (label, files) in programs(fail) {
        if only.as_ref().is_some_and(|o| !label.contains(o.as_str())) { continue; }
        unsafe { std::env::set_var("Z3_LABEL", &label); }
        let Some((p, t)) = frontend(&files) else { println!("{label}: does not check"); continue };
        if which == "vac" {
            for (path, _, what) in bmb::verify::vacuous_ends(&p, &t) {
                println!("VAC {label} {path}: {what}");
            }
        } else {
            for (path, _, v) in bmb::verify::preconditions_satisfiable(&p, &t) {
                println!("PRE {label} {path} {v:?}");
            }
        }
    }
}
