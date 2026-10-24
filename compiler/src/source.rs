//! Source files and byte spans.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub file: u32,
    pub lo: u32,
    pub hi: u32,
}

impl Span {
    pub fn to(self, other: Span) -> Span {
        Span {
            file: self.file,
            lo: self.lo.min(other.lo),
            hi: self.hi.max(other.hi),
        }
    }
}

pub struct SourceFile {
    pub name: String,
    pub text: String,
    line_starts: Vec<u32>,
}

#[derive(Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn add(&mut self, name: String, text: String) -> u32 {
        let mut line_starts = vec![0u32];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i as u32 + 1);
            }
        }
        self.files.push(SourceFile {
            name,
            text,
            line_starts,
        });
        (self.files.len() - 1) as u32
    }

    pub fn file(&self, id: u32) -> &SourceFile {
        &self.files[id as usize]
    }

    pub fn name(&self, id: u32) -> &str {
        &self.files[id as usize].name
    }

    /// 1-based line and column; the column counts characters, not bytes.
    pub fn line_col(&self, id: u32, pos: u32) -> (u32, u32) {
        let f = &self.files[id as usize];
        let line = match f.line_starts.binary_search(&pos) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let start = f.line_starts[line] as usize;
        let end = (pos as usize).min(f.text.len());
        let col = f.text[start..end].chars().count() as u32 + 1;
        (line as u32 + 1, col)
    }
}
