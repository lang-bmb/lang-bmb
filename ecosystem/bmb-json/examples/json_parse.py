"""JSON parsing demo using bmb-json — validate, extract fields, iterate arrays."""
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'bindings', 'python'))
if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)

import bmb_json

# Sample API-style JSON payload
payload = '{"name":"Alice","age":30,"city":"Seoul","scores":[95,87,92,78,100]}'

# Validation
print("=== Validation ===")
print(f"Valid JSON  : {bmb_json.validate(payload)}")
print(f"Invalid str : {bmb_json.validate('{bad json}')}")

# Type detection
print(f"\nPayload type: {bmb_json.get_type(payload)}")

# Field extraction
print("\n=== Field Extraction ===")
print(f"name : {bmb_json.get_string(payload, 'name')}")
print(f"age  : {bmb_json.get_number(payload, 'age')}")
print(f"city : {bmb_json.get_string(payload, 'city')}")

# Array access
scores_json = bmb_json.get(payload, "scores")
n = bmb_json.array_len(scores_json)
print(f"\n=== Scores array ({n} items) ===")
total = 0
for i in range(n):
    item = bmb_json.array_get(scores_json, i)
    score = int(item)
    total += score
    print(f"  scores[{i}] = {score}")
print(f"Average score: {total / n:.1f}")

# Pretty-print (stringify normalises whitespace)
print("\n=== Stringified ===")
print(bmb_json.stringify(payload))
