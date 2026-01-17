#!/bin/bash
# Download FCC ULS data files for development and testing
#
# FCC ULS Data URL Structure:
#   Weekly (complete) databases: https://data.fcc.gov/download/pub/uls/complete/
#   Daily transaction files:      https://data.fcc.gov/download/pub/uls/daily/
#
# File naming:
#   Complete: l_<service>.zip (licenses) or a_<service>.zip (applications)
#   Daily:    l_<abbrev>_<day>.zip where day is mon,tue,wed,thu,fri,sat
#
# Service codes for amateur:
#   amat = amateur (complete)
#   am = amateur (daily)

set -e

BASE_URL="https://data.fcc.gov/download/pub/uls"
DATA_DIR="${1:-./data/fcc}"

mkdir -p "$DATA_DIR/complete"
mkdir -p "$DATA_DIR/daily"
mkdir -p "$DATA_DIR/extracted"

download_file() {
    local url="$1"
    local dest="$2"

    echo "Downloading: $url"
    if command -v curl &> /dev/null; then
        curl -L -o "$dest" "$url" --progress-bar
    elif command -v wget &> /dev/null; then
        wget -O "$dest" "$url"
    else
        echo "Error: Neither curl nor wget is available"
        exit 1
    fi
    echo "Saved to: $dest"
}

# Download weekly (complete) amateur license database
echo "=== Downloading Weekly Amateur License Database ==="
download_file "$BASE_URL/complete/l_amat.zip" "$DATA_DIR/complete/l_amat.zip"

# Download weekly amateur application database
echo ""
echo "=== Downloading Weekly Amateur Application Database ==="
download_file "$BASE_URL/complete/a_amat.zip" "$DATA_DIR/complete/a_amat.zip"

# Download daily amateur transaction files
echo ""
echo "=== Downloading Daily Amateur Transaction Files ==="
for day in mon tue wed thu fri sat; do
    download_file "$BASE_URL/daily/l_am_$day.zip" "$DATA_DIR/daily/l_am_$day.zip" || true
done

# Extract files for analysis
echo ""
echo "=== Extracting Files ==="
unzip -o "$DATA_DIR/complete/l_amat.zip" -d "$DATA_DIR/extracted/l_amat/" 2>/dev/null || true

echo ""
echo "=== Download Complete ==="
echo ""
echo "Files downloaded to: $DATA_DIR"
echo ""
echo "Extracted amateur license DAT files:"
ls -la "$DATA_DIR/extracted/l_amat/" 2>/dev/null || echo "  (extraction may have failed)"
echo ""
echo "File sizes:"
du -sh "$DATA_DIR"/*/* 2>/dev/null || true
