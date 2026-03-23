"""
bmb-json: High-performance JSON parsing powered by BMB
https://github.com/iyulab/lang-bmb
"""

from setuptools import setup
import os

here = os.path.dirname(os.path.abspath(__file__))

setup(
    name='bmb-json',
    version='0.1.0',
    description='High-performance JSON parsing and generation powered by BMB',
    long_description=open(os.path.join(here, 'README.md')).read() if os.path.exists(os.path.join(here, 'README.md')) else '',
    long_description_content_type='text/markdown',
    author='iyulab',
    author_email='iyulab@example.com',
    url='https://github.com/iyulab/lang-bmb',
    packages=['bmb_json'],
    package_dir={'bmb_json': 'bindings/python'},
    package_data={'bmb_json': ['*.dll', '*.so', '*.dylib']},
    python_requires='>=3.8',
    classifiers=[
        'Development Status :: 4 - Beta',
        'Intended Audience :: Developers',
        'Programming Language :: Python :: 3',
        'Topic :: Software Development :: Libraries',
        'Topic :: Internet :: WWW/HTTP',
    ],
    keywords='json parser serializer performance bmb',
)
