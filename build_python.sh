#!/bin/bash
# Script to build the Python package

set -e

# Check if maturin is installed
if ! command -v maturin &> /dev/null; then
    echo "maturin is not installed. Installing..."
    pip install maturin
fi

# Build the package
echo "Building Python package..."
maturin build --release

echo "Package built successfully!"
echo "To install the package locally, run:"
echo "pip install target/wheels/<wheel-file>"
echo ""
echo "To publish to PyPI, run:"
echo "maturin publish" 