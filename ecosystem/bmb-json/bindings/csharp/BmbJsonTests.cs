using BmbJsonLib;

int pass = 0, fail = 0;

void Check(string name, bool condition)
{
    if (condition) { Console.WriteLine($"  PASS  {name}"); pass++; }
    else           { Console.Error.WriteLine($"  FAIL  {name}"); fail++; }
}

Console.WriteLine("=== BmbJson C# bindings smoke test ===");

string obj  = """{"name":"Alice","age":30,"active":true}""";
string arr  = "[1,2,3]";
string num  = "42";

// Validate
Check("Validate object", Json.Validate(obj));
Check("Validate array",  Json.Validate(arr));
Check("Validate bad",    !Json.Validate("{bad}"));

// Type
Check("Type object", Json.Type(obj) == "object");
Check("Type array",  Json.Type(arr) == "array");
Check("Type number", Json.Type(num) == "number");

// Object operations
Check("GetNumber(age)=30",  Json.GetNumber(obj, "age") == 30);
Check("GetString(name)",    Json.GetString(obj, "name") == "Alice");
Check("HasKey(active)",     Json.HasKey(obj, "active"));
Check("!HasKey(missing)",   !Json.HasKey(obj, "missing"));
Check("ObjectLen=3",        Json.ObjectLen(obj) == 3);
Check("GetBool(active)=1",  Json.GetBool(obj, "active") == 1);

// Array operations
Check("ArrayLen=3",         Json.ArrayLen(arr) == 3);

// Stringify
string roundtrip = Json.Stringify(obj);
Check("Stringify non-empty", roundtrip.Length > 0);

Console.WriteLine($"\n{pass} passed, {fail} failed");
Environment.Exit(fail > 0 ? 1 : 0);
