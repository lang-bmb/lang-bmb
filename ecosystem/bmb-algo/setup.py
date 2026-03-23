"""
bmb-algo: High-performance algorithms powered by BMB
https://github.com/iyulab/lang-bmb
"""

from setuptools import setup, find_packages
import os

here = os.path.dirname(os.path.abspath(__file__))

setup(
    name='bmb-algo',
    version='0.2.0',
    description='Blazing fast algorithms powered by BMB — 6.8x faster than C on knapsack',
    long_description=open(os.path.join(here, 'README.md')).read() if os.path.exists(os.path.join(here, 'README.md')) else '',
    long_description_content_type='text/markdown',
    author='iyulab',
    author_email='iyulab@example.com',
    url='https://github.com/iyulab/lang-bmb',
    packages=['bmb_algo'],
    package_dir={'bmb_algo': 'bindings/python'},
    package_data={'bmb_algo': ['*.dll', '*.so', '*.dylib']},
    python_requires='>=3.8',
    classifiers=[
        'Development Status :: 4 - Beta',
        'Intended Audience :: Developers',
        'Programming Language :: Python :: 3',
        'Topic :: Scientific/Engineering :: Mathematics',
    ],
    keywords='algorithm knapsack lcs dijkstra floyd sort search bmb',
)
