//! BMB v2 compiler. Milestone 1 front end: parse, check, report.

pub mod ast;
pub mod check;
pub mod diag;
pub mod lexer;
pub mod parser;
pub mod source;
pub mod toolchain;

use diag::Diags;
use source::SourceMap;

/// Result of checking a set of source files.
pub struct CheckOutput {
    pub ok: bool,
    /// Diagnostics as deterministic JSONL.
    pub diagnostics: String,
}

/// Parses and checks the given files as one program. File order does not affect
/// the result beyond the `file` field of diagnostics.
pub fn check_sources(files: &[(String, String)]) -> CheckOutput {
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
    CheckOutput {
        ok: !diags.has_errors(),
        diagnostics: diags.to_jsonl(&sm),
    }
}
