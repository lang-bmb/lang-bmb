//! A set of `i64` values stored as at most 8 intervals.

/// The values `lo[i]..=hi[i]` for `i < len`.
///
/// Invariant (every method may rely on it and must keep it): `len <= 8`;
/// every span is nonempty (`lo[i] <= hi[i]`); the spans are in increasing
/// order and two neighbours never touch: `hi[i] + 1 < lo[i + 1]` (there is
/// at least one value between them). So each set of values has exactly one
/// list.
#[derive(Clone, Debug)]
pub struct Spans {
    lo: [i64; 8],
    hi: [i64; 8],
    len: usize,
}

impl Spans {
    pub fn new() -> Spans {
        Spans { lo: [0; 8], hi: [0; 8], len: 0 }
    }

    pub fn count(&self) -> usize {
        self.len
    }

    /// The first value of the `i`-th span.
    pub fn first(&self, i: usize) -> Option<i64> {
        if i < self.len {
            Some(self.lo[i])
        } else {
            None
        }
    }

    /// The last value of the `i`-th span.
    pub fn last(&self, i: usize) -> Option<i64> {
        if i < self.len {
            Some(self.hi[i])
        } else {
            None
        }
    }

    pub fn contains(&self, x: i64) -> bool {
        for i in 0..self.len {
            if x < self.lo[i] {
                return false;
            }
            if x <= self.hi[i] {
                return true;
            }
        }
        false
    }

    /// Appends the span `lo..=hi` after the last one. `false` (and no
    /// change) when `lo > hi`, when the span does not lie above the last one
    /// with a gap, or when the list is full.
    pub fn push(&mut self, lo: i64, hi: i64) -> bool {
        if lo > hi || self.len == 8 {
            return false;
        }
        if self.len > 0 && (self.hi[self.len - 1] >= lo || self.hi[self.len - 1] + 1 == lo) {
            return false;
        }
        self.lo[self.len] = lo;
        self.hi[self.len] = hi;
        self.len += 1;
        true
    }
}
