//! BMB v2 compiler. Milestone 1: parse, check, prove.

pub mod ast;
pub mod check;
pub mod diag;
pub mod lexer;
pub mod parser;
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
    for s in &outcome.statuses {
        let status = if s.failed == 0 { "verified" } else { "failed" };
        output.push_str(&format!(
            "{{\"def\":{},\"status\":\"{status}\",\"obligations\":{},\"failed\":{}}}\n",
            diag::json_str(&s.path),
            s.obligations,
            s.failed
        ));
    }
    VerifyOutput {
        ok: !fe.diags.has_errors() && outcome.statuses.iter().all(|s| s.failed == 0),
        output,
        scripts: outcome.scripts,
    }
}
