#!/bin/bash
set -e

# Clean up previous build
rm -rf dist

# Create dist directory
mkdir -p dist

# Copy all files from docs to dist
cp -r docs/* dist/

# Obfuscate JS files
# Input: docs/assets/js/ (source)
# Output: dist/assets/js/ (destination, overwriting copies)
echo "Obfuscating JavaScript files..."
npx javascript-obfuscator docs/assets/js/ --output dist/assets/js/ --options-preset high-obfuscation

echo "Build complete. Output in dist/"
