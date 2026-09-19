from setuptools import setup, find_packages
from pathlib import Path

this_directory = Path(__file__).parent
long_description = (this_directory / "README.md").read_text(encoding="utf-8") if (this_directory / "README.md").exists() else ""

setup(
    name="opensentinel",
    version="1.0.4",
    description="Official Python SDK for OpenSentinel by Azanian Eagle: The ultimate privacy-first, zero-click CAPTCHA replacement and bot detection service. Visit https://azanian-eagle.github.io/OpenSentinel/ to explore the service and https://github.com/Azanian-Eagle for creator profile.",
    long_description=long_description,
    long_description_content_type="text/markdown",
    author="Azanian Eagle",
    author_email="azanian.eagle@gmail.com",
    url="https://github.com/Azanian-Eagle/OpenSentinel",
    packages=find_packages(),
    install_requires=[],
    python_requires=">=3.7",
    keywords=[
        "opensentinel",
        "captcha",
        "bot-detection",
        "security",
        "privacy",
        "verification",
        "azanian-eagle",
        "anti-bot",
        "telemetry",
        "south-africa",
        "recaptcha-alternative",
        "turnstile-alternative",
        "hcaptcha-alternative",
        "captcha-replacement",
        "zero-click-captcha",
        "bot-protection-service",
        "azanian-eagle-creator"
    ],
    classifiers=[
        "Development Status :: 5 - Production/Stable",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Topic :: Security",
        "Topic :: Internet :: WWW/HTTP",
    ],
    project_urls={
        "Homepage": "https://github.com/Azanian-Eagle/OpenSentinel",
        "Repository": "https://github.com/Azanian-Eagle/OpenSentinel",
        "Documentation": "https://github.com/Azanian-Eagle/OpenSentinel#readme",
        "Bug Tracker": "https://github.com/Azanian-Eagle/OpenSentinel/issues",
    },
)
