//! Tokenizer. Comments are `//` only and are discarded.

use crate::ast::Radix;
use crate::diag::{Diag, Diags};
use crate::source::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Ident(String),
    Int(u128, Radix),
    /// A floating-point literal `d.d[e[+-]d]` (ADR-0061 §2): the bits of
    /// the nearest `f64` (correctly rounded, as Rust's `str::parse`).
    Float(u64),
    Str(String),
    /// `'a'`, `'\u{1F600}'`: a Unicode scalar value (ADR-0050 §1).
    Char(u32),
    /// `b'a'` (ADR-0044 §2).
    Byte(u8),
    /// `b"..."` (ADR-0044 §2).
    Bytes(Vec<u8>),
    P(&'static str),
    /// A loop label `'outer` (ADR-0049 §1): before `: while`/`: loop`/
    /// `: for`, or after `break`/`continue`. Elsewhere `'a` is a lifetime
    /// (E0208).
    Label(String),
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tok: Tok,
    pub span: Span,
}

const PUNCT3: &[&str] = &["==>", "..="];
const PUNCT2: &[&str] = &["::", "->", "=>", "==", "!=", "<=", ">=", "&&", "||", ".."];
const PUNCT1: &[&str] = &[
    "(", ")", "{", "}", "[", "]", ",", ";", ":", ".", "|", "#", "!", "&", "=", "<", ">", "+", "-",
    "*", "/", "%", "?", "@", "^", "~", "$",
];

pub fn lex(file: u32, text: &str, diags: &mut Diags) -> Vec<Token> {
    let b = text.as_bytes();
    let mut i = 0usize;
    let mut out = Vec::new();
    let sp = |lo: usize, hi: usize| Span {
        file,
        lo: lo as u32,
        hi: hi as u32,
    };
    'outer: while i < b.len() {
        let c = b[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if c == b'/' && b.get(i + 1) == Some(&b'/') {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if c == b'/' && b.get(i + 1) == Some(&b'*') {
            let lo = i;
            i += 2;
            while i < b.len() && !(b[i] == b'*' && b.get(i + 1) == Some(&b'/')) {
                i += 1;
            }
            i = (i + 2).min(b.len());
            diags.push(
                Diag::new("E0106", sp(lo, i), "block comments are not allowed")
                    .fix("use `//` line comments", None),
            );
            continue;
        }
        if c == b'b' && matches!(b.get(i + 1), Some(b'\'' | b'"')) {
            let lo = i;
            let quote = b[i + 1];
            i += 2;
            let mut bytes = Vec::new();
            let mut non_ascii = false;
            while i < b.len() && b[i] != quote && !(quote == b'\'' && b[i] == b'\n') {
                if b[i] == b'\\' {
                    match escape(b, i) {
                        Some((v, n)) => {
                            bytes.push(v);
                            i += n;
                        }
                        None => {
                            let n = 1 + text[i + 1..].chars().next().map_or(0, char::len_utf8);
                            diags.push(Diag::new(
                                "E0101",
                                sp(i, i + n),
                                "unknown escape: byte literals take \\n \\r \\t \\\\ \\' \\\" \\0 \\xHH",
                            ));
                            i += n;
                        }
                    }
                    continue;
                }
                non_ascii |= b[i] >= 0x80;
                bytes.push(b[i]);
                i += 1;
            }
            if i >= b.len() || b[i] != quote {
                diags.push(Diag::new("E0102", sp(lo, i), "unterminated byte literal"));
                continue;
            }
            i += 1;
            let q = if quote == b'"' { '"' } else { '\'' };
            if non_ascii {
                // As in Rust: other bytes are written as escapes.
                diags.push(
                    Diag::new(
                        "E0101",
                        sp(lo, i),
                        "a byte literal holds ASCII characters and escapes",
                    )
                    .fix(
                        "write the other bytes as `\\xHH`",
                        Some(byte_text(&bytes, q)),
                    ),
                );
            }
            let tok = if quote == b'"' {
                Tok::Bytes(bytes)
            } else {
                if bytes.len() != 1 {
                    diags.push(Diag::new(
                        "E0101",
                        sp(lo, i),
                        "a byte literal holds one byte; use `b\"...\"` for several",
                    ));
                }
                Tok::Byte(bytes.first().copied().unwrap_or(0))
            };
            out.push(Token {
                tok,
                span: sp(lo, i),
            });
            continue;
        }
        if c.is_ascii_alphabetic() || c == b'_' {
            let lo = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            out.push(Token {
                tok: Tok::Ident(text[lo..i].to_string()),
                span: sp(lo, i),
            });
            continue;
        }
        // A floating-point literal (ADR-0061 §2), or a malformed one: `.5`
        // where no field name can follow a `.`, `1.`, `1e5` (E0109).
        let after_value =
            i > 0 && (b[i - 1].is_ascii_alphanumeric() || matches!(b[i - 1], b'_' | b')' | b']'));
        if (c == b'.' && !after_value && b.get(i + 1).is_some_and(u8::is_ascii_digit))
            || (c.is_ascii_digit() && float_start(b, i))
        {
            let lo = i;
            let f = float_parts(b, i);
            i = f.end;
            let fixed = f.fixed();
            let shape = if f.int.is_empty() {
                Some("has no digits before the point")
            } else if f.frac.is_none() {
                Some("has no point")
            } else if f.frac.as_deref() == Some("") {
                Some("has no digits after the point")
            } else {
                None
            };
            if let Some(why) = shape {
                diags.push(
                    Diag::new(
                        "E0109",
                        sp(lo, i),
                        format!("float literal `{}` {why}", &text[lo..i]),
                    )
                    .fix(
                        "a float literal has digits on both sides of the point: `1.0`, `0.5`, `1.0e-9`",
                        f.exp_ok().then(|| fixed.clone()),
                    ),
                );
            } else if !f.exp_ok() {
                diags.push(
                    Diag::new(
                        "E0109",
                        sp(lo, i),
                        format!("float literal `{}` has no exponent digits", &text[lo..i]),
                    )
                    .fix("write the exponent's digits after `e`: `1.0e-9`", None),
                );
            }
            let value: f64 = if f.exp_ok() {
                fixed.parse().unwrap_or(0.0)
            } else {
                0.0
            };
            if value.is_infinite() {
                diags.push(
                    Diag::new(
                        "E0406",
                        sp(lo, i),
                        format!("float literal `{}` is too large for `f64`", &text[lo..i]),
                    )
                    .fix(
                        "the largest finite `f64` is `f64::MAX`; infinity is `f64::INFINITY`",
                        None,
                    ),
                );
            }
            if i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                let slo = i;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                    i += 1;
                }
                let suffix = &text[slo..i];
                let d = Diag::new(
                    "E0104",
                    sp(lo, i),
                    format!("literal suffix `{suffix}` is not allowed"),
                );
                diags.push(if suffix == "f64" || suffix == "f32" {
                    d.fix("remove the suffix", Some(text[lo..slo].to_string()))
                } else {
                    d.fix(
                        "a float literal is digits, a point, digits and an optional exponent: `6.02e23`",
                        None,
                    )
                });
            }
            out.push(Token {
                tok: Tok::Float(value.to_bits()),
                span: sp(lo, i),
            });
            continue;
        }
        if c.is_ascii_digit() {
            let lo = i;
            // `0x`, `0o`, `0b` and `_` separators (ADR-0048 §3), as in Rust.
            let radix = match (c, b.get(i + 1)) {
                (b'0', Some(b'x')) => Radix::Hex,
                (b'0', Some(b'o')) => Radix::Oct,
                (b'0', Some(b'b')) => Radix::Bin,
                _ => Radix::Dec,
            };
            if radix != Radix::Dec {
                i += 2;
            }
            let base: u32 = match radix {
                Radix::Dec => 10,
                Radix::Hex => 16,
                Radix::Oct => 8,
                Radix::Bin => 2,
            };
            // Digits: hexadecimal ones in a `0x` literal, decimal ones
            // otherwise (a decimal digit outside the base is an error, as in
            // Rust); what follows is a suffix.
            let digit = |ch: u8| {
                if radix == Radix::Hex {
                    ch.is_ascii_hexdigit()
                } else {
                    ch.is_ascii_digit()
                }
            };
            let mut v: u128 = 0;
            let mut overflow = false;
            let mut any = false;
            let mut bad: Option<(usize, char)> = None;
            while i < b.len() && (digit(b[i]) || b[i] == b'_') {
                if b[i] != b'_' {
                    any = true;
                    let d = (b[i] as char).to_digit(16).expect("a digit");
                    if d >= base {
                        bad.get_or_insert((i, b[i] as char));
                    } else {
                        match v
                            .checked_mul(base as u128)
                            .and_then(|x| x.checked_add(d as u128))
                        {
                            Some(x) => v = x,
                            None => overflow = true,
                        }
                    }
                }
                i += 1;
            }
            let digits_end = i;
            let name = match radix {
                Radix::Hex => "hexadecimal",
                Radix::Oct => "octal",
                Radix::Bin => "binary",
                Radix::Dec => "decimal",
            };
            if let Some((at, ch)) = bad {
                let allowed = if radix == Radix::Bin {
                    "`0` and `1`"
                } else {
                    "`0` to `7`"
                };
                diags.push(
                    Diag::new(
                        "E0108",
                        sp(at, at + 1),
                        format!("`{ch}` is not {} {name} digit", if radix == Radix::Oct { "an" } else { "a" }),
                    )
                    .fix(
                        format!(
                            "{name} digits are {allowed}; write the value in another base (`0x` hexadecimal, or decimal)"
                        ),
                        None,
                    ),
                );
            }
            let no_digits = !any && radix != Radix::Dec;
            if no_digits {
                diags.push(
                    Diag::new(
                        "E0108",
                        sp(lo, digits_end),
                        format!("`{}` has no {name} digits", &text[lo..digits_end]),
                    )
                    .fix(
                        format!("write the digits after the prefix: `{}`", radix.text(0)),
                        None,
                    ),
                );
            }
            if i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                let slo = i;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                    i += 1;
                }
                let suffix = &text[slo..i];
                let written = &text[lo..digits_end];
                const TYPES: [&str; 12] = [
                    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128",
                    "usize",
                ];
                let d = if no_digits {
                    // Reported above: the suffix is what the digits would be.
                    None
                } else if TYPES.contains(&suffix) {
                    Diag::new(
                        "E0104",
                        sp(lo, i),
                        format!("literal suffix `{suffix}` is not allowed"),
                    )
                    .fix("remove the suffix", Some(written.to_string()))
                    .into()
                } else {
                    Diag::new(
                        "E0104",
                        sp(lo, i),
                        format!("literal suffix `{suffix}` is not allowed"),
                    )
                    .fix(
                        "integer literals are digits in base 10, or after `0x`, `0o` or `0b`",
                        None,
                    )
                    .into()
                };
                if let Some(d) = d {
                    diags.push(d);
                }
            }
            if overflow {
                diags.push(Diag::new(
                    "E0105",
                    sp(lo, i),
                    "integer literal does not fit in 128 bits",
                ));
            }
            out.push(Token {
                tok: Tok::Int(v, radix),
                span: sp(lo, i),
            });
            continue;
        }
        if c == b'"' {
            let lo = i;
            i += 1;
            let mut s = String::new();
            while i < b.len() && b[i] != b'"' {
                if b[i] == b'\\' {
                    match char_escape(text, i) {
                        Ok((v, n)) => {
                            s.push(v);
                            i += n;
                        }
                        Err(n) => {
                            diags.push(
                                Diag::new("E0101", sp(i, i + n), ESCAPES).fix(ESCAPE_FIX, None),
                            );
                            i += n;
                        }
                    }
                    continue;
                }
                let ch = text[i..].chars().next().unwrap();
                s.push(ch);
                i += ch.len_utf8();
            }
            if i >= b.len() {
                diags.push(Diag::new("E0102", sp(lo, i), "unterminated string literal"));
                break;
            }
            i += 1;
            out.push(Token {
                tok: Tok::Str(s),
                span: sp(lo, i),
            });
            continue;
        }
        // `'a'`, `'\n'`, `'\u{e9}'`: a character literal (ADR-0050 §1).
        if c == b'\'' {
            let (v, j) = if b.get(i + 1) == Some(&b'\\') {
                match char_escape(text, i + 1) {
                    Ok((v, n)) => (Some(v), i + 1 + n),
                    Err(n) => (None, i + 1 + n),
                }
            } else if let Some(ch) = text.get(i + 1..).and_then(|t| t.chars().next()) {
                (Some(ch), i + 1 + ch.len_utf8())
            } else {
                (None, i + 1)
            };
            if b.get(j) == Some(&b'\'') && j > i + 1 {
                match v {
                    Some(v) if !(v == '\'' && b[i + 1] != b'\\') => {
                        out.push(Token {
                            tok: Tok::Char(v as u32),
                            span: sp(i, j + 1),
                        });
                    }
                    _ => {
                        diags.push(Diag::new("E0101", sp(i + 1, j), ESCAPES).fix(ESCAPE_FIX, None));
                        out.push(Token {
                            tok: Tok::Char(0),
                            span: sp(i, j + 1),
                        });
                    }
                }
                i = j + 1;
                continue;
            }
        }
        if c == b'\'' {
            let lo = i;
            i += 1;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            // A label: after `break`/`continue`, or before a `:` (a loop
            // label; a lifetime takes none in BMB). Comments may come between.
            let after_jump = matches!(out.last(), Some(Token { tok: Tok::Ident(k), .. }) if k == "break" || k == "continue");
            let mut rest = &text[i..];
            loop {
                let t = rest.trim_start();
                match t.strip_prefix("//") {
                    Some(c) => rest = c.find('\n').map_or("", |n| &c[n..]),
                    None => {
                        rest = t;
                        break;
                    }
                }
            }
            let before_loop = rest.starts_with(':') && !rest.starts_with("::");
            if i > lo + 1 && (after_jump || before_loop) && !(i < b.len() && b[i] == b'\'') {
                out.push(Token {
                    tok: Tok::Label(text[lo + 1..i].to_string()),
                    span: sp(lo, i),
                });
                continue;
            }
            if i < b.len() && b[i] == b'\'' {
                i += 1;
                diags.push(
                    Diag::new(
                        "E0101",
                        sp(lo, i),
                        "a character literal holds one character",
                    )
                    .fix("several characters are a string: `\"...\"`", None),
                );
                // Recovery: a character, so parsing goes on.
                out.push(Token {
                    tok: Tok::Char(0),
                    span: sp(lo, i),
                });
            } else {
                diags.push(
                    Diag::new("E0208", sp(lo, i), "lifetime annotations are not allowed").fix(
                        "remove the lifetime; references are second-class and need none",
                        Some(String::new()),
                    ),
                );
            }
            continue;
        }
        for set in [PUNCT3, PUNCT2, PUNCT1] {
            for p in set {
                if text[i..].starts_with(p) {
                    out.push(Token {
                        tok: Tok::P(p),
                        span: sp(i, i + p.len()),
                    });
                    i += p.len();
                    continue 'outer;
                }
            }
        }
        let ch = text[i..].chars().next().unwrap();
        diags.push(Diag::new(
            "E0101",
            sp(i, i + ch.len_utf8()),
            format!("unexpected character `{ch}`"),
        ));
        i += ch.len_utf8();
    }
    out.push(Token {
        tok: Tok::Eof,
        span: sp(b.len(), b.len()),
    });
    out
}

/// Whether the decimal digits at `b[i]` begin a float literal, written
/// well (`1.5`, `2.0e-3`) or not (`1.`, `1e5`): a point and a digit, a point
/// that neither starts a range (`0..n`) nor a method or field name
/// (`1.max(2)`), or an exponent right after the digits.
fn float_start(b: &[u8], i: usize) -> bool {
    if b[i] == b'0' && matches!(b.get(i + 1), Some(b'x' | b'o' | b'b')) {
        return false;
    }
    let mut j = i;
    while j < b.len() && (b[j].is_ascii_digit() || b[j] == b'_') {
        j += 1;
    }
    match b.get(j) {
        Some(b'.') => match b.get(j + 1) {
            Some(c) if c.is_ascii_digit() => true,
            Some(c) => !(c.is_ascii_alphabetic() || *c == b'_' || *c == b'.'),
            None => true,
        },
        Some(b'e' | b'E') => match b.get(j + 1) {
            Some(c) if c.is_ascii_digit() => true,
            Some(b'+' | b'-') => b.get(j + 2).is_some_and(u8::is_ascii_digit),
            _ => false,
        },
        _ => false,
    }
}

/// The parts of a float literal (digits without `_` separators).
struct FloatParts {
    int: String,
    /// The digits after the point; `None` without a point.
    frac: Option<String>,
    /// The exponent's sign and digits, after `e`/`E`.
    exp: Option<(String, String)>,
    end: usize,
}

impl FloatParts {
    fn exp_ok(&self) -> bool {
        self.exp.as_ref().is_none_or(|(_, d)| !d.is_empty())
    }

    /// The literal with the missing digits written (`.5` -> `0.5`, `1.` ->
    /// `1.0`, `1e5` -> `1.0e5`).
    fn fixed(&self) -> String {
        let int = if self.int.is_empty() { "0" } else { &self.int };
        let frac = match self.frac.as_deref() {
            None | Some("") => "0",
            Some(f) => f,
        };
        let exp = match &self.exp {
            Some((sign, d)) => format!("e{sign}{d}"),
            None => String::new(),
        };
        format!("{int}.{frac}{exp}")
    }
}

fn float_parts(b: &[u8], mut i: usize) -> FloatParts {
    let digits = |i: &mut usize| {
        let mut d = String::new();
        while *i < b.len() && (b[*i].is_ascii_digit() || b[*i] == b'_') {
            if b[*i] != b'_' {
                d.push(b[*i] as char);
            }
            *i += 1;
        }
        d
    };
    let int = digits(&mut i);
    let frac = if b.get(i) == Some(&b'.') {
        i += 1;
        Some(digits(&mut i))
    } else {
        None
    };
    let exp = if matches!(b.get(i), Some(b'e' | b'E')) {
        i += 1;
        let sign = match b.get(i) {
            Some(b'-') => {
                i += 1;
                "-"
            }
            Some(b'+') => {
                i += 1;
                ""
            }
            _ => "",
        };
        Some((sign.to_string(), digits(&mut i)))
    } else {
        None
    };
    FloatParts {
        int,
        frac,
        exp,
        end: i,
    }
}

/// The normal form of a float literal's value (ADR-0061 §2): the shortest
/// text that reads back as the same `f64` (Rust's `{:?}`), with a point in
/// the mantissa (`1e-9` is written `1.0e-9`, so it is a literal again).
pub fn float_text(bits: u64) -> String {
    let s = format!("{:?}", f64::from_bits(bits));
    match s.split_once('e') {
        Some((m, e)) if !m.contains('.') => format!("{m}.0e{e}"),
        _ => s,
    }
}

/// The escape at `b[i]` (a backslash) in a byte literal: its value and its
/// length in the source (ADR-0044 §2, as Rust).
fn escape(b: &[u8], i: usize) -> Option<(u8, usize)> {
    let v = match *b.get(i + 1)? {
        b'n' => b'\n',
        b'r' => b'\r',
        b't' => b'\t',
        b'\\' => b'\\',
        b'\'' => b'\'',
        b'"' => b'"',
        b'0' => 0,
        b'x' => {
            let h = std::str::from_utf8(b.get(i + 2..i + 4)?).ok()?;
            return Some((u8::from_str_radix(h, 16).ok()?, 4));
        }
        _ => return None,
    };
    Some((v, 2))
}

/// The escapes of string and character literals (ADR-0050 §1), as in Rust.
/// The help of an unknown escape.
const ESCAPE_FIX: &str =
    "write the character itself, or one of those escapes (a code point is `\\u{e9}`)";
const ESCAPES: &str = "unknown escape: string and character literals take \\n \\r \\t \\\\ \\' \\\" \\0 \\xHH (up to 7f) \\u{H..}";

/// The escape at `text[i]` (a `\\`) of a string or character literal: its
/// character and length, or the length to skip when it is not one.
fn char_escape(text: &str, i: usize) -> Result<(char, usize), usize> {
    let b = text.as_bytes();
    let Some(&e) = b.get(i + 1) else {
        return Err(1);
    };
    let simple = match e {
        b'n' => Some('\n'),
        b'r' => Some('\r'),
        b't' => Some('\t'),
        b'\\' => Some('\\'),
        b'\'' => Some('\''),
        b'"' => Some('"'),
        b'0' => Some('\0'),
        _ => None,
    };
    if let Some(c) = simple {
        return Ok((c, 2));
    }
    match e {
        b'x' => {
            let h = text.get(i + 2..i + 4).ok_or(2usize)?;
            match u8::from_str_radix(h, 16) {
                Ok(v) if v <= 0x7f && h.bytes().all(|c| c.is_ascii_hexdigit()) => {
                    Ok((v as char, 4))
                }
                _ => Err(4.min(b.len() - i)),
            }
        }
        b'u' if b.get(i + 2) == Some(&b'{') => {
            let close = text[i + 3..].find('}').map(|k| i + 3 + k);
            let Some(close) = close else {
                return Err(3);
            };
            let n = close + 1 - i;
            let h: String = text[i + 3..close].chars().filter(|&c| c != '_').collect();
            if h.is_empty() || h.len() > 6 || !h.bytes().all(|c| c.is_ascii_hexdigit()) {
                return Err(n);
            }
            u32::from_str_radix(&h, 16)
                .ok()
                .and_then(char::from_u32)
                .map(|c| (c, n))
                .ok_or(n)
        }
        _ => Err(1 + text[i + 1..].chars().next().map_or(0, char::len_utf8)),
    }
}

/// `s` written as a string literal (`q` is `"`) or a character literal
/// (`q` is `'`): characters as themselves except the quote, `\\`, and the
/// control characters, which are escapes. The normal form uses it.
pub fn str_text(s: &str, q: char) -> String {
    let mut o = String::from(q);
    for c in s.chars() {
        match c {
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            '\\' => o.push_str("\\\\"),
            '\0' => o.push_str("\\0"),
            c if c == q => {
                o.push('\\');
                o.push(q);
            }
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                o.push_str(&format!("\\u{{{:x}}}", c as u32));
            }
            c => o.push(c),
        }
    }
    o.push(q);
    o
}

/// `bytes` written as a byte literal with quote `q` (`'` or `"`): printable
/// ASCII as itself, the rest as escapes. The normal form and fixes use it.
pub fn byte_text(bytes: &[u8], q: char) -> String {
    let mut s = format!("b{q}");
    for &c in bytes {
        match c {
            b'\n' => s.push_str("\\n"),
            b'\r' => s.push_str("\\r"),
            b'\t' => s.push_str("\\t"),
            b'\\' => s.push_str("\\\\"),
            0 => s.push_str("\\0"),
            c if c as char == q => {
                s.push('\\');
                s.push(q);
            }
            0x20..=0x7e => s.push(c as char),
            c => s.push_str(&format!("\\x{c:02x}")),
        }
    }
    s.push(q);
    s
}
