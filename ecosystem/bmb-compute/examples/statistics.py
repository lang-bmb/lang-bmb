"""Statistics demo using bmb-compute — sum, mean, min, max, range, variance."""
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'bindings', 'python'))
if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)

import bmb_compute

# Dataset: monthly sales figures (integer units)
sales = [142, 198, 175, 210, 163, 229, 188, 245, 172, 195, 211, 234]
months = ["Jan","Feb","Mar","Apr","May","Jun",
          "Jul","Aug","Sep","Oct","Nov","Dec"]

print("=== Monthly Sales Dataset ===")
for m, v in zip(months, sales):
    bar = "#" * (v // 10)
    print(f"  {m}: {v:>4}  {bar}")

print("\n=== Statistics ===")
total    = bmb_compute.sum(sales)
# mean_scaled returns mean * 100 (fixed-point, avoids floats in BMB)
mean_fp  = bmb_compute.mean_scaled(sales)
lo       = bmb_compute.min_val(sales)
hi       = bmb_compute.max_val(sales)
spread   = bmb_compute.range_val(sales)
# variance_scaled returns variance * 100
var_fp   = bmb_compute.variance_scaled(sales)

print(f"  Count    : {len(sales)}")
print(f"  Sum      : {total}")
print(f"  Mean     : {mean_fp / 100:.2f}")
print(f"  Min      : {lo}  ({months[sales.index(lo)]})")
print(f"  Max      : {hi}  ({months[sales.index(hi)]})")
print(f"  Range    : {spread}")
print(f"  Variance : {var_fp / 100:.2f}")
print(f"  Std Dev  : {(var_fp / 100) ** 0.5:.2f}")
