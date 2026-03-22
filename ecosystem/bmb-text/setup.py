"""
bmb-text: High-performance string processing powered by BMB
https://github.com/iyulab/lang-bmb
"""

from setuptools import setup, find_packages
import os

here = os.path.dirname(os.path.abspath(__file__))

setup(
    name='bmb-text',
    version='0.1.0',
    description='Fast string search, matching, and analysis powered by BMB',
    long_description=open(os.path.join(here, 'README.md')).read() if os.path.exists(os.path.join(here, 'README.md')) else '',
    long_description_content_type='text/markdown',
    author='iyulab',
    author_email='iyulab@example.com',
    url='https://github.com/iyulab/lang-bmb',
    license='MIT',
    packages=['bmb_text'],
    package_dir={'bmb_text': 'bindings/python'},
    package_data={'bmb_text': ['*.dll', '*.so', '*.dylib']},
    python_requires='>=3.8',
    classifiers=[
        'Development Status :: 3 - Alpha',
        'Intended Audience :: Developers',
        'License :: OSI Approved :: MIT License',
        'Programming Language :: Python :: 3',
        'Topic :: Text Processing',
    ],
    keywords='string search kmp palindrome tokenizer bmb',
)
