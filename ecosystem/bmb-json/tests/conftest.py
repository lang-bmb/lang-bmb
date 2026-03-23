"""
conftest.py for bmb-json pytest suite.

Adds the bindings/python directory to sys.path and registers MSYS2 DLL
directories on Windows so that ctypes can find the runtime dependencies.
"""

import sys
import os

# Make the Python binding importable without installation.
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'bindings', 'python'))

# Windows: register DLL search paths so ctypes can find MinGW/UCRT runtime DLLs.
if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)
