# 143. Min Cost Climbing Stairs

각 계단에 비용이 있다. 한 번에 1칸 또는 2칸 이동 가능. 꼭대기까지 가는 최소 비용을 반환하라. (인덱스 0 또는 1에서 시작 가능)

## 입력
- 첫째 줄: 계단 수 n
- 다음 n줄: 각 계단 비용

## 출력
- 최소 비용

## 예시
```
입력:
3
10
15
20
출력:
15
```

## BMB Notes

DP: `dp[i] = cost[i] + min(dp[i-1], dp[i-2])`. 배열 없이 재귀로:

```bmb
fn read_costs(n: i64, costs: String) -> String
= if n <= 0 { costs }
  else {
    let c = read_int();
    read_costs(n - 1, costs + int_to_string(c) + ",")
  };

fn parse_cost(s: String, idx: i64) -> i64
= ...;

fn min_cost(costs: String, n: i64) -> i64
= if n == 0 { parse_cost(costs, 0) }
  else if n == 1 { parse_cost(costs, 1) }
  else {
    let c = parse_cost(costs, n);
    c + min(min_cost(costs, n-1), min_cost(costs, n-2))
  };
```

더 간단하게: vec를 사용해 입력을 읽고 DP 계산.

```bmb
fn read_into_vec(v: i64, n: i64) -> i64
= if n <= 0 { v }
  else { let x = read_int(); read_into_vec(vec_push(v, x), n - 1) };

fn dp_step(cost: i64, i: i64, p1: i64, p2: i64) -> i64
= if i >= vec_len(cost) { min(p1, p2) }
  else {
    let c = vec_get(cost, i) + min(p1, p2);
    dp_step(cost, i + 1, c, p1)
  };

fn main() -> i64 = {
    let n = read_int();
    let cost = read_into_vec(vec_new(), n);
    println(dp_step(cost, 2, vec_get(cost, 1), vec_get(cost, 0)));
    0
};
```

LeetCode #746 (external)
