use std::io::{self, Read};

fn quicksort(arr: &mut [i64]) {
    if arr.len() <= 1 {
        return;
    }
    let pivot_idx = partition(arr);
    let (left, right) = arr.split_at_mut(pivot_idx);
    quicksort(left);
    quicksort(&mut right[1..]);
}

fn partition(arr: &mut [i64]) -> usize {
    let hi = arr.len() - 1;
    let pivot = arr[hi];
    let mut i = 0usize;
    for j in 0..hi {
        if arr[j] <= pivot {
            arr.swap(i, j);
            i += 1;
        }
    }
    arr.swap(i, hi);
    i
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut nums = input.split_whitespace().map(|s| s.parse::<i64>().unwrap());

    let n = nums.next().unwrap() as usize;
    let mut arr: Vec<i64> = nums.take(n).collect();

    quicksort(&mut arr);

    let parts: Vec<String> = arr.iter().map(|x| x.to_string()).collect();
    println!("{}", parts.join(" "));
}
