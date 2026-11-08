// Rust translation of tests/run/std_slice.bmb (backlog 4B-02), for its
// expected output (`rustc --edition 2021`).
struct Rec {
    key: u64,
    tag: u8,
}

fn found(r: Result<usize, usize>) {
    match r {
        Ok(_) => println!("found"),
        Err(j) => {
            print!("insert at ");
            println!("{j}");
        }
    }
}

fn main() {
    let mut recs = vec![
        Rec { key: 3, tag: b'a' },
        Rec { key: 1, tag: b'b' },
        Rec { key: 3, tag: b'c' },
        Rec { key: 2, tag: b'd' },
        Rec { key: 1, tag: b'e' },
        Rec { key: 3, tag: b'f' },
    ];
    recs.sort_by_key(|r| r.key);
    let line: String = recs.iter().map(|r| r.tag as char).collect();
    println!("{line}");
    let sorted: Vec<u64> = recs.iter().map(|r| r.key).collect();
    found(sorted.binary_search(&2));
    found(sorted.binary_search(&0));
    found(sorted.binary_search(&4));
}
