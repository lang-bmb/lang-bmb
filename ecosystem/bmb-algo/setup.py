"""
bmb-algo setup shim.

Metadata lives in pyproject.toml. This file exists to produce a wheel
tagged `py3-none-<platform>` (e.g., py3-none-win_amd64) because we ship
prebuilt native shared libraries that are loaded via ctypes, not as CPython
extensions.

Without this:
  - Plain pyproject.toml build → `py3-none-any` pure-python wheel; a Linux
    user would pip-install a Windows .dll.
  - `has_ext_modules=True` alone → `cp3XX-cp3XX-<platform>`; Python 3.13
    user on the correct OS still wouldn't get the 3.12-built wheel.

The correct tag is `py3-none-<platform>`: platform-specific, Python-version
independent, ABI independent — matching what the binary actually requires.
"""

from setuptools import setup
from setuptools.dist import Distribution

try:
    from setuptools.command.bdist_wheel import bdist_wheel as _bdist_wheel
except ImportError:  # older setuptools / standalone wheel package
    from wheel.bdist_wheel import bdist_wheel as _bdist_wheel


class BinaryDistribution(Distribution):
    def has_ext_modules(self):
        return True


class bdist_wheel_platform(_bdist_wheel):
    def finalize_options(self):
        super().finalize_options()
        # Not pure Python; must tag with platform.
        self.root_is_pure = False

    def get_tag(self):
        _, _, plat = super().get_tag()
        return "py3", "none", plat


setup(
    distclass=BinaryDistribution,
    cmdclass={"bdist_wheel": bdist_wheel_platform},
)
