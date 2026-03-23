"""
conftest.py - pytest configuration for bmb-algo tests.

Adds the bindings/python directory to sys.path so tests can be run both from
a pip install AND directly from the repository without installation.
"""

import sys
import os

# Allow running tests from the repo without pip install
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'bindings', 'python'))

# Windows DLL directory — needed so ctypes can find GCC/MSYS2 runtime DLLs
if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)
