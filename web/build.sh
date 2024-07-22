#!/bin/bash
set -e

# Build the Wasm package
wasm-pack build --target web

# remove dist directory if it exists
if [ ! -d "dist" ]; then
  rm -rf dist
fi

# Create a dist directory
mkdir -p dist

# Copy the generated Wasm files
cp -r pkg dist/

# Copy static files
cp -r static/* dist/

# To prevent github action from skipping
rm dist/pkg/.gitignore

echo "Build complete. Output is in the dist directory."
