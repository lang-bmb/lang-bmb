"""
bmb-compute: High-performance numeric computation powered by BMB
https://github.com/iyulab/lang-bmb
"""

from setuptools import setup
import os

here = os.path.dirname(os.path.abspath(__file__))

setup(
    name='bmb-compute',
    version='0.1.0',
    description='High-performance numeric computation powered by BMB — math, statistics, random, vector ops',
    long_description=open(os.path.join(here, 'README.md')).read() if os.path.exists(os.path.join(here, 'README.md')) else '',
    long_description_content_type='text/markdown',
    author='iyulab',
    author_email='iyulab@example.com',
    url='https://github.com/iyulab/lang-bmb',
    packages=['bmb_compute'],
    package_dir={'bmb_compute': 'bindings/python'},
    package_data={'bmb_compute': ['*.dll', '*.so', '*.dylib']},
    python_requires='>=3.8',
    classifiers=[
        'Development Status :: 4 - Beta',
        'Intended Audience :: Developers',
        'Intended Audience :: Science/Research',
        'Programming Language :: Python :: 3',
        'Topic :: Scientific/Engineering :: Mathematics',
    ],
    keywords='math statistics random vector numeric computation bmb',
)
