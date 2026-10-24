//! BMB v2 compiler. Milestone 1: parse, check, prove.

pub mod ast;
pub mod check;
pub mod codegen;
pub mod diag;
pub mod fmt;
pub mod lexer;
pub mod map;
pub mod parser;
pub mod sha256;
pub mod smt;
pub mod source;
pub mod toolchain;
pub mod verify;

use std::collections::BTreeMap;

use diag::Diags;
use source::SourceMap;

/// Result of checking a set of source files.
pub struct CheckOutput {
    pub ok: bool,
    /// Diagnostics as deterministic JSONL.
    pub diagnostics: String,
}

/// Result of checking and proving a set of source files.
pub struct VerifyOutput {
    pub ok: bool,
    /// Diagnostics, then one status line per definition, as deterministic JSONL.
    pub output: String,
    /// SMT-LIB script per definition (empty when checking failed).
    pub scripts: BTreeMap<String, String>,
}

struct Frontend {
    sm: SourceMap,
    diags: Diags,
    program: ast::Program,
    tables: verify::Tables,
}

fn frontend(files: &[(String, String)]) -> Frontend {
    let mut sm = SourceMap::default();
    let mut diags = Diags::default();
    let mut program = ast::Program { mods: Vec::new() };
    for (name, text) in files {
        let id = sm.add(name.clone(), text.clone());
        let toks = lexer::lex(id, text, &mut diags);
        let mut p = parser::Parser::new(toks, &mut diags);
        program.mods.extend(p.parse_program().mods);
    }
    let mut c = check::Checker::new(&mut diags);
    c.check_program(&program);
    let tables = c.into_tables();
    Frontend {
        sm,
        diags,
        program,
        tables,
    }
}

/// Parses and checks the given files as one program.
pub fn check_sources(files: &[(String, String)]) -> CheckOutput {
    let fe = frontend(files);
    CheckOutput {
        ok: !fe.diags.has_errors(),
        diagnostics: fe.diags.to_jsonl(&fe.sm),
    }
}

/// Parses, checks and proves the given files. Proof runs only on a program
/// that checks cleanly.
pub fn verify_sources(files: &[(String, String)]) -> VerifyOutput {
    let mut fe = frontend(files);
    if fe.diags.has_errors() {
        return VerifyOutput {
            ok: false,
            output: fe.diags.to_jsonl(&fe.sm),
            scripts: BTreeMap::new(),
        };
    }
    let outcome = verify::verify_program(&fe.program, &fe.tables, &mut fe.diags);
    let mut output = fe.diags.to_jsonl(&fe.sm);
    output.push_str(&status_lines(&outcome));
    VerifyOutput {
        ok: !fe.diags.has_errors() && outcome.statuses.iter().all(|s| s.failed == 0),
        output,
        scripts: outcome.scripts,
    }
}

/// Result of `emit_llvm_sources`.
pub struct LlvmOutput {
    /// Diagnostics and statuses, as from `verify_sources`.
    pub verify: VerifyOutput,
    /// LLVM IR, when the program checks (proof may be partial).
    pub ir: Option<String>,
    /// Runtime checks kept because their obligation is unproved.
    pub checks: u32,
    pub has_main: bool,
}

/// Checks, proves and lowers to LLVM IR. IR is produced even when some
/// obligations are unproved; those keep runtime checks. Building a binary
/// additionally requires `verify.ok`.
pub fn emit_llvm_sources(files: &[(String, String)]) -> LlvmOutput {
    let mut fe = frontend(files);
    if fe.diags.has_errors() {
        return LlvmOutput {
            verify: VerifyOutput {
                ok: false,
                output: fe.diags.to_jsonl(&fe.sm),
                scripts: BTreeMap::new(),
            },
            ir: None,
            checks: 0,
            has_main: false,
        };
    }
    let outcome = verify::verify_program(&fe.program, &fe.tables, &mut fe.diags);
    let module = codegen::emit(&fe.program, &fe.tables, &outcome.proven);
    let mut output = fe.diags.to_jsonl(&fe.sm);
    output.push_str(&status_lines(&outcome));
    LlvmOutput {
        verify: VerifyOutput {
            ok: !fe.diags.has_errors() && outcome.statuses.iter().all(|s| s.failed == 0),
            output,
            scripts: outcome.scripts,
        },
        ir: Some(module.ir),
        checks: module.checks,
        has_main: module.main.is_some(),
    }
}

fn status_lines(outcome: &verify::Outcome) -> String {
    let mut out = String::new();
    for s in &outcome.statuses {
        let status = if s.failed == 0 { "verified" } else { "failed" };
        out.push_str(&format!(
            "{{\"def\":{},\"status\":\"{status}\",\"obligations\":{},\"failed\":{}}}\n",
            diag::json_str(&s.path),
            s.obligations,
            s.failed
        ));
    }
    out
}

/// Normal form of a program (ADR-0019): top-level unit name -> its text.
/// `Err` carries diagnostics when the program does not check.
pub fn format_sources(files: &[(String, String)]) -> Result<BTreeMap<String, String>, String> {
    let fe = frontend(files);
    if fe.diags.has_errors() {
        return Err(fe.diags.to_jsonl(&fe.sm));
    }
    Ok(fmt::format_units(&fe.program))
}

/// Definition hashes as JSONL (ADR-0019 §2).
pub fn hash_sources(files: &[(String, String)]) -> Result<String, String> {
    let fe = frontend(files);
    if fe.diags.has_errors() {
        return Err(fe.diags.to_jsonl(&fe.sm));
    }
    let mut out = String::new();
    for h in fmt::hashes(&fe.program, &fe.tables.fns) {
        out.push_str(&format!(
            "{{\"def\":{},\"body\":\"{}\",\"iface\":\"{}\"}}\n",
            diag::json_str(&h.path),
            h.body,
            h.iface
        ));
    }
    Ok(out)
}

/// Builds the maps and answers a request against them (ADR-0020).
/// `request` is `["map", level]` or `["query", name, args..]`.
pub fn map_request(files: &[(String, String)], request: &[String]) -> Result<String, String> {
    let mut fe = frontend(files);
    if fe.diags.has_errors() {
        return Err(fe.diags.to_jsonl(&fe.sm));
    }
    let outcome = verify::verify_program(&fe.program, &fe.tables, &mut fe.diags);
    let model = map::Model::build(&fe.program, &fe.tables, &fe.sm, &outcome);
    match request.first().map(String::as_str) {
        Some("map") => {
            let level = request.get(1).map(String::as_str).unwrap_or("all");
            if !matches!(level, "L0" | "L1" | "L2" | "all") {
                return Err(format!("unknown map level `{level}`"));
            }
            Ok(model.map(level))
        }
        Some("query") => {
            let name = request.get(1).ok_or("missing query name")?;
            model
                .query(name, &request[2..])
                .map(|body| format!("{}\n{body}", map::HEADER))
        }
        _ => Err("expected `map` or `query`".into()),
    }
}
