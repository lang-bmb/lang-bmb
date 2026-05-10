using BmbCryptoLib;

int pass = 0, fail = 0;

void Check(string name, bool condition)
{
    if (condition) { Console.WriteLine($"  PASS  {name}"); pass++; }
    else           { Console.Error.WriteLine($"  FAIL  {name}"); fail++; }
}

Console.WriteLine("=== BmbCrypto C# bindings smoke test ===");

// SHA-256 (known value for "hello")
string sha256 = Crypto.Sha256("hello");
Check("Sha256 length=64",  sha256.Length == 64);
Check("Sha256 hex chars",  sha256.All(c => "0123456789abcdef".Contains(c)));
Check("Sha256 deterministic", Crypto.Sha256("hello") == sha256);

// MD5
string md5 = Crypto.Md5("hello");
Check("Md5 non-empty",     md5.Length > 0);

// Base64
string b64 = Crypto.Base64Encode("Hello, World!");
Check("Base64 non-empty",  b64.Length > 0);
string decoded = Crypto.Base64Decode(b64);
Check("Base64 roundtrip",  decoded == "Hello, World!");

// Rot13
Check("Rot13 hello=uryyb",  Crypto.Rot13("hello") == "uryyb");
Check("Rot13 double",        Crypto.Rot13(Crypto.Rot13("hello")) == "hello");

// HMAC-SHA256
string hmac = Crypto.HmacSha256("key", "message");
Check("HmacSha256 non-empty", hmac.Length > 0);
Check("HmacSha256 deterministic", Crypto.HmacSha256("key", "message") == hmac);

Console.WriteLine($"\n{pass} passed, {fail} failed");
Environment.Exit(fail > 0 ? 1 : 0);
