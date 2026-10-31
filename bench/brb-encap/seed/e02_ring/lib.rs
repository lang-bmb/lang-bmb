//! A ring buffer of at most 8 `i64` values.

/// A first-in, first-out queue of at most 8 values.
///
/// Invariant (every method may rely on it and must keep it): `head < 8` and
/// `len <= 8`. The values, oldest first, are `buf[(head + i) % 8]` for
/// `i < len`.
#[derive(Clone, Debug)]
pub struct Ring {
    buf: [i64; 8],
    head: usize,
    len: usize,
}

impl Ring {
    pub fn new() -> Ring {
        Ring { buf: [0; 8], head: 0, len: 0 }
    }

    /// Adds `x` as the newest value; `false` when full: the value is not
    /// stored.
    pub fn push(&mut self, x: i64) -> bool {
        if self.len == 8 {
            return false;
        }
        self.buf[(self.head + self.len) % 8] = x;
        self.len += 1;
        true
    }

    /// Removes and returns the oldest value; `None` when empty.
    pub fn pop(&mut self) -> Option<i64> {
        if self.len == 0 {
            return None;
        }
        let x = self.buf[self.head];
        self.head = (self.head + 1) % 8;
        self.len -= 1;
        Some(x)
    }

    /// The `i`-th value from the oldest.
    pub fn get(&self, i: usize) -> Option<i64> {
        if i >= self.len {
            return None;
        }
        Some(self.buf[(self.head + i) % 8])
    }

    pub fn size(&self) -> usize {
        self.len
    }
}
