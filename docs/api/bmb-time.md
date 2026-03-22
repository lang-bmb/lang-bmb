# bmb-time — Time Module API

Monotonic clock, sleep, and duration utilities.

## Clock Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `now_ns()` | `() -> i64` | Current monotonic time in nanoseconds |
| `now_ms()` | `() -> i64` | Current monotonic time in milliseconds |

**Contracts**: `post ret >= 0`

**Note**: Uses `CLOCK_MONOTONIC` (Unix) or `QueryPerformanceCounter` (Windows).

## Sleep

| Function | Signature | Description |
|----------|-----------|-------------|
| `sleep_ms(ms)` | `(i64) -> i64` | Sleep for given milliseconds |

**Contracts**: `pre ms >= 0`

## Duration Converters

| Function | Signature | Contract |
|----------|-----------|----------|
| `secs_to_ms(s)` | `(i64) -> i64` | `pre s >= 0`, `post ret == s * 1000` |
| `secs_to_ns(s)` | `(i64) -> i64` | `pre s >= 0`, `post ret == s * 1000000000` |
| `ms_to_ns(ms)` | `(i64) -> i64` | `pre ms >= 0`, `post ret == ms * 1000000` |
| `ns_to_ms(ns)` | `(i64) -> i64` | `pre ns >= 0`, `post ret == ns / 1000000` |
| `ns_to_secs(ns)` | `(i64) -> i64` | `pre ns >= 0`, `post ret == ns / 1000000000` |
| `ms_to_secs(ms)` | `(i64) -> i64` | `pre ms >= 0`, `post ret == ms / 1000` |

## Elapsed Helpers

| Function | Signature | Contract |
|----------|-----------|----------|
| `elapsed_ns(start, end)` | `(i64, i64) -> i64` | `pre start >= 0 and end >= start` |
| `elapsed_ms(start, end)` | `(i64, i64) -> i64` | `pre start >= 0 and end >= start` |

## Example

```bmb
use time::now_ms;
use time::elapsed_ms;
use time::sleep_ms;

fn main() -> i64 = {
    let t0 = now_ms();
    sleep_ms(100);
    let t1 = now_ms();
    let duration = elapsed_ms(t0, t1);
    println(duration);    // ~100
    0
};
```
