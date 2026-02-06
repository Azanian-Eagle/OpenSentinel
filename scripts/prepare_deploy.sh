#!/bin/bash
set -e
set -x

# Clean up previous build
rm -rf dist
mkdir -p dist

# 1. Create /docs subdirectory by copying the source docs folder
# This results in dist/docs/ containing the site.
echo "Creating dist/docs/..."
cp -r docs dist/

# 2. Copy the contents of docs to the root of dist
# This results in dist/ containing the site at root.
echo "Populating dist/ root..."
cp -r docs/* dist/

# Obfuscate JS files for Root
echo "Obfuscating JavaScript files (Root)..."
# We target dist/assets explicitly
npx javascript-obfuscator dist/assets/js/ --output dist/assets/js/ --options-preset high-obfuscation

# Obfuscate JS files for Docs subdirectory
echo "Obfuscating JavaScript files (Docs subdirectory)..."
# We target dist/docs/assets explicitly
npx javascript-obfuscator dist/docs/assets/js/ --output dist/docs/assets/js/ --options-preset high-obfuscation

# Add .nojekyll to prevent Jekyll processing
touch dist/.nojekyll

# Ensure correct permissions for static hosting
# Directories: 755 (drwxr-xr-x)
find dist -type d -exec chmod 755 {} +
# Files: 644 (-rw-r--r--)
find dist -type f -exec chmod 644 {} +

echo "Build complete. Output in dist/"
ls -la dist
ls -la dist/docs
