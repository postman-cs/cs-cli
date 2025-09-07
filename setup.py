"""Setup script for Customer Transcripts CLI - Standalone Team Calls Extractor."""

from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="customer-transcripts-cli",
    version="1.0.0",
    author="Customer Success Team",
    description="Extract customer calls from Gong and save as markdown transcripts",
    long_description=long_description,
    long_description_content_type="text/markdown",
    packages=find_packages(),
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
    ],
    python_requires=">=3.8",
    install_requires=[
        "structlog>=23.1.0",
        "click>=8.0.0", 
        "rich>=13.0.0",
        "browser-cookie3==0.19.1",
        "pydantic>=2.0.0",
        "curl-cffi>=0.5.0",
        "asyncio>=3.4.3",
    ],
    entry_points={
        "console_scripts": [
            "customer-transcripts=team_calls.cli:cli",
            "team-calls=team_calls.cli:cli",
        ],
    },
    include_package_data=True,
    zip_safe=False,
)