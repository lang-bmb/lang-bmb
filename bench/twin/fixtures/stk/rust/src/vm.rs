//! vm: the stack and its operations; each returns 0 or an error code

// Error codes: 1 stack underflow, 2 overflow, 3 division by zero,
// 4 unknown token (main), 5 stack full.

/// The most values the stack holds.
pub const LIMIT: usize = 4096;

pub fn push(st: &mut Vec<i64>, v: i64) -> u8 {
    if st.len() >= LIMIT {
        return 5;
    }
    st.push(v);
    0
}

/// `a op b` for op 1 add, 2 sub, 3 mul, 4 div; None when the result does not
/// fit or b is 0 for div.
pub fn apply(op: u8, a: i64, b: i64) -> Option<i64> {
    match op {
        1 => a.checked_add(b),
        2 => a.checked_sub(b),
        3 => a.checked_mul(b),
        _ => a.checked_div(b),
    }
}

/// Replaces the two top values a, b (b on top) with `a op b`. On an error the
/// stack is left as it was.
pub fn binary(st: &mut Vec<i64>, op: u8) -> u8 {
    if st.len() < 2 {
        return 1;
    }
    let b = st.pop().unwrap();
    let a = st.pop().unwrap();
    match apply(op, a, b) {
        Some(v) => {
            st.push(v);
            0
        }
        None => {
            st.push(a);
            st.push(b);
            if op == 4 && b == 0 {
                3
            } else {
                2
            }
        }
    }
}

pub fn dup(st: &mut Vec<i64>) -> u8 {
    if st.len() >= LIMIT {
        return 5;
    }
    match st.last() {
        Some(&x) => {
            st.push(x);
            0
        }
        None => 1,
    }
}

pub fn swap(st: &mut Vec<i64>) -> u8 {
    let n = st.len();
    if n < 2 {
        return 1;
    }
    st.swap(n - 2, n - 1);
    0
}

pub fn drop(st: &mut Vec<i64>) -> u8 {
    match st.pop() {
        Some(_) => 0,
        None => 1,
    }
}

/// Prints the top value on a line of its own.
pub fn print(st: &mut Vec<i64>) -> u8 {
    match st.last() {
        Some(x) => {
            println!("{x}");
            0
        }
        None => 1,
    }
}
