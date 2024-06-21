#!/bin/bash
set -e

# Build the Wasm package
wasm-pack build --target web

# Create a dist directory
mkdir -p dist

# Copy the generated Wasm files
cp -r pkg dist/

# Copy static files
cp static/* dist/

echo "Build complete. Output is in the dist directory."
