"""
Setup configuration for LIMINAL package.
"""

from setuptools import setup, find_packages
import os

# Read the README for long description
with open("docs/README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

# Read requirements
with open("requirements.txt", "r", encoding="utf-8") as fh:
    requirements = [line.strip() for line in fh if line.strip() and not line.startswith("#")]

setup(
    name="liminal",
    version="0.1.0-alpha",
    author="LIMINAL Development Team",
    description="Physics-Based Memory Architecture for Large Language Models",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/your-org/LIMINAL",
    packages=find_packages(),
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Science/Research",
        "Topic :: Scientific/Engineering :: Artificial Intelligence",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
    ],
    python_requires=">=3.10",
    install_requires=requirements,
    extras_require={
        "dev": [
            "pytest>=7.4.0",
            "pytest-cov>=4.1.0",
            "black>=23.0.0",
            "flake8>=6.0.0",
            "mypy>=1.4.0",
        ],
        "docs": [
            "sphinx>=7.0.0",
            "sphinx-rtd-theme>=1.2.0",
        ],
    },
    entry_points={
        "console_scripts": [
            "liminal=liminal.cli:main",
        ],
    },
    include_package_data=True,
    package_data={
        "liminal": ["config/*.yaml", "data/*.json"],
    },
)