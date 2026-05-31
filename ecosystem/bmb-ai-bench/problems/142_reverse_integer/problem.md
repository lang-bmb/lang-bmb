# 142. Reverse Integer

부호 있는 32비트 정수 x의 자릿수를 뒤집어라. 결과가 [-2^31, 2^31-1] 범위를 벗어나면 0을 반환.

## 입력
- 첫째 줄: 정수 x

## 출력
- 뒤집은 수 (오버플로 시 0)

## 예시
```
입력:
123
출력:
321
```

## BMB Notes

```bmb
fn reverse(x: i64, acc: i64) -> i64
= if x == 0 { acc }
  else { reverse(x / 10, acc * 10 + x % 10) };

fn main() -> i64 = {
    let x = read_int();
    let sign = if x < 0 { 0 - 1 } else { 1 };
    let abs_x = if x < 0 { 0 - x } else { x };
    let rev = reverse(abs_x, 0) * sign;
    let limit = 2147483647;
    if rev > limit or rev < 0 - limit - 1 { println(0) }
    else { println(rev) };
    0
};
```

LeetCode #7 (external)
