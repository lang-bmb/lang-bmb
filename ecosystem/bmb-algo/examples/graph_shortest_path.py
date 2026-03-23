"""Shortest-path demo using bmb-algo — Dijkstra + Floyd-Warshall."""
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'bindings', 'python'))
if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)

import bmb_algo

INF = 10**9  # represents no direct edge

# 5-node city graph (adjacency matrix).  0 = no self-loop, INF = no edge.
# Nodes: 0=Home  1=Work  2=Shop  3=Gym  4=Park
adj = [
    [0,   4,   INF, 8,   INF],
    [4,   0,   8,   11,  INF],
    [INF, 8,   0,   7,   2  ],
    [8,   11,  7,   0,   9  ],
    [INF, INF, 2,   9,   0  ],
]
labels = ["Home", "Work", "Shop", "Gym", "Park"]

# --- Dijkstra from Home (node 0) ---
dist = bmb_algo.dijkstra(adj, source=0)
print("Dijkstra shortest distances from Home:")
for i, d in enumerate(dist):
    tag = "unreachable" if d >= INF else f"{d}"
    print(f"  Home -> {labels[i]:<6}: {tag}")

# --- Floyd-Warshall (all-pairs) ---
all_pairs = bmb_algo.floyd_warshall(adj)
print("\nFloyd-Warshall all-pairs shortest paths:")
print(f"  {'':6}", end="")
for lbl in labels:
    print(f"  {lbl:<6}", end="")
print()
for i, row in enumerate(all_pairs):
    print(f"  {labels[i]:<6}", end="")
    for d in row:
        cell = "  INF   " if d >= INF else f"  {d:<6}"
        print(cell, end="")
    print()
