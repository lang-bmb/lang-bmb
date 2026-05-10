using BmbAlgo;

// Smoke tests — run as a console app, not a test framework,
// to keep the scaffold dependency-free.

int pass = 0, fail = 0;

void Check(string name, bool condition)
{
    if (condition) { Console.WriteLine($"  PASS  {name}"); pass++; }
    else           { Console.Error.WriteLine($"  FAIL  {name}"); fail++; }
}

Console.WriteLine("=== BmbAlgo C# bindings smoke test ===");

// Math
Check("Gcd(12,8)=4",        Algo.Gcd(12, 8) == 4);
Check("Lcm(4,6)=12",        Algo.Lcm(4, 6) == 12);
Check("Fibonacci(10)=55",   Algo.Fibonacci(10) == 55);
Check("PrimeCount(10)=4",   Algo.PrimeCount(10) == 4);
Check("ModPow(2,10,1000)=24", Algo.ModPow(2, 10, 1000) == 24);
Check("NQueens(8)=92",      Algo.NQueens(8) == 92);
Check("IsPrime(17)",        Algo.IsPrime(17));
Check("!IsPrime(18)",       !Algo.IsPrime(18));
Check("DigitSum(123)=6",    Algo.DigitSum(123) == 6);
Check("BitPopcount(7)=3",   Algo.BitPopcount(7) == 3);
Check("IsPalindromeNum(121)",  Algo.IsPalindromeNum(121));
Check("!IsPalindromeNum(123)", !Algo.IsPalindromeNum(123));

// Bit ops
Check("BitSet(0,3)=8",      Algo.BitSet(0, 3) == 8);
Check("BitClear(8,3)=0",    Algo.BitClear(8, 3) == 0);
Check("BitTest(8,3)",       Algo.BitTest(8, 3));
Check("BitToggle(8,3)=0",   Algo.BitToggle(8, 3) == 0);

// Array ops
long[] arr = [64, 34, 25, 12, 22, 11, 90];
Check("ArraySum",    Algo.ArraySum(arr) == 258);
Check("ArrayMin",    Algo.ArrayMin(arr) == 11);
Check("ArrayMax",    Algo.ArrayMax(arr) == 90);
Check("MaxSubarray([64,34,25,12,22,11,90])=258", Algo.MaxSubarray(arr) == 258);

long[] sorted = [1, 2, 3, 4, 5];
Check("IsSorted",       Algo.IsSorted(sorted));
Check("!IsSorted arr",  !Algo.IsSorted(arr));
Check("BinarySearch(3)=2", Algo.BinarySearch(sorted, 3) == 2);
Check("BinarySearch(9)=-1", Algo.BinarySearch(sorted, 9) == -1);

// Sorting
long[] qs = Algo.QuickSort(arr);
Check("QuickSort", qs[0] == 11 && qs[6] == 90);
long[] ms = Algo.MergeSort(arr);
Check("MergeSort", ms[0] == 11 && ms[6] == 90);

// Knapsack
long[] weights = [2, 3, 4, 5];
long[] values  = [3, 4, 5, 6];
Check("Knapsack(cap=5)=7", Algo.Knapsack(weights, values, 5) == 7);

// LIS
long[] lisArr = [10, 9, 2, 5, 3, 7, 101, 18];
Check("Lis=4", Algo.Lis(lisArr) == 4);

// Coin change
long[] coins = [1, 5, 10, 25];
Check("CoinChange(30)=2", Algo.CoinChange(coins, 30) == 2);

// String ops
Check("Lcs('abcde','ace')=3",          Algo.Lcs("abcde", "ace") == 3);
Check("EditDistance('intention','execution')=5", Algo.EditDistance("intention", "execution") == 5);
Check("Djb2Hash non-zero",             Algo.Djb2Hash("hello") != 0);

// TwoSum
long[] ts = [2, 7, 11, 15];
var (i, j) = Algo.TwoSum(ts, 9);
Check("TwoSum(9) => (0,1)", i == 0 && j == 1);

Console.WriteLine($"\n{pass} passed, {fail} failed");
Environment.Exit(fail > 0 ? 1 : 0);
