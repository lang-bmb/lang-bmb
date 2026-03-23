"""Knapsack solver using bmb-algo — 0/1 knapsack via dynamic programming."""
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'bindings', 'python'))
if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)

import bmb_algo

# Items: (name, weight, value)
items = [
    ("Laptop",     3, 4000),
    ("Headphones", 1,  500),
    ("Camera",     2, 1200),
    ("Tablet",     2,  900),
    ("Phone",      1,  700),
    ("Charger",    1,  150),
]

capacity = 6  # kg

weights = [w for _, w, _ in items]
values  = [v for _, _, v in items]

best_value = bmb_algo.knapsack(weights, values, capacity)

print(f"Knapsack capacity : {capacity} kg")
print(f"Available items   : {len(items)}")
print()
print("Item breakdown:")
for name, w, v in items:
    print(f"  {name:<12}  weight={w} kg  value=${v}")
print()
print(f"Optimal total value: ${best_value}")

# Bonus: compare a few sub-problems
print("\nSub-problem sweep (capacity 1–6):")
for cap in range(1, capacity + 1):
    val = bmb_algo.knapsack(weights, values, cap)
    print(f"  cap={cap} kg -> ${val}")
