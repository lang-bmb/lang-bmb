# Phone Keypad

Given n keypad presses (digits 2-9), output total letters on those keys.

## Input
- First integer: n
- Next n integers: keypad keys pressed (2-9)

## Output
Total letter count (one integer)

## Letter counts
2=3, 3=3, 4=3, 5=3, 6=3, 7=4, 8=3, 9=4

## Example
`3 2 3 4` -> 3+3+3=9

## BMB Notes
- Lookup via if-else chain; 7 and 9 have 4 letters, all others (2-6, 8) have 3
```
let n: i64 = read_int();
let mut total: i64 = 0;
for _i in 0..n {
    let k: i64 = read_int();
    let letters: i64 = if k == 7 { 4 } else { if k == 9 { 4 } else { 3 } };
    set total = total + letters
};
println(total);
0
