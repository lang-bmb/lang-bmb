//! A sorted set of at most 8 distinct `i64` values.

/// A set of at most 8 distinct values.
///
/// Invariant (every method may rely on it and must keep it): `len <= 8`, and
/// the values `items[0..len]` are strictly increasing.
#[derive(Clone, Debug)]
pub struct SortedSet {
    items: [i64; 8],
    len: usize,
}

impl SortedSet {
    pub fn new() -> SortedSet {
        SortedSet { items: [0; 8], len: 0 }
    }

    pub fn size(&self) -> usize {
        self.len
    }

    /// The `i`-th smallest value.
    pub fn at(&self, i: usize) -> Option<i64> {
        if i < self.len {
            Some(self.items[i])
        } else {
            None
        }
    }

    pub fn contains(&self, x: i64) -> bool {
        for i in 0..self.len {
            if self.items[i] == x {
                return true;
            }
            if self.items[i] > x {
                return false;
            }
        }
        false
    }

    /// Adds `x`; `false` (and no change) when `x` is there already or the
    /// set is full.
    pub fn insert(&mut self, x: i64) -> bool {
        if self.len == 8 {
            return false;
        }
        let mut p = 0;
        while p < self.len && self.items[p] < x {
            p += 1;
        }
        if p < self.len && self.items[p] == x {
            return false;
        }
        let mut j = self.len;
        while j > p {
            self.items[j] = self.items[j - 1];
            j -= 1;
        }
        self.items[p] = x;
        self.len += 1;
        true
    }

    /// Removes `x`; `false` when `x` is not there.
    pub fn remove(&mut self, x: i64) -> bool {
        let mut p = 0;
        while p < self.len && self.items[p] != x {
            p += 1;
        }
        if p == self.len {
            return false;
        }
        for j in p..self.len - 1 {
            self.items[j] = self.items[j + 1];
        }
        self.len -= 1;
        true
    }
}
