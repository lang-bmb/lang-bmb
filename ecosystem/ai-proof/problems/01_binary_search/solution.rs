use std::io::{self, Read};

fn binary_search(arr: &[i64], target: i64) -> i64 {
    let mut lo: i64 = 0;
    let mut hi: i64 = arr.len() as i64 - 1;
    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        let val = arr[mid as usize];
        if val == target {
            return mid;
        } else if val < target {
            lo = mid + 1;
        } else {
            hi = mid - 1;
        }
    }
    -1
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut nums = input.split_whitespace().map(|s| s.parse::<i64>().unwrap());

    let target = nums.next().unwrap();
    let n = nums.next().unwrap() as usize;
    let arr: Vec<i64> = nums.take(n).collect();

    let result = binary_search(&arr, target);
    println!("{}", result);
}
