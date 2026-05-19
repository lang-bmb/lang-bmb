# Calendar Day

Given month and day, compute day-of-year (1-indexed, non-leap year).

## Input
- First integer: **t** (number of test cases)
- Each test case: **month day** (two integers in this order)

## Output
Day number (1=Jan1, 365=Dec31), one per line (t lines total)

## Month lengths (non-leap)
Jan=31, Feb=28, Mar=31, Apr=30, May=31, Jun=30, Jul=31, Aug=31, Sep=30, Oct=31, Nov=30, Dec=31

## Example
(month=1, day=1) → 1; (month=12, day=31) → 365; (month=3, day=1) → 60

## IMPORTANT: t is the query count

```
let t: i64 = read_int();
let mut i: i64 = 0;
while i < t {
    let month: i64 = read_int();
    let day: i64 = read_int();
    // sum days from Jan up to (month-1), then add day
    println(day_of_year(month, day));
    set i = i + 1;
};
0
```

## Implementation Hint

```
fn days_before_month(m: i64) -> i64 = {
    let months: i64 = vec_new();
    vec_push(months, 0);   // padding (month is 1-indexed)
    vec_push(months, 0);   // before Jan: 0
    vec_push(months, 31);  // before Feb: 31
    vec_push(months, 59);  // before Mar: 59
    vec_push(months, 90);  // before Apr
    vec_push(months, 120); // before May
    vec_push(months, 151); // before Jun
    vec_push(months, 181); // before Jul
    vec_push(months, 212); // before Aug
    vec_push(months, 243); // before Sep
    vec_push(months, 273); // before Oct
    vec_push(months, 304); // before Nov
    vec_push(months, 334); // before Dec
    vec_get(months, m)
};

fn day_of_year(month: i64, day: i64) -> i64 = days_before_month(month) + day;
```
