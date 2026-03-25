use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut nums = input.split_whitespace().map(|s| s.parse::<i64>().unwrap());
    let n = nums.next().unwrap() as usize;
    let a: Vec<i64> = (0..n * n).map(|_| nums.next().unwrap()).collect();
    let b: Vec<i64> = (0..n * n).map(|_| nums.next().unwrap()).collect();
    let mut c = vec![0i64; n * n];
    for i in 0..n {
        for k in 0..n {
            for j in 0..n {
                c[i * n + j] += a[i * n + k] * b[k * n + j];
            }
        }
    }
    for i in 0..n {
        let row: Vec<String> = (0..n).map(|j| c[i * n + j].to_string()).collect();
        println!("{}", row.join(" "));
    }
}
