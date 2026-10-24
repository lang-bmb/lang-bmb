// Shared by the Rust baselines. `--cfg unchecked` turns every index into
// `get_unchecked`; otherwise indexing is ordinary safe Rust.
#[cfg(not(unchecked))]
macro_rules! get { ($a:expr, $i:expr) => { $a[$i] }; }
#[cfg(not(unchecked))]
macro_rules! set { ($a:expr, $i:expr, $v:expr) => { $a[$i] = $v }; }
#[cfg(unchecked)]
#[allow(unused_unsafe)]
macro_rules! get { ($a:expr, $i:expr) => { unsafe { *$a.get_unchecked($i) } }; }
#[cfg(unchecked)]
macro_rules! set { ($a:expr, $i:expr, $v:expr) => { unsafe { *$a.get_unchecked_mut($i) = $v } }; }
