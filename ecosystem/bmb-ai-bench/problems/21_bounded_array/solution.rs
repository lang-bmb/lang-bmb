use std::io::{self, Read};

fn bounded_get(arr: &[i64], idx: usize) -> i64 {
    assert!(idx < arr.len(), "index out of bounds");
    arr[idx]
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut nums = input.split_whitespace().map(|s| s.parse::<i64>().unwrap());

    let n = nums.next().unwrap() as usize;
    let arr: Vec<i64> = nums.by_ref().take(n).collect();
    let idx = nums.next().unwrap() as usize;

    let val = bounded_get(&arr, idx);
    println!("{}", val);
}
