use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut nums = input.split_whitespace().map(|s| s.parse::<i64>().unwrap());
    let n = nums.next().unwrap() as usize;
    let mut arr: Vec<i64> = (0..n).map(|_| nums.next().unwrap()).collect();
    let val = nums.next().unwrap();
    let pos = arr.partition_point(|&x| x <= val);
    arr.insert(pos, val);
    let strs: Vec<String> = arr.iter().map(|x| x.to_string()).collect();
    println!("{}", strs.join(" "));
}
