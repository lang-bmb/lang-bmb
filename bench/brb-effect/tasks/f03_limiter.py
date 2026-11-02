from cases import I64_MAX

ID = "f03-limiter"
NAME = "limiter"
PARAMS = [("times", "&[i64]"), ("cap", "i64"), ("every", "i64")]
RET = "i64"
STATEMENT = (
    "Run a token-bucket rate limiter. The bucket holds at most cap tokens; it starts full, with its refill mark "
    "at time 0. Request k arrives at time times[k]. For each request, in order: first refill, where "
    "g = (times[k] - mark) / every (integer division) whole periods have passed since the mark, the bucket gains g "
    "tokens but never holds more than cap, and the mark moves forward by g * every; then, if the bucket holds at "
    "least one token, the request is accepted, one token is removed and you print 1, otherwise print 0 "
    "(print each value with print_i64 in BMB, or println!(\"{}\", value) in Rust). Return the number of accepted "
    "requests.")
DOMAIN = "times is non-decreasing and every time is >= 0; cap >= 0; every >= 1."
BMB_DOMAIN = """spec fn dom(times: &[i64], cap: i64, every: i64) -> bool {
    cap >= 0 && every >= 1
        && (forall|i: usize| i < times.len() ==> #[trigger] times[i] >= 0)
        && (forall|p: usize, q: usize| p < q && q < times.len() ==> #[trigger] times[p] <= #[trigger] times[q])
}"""


def domain(times, cap, every):
    return cap >= 0 and every >= 1 and all(t >= 0 for t in times) and all(
        times[i] <= times[i + 1] for i in range(len(times) - 1))


def model(times, cap, every):
    tokens, mark, accepted, out = cap, 0, 0, []
    for t in times:
        g = (t - mark) // every
        tokens = min(cap, tokens + g)
        mark += g * every
        if tokens > 0:
            tokens -= 1
            accepted += 1
            out.append(1)
        else:
            out.append(0)
    return accepted, out


def times(g, n, every):
    t = g.choice([0, 0, g.int(0, 100), g.int(0, 1 << 40), I64_MAX - g.int(0, 1 << 20)])
    out = []
    for _ in range(n):
        k = g.rng.randrange(6)
        step = (0 if k == 0 else g.int(0, 3) if k == 1 else g.int(0, 3 * every) if k == 2 and every < (1 << 60)
                else g.int(0, 1 << 40) if k == 3 else g.int(0, I64_MAX) if k == 4 else every - 1)
        t = min(I64_MAX, t + step)
        out.append(t)
    return out


def cases(g):
    yield ([], 0, 1)
    yield ([0, 0, 0], 2, 1)
    yield ([0, 1, 2, 3], 1, 2)
    yield ([5, 9, 10, 14, 15], 1, 5)
    yield ([0, I64_MAX], I64_MAX, 1)
    yield ([0, 0, I64_MAX, I64_MAX], 1, 1)
    yield ([I64_MAX, I64_MAX], 0, I64_MAX)
    for _ in range(460):
        every = g.choice([1, 1, 2, 3, 5, 7, 10, 100, g.int(1, 1000), 1 << 32, I64_MAX // 3, I64_MAX])
        cap = g.choice([0, 1, 1, 2, 3, 5, g.int(0, 20), 1 << 40, I64_MAX - 1, I64_MAX])
        yield (times(g, g.choice([1, 2, 3, 5, 8, g.int(0, 16)]), every), cap, every)


BMB_REF = """spec fn bucket(tokens: i64, cap: i64) -> bool { 0 <= tokens && tokens <= cap }

spec fn span(mark: i64, t: i64) -> bool { 0 <= mark && mark <= t }

spec fn period(every: i64) -> bool { every >= 1 }

spec fn times_ok(times: &[i64]) -> bool {
    (times.len() == 0 || times[0] >= 0)
        && (forall|i: usize| 0 < i && i < times.len() ==> times[i - 1] <= #[trigger] times[i])
}

#[effects(pure)]
#[uses(bucket)]
fn refill(tokens: i64, g: i64, cap: i64) -> (r: i64)
    requires bucket(tokens, cap),
    ensures bucket(r, cap),
{
    if g >= cap - tokens { cap } else { if g > 0 { tokens + g } else { tokens } }
}

#[effects(pure)]
#[uses(span, period)]
fn periods(mark: i64, t: i64, every: i64) -> (r: i64)
    requires span(mark, t), period(every),
{
    (t - mark) / every
}

#[effects(pure)]
#[uses(span, period)]
fn advance(mark: i64, t: i64, every: i64) -> (r: i64)
    requires span(mark, t), period(every),
    ensures span(r, t),
{
    t - (t - mark) % every
}

#[effects(io)]
#[uses(bucket, span, period, times_ok)]
fn limiter(times: &[i64], cap: i64, every: i64) -> (r: i64)
    requires times_ok(times), bucket(cap, cap), period(every),
{
    let mut tokens: i64 = cap;
    let mut mark: i64 = 0;
    let mut accepted: i64 = 0;
    let mut k: usize = 0;
    while k < times.len()
        invariant k <= times.len(), bucket(tokens, cap), accepted <= k,
            k == 0 ==> mark == 0, k > 0 ==> span(mark, times[k - 1]),
        decreases times.len() - k,
    {
        let t: i64 = times[k];
        tokens = refill(tokens, periods(mark, t, every), cap);
        mark = advance(mark, t, every);
        if tokens > 0 {
            tokens = tokens - 1;
            accepted = accepted + 1;
            print_i64(1);
        } else {
            print_i64(0);
        }
        k = k + 1;
    }
    accepted
}"""

# Weak: the mark moves to the request time, dropping the part of a period
# that had already passed.
BMB_WEAK = """spec fn bucket(tokens: i64, cap: i64) -> bool { 0 <= tokens && tokens <= cap }

spec fn span(mark: i64, t: i64) -> bool { 0 <= mark && mark <= t }

spec fn period(every: i64) -> bool { every >= 1 }

spec fn times_ok(times: &[i64]) -> bool {
    (times.len() == 0 || times[0] >= 0)
        && (forall|i: usize| 0 < i && i < times.len() ==> times[i - 1] <= #[trigger] times[i])
}

#[effects(pure)]
#[uses(bucket)]
fn refill(tokens: i64, g: i64, cap: i64) -> (r: i64)
    requires bucket(tokens, cap),
    ensures bucket(r, cap),
{
    if g >= cap - tokens { cap } else { if g > 0 { tokens + g } else { tokens } }
}

#[effects(pure)]
#[uses(span, period)]
fn periods(mark: i64, t: i64, every: i64) -> (r: i64)
    requires span(mark, t), period(every),
{
    (t - mark) / every
}

#[effects(pure)]
#[uses(span, period)]
fn advance(mark: i64, t: i64, every: i64) -> (r: i64)
    requires span(mark, t), period(every),
    ensures span(r, t),
{
    t - (t - mark) % every
}

#[effects(io)]
#[uses(bucket, span, period, times_ok)]
fn limiter(times: &[i64], cap: i64, every: i64) -> (r: i64)
    requires times_ok(times), bucket(cap, cap), period(every),
{
    let mut tokens: i64 = cap;
    let mut mark: i64 = 0;
    let mut accepted: i64 = 0;
    let mut k: usize = 0;
    while k < times.len()
        invariant k <= times.len(), bucket(tokens, cap), accepted <= k,
            k == 0 ==> mark == 0, k > 0 ==> span(mark, times[k - 1]),
        decreases times.len() - k,
    {
        let t: i64 = times[k];
        tokens = refill(tokens, periods(mark, t, every), cap);
        mark = t;
        if tokens > 0 {
            tokens = tokens - 1;
            accepted = accepted + 1;
            print_i64(1);
        } else {
            print_i64(0);
        }
        k = k + 1;
    }
    accepted
}"""

RUST_REF = """pub fn limiter(times: &[i64], cap: i64, every: i64) -> i64 {
    let (mut tokens, mut mark, mut accepted) = (cap, 0i64, 0i64);
    for &t in times {
        let g = (t - mark) / every;
        tokens = if g >= cap - tokens { cap } else { tokens + g };
        mark += g * every;
        if tokens > 0 {
            tokens -= 1;
            accepted += 1;
            println!("1");
        } else {
            println!("0");
        }
    }
    accepted
}
"""

# Weak: (tokens + g).min(cap) overflows when many periods pass with a large
# cap; its tests use small times.
RUST_WEAK = """pub fn limiter(times: &[i64], cap: i64, every: i64) -> i64 {
    let (mut tokens, mut mark, mut accepted) = (cap, 0i64, 0i64);
    for &t in times {
        let g = (t - mark) / every;
        tokens = (tokens + g).min(cap);
        mark += g * every;
        if tokens > 0 {
            tokens -= 1;
            accepted += 1;
            println!("1");
        } else {
            println!("0");
        }
    }
    accepted
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn limits() {
        assert_eq!(limiter(&[], 3, 1), 0);
        assert_eq!(limiter(&[0, 0, 0], 2, 1), 2);
        assert_eq!(limiter(&[0, 0, 5, 5], 1, 5), 2);
        assert_eq!(limiter(&[1, 2, 3], 0, 1), 0);
    }
}
"""


def _rust(refill, mark="mark += g * every;", gain="(t - mark) / every", start="cap"):
    return ("pub fn limiter(times: &[i64], cap: i64, every: i64) -> i64 {\n"
            "    let (mut tokens, mut mark, mut accepted, mut last) = (" + start + ", 0i64, 0i64, 0i64);\n"
            "    for &t in times {\n"
            "        let g = " + gain + ";\n        " + refill + "\n        " + mark + "\n        last = t;\n"
            "        if tokens > 0 { tokens -= 1; accepted += 1; println!(\"1\"); } else { println!(\"0\"); }\n"
            "    }\n    let _ = last;\n    accepted\n}")


_REFILL = "tokens = if g >= cap - tokens { cap } else { tokens + g };"

RUST_MUTANTS = [
    _rust("tokens = (tokens + g).min(cap);"),               # overflow
    _rust(_REFILL, mark="mark = t;"),                       # drops the partial period
    _rust(_REFILL, mark="mark = t;", gain="(t - last) / every"),  # periods since the last request
    _rust(_REFILL, start="0"),                              # starts empty
]
