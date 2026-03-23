import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'bindings', 'python'))

if sys.platform == 'win32' and hasattr(os, 'add_dll_directory'):
    for p in [r'C:\msys64\ucrt64\bin', r'C:\msys64\mingw64\bin']:
        if os.path.isdir(p):
            os.add_dll_directory(p)
