// Rust translation of tests/run/std_str.bmb (backlog 4B-02), for its
// expected output (`rustc --edition 2021`). BMB's functions return the
// positions where Rust's return the slices.
fn yes(b: bool) {
    println!("{b}");
}

fn at(r: Option<usize>) {
    match r {
        None => println!("none"),
        Some(i) => println!("{i}"),
    }
}

fn trimmed(s: &str) {
    println!("[{}]", s.trim_ascii());
}

fn rest(s: &str, p: &str) {
    match s.strip_prefix(p) {
        None => println!("-"),
        Some(r) => println!("{r}"),
    }
}

fn head(s: &str, p: &str) {
    match s.strip_suffix(p) {
        None => println!("-"),
        Some(r) => println!("{r}"),
    }
}

fn chars(s: &str) {
    let mut n = 0u64;
    for c in s.chars() {
        println!("{}", c as u32);
        n += 1;
    }
    println!("{n}");
}

fn main() {
    yes("hello".starts_with("he"));
    yes("hello".starts_with("lo"));
    yes("".starts_with(""));
    yes("hello".ends_with("llo"));
    yes("hello".ends_with("hel"));
    at("hello world".find("o"));
    at("hello world".find("world"));
    at("hello".find("xyz"));
    at("abc".find(""));
    let owned = String::from("  owned text \t");
    trimmed(owned.as_str());
    trimmed("\n\n");
    trimmed("");
    yes(owned.starts_with("  ow"));
    at(owned.find("text"));
    rest("key=value", "key=");
    rest("key=value", "val");
    rest("héllo", "h");
    head("file.txt", ".txt");
    head("file.txt", ".rs");
    chars("aé€😀");
    chars("");
}
