# Base Convert

Convert decimal n to base b. Output each digit as its decimal value, concatenated.

## Input
- First integer: t
- Each test case: number base

## Output
For each: digits of (number in base b), each digit printed as decimal, concatenated (one per line)

## Notes
- Special case: 0 -> "0"
- For base 16: digit 15 prints as "15" (two chars), not 'F'

## Example
- 10 in base 2 -> digits [1,0,1,0] -> "1010"
- 255 in base 16 -> digits [15,15] -> "1515"
