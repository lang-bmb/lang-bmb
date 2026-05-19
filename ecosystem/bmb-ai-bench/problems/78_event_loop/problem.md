# Event Loop

Process n events. Each event has a type and a value. Accumulate the values and print the running total after each event (event type is ignored - all add to accumulator).

## Input
- First integer: n
- Next 2n integers: type value pairs

## Output
Running total after each event (n lines)

## Example
`3 1 10 2 20 3 30` -> events (1,10),(2,20),(3,30) -> running totals: 10, 30, 60

## BMB Notes
- Read BOTH type and value for each event; type is discarded
```
let n: i64 = read_int();
let mut acc: i64 = 0;
for _i in 0..n {
    let _t = read_int();   // event type (ignored)
    let val: i64 = read_int();
    acc = acc + val;
    println(acc)
};
0
```
