//! The compact agent view of tool output (backlog 2D-23, 2D-25, ADR-0068).
//!
//! `bmb mcp --compact` passes every tool result through `compact`. The maps
//! and the CLI stay JSONL (ADR-0020); this is a projection of that text for
//! the model, computed from it alone, so it is deterministic and carries the
//! same answers:
//!
//! - map header: dropped.
//! - L1 surface: the definition's normal form without its body (attributes,
//!   signature, contract clauses), after a `// PATH` line with the proof
//!   status. Hashes, file, line and proof cost are not shown.
//! - L0: `#[summary("..")] mod PATH;` for a unit, `KIND PATH` for a definition.
//! - L2: `FROM KIND TO`, with ` arg N` and ` <TYPES>` when the edge has them.
//! - slice: the definition's text and each dependency's text under `// PATH`.
//! - verify: diagnostics as they are; the status lines of verified
//!   definitions become one `{"verified":N}` line, the others lose their
//!   proof cost.
//! - any other JSON row: without `level`, with empty arrays, `null` and empty
//!   strings left out and hashes cut to 12 hex digits (ADR-0019 §2).
//!
//! Lines that are not JSON objects pass through unchanged.

use crate::diag::json_str;
use crate::json::{self, Value};

/// Hash keys of map rows; shown cut to `sha256:` and 12 hex digits.
const HASH_KEYS: [&str; 4] = ["hash", "body", "iface", "id"];

/// The compact view of a tool result.
pub fn compact(text: &str) -> String {
    let mut out = String::new();
    let mut verified = 0usize;
    for line in text.lines() {
        let row = match json::parse(line) {
            Ok(v @ Value::Object(_)) => v,
            _ => {
                out.push_str(line);
                out.push('\n');
                continue;
            }
        };
        if row.get("schema").is_some() && row.get("version").is_some() {
            continue;
        }
        if is_status(&row) {
            if str_of(&row, "status") == "verified" {
                verified += 1;
            } else {
                out.push_str(&keep(&row, &["def", "status", "obligations", "failed"]).to_json());
                out.push('\n');
            }
            continue;
        }
        let rendered = match (
            row.get("level").and_then(Value::as_str),
            row.get("kind").and_then(Value::as_str),
        ) {
            (Some("L1"), _) => l1(&row),
            (Some("L0"), Some("unit")) => format!(
                "#[summary({})] mod {};\n",
                json_str(str_of(&row, "summary")),
                str_of(&row, "path")
            ),
            (Some("L0"), _) => format!("{} {}\n", str_of(&row, "def_kind"), str_of(&row, "path")),
            (Some("L2"), _) => l2(&row),
            _ if row.get("code").is_some() && row.get("deps").is_some() => slice(&row),
            _ if is_diagnostic(&row) => format!("{}\n", row.to_json()),
            _ => format!("{}\n", generic(&row).to_json()),
        };
        out.push_str(&rendered);
    }
    if verified > 0 {
        out.push_str(&format!("{{\"verified\":{verified}}}\n"));
    }
    out
}

fn str_of<'v>(row: &'v Value, key: &str) -> &'v str {
    row.get(key).and_then(Value::as_str).unwrap_or("")
}

fn num_of(row: &Value, key: &str) -> i64 {
    row.get(key).and_then(Value::as_i64).unwrap_or(0)
}

fn strs_of(row: &Value, key: &str) -> Vec<String> {
    match row.get(key) {
        Some(Value::Array(xs)) => xs
            .iter()
            .filter_map(|x| x.as_str().map(String::from))
            .collect(),
        _ => Vec::new(),
    }
}

/// A `verify` status line: `{"def","status","obligations","failed",..}`.
fn is_status(row: &Value) -> bool {
    row.get("def").is_some() && row.get("status").is_some() && row.get("obligations").is_some()
}

fn is_diagnostic(row: &Value) -> bool {
    row.get("code").is_some() && row.get("severity").is_some()
}

fn keep(row: &Value, keys: &[&str]) -> Value {
    match row {
        Value::Object(kv) => Value::Object(
            kv.iter()
                .filter(|(k, _)| keys.contains(&k.as_str()))
                .cloned()
                .collect(),
        ),
        other => other.clone(),
    }
}

/// `level` dropped, empty values left out, hashes cut to 12 hex digits.
fn generic(row: &Value) -> Value {
    let Value::Object(kv) = row else {
        return row.clone();
    };
    let empty = |v: &Value| match v {
        Value::Null => true,
        Value::Str(s) => s.is_empty(),
        Value::Array(xs) => xs.is_empty(),
        _ => false,
    };
    Value::Object(
        kv.iter()
            .filter(|(k, v)| k != "level" && !empty(v))
            .map(|(k, v)| match v {
                Value::Str(s) if HASH_KEYS.contains(&k.as_str()) && s.starts_with("sha256:") => {
                    (k.clone(), Value::Str(s.chars().take(7 + 12).collect()))
                }
                _ => (k.clone(), v.clone()),
            })
            .collect(),
    )
}

/// An L1 row as the definition's normal form without a body (ADR-0019 §1:
/// one attribute per line, each contract clause on its own line, four spaces,
/// items joined by commas with a trailing comma).
fn l1(row: &Value) -> String {
    let path = str_of(row, "path");
    let kind = str_of(row, "def_kind");
    let mut out = format!("// {path}");
    match str_of(row, "status") {
        "verified" => out.push_str(": verified"),
        "failed" => out.push_str(&format!(
            ": failed {} of {} obligations",
            num_of(row, "failed"),
            num_of(row, "obligations")
        )),
        _ => {}
    }
    out.push('\n');
    if kind == "fn" {
        let effects = str_of(row, "effects");
        if !effects.is_empty() {
            out.push_str(&format!("#[effects({effects})]\n"));
        }
    }
    let uses = strs_of(row, "uses");
    if !uses.is_empty() {
        out.push_str(&format!("#[uses({})]\n", uses.join(", ")));
    }
    out.push_str(str_of(row, "signature"));
    out.push('\n');
    for (clause, items) in [
        ("requires", strs_of(row, "requires")),
        ("ensures", strs_of(row, "ensures")),
    ] {
        if !items.is_empty() {
            out.push_str(&format!("    {clause} {},\n", items.join(", ")));
        }
    }
    let dec = str_of(row, "decreases");
    if !dec.is_empty() {
        out.push_str(&format!("    decreases {dec},\n"));
    }
    out
}

fn l2(row: &Value) -> String {
    let mut out = format!(
        "{} {} {}",
        str_of(row, "from"),
        str_of(row, "kind"),
        str_of(row, "to")
    );
    if let Some(a) = row.get("arg").and_then(Value::as_i64) {
        out.push_str(&format!(" arg {a}"));
    }
    let targs = str_of(row, "type_args");
    if !targs.is_empty() {
        out.push_str(&format!(" <{targs}>"));
    }
    out.push('\n');
    out
}

fn slice(row: &Value) -> String {
    let mut out = format!("// {}\n{}", str_of(row, "path"), str_of(row, "code"));
    if let Some(Value::Array(deps)) = row.get("deps") {
        for d in deps {
            out.push_str(&format!("// {}\n{}", str_of(d, "path"), str_of(d, "text")));
        }
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::compact;

    #[test]
    fn l1_row_is_the_bodiless_normal_form() {
        let row = r#"{"level":"L1","path":"app::report","def_kind":"fn","signature":"fn report(a: &[i64]) -> ()","requires":["report_ok(a)","stats::bounded_elems(a)"],"ensures":[],"decreases":null,"effects":"io","uses":["app::report_ok"],"status":"verified","obligations":5,"failed":0,"body":"sha256:1f00","iface":"sha256:4543","id":"sha256:1f00","file":"src/app.bmb","line":23,"rlimit_max":771}"#;
        assert_eq!(
            compact(row),
            "// app::report: verified\n#[effects(io)]\n#[uses(app::report_ok)]\nfn report(a: &[i64]) -> ()\n    requires report_ok(a), stats::bounded_elems(a),\n"
        );
    }

    #[test]
    fn verified_status_lines_fold_into_a_count() {
        let out = "{\"code\":\"E0802\",\"severity\":\"error\",\"message\":\"m\",\"file\":\"f\",\"line\":1}\n{\"def\":\"a::f\",\"status\":\"verified\",\"obligations\":1,\"failed\":0,\"rlimit\":2,\"rlimit_max\":2}\n{\"def\":\"a::g\",\"status\":\"failed\",\"obligations\":3,\"failed\":1,\"rlimit\":9,\"rlimit_max\":5}\n";
        assert_eq!(
            compact(out),
            "{\"code\":\"E0802\",\"severity\":\"error\",\"message\":\"m\",\"file\":\"f\",\"line\":1}\n{\"def\":\"a::g\",\"status\":\"failed\",\"obligations\":3,\"failed\":1}\n{\"verified\":1}\n"
        );
    }

    #[test]
    fn other_rows_drop_empty_fields_and_cut_hashes() {
        let row = r#"{"hash":"sha256:22ccbc315f497d6212143024bc833446be6b1189a4c9e0d243a81a417a7c7bc5","defs":["a::f","b::g"],"none":[]}"#;
        assert_eq!(
            compact(row),
            "{\"hash\":\"sha256:22ccbc315f49\",\"defs\":[\"a::f\",\"b::g\"]}\n"
        );
        assert_eq!(compact("not json\n"), "not json\n");
    }
}
