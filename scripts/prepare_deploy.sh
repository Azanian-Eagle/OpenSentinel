#!/bin/bash
set -e

# Clean up previous build
rm -rf dist

# Create dist directory
mkdir -p dist

# Copy all files from docs to dist (Root deployment)
echo "Copying files to dist/ (Root)..."
cp -r docs/* dist/

# Create docs subdirectory for /docs path support
echo "Copying files to dist/docs/ (Docs subdirectory)..."
mkdir -p dist/docs
cp -r docs/* dist/docs/

# Obfuscate JS files for Root
echo "Obfuscating JavaScript files (Root)..."
npx javascript-obfuscator docs/assets/js/ --output dist/assets/js/ --options-preset high-obfuscation

# Obfuscate JS files for Docs
echo "Obfuscating JavaScript files (Docs subdirectory)..."
npx javascript-obfuscator docs/assets/js/ --output dist/docs/assets/js/ --options-preset high-obfuscation

# Add .nojekyll to prevent Jekyll processing
touch dist/.nojekyll

echo "Build complete. Output in dist/"
