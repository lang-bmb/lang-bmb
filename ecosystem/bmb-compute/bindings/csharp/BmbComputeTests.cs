using BmbComputeLib;

int pass = 0, fail = 0;

void Check(string name, bool condition)
{
    if (condition) { Console.WriteLine($"  PASS  {name}"); pass++; }
    else           { Console.Error.WriteLine($"  FAIL  {name}"); fail++; }
}

Console.WriteLine("=== BmbCompute C# bindings smoke test ===");

// Scalar math
Check("Abs(-5)=5",         Compute.Abs(-5) == 5);
Check("Min(3,7)=3",        Compute.Min(3, 7) == 3);
Check("Max(3,7)=7",        Compute.Max(3, 7) == 7);
Check("Clamp(10,0,5)=5",   Compute.Clamp(10, 0, 5) == 5);
Check("Sign(-3)=-1",       Compute.Sign(-3) == -1);
Check("Sign(5)=1",         Compute.Sign(5) == 1);
Check("IPow(2,10)=1024",   Compute.IPow(2, 10) == 1024);
Check("Sqrt(16)=4",        Compute.Sqrt(16) == 4);
Check("Factorial(5)=120",  Compute.Factorial(5) == 120);
Check("IsPowerOfTwo(8)",    Compute.IsPowerOfTwo(8));
Check("!IsPowerOfTwo(7)",   !Compute.IsPowerOfTwo(7));
Check("NextPowerOfTwo(5)=8", Compute.NextPowerOfTwo(5) == 8);

// Array ops
long[] data = [1, 2, 3, 4, 5];
Check("Sum([1..5])=15",   Compute.Sum(data) == 15);
Check("MinVal=1",         Compute.MinVal(data) == 1);
Check("MaxVal=5",         Compute.MaxVal(data) == 5);
Check("RangeVal=4",       Compute.RangeVal(data) == 4);
Check("DotProduct",       Compute.DotProduct(data, [2, 2, 2, 2, 2]) == 30);

Console.WriteLine($"\n{pass} passed, {fail} failed");
Environment.Exit(fail > 0 ? 1 : 0);
