//! Nesting limits (backlog 1A-11): deep input is a diagnostic, not a crash, and
//! programs just inside the limit go through the whole pipeline on the stack
//! the CLI and the MCP server use (`bmb::with_stack`).

use bmb::parser::MAX_NESTING;

fn program(body: &str) -> Vec<(String, String)> {
    vec![(
        "deep.bmb".to_string(),
        format!(
            "#[summary(\"깊은 중첩\")]\nmod deep {{\n    #[effects(io)]\n    fn main() -> () {{\n        let x: i64 = {body};\n        print_i64(x)\n    }}\n}}\n"
        ),
    )]
}

fn parens(n: usize) -> String {
    format!("{}1{}", "(".repeat(n), ")".repeat(n))
}

fn ifs(n: usize) -> String {
    let mut s = String::from("1");
    for _ in 0..n {
        s = format!("if true {{ {s} }} else {{ 2 }}");
    }
    s
}

fn else_ifs(n: usize) -> String {
    format!(
        "{}1 }} else {{ 2 }}",
        "if false { 0 } else ".repeat(n) + "if true {"
    )
}

fn blocks(n: usize) -> String {
    format!("{}1{}", "{ ".repeat(n), " }".repeat(n))
}

fn negs(n: usize) -> String {
    format!("{}1", "- ".repeat(n))
}

/// `0 + 0 + .. + 1`: iterative to parse, but a left-nested tree.
fn sums(n: usize) -> String {
    format!("{}1", "0 + ".repeat(n))
}

/// `[[..[1]..][0]..][0]`: nested arrays indexed back down.
fn indexes(n: usize) -> String {
    format!("{}1{}{}", "[".repeat(n), "]".repeat(n), "[0]".repeat(n))
}

/// `1.wrapping_add(0).wrapping_add(0)..`: a left-nested method chain.
fn methods(n: usize) -> String {
    format!("1{}", ".wrapping_add(0)".repeat(n))
}

/// Left-nested `||` and nested `!` (no overflow obligations, so proving stays fast).
fn ors(n: usize) -> String {
    format!("if {}true {{ 1 }} else {{ 2 }}", "false || ".repeat(n))
}

fn nots(n: usize) -> String {
    format!("if {}true {{ 1 }} else {{ 2 }}", "!".repeat(n & !1))
}

fn implications(n: usize) -> String {
    // `==>` is spec-only; the checker rejects it here, but must not crash.
    format!("{}true", "true ==> ".repeat(n))
}

#[test]
fn deep_nesting_is_a_diagnostic() {
    bmb::with_stack(|| {
        let n = 50_000;
        for body in [
            parens(n),
            blocks(n),
            negs(n),
            sums(n),
            indexes(n),
            methods(n),
            implications(n),
            ifs(5_000),
            else_ifs(n),
        ] {
            let out = bmb::check_sources(&program(&body));
            assert!(!out.ok);
            assert!(out.diagnostics.contains("\"E0107\""), "{}", out.diagnostics);
        }
    });
}

#[test]
fn deep_types_and_units_are_a_diagnostic() {
    bmb::with_stack(|| {
        let n = 50_000;
        let ty = format!(
            "#[summary(\"t\")]\nmod t {{\n    #[effects(pure)]\n    fn f(a: &{}i64{}) -> () {{ () }}\n}}\n",
            "[".repeat(n),
            "]".repeat(n)
        );
        let units = format!(
            "{}{}",
            "#[summary(\"u\")] mod u { ".repeat(n),
            "}".repeat(n)
        );
        for text in [ty, units] {
            let out = bmb::check_sources(&[("deep.bmb".into(), text)]);
            assert!(!out.ok);
            assert!(out.diagnostics.contains("\"E0107\""), "{}", out.diagnostics);
        }
    });
}

#[test]
fn programs_inside_the_limit_pass_the_whole_pipeline() {
    bmb::with_stack(|| {
        // Each kind uses at most one level per syntactic level, plus a few for
        // the surrounding function.
        let n = MAX_NESTING - 8;
        let half = n / 2 - 1;
        for body in [
            parens(n),
            blocks(n),
            ors(n),
            nots(n),
            methods(n),
            ifs(half),
            else_ifs(n),
        ] {
            let files = program(&body);
            let out = bmb::check_sources(&files);
            assert!(out.ok, "{}", out.diagnostics);
            let v = bmb::verify_sources(&files);
            assert!(v.ok, "{}", v.output);
            let ir = bmb::emit_llvm_sources(&files);
            assert!(ir.ir.is_some());
            bmb::format_sources(&files).unwrap();
            bmb::map_request(&files, &["map".into(), "all".into()]).unwrap();
        }
    });
}
