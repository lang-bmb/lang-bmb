use std::io::{self, Read};

fn merge_sort(arr: &mut Vec<i64>) {
    let n = arr.len();
    if n <= 1 { return; }
    let mid = n / 2;
    let mut left: Vec<i64> = arr[..mid].to_vec();
    let mut right: Vec<i64> = arr[mid..].to_vec();
    merge_sort(&mut left);
    merge_sort(&mut right);
    let (mut i, mut j, mut k) = (0, 0, 0);
    while i < left.len() && j < right.len() {
        if left[i] <= right[j] { arr[k] = left[i]; i += 1; }
        else { arr[k] = right[j]; j += 1; }
        k += 1;
    }
    while i < left.len() { arr[k] = left[i]; i += 1; k += 1; }
    while j < right.len() { arr[k] = right[j]; j += 1; k += 1; }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut nums = input.split_whitespace().map(|s| s.parse::<i64>().unwrap());
    let n = nums.next().unwrap() as usize;
    let mut arr: Vec<i64> = nums.take(n).collect();
    merge_sort(&mut arr);
    let strs: Vec<String> = arr.iter().map(|x| x.to_string()).collect();
    println!("{}", strs.join(" "));
}
