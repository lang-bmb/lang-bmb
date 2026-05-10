# BmbAlgo — C# Bindings

High-performance algorithms powered by BMB, accessible from C# via P/Invoke.

## Requirements

- .NET 8+ or .NET 10+
- `bmb_algo.dll` (Windows) / `libbmb_algo.so` (Linux) / `libbmb_algo.dylib` (macOS)  
  Place the shared library in the same directory as your executable,  
  or set `PATH` / `LD_LIBRARY_PATH` / `DYLD_LIBRARY_PATH` to include its directory.

## Build the native library

```bash
# From the repo root:
cargo build --release --features llvm --target x86_64-pc-windows-gnu  # Windows
cargo build --release --features llvm                                   # Linux/macOS

# Then build the shared library:
./target/release/bmb build ecosystem/bmb-algo/src/lib.bmb --lib -o bmb_algo
```

## Usage

```csharp
using BmbAlgo;

// Math
Console.WriteLine(Algo.Fibonacci(10));          // 55
Console.WriteLine(Algo.PrimeCount(100));        // 25
Console.WriteLine(Algo.NQueens(8));             // 92
Console.WriteLine(Algo.ModPow(2, 10, 1000));    // 24

// Arrays
long[] arr = [64, 34, 25, 12, 22, 11, 90];
long[] sorted = Algo.QuickSort(arr);
Console.WriteLine(Algo.ArraySum(arr));          // 258
Console.WriteLine(Algo.MaxSubarray(arr));       // 258

// Knapsack
long[] weights = [2, 3, 4, 5];
long[] values  = [3, 4, 5, 6];
Console.WriteLine(Algo.Knapsack(weights, values, 5));  // 7

// Strings
Console.WriteLine(Algo.EditDistance("intention", "execution"));  // 5
Console.WriteLine(Algo.Lcs("abcde", "ace"));                    // 3
```

## API

| Method | Description |
|--------|-------------|
| `Gcd(a,b)` | Greatest common divisor |
| `Lcm(a,b)` | Least common multiple |
| `Fibonacci(n)` | n-th Fibonacci (0-indexed) |
| `PrimeCount(n)` | Count primes ≤ n |
| `ModPow(base,exp,mod)` | Fast modular exponentiation |
| `NQueens(n)` | N-Queens solution count |
| `IsPrime(n)` | Primality test |
| `Knapsack(weights,values,cap)` | 0/1 Knapsack |
| `QuickSort(arr)` | Sort (returns copy) |
| `MergeSort(arr)` | Stable sort (returns copy) |
| `BinarySearch(arr,target)` | Binary search in sorted array |
| `MaxSubarray(arr)` | Kadane's algorithm |
| `Lis(arr)` | Longest Increasing Subsequence length |
| `CoinChange(coins,amount)` | Minimum coins |
| `EditDistance(a,b)` | Levenshtein distance |
| `Lcs(a,b)` | Longest Common Subsequence length |
| `TwoSum(arr,target)` | Two-sum index pair |
| ... | 55 functions total — see `BmbAlgo.cs` |

## License

MIT
