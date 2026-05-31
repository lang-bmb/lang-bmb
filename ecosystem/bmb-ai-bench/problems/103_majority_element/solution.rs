use std::io::{self, Read};
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut iter = input.split_whitespace();
    let n: usize = iter.next().unwrap().parse().unwrap();
    let arr: Vec<i64> = (0..n).map(|_| iter.next().unwrap().parse().unwrap()).collect();
    let (mut candidate, mut count) = (arr[0], 1i64);
    for &val in &arr[1..] {
        if count == 0 { candidate = val; count = 1; }
        else if val == candidate { count += 1; }
        else { count -= 1; }
    }
    println!("{}", candidate);
}
