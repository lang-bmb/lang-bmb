# State Machine

## Description

Simulate a simple state machine that starts at state 0 and processes a sequence of commands.

Commands:
- `1` — increment: state += 1
- `2` — decrement: state -= 1
- `3` — double: state *= 2
- `4` — reset: state = 0
- `5` — negate: state = -state

**Input** (stdin):
- First integer: `n`, number of commands
- Next `n` integers: the commands (1–5), space-separated

**Output** (stdout):
- Print the final state value after processing all commands

## Example

Input:
```
6 1 1 1 3 3 2
```

Output:
```
11
```

Explanation:
- Start: state=0
- cmd 1: state=1
- cmd 1: state=2
- cmd 1: state=3
- cmd 3: state=6
- cmd 3: state=12
- cmd 2: state=11

## Additional Examples

- `3 1 1 1` → 3 (three increments)
- `4 1 1 1 5` → -3 (increment 3 times then negate)
- `2 1 4` → 0 (increment then reset)
- `0` → 0 (no commands, stays at 0)

## Constraints

- 0 ≤ n ≤ 100
- Each command is one of: 1, 2, 3, 4, 5
- State fits in a 64-bit signed integer throughout

## Category

Simulation / state machine
