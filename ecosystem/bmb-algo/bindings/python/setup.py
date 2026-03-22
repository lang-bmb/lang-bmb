from setuptools import setup, find_packages

setup(
    name="bmb-algo",
    version="0.1.0",
    description="High-performance algorithms powered by BMB — 6.8x faster than C",
    long_description=open("../../README.md").read(),
    long_description_content_type="text/markdown",
    author="BMB Team",
    author_email="contact@bmb-lang.dev",
    url="https://github.com/iyulab/lang-bmb",
    project_urls={
        "Documentation": "https://github.com/iyulab/lang-bmb/tree/main/ecosystem/bmb-algo",
        "Source": "https://github.com/iyulab/lang-bmb",
        "Benchmarks": "https://github.com/iyulab/lang-bmb/tree/main/ecosystem/benchmark-bmb",
    },
    py_modules=["bmb_algo"],
    package_data={"": ["*.dll", "*.so", "*.dylib"]},
    include_package_data=True,
    python_requires=">=3.8",
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "Intended Audience :: Science/Research",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Topic :: Scientific/Engineering :: Mathematics",
        "Topic :: Software Development :: Libraries",
    ],
    keywords="algorithms knapsack lcs dijkstra floyd performance bmb",
)
