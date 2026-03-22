"""
bmb-crypto: High-performance cryptographic functions powered by BMB
https://github.com/iyulab/lang-bmb
"""

from setuptools import setup, find_packages
import os

here = os.path.dirname(os.path.abspath(__file__))

setup(
    name='bmb-crypto',
    version='0.2.0',
    description='Fast cryptographic functions (SHA-256, MD5, HMAC, Base64/32, CRC32) powered by BMB',
    long_description=open(os.path.join(here, 'README.md')).read() if os.path.exists(os.path.join(here, 'README.md')) else '',
    long_description_content_type='text/markdown',
    author='iyulab',
    author_email='iyulab@example.com',
    url='https://github.com/iyulab/lang-bmb',
    license='MIT',
    packages=['bmb_crypto'],
    package_dir={'bmb_crypto': 'bindings/python'},
    package_data={'bmb_crypto': ['*.dll', '*.so', '*.dylib']},
    python_requires='>=3.8',
    classifiers=[
        'Development Status :: 4 - Beta',
        'Intended Audience :: Developers',
        'License :: OSI Approved :: MIT License',
        'Programming Language :: Python :: 3',
        'Topic :: Security :: Cryptography',
    ],
    keywords='sha256 md5 hmac base64 base32 crc32 cryptography bmb',
)
