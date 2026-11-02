//! A binary min-heap of at most 8 `i64` values.

/// The values `a[0..len]` in heap order.
///
/// Invariant (every method may rely on it and must keep it): `len <= 8`,
/// and no value is smaller than its parent's: `a[(i - 1) / 2] <= a[i]` for
/// `0 < i < len`. So `a[0]` is the smallest.
#[derive(Clone, Debug)]
pub struct Heap {
    a: [i64; 8],
    len: usize,
}

impl Heap {
    pub fn new() -> Heap {
        Heap { a: [0; 8], len: 0 }
    }

    pub fn size(&self) -> usize {
        self.len
    }

    /// The smallest value.
    pub fn peek(&self) -> Option<i64> {
        if self.len == 0 {
            None
        } else {
            Some(self.a[0])
        }
    }

    /// Adds `x`; `false` when full: the value is not stored.
    pub fn push(&mut self, x: i64) -> bool {
        if self.len == 8 {
            return false;
        }
        let mut k = self.len;
        self.a[k] = x;
        self.len += 1;
        while k > 0 && self.a[(k - 1) / 2] > self.a[k] {
            let p = (k - 1) / 2;
            self.a.swap(p, k);
            k = p;
        }
        true
    }
}
