#!/bin/bash
# collect_daily_files.sh - Archive FCC daily transaction files with canonical dates
#
# Downloads daily transaction files (Mon-Sat) and archives them with their
# canonical creation date extracted from the internal "counts" file.
#
# Usage: ./scripts/collect_daily_files.sh [--service SERVICE] [--archive-dir DIR]

set -euo pipefail

# Configuration
FCC_BASE_URL="https://data.fcc.gov/download/pub/uls/daily"
SERVICES=("am" "gm")  # Amateur, GMRS
WEEKDAYS=("mon" "tue" "wed" "thu" "fri" "sat")
ARCHIVE_DIR="${ARCHIVE_DIR:-data/archives/daily}"
TEMP_DIR="${TMPDIR:-/tmp}/fcc_daily_$$"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() { echo -e "${GREEN}[INFO]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $*" >&2; }

cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

# Extract canonical date from counts file inside ZIP
# Returns YYYY-MM-DD format or empty string if not found
extract_canonical_date() {
    local zip_file="$1"
    
    # Extract counts file and parse "File Creation Date: Sun Jan 18 12:01:25 EST 2026"
    local date_line
    date_line=$(unzip -p "$zip_file" counts 2>/dev/null | grep "^File Creation Date:" || true)
    
    if [[ -z "$date_line" ]]; then
        return 1
    fi
    
    # Parse: "File Creation Date: Sun Jan 18 12:01:25 EST 2026"
    # Extract: Jan 18 ... 2026
    local raw_date
    raw_date=$(echo "$date_line" | sed 's/File Creation Date: //')
    
    # Use date command to convert to YYYY-MM-DD
    # Input example: "Sun Jan 18 12:01:25 EST 2026"
    local canonical_date
    canonical_date=$(date -d "$raw_date" +%Y-%m-%d 2>/dev/null || true)
    
    if [[ -z "$canonical_date" ]]; then
        return 1
    fi
    
    echo "$canonical_date"
}

# Download and archive a single daily file
process_daily_file() {
    local service="$1"
    local weekday="$2"
    
    local url="${FCC_BASE_URL}/l_${service}_${weekday}.zip"
    local temp_file="${TEMP_DIR}/l_${service}_${weekday}.zip"
    
    log_info "Checking $url..."
    
    # Download with curl, save ETag
    local http_code
    http_code=$(curl -sS -w "%{http_code}" -o "$temp_file" "$url" 2>/dev/null || echo "000")
    
    if [[ "$http_code" != "200" ]]; then
        log_warn "Skipping l_${service}_${weekday}.zip (HTTP $http_code)"
        rm -f "$temp_file"
        return 0
    fi
    
    # Extract canonical date from counts file
    local canonical_date
    if ! canonical_date=$(extract_canonical_date "$temp_file"); then
        log_warn "Could not extract date from l_${service}_${weekday}.zip counts file"
        rm -f "$temp_file"
        return 0
    fi
    
    # Target filename with canonical date
    local target_file="${ARCHIVE_DIR}/l_${service}_${canonical_date}.zip"
    
    # Check if already archived
    if [[ -f "$target_file" ]]; then
        log_info "Already archived: l_${service}_${canonical_date}.zip"
        rm -f "$temp_file"
        return 0
    fi
    
    # Move to archive
    mkdir -p "$ARCHIVE_DIR"
    mv "$temp_file" "$target_file"
    log_info "Archived: l_${service}_${canonical_date}.zip (from ${weekday})"
}

main() {
    log_info "FCC Daily File Collector"
    log_info "Archive directory: $ARCHIVE_DIR"
    
    mkdir -p "$TEMP_DIR"
    mkdir -p "$ARCHIVE_DIR"
    
    local count=0
    for service in "${SERVICES[@]}"; do
        for weekday in "${WEEKDAYS[@]}"; do
            if process_daily_file "$service" "$weekday"; then
                ((count++)) || true
            fi
        done
    done
    
    log_info "Processed $count files"
}

main "$@"
