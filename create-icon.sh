#!/bin/bash

# Create .icns file from a single 1024x1024 PNG
# Usage: ./create-icon.sh icon_1024x1024.png

set -e

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <1024x1024.png>"
    echo "Please provide a 1024x1024 PNG file"
    exit 1
fi

SOURCE_PNG="$1"
ICONSET_DIR="AppIcon.iconset"

if [ ! -f "$SOURCE_PNG" ]; then
    echo "Error: File $SOURCE_PNG not found"
    exit 1
fi

echo "Creating icon set from $SOURCE_PNG..."

# Create iconset directory
rm -rf "$ICONSET_DIR"
mkdir "$ICONSET_DIR"

# Generate all required sizes
sips -z 16 16     "$SOURCE_PNG" --out "$ICONSET_DIR/icon_16x16.png"      > /dev/null
sips -z 32 32     "$SOURCE_PNG" --out "$ICONSET_DIR/icon_16x16@2x.png"   > /dev/null
sips -z 32 32     "$SOURCE_PNG" --out "$ICONSET_DIR/icon_32x32.png"      > /dev/null
sips -z 64 64     "$SOURCE_PNG" --out "$ICONSET_DIR/icon_32x32@2x.png"   > /dev/null
sips -z 128 128   "$SOURCE_PNG" --out "$ICONSET_DIR/icon_128x128.png"    > /dev/null
sips -z 256 256   "$SOURCE_PNG" --out "$ICONSET_DIR/icon_128x128@2x.png" > /dev/null
sips -z 256 256   "$SOURCE_PNG" --out "$ICONSET_DIR/icon_256x256.png"    > /dev/null
sips -z 512 512   "$SOURCE_PNG" --out "$ICONSET_DIR/icon_256x256@2x.png" > /dev/null
sips -z 512 512   "$SOURCE_PNG" --out "$ICONSET_DIR/icon_512x512.png"    > /dev/null
sips -z 1024 1024 "$SOURCE_PNG" --out "$ICONSET_DIR/icon_512x512@2x.png" > /dev/null

# Create the .icns file
iconutil -c icns "$ICONSET_DIR" -o AppIcon.icns

# Clean up
rm -rf "$ICONSET_DIR"

echo "✅ Created AppIcon.icns"
echo ""
echo "Icon file ready for use in your .app bundle!"