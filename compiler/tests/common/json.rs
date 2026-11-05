//! A JSON reader and compact writer for tests that talk to the CLI (backlog
//! 4A-04), independent of the compiler's own: RFC 8259 values, object keys
//! in source order, numbers as their text. The writer escapes `"`, `\`,
//! `\n`, `\r`, `\t` and other control characters and nothing else, so a
//! value read from the server's output writes back to the same bytes.

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Num(String),
    Str(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(kv) => kv.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Num(n) => n.parse().ok(),
            _ => None,
        }
    }

    pub fn to_json(&self) -> String {
        match self {
            Value::Null => "null".into(),
            Value::Bool(b) => b.to_string(),
            Value::Num(n) => n.clone(),
            Value::Str(s) => quote(s),
            Value::Array(xs) => format!(
                "[{}]",
                xs.iter().map(Value::to_json).collect::<Vec<_>>().join(",")
            ),
            Value::Object(kv) => format!(
                "{{{}}}",
                kv.iter()
                    .map(|(k, v)| format!("{}:{}", quote(k), v.to_json()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }
}

/// `s` as a JSON string.
pub fn quote(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

pub fn parse(s: &str) -> Result<Value, String> {
    let mut r = Reader {
        b: s.as_bytes(),
        s,
        i: 0,
    };
    r.ws();
    let v = r.value()?;
    r.ws();
    if r.i != r.b.len() {
        return Err(format!("trailing text at byte {}", r.i));
    }
    Ok(v)
}

struct Reader<'a> {
    b: &'a [u8],
    s: &'a str,
    i: usize,
}

impl Reader<'_> {
    fn ws(&mut self) {
        while self.i < self.b.len() && matches!(self.b[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }

    fn eat(&mut self, c: u8) -> Result<(), String> {
        if self.b.get(self.i) == Some(&c) {
            self.i += 1;
            Ok(())
        } else {
            Err(format!("expected `{}` at byte {}", c as char, self.i))
        }
    }

    fn word(&mut self, w: &str, v: Value) -> Result<Value, String> {
        if self.s[self.i..].starts_with(w) {
            self.i += w.len();
            Ok(v)
        } else {
            Err(format!("bad literal at byte {}", self.i))
        }
    }

    fn value(&mut self) -> Result<Value, String> {
        match self.b.get(self.i) {
            Some(b'{') => {
                self.i += 1;
                let mut kv = Vec::new();
                self.ws();
                if self.b.get(self.i) == Some(&b'}') {
                    self.i += 1;
                    return Ok(Value::Object(kv));
                }
                loop {
                    self.ws();
                    let k = self.string()?;
                    self.ws();
                    self.eat(b':')?;
                    self.ws();
                    kv.push((k, self.value()?));
                    self.ws();
                    if self.b.get(self.i) == Some(&b',') {
                        self.i += 1;
                    } else {
                        self.eat(b'}')?;
                        return Ok(Value::Object(kv));
                    }
                }
            }
            Some(b'[') => {
                self.i += 1;
                let mut xs = Vec::new();
                self.ws();
                if self.b.get(self.i) == Some(&b']') {
                    self.i += 1;
                    return Ok(Value::Array(xs));
                }
                loop {
                    self.ws();
                    xs.push(self.value()?);
                    self.ws();
                    if self.b.get(self.i) == Some(&b',') {
                        self.i += 1;
                    } else {
                        self.eat(b']')?;
                        return Ok(Value::Array(xs));
                    }
                }
            }
            Some(b'"') => Ok(Value::Str(self.string()?)),
            Some(b't') => self.word("true", Value::Bool(true)),
            Some(b'f') => self.word("false", Value::Bool(false)),
            Some(b'n') => self.word("null", Value::Null),
            Some(c) if *c == b'-' || c.is_ascii_digit() => {
                let start = self.i;
                while self.i < self.b.len()
                    && matches!(
                        self.b[self.i],
                        b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9'
                    )
                {
                    self.i += 1;
                }
                Ok(Value::Num(self.s[start..self.i].to_string()))
            }
            _ => Err(format!("unexpected input at byte {}", self.i)),
        }
    }

    fn hex4(&mut self) -> Result<u32, String> {
        let h = self.s.get(self.i..self.i + 4).ok_or("short \\u escape")?;
        self.i += 4;
        u32::from_str_radix(h, 16).map_err(|e| e.to_string())
    }

    fn string(&mut self) -> Result<String, String> {
        self.eat(b'"')?;
        let mut out = String::new();
        loop {
            let c = self.s[self.i..]
                .chars()
                .next()
                .ok_or("unterminated string")?;
            self.i += c.len_utf8();
            match c {
                '"' => return Ok(out),
                '\\' => {
                    let e = *self.b.get(self.i).ok_or("unterminated escape")?;
                    self.i += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let mut u = self.hex4()?;
                            if (0xd800..0xdc00).contains(&u) && self.s[self.i..].starts_with("\\u")
                            {
                                self.i += 2;
                                let lo = self.hex4()?;
                                u = 0x10000 + ((u - 0xd800) << 10) + (lo - 0xdc00);
                            }
                            out.push(char::from_u32(u).ok_or("bad \\u escape")?);
                        }
                        _ => return Err(format!("bad escape at byte {}", self.i)),
                    }
                }
                c => out.push(c),
            }
        }
    }
}
