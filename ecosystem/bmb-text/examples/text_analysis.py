"""Text analysis demo using bmb-text — word count, search, replace, palindromes."""
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'bindings', 'python'))
if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)

import bmb_text

text = "The quick brown fox jumps over the lazy dog. The dog barked loudly."

print("=== Input Text ===")
print(text)

# Word and pattern counts
print("\n=== Analysis ===")
print(f"Word count      : {bmb_text.word_count(text)}")
print(f"Occurrences of 'the' (case-sensitive): {bmb_text.str_count(text.lower(), 'the')}")
print(f"First 'dog' at  : index {bmb_text.str_find(text, 'dog')}")
print(f"Last  'dog' at  : index {bmb_text.str_rfind(text, 'dog')}")
print(f"Contains 'fox'  : {bmb_text.str_contains(text, 'fox')}")
print(f"Starts with 'The': {bmb_text.str_starts_with(text, 'The')}")

# Transform
replaced = bmb_text.str_replace_all(text, "dog", "cat")
print("\n=== Replace 'dog' -> 'cat' ===")
print(replaced)

# Case transforms
print("\n=== Case Transforms ===")
print(f"Upper: {bmb_text.to_upper('Hello BMB')}")
print(f"Lower: {bmb_text.to_lower('Hello BMB')}")
print(f"Trim : '{bmb_text.trim('   spaces around   ')}'")

# Palindrome check
words = ["racecar", "level", "hello", "madam", "world"]
print("\n=== Palindrome Check ===")
for w in words:
    print(f"  {w:<10}: {bmb_text.is_palindrome(w)}")
