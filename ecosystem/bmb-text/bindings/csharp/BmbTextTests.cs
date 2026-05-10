using BmbTextLib;

int pass = 0, fail = 0;

void Check(string name, bool condition)
{
    if (condition) { Console.WriteLine($"  PASS  {name}"); pass++; }
    else           { Console.Error.WriteLine($"  FAIL  {name}"); fail++; }
}

Console.WriteLine("=== BmbText C# bindings smoke test ===");

// Search
Check("StrFind('hello','ll')=2",     Text.StrFind("hello world", "ll") == 2);
Check("StrFind missing=-1",           Text.StrFind("hello", "xyz") == -1);
Check("StrContains",                  Text.StrContains("hello world", "world"));
Check("!StrContains missing",         !Text.StrContains("hello", "xyz"));
Check("StrStartsWith",                Text.StrStartsWith("hello", "hel"));
Check("StrEndsWith",                  Text.StrEndsWith("hello", "llo"));
Check("StrCount('hello','l')=2",      Text.StrCount("hello", "l") == 2);
Check("IsPalindrome('racecar')",      Text.IsPalindrome("racecar"));
Check("!IsPalindrome('hello')",       !Text.IsPalindrome("hello"));
Check("WordCount('hello world')=2",   Text.WordCount("hello world") == 2);
Check("TextLen('hello')=5",           Text.TextLen("hello") == 5);
Check("Hamming('abc','axc')=1",       Text.StrHamming("abc", "axc") == 1);

// Transform
Check("StrToUpper",       Text.StrToUpper("hello") == "HELLO");
Check("StrToLower",       Text.StrToLower("HELLO") == "hello");
Check("StrTrim",          Text.StrTrim("  hi  ") == "hi");
Check("StrReverse",       Text.StrReverse("hello") == "olleh");
Check("StrRepeat('ab',3)", Text.StrRepeat("ab", 3) == "ababab");
Check("StrReplace first", Text.StrReplace("aaa", "a", "b") == "baa");
Check("StrReplaceAll",    Text.StrReplaceAll("aaa", "a", "b") == "bbb");

Console.WriteLine($"\n{pass} passed, {fail} failed");
Environment.Exit(fail > 0 ? 1 : 0);
