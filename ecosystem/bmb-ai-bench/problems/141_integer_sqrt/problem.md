# 141. Integer Square Root

음이 아닌 정수 n의 정수 제곱근(floor)을 반환하라. (sqrt 라이브러리 함수 사용 불가)

## 입력
- 첫째 줄: 정수 n (0 <= n <= 2^31 - 1)

## 출력
- floor(sqrt(n))

## 예시
```
입력:
8
출력:
2
```

## BMB Notes

이진 탐색으로 floor(sqrt(n)) 계산:

```bmb
fn isqrt(lo: i64, hi: i64, n: i64) -> i64
= if lo > hi { lo - 1 }
  else {
    let mid = lo + (hi - lo) / 2;
    if mid * mid <= n { isqrt(mid + 1, hi, n) }
    else { isqrt(lo, mid - 1, n) }
  };

fn main() -> i64 = {
    let n = read_int();
    println(isqrt(0, n, n));
    0
};
```

LeetCode #69 (external)
