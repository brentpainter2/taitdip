#!/bin/bash

# 1. Extract OLD version (before bumping) from Cargo.toml
OLD_VERSION=$(grep '^version =' Cargo.toml | sed -E 's/version = "(.*)"/\1/')

# 2. Bump the version
# This increments the patch version (e.g., 1.0.4 -> 1.0.5)
# Requires 'cargo install cargo-edit'
cargo set-version --bump patch

# 3. Extract new metadata
VERSION=$(grep '^version =' Cargo.toml | sed -E 's/version = "(.*)"/\1/')
BINARY_NAME=$(grep '^name =' Cargo.toml | sed -E 's/name = "(.*)"/\1/')

# 4. Archive the old source code
echo "Archiving old source code to oldversions/src_${OLD_VERSION}/ ..."
mkdir -p oldversions
if [ -d "src" ]; then
    cp -R src "oldversions/src_${OLD_VERSION}/"
else
    echo "Warning: src directory not found, skipping archive."
fi

# 5. Build the release binary
# Since you're using env!("CARGO_PKG_VERSION"), the new $VERSION is baked in here.
cargo build --release

# 6. Define Paths for tait_dip
DIST_DIR="/Users/brentpainter/Desktop/Rust/tait_dip/dist"
BIN_DEST="$DIST_DIR/tait_dip"

# 7. Create the directory structure
mkdir -p "$BIN_DEST/config"

# 8. Copy the versioned binary
# This takes the compiled file and renames it with the version suffix
cp -f "target/release/$BINARY_NAME" "$BIN_DEST/$BINARY_NAME-$VERSION"

# 9. Copy the config folder contents
if [ -d "config" ]; then
    cp -R config/. "$BIN_DEST/config/"
fi

echo "------------------------------------------------"
echo "✅ tait_dip Deployment Complete"
echo "Binary: $BIN_DEST/$BINARY_NAME-$VERSION"
echo "Config: $BIN_DEST/config/"
echo "Old source archived: oldversions/src_${OLD_VERSION}/"
echo "------------------------------------------------"