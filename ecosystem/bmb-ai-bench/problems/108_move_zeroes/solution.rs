use std::io::{self, Read};
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut iter = input.split_whitespace();
    let n: usize = iter.next().unwrap().parse().unwrap();
    let mut a: Vec<i64> = (0..n).map(|_| iter.next().unwrap().parse().unwrap()).collect();
    let nz: usize = a.iter().filter(|&&x| x != 0).count();
    let mut out: Vec<i64> = a.iter().filter(|&&x| x != 0).cloned().collect();
    out.resize(n, 0);
    let s: Vec<String> = out.iter().map(|x| x.to_string()).collect();
    println!("{}", s.join(" "));
}
