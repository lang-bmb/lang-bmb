use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut nums = input.split_whitespace().map(|s| s.parse::<i64>().unwrap());
    let n = nums.next().unwrap() as usize;
    let arr: Vec<i64> = (0..n).map(|_| nums.next().unwrap()).collect();
    let mut prefix = vec![0i64; n + 1];
    for i in 0..n { prefix[i + 1] = prefix[i] + arr[i]; }
    let q = nums.next().unwrap();
    for _ in 0..q {
        let l = nums.next().unwrap() as usize;
        let r = nums.next().unwrap() as usize;
        println!("{}", prefix[r + 1] - prefix[l]);
    }
}
