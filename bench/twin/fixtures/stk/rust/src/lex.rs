//! lex: the tokens of a line, as byte ranges.

/// A space between tokens: space, tab or carriage return.
pub fn space(b: u8) -> bool {
    b == 32 || b == 9 || b == 13
}

/// The end of the token that starts at `lo`: the first space at or after it,
/// or `hi`. Requires `lo < hi <= s.len()`.
pub fn token_end(s: &[u8], lo: usize, hi: usize) -> usize {
    let mut i = lo + 1;
    while i < hi && !space(s[i]) {
        i += 1;
    }
    i
}

/// Whether `s[lo..hi]` is the word `w`.
pub fn is(s: &[u8], lo: usize, hi: usize, w: &str) -> bool {
    &s[lo..hi] == w.as_bytes()
}
