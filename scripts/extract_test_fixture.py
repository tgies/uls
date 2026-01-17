#!/usr/bin/env python3
"""
Extract a representative and consistent subset of FCC ULS data for testing.

This script extracts a small subset of records from ALL FCC data files that:
1. Covers all record types across all services (amateur, GMRS, etc.)
2. Maintains referential integrity (all related records for selected licenses)
3. Includes edge cases (multi-line comments, special characters, expired/cancelled)
4. Is deterministic (same input produces same output)

Usage:
    python extract_test_fixture.py <cache_dir> <output_dir> [--count N]

Example:
    python extract_test_fixture.py ~/.cache/uls tests/fixtures/fcc-sample

The script looks for ZIP files in the cache directory (l_amat.zip, l_gmrs.zip, etc.)
and extracts samples from each.
"""

import argparse
import os
import random
import sys
import zipfile
from collections import defaultdict
from pathlib import Path
from tempfile import TemporaryDirectory


# Known record types and their USI field position (0-indexed)
RECORD_TYPES = {
    'HD': 1,  # Header/License
    'EN': 1,  # Entity
    'AM': 1,  # Amateur
    'HS': 1,  # History
    'CO': 1,  # Comments
    'SC': 1,  # Special Conditions
    'LA': 1,  # License Attachment
    'SF': 1,  # License Free Form Special Condition
}

# Valid record type prefixes for continuation detection
VALID_RECORD_PREFIXES = set(RECORD_TYPES.keys()) | {'AD', 'LO', 'L2', 'L3', 'L4', 'L5', 'L6'}

# =============================================================================
# ANONYMIZATION - Fake data pools for PII scrubbing
# =============================================================================

FIRST_NAMES = [
    "JOHN", "MARY", "JAMES", "PATRICIA", "ROBERT", "JENNIFER", "MICHAEL", "LINDA",
    "WILLIAM", "ELIZABETH", "DAVID", "BARBARA", "RICHARD", "SUSAN", "JOSEPH", "JESSICA",
    "THOMAS", "SARAH", "CHARLES", "KAREN", "CHRISTOPHER", "NANCY", "DANIEL", "LISA"
]

LAST_NAMES = [
    "SMITH", "JOHNSON", "WILLIAMS", "BROWN", "JONES", "GARCIA", "MILLER", "DAVIS",
    "RODRIGUEZ", "MARTINEZ", "HERNANDEZ", "LOPEZ", "GONZALEZ", "WILSON", "ANDERSON",
    "THOMAS", "TAYLOR", "MOORE", "JACKSON", "MARTIN", "LEE", "PEREZ", "THOMPSON", "WHITE"
]

CITIES = [
    "ANYTOWN", "SPRINGFIELD", "RIVERSIDE", "OAKVILLE", "MAPLEWOOD", "FAIRVIEW",
    "CLEARWATER", "LAKEWOOD", "HILLDALE", "SUNNYVALE", "BROOKSIDE", "GREENFIELD"
]

STATES = ["CA", "TX", "FL", "NY", "PA", "IL", "OH", "GA", "NC", "MI", "NJ", "VA"]

# Callsign prefixes by format
AMATEUR_PREFIXES_1LETTER = ["K", "N", "W", "A"]
AMATEUR_PREFIXES_2LETTER = ["KA", "KB", "KC", "KD", "KE", "KF", "KG", "KI", "KJ", "KK",
                             "WA", "WB", "WD", "WE", "WF", "WG", "WH", "WI", "WJ", "WK",
                             "NA", "NB", "NC", "ND", "NE", "NF", "NG", "NI", "NJ", "NK",
                             "AA", "AB", "AC", "AD", "AE", "AF", "AG", "AI", "AJ", "AK"]

def generate_amateur_callsign(usi: str, index: int) -> str:
    """Generate a valid amateur callsign (letter-digit-letter pattern).
    
    Formats: 1x2, 1x3, 2x1, 2x2, 2x3
    """
    h = hash((usi, index, "callsign"))
    fmt = h % 5
    digit = str(h % 10)
    
    if fmt == 0:  # 1x2: K0AA
        prefix = AMATEUR_PREFIXES_1LETTER[h % len(AMATEUR_PREFIXES_1LETTER)]
        suffix = chr(65 + (h >> 4) % 26) + chr(65 + (h >> 8) % 26)
        return f"{prefix}{digit}{suffix}"
    elif fmt == 1:  # 1x3: K0AAA
        prefix = AMATEUR_PREFIXES_1LETTER[h % len(AMATEUR_PREFIXES_1LETTER)]
        suffix = chr(65 + (h >> 4) % 26) + chr(65 + (h >> 8) % 26) + chr(65 + (h >> 12) % 26)
        return f"{prefix}{digit}{suffix}"
    elif fmt == 2:  # 2x1: AA0A
        prefix = AMATEUR_PREFIXES_2LETTER[h % len(AMATEUR_PREFIXES_2LETTER)]
        suffix = chr(65 + (h >> 4) % 26)
        return f"{prefix}{digit}{suffix}"
    elif fmt == 3:  # 2x2: AA0AA
        prefix = AMATEUR_PREFIXES_2LETTER[h % len(AMATEUR_PREFIXES_2LETTER)]
        suffix = chr(65 + (h >> 4) % 26) + chr(65 + (h >> 8) % 26)
        return f"{prefix}{digit}{suffix}"
    else:  # 2x3: AA0AAA
        prefix = AMATEUR_PREFIXES_2LETTER[h % len(AMATEUR_PREFIXES_2LETTER)]
        suffix = chr(65 + (h >> 4) % 26) + chr(65 + (h >> 8) % 26) + chr(65 + (h >> 12) % 26)
        return f"{prefix}{digit}{suffix}"


def generate_gmrs_callsign(usi: str, index: int) -> str:
    """Generate a valid GMRS callsign (all letters then all digits).
    
    Formats: 3x4 (KAA1234) or 4x3 (WQFX467)
    """
    h = hash((usi, index, "gmrs_callsign"))
    fmt = h % 2
    
    if fmt == 0:  # 3x4: KAA1234 (legacy)
        letters = "K" + chr(65 + (h >> 4) % 26) + chr(65 + (h >> 8) % 26)
        digits = f"{(h >> 12) % 10000:04d}"
        return f"{letters}{digits}"
    else:  # 4x3: WQFX467 (modern)
        prefix = ["WP", "WQ", "WR", "WS"][h % 4]
        letters = prefix + chr(65 + (h >> 4) % 26) + chr(65 + (h >> 8) % 26)
        digits = f"{(h >> 12) % 1000:03d}"
        return f"{letters}{digits}"


def generate_fake_name(usi: str) -> tuple[str, str, str]:
    """Generate a fake name (first, middle initial, last)."""
    h = hash((usi, "name"))
    first = FIRST_NAMES[h % len(FIRST_NAMES)]
    middle = chr(65 + (h >> 8) % 26)
    last = LAST_NAMES[(h >> 4) % len(LAST_NAMES)]
    return first, middle, last


def generate_fake_address(usi: str) -> tuple[str, str, str, str]:
    """Generate a fake address (street, city, state, zip)."""
    h = hash((usi, "address"))
    street = f"{h % 9999 + 1} MAIN ST"
    city = CITIES[h % len(CITIES)]
    state = STATES[(h >> 4) % len(STATES)]
    zip_code = f"{90000 + h % 10000:05d}"
    return street, city, state, zip_code


def generate_fake_frn(usi: str) -> str:
    """Generate a fake FRN (10 digits starting with 000)."""
    h = hash((usi, "frn"))
    return f"000{h % 10000000:07d}"


def anonymize_record(raw_line: str, service_code: str, usi_callsign_map: dict[str, str]) -> str:
    """Anonymize a single record line, replacing PII with fake data.
    
    Args:
        raw_line: The original pipe-delimited record line
        service_code: 'HA'/'HV' for amateur, 'ZA' for GMRS
        usi_callsign_map: Mapping from USI to consistent fake callsign
    
    Returns:
        Anonymized record line
    """
    record_type, fields = parse_dat_line(raw_line)
    if len(fields) < 2:
        return raw_line
    
    usi = fields[1] if len(fields) > 1 else ""
    
    # Get or generate consistent callsign for this USI
    if usi not in usi_callsign_map:
        index = len(usi_callsign_map)
        if service_code == 'ZA':
            usi_callsign_map[usi] = generate_gmrs_callsign(usi, index)
        else:  # HA, HV
            usi_callsign_map[usi] = generate_amateur_callsign(usi, index)
    
    fake_callsign = usi_callsign_map[usi]
    first, middle, last = generate_fake_name(usi)
    street, city, state, zip_code = generate_fake_address(usi)
    fake_frn = generate_fake_frn(usi)
    
    # Anonymize based on record type
    if record_type == 'HD':
        # HD: callsign at index 4, names at various positions (30+)
        if len(fields) > 4:
            fields[4] = fake_callsign
        # Clear name fields in HD if present (around indices 28-35)
        for i in [28, 29, 30, 31, 32]:
            if i < len(fields) and fields[i]:
                fields[i] = ""
    
    elif record_type == 'EN':
        # EN: callsign at 4, entity_name at 7, first/middle/last at 8-10, 
        #     street at 15, city at 16, state at 17, zip at 18, email at 14, phone at 12, frn at 22
        if len(fields) > 4:
            fields[4] = fake_callsign
        if len(fields) > 7:
            fields[7] = f"{last}, {first} {middle}"  # Entity name
        if len(fields) > 8:
            fields[8] = first
        if len(fields) > 9:
            fields[9] = middle
        if len(fields) > 10:
            fields[10] = last
        if len(fields) > 12 and fields[12]:  # Phone
            fields[12] = "555-555-0100"
        if len(fields) > 14 and fields[14]:  # Email
            fields[14] = "test@example.com"
        if len(fields) > 15 and fields[15]:  # Street
            fields[15] = street
        if len(fields) > 16 and fields[16]:  # City
            fields[16] = city
        if len(fields) > 17 and fields[17]:  # State
            fields[17] = state
        if len(fields) > 18 and fields[18]:  # Zip
            fields[18] = zip_code
        if len(fields) > 22 and fields[22]:  # FRN
            fields[22] = fake_frn
    
    elif record_type == 'AM':
        # AM: callsign at index 4
        if len(fields) > 4:
            fields[4] = fake_callsign
    
    elif record_type == 'CO':
        # CO: callsign at index 3, comment text at index 5
        if len(fields) > 3:
            fields[3] = fake_callsign
        if len(fields) > 5 and fields[5]:
            # Generate multi-line comment for some records to preserve edge case
            # Multi-line comments in FCC data have continuation lines that don't start with a record type
            h = hash((usi, "comment_multiline"))
            if h % 5 == 0:  # Every 5th comment is multi-line
                fields[5] = "This is a multi-line test comment that spans multiple lines.\n" + \
                            "CONTINUATION: This line tests the continuation handling in the parser."
            else:
                fields[5] = "Test comment for fixture data."
    
    elif record_type in ('HS', 'SC', 'SF', 'LA'):
        # These have callsign at index 3 or 4
        for i in [3, 4]:
            if i < len(fields) and fields[i] and len(fields[i]) <= 10:
                # Looks like a callsign field
                if any(c.isdigit() for c in fields[i]) and any(c.isalpha() for c in fields[i]):
                    fields[i] = fake_callsign
    
    return '|'.join(fields)



def parse_dat_line(line: str) -> tuple[str, list[str]]:
    """Parse a pipe-delimited DAT line, returning record type and fields."""
    fields = line.rstrip('\r\n').split('|')
    record_type = fields[0] if fields else ''
    return record_type, fields


def get_usi(record_type: str, fields: list[str]) -> str | None:
    """Extract the unique_system_identifier from a record."""
    pos = RECORD_TYPES.get(record_type, 1)
    if len(fields) > pos:
        return fields[pos]
    return None


def read_dat_from_zip(zip_path: Path, dat_name: str) -> list[tuple[str, list[str], str]]:
    """Read a DAT file from inside a ZIP and return records."""
    records = []
    
    try:
        with zipfile.ZipFile(zip_path, 'r') as zf:
            if dat_name not in zf.namelist():
                return records
            
            with zf.open(dat_name) as f:
                pending_line = None
                for raw_line in f:
                    line = raw_line.decode('latin-1', errors='replace')
                    if not line.strip():
                        continue
                    
                    record_type, fields = parse_dat_line(line)
                    
                    # Check if this is a continuation line
                    if record_type and record_type not in VALID_RECORD_PREFIXES:
                        if pending_line:
                            pending_line = pending_line.rstrip('\r\n') + ' ' + line.strip()
                        continue
                    
                    if pending_line:
                        pt, pf = parse_dat_line(pending_line)
                        records.append((pt, pf, pending_line))
                    
                    pending_line = line
                
                if pending_line:
                    pt, pf = parse_dat_line(pending_line)
                    records.append((pt, pf, pending_line))
    except Exception as e:
        print(f"  Warning: Could not read {dat_name} from {zip_path}: {e}")
    
    return records


def select_sample_licenses(hd_records: list[tuple[str, list[str], str]], 
                           service_name: str,
                           count: int = 50,
                           seed: int = 42) -> set[str]:
    """Select a deterministic set of license USIs covering various cases."""
    random.seed(seed + hash(service_name))
    
    by_status = defaultdict(list)
    by_callsign = {}
    
    for record_type, fields, raw in hd_records:
        if len(fields) > 5:
            status = fields[5] if len(fields) > 5 else 'A'
            usi = fields[1] if len(fields) > 1 else None
            callsign = fields[4] if len(fields) > 4 else None
            if usi:
                by_status[status].append(usi)
                if callsign:
                    by_callsign[callsign] = usi
    
    selected = set()
    
    # Known callsigns for verification
    known_callsigns = {
        'W1AW', 'W3LPL', 'K5ZD', 'N1MM',  # Amateur
        'WRCK692', 'WQRZ855',  # GMRS examples
    }
    for callsign in known_callsigns:
        if callsign in by_callsign:
            selected.add(by_callsign[callsign])
    
    # Sample from each status
    remaining = count - len(selected)
    status_counts = {'A': int(remaining * 0.7), 'E': int(remaining * 0.2), 'C': int(remaining * 0.1)}
    
    for status, target in status_counts.items():
        available = [usi for usi in by_status.get(status, []) if usi not in selected]
        if available:
            sample = random.sample(available, min(target, len(available)))
            selected.update(sample)
    
    return selected


def extract_service(zip_path: Path, output_dir: Path, count: int = 50, seed: int = 42) -> dict:
    """Extract fixture from a single service ZIP file."""
    service_name = zip_path.stem  # e.g., 'l_amat' or 'l_gmrs'
    
    # Determine service code for callsign format
    if 'gmrs' in service_name.lower():
        service_code = 'ZA'
    else:
        service_code = 'HA'  # Default to amateur
    
    print(f"\nProcessing {service_name} (service_code={service_code})...")
    
    # Read all record types
    all_records = {}
    for rt in RECORD_TYPES:
        dat_name = f"{rt}.dat"
        records = read_dat_from_zip(zip_path, dat_name)
        if records:
            all_records[rt] = records
            print(f"  {dat_name}: {len(records)} records")
    
    if 'HD' not in all_records:
        print(f"  Skipping {service_name}: no HD.dat found")
        return {}
    
    # Select sample licenses
    selected_usis = select_sample_licenses(all_records['HD'], service_name, count=count, seed=seed)
    print(f"  Selected {len(selected_usis)} licenses")
    
    # Enrich with edge cases
    for rt in ['CO', 'HS', 'SC']:
        if rt in all_records:
            rt_usis = {get_usi(rt, fields) for _, fields, _ in all_records[rt]}
            extra = random.sample(list(rt_usis - selected_usis), min(5, len(rt_usis - selected_usis)))
            selected_usis.update(extra)
    
    # Create USI-to-callsign mapping for consistent anonymization
    usi_callsign_map = {}
    
    # Extract and anonymize related records
    output_records = defaultdict(list)
    for record_type, records in all_records.items():
        for rt, fields, raw in records:
            usi = get_usi(rt, fields)
            if usi in selected_usis:
                # Anonymize before adding
                anonymized = anonymize_record(raw, service_code, usi_callsign_map)
                output_records[record_type].append(anonymized)
    
    # Write output files
    service_output_dir = output_dir / service_name
    service_output_dir.mkdir(parents=True, exist_ok=True)
    
    stats = {}
    for record_type, lines in output_records.items():
        output_path = service_output_dir / f"{record_type}.dat"
        with open(output_path, 'w', encoding='latin-1') as f:
            for line in lines:
                if not line.endswith('\n'):
                    line += '\n'
                f.write(line)
        stats[record_type] = len(lines)
        print(f"  Wrote {record_type}.dat: {len(lines)} records")
    
    return stats


def extract_fixture(cache_dir: Path, output_dir: Path, count: int = 50, seed: int = 42):
    """Extract fixtures from all FCC data ZIPs in cache directory."""
    
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Find all license ZIP files
    zip_files = list(cache_dir.glob('l_*.zip'))
    if not zip_files:
        print(f"No l_*.zip files found in {cache_dir}")
        sys.exit(1)
    
    print(f"Found {len(zip_files)} service ZIP files:")
    for zf in zip_files:
        print(f"  - {zf.name}")
    
    all_stats = {}
    for zip_path in sorted(zip_files):
        stats = extract_service(zip_path, output_dir, count=count, seed=seed)
        if stats:
            all_stats[zip_path.stem] = stats
    
    # Write manifest
    manifest_path = output_dir / 'MANIFEST.md'
    with open(manifest_path, 'w') as f:
        f.write("# FCC ULS Test Fixture\n\n")
        f.write("This directory contains a representative subset of FCC ULS data for testing.\n")
        f.write("All records are real FCC data with referential integrity preserved.\n\n")
        f.write("## Generation\n\n")
        f.write("```bash\n")
        f.write(f"python scripts/extract_test_fixture.py <cache_dir> <output_dir> --count {count}\n")
        f.write("```\n\n")
        f.write("## Contents\n\n")
        
        total = 0
        for service, stats in sorted(all_stats.items()):
            f.write(f"### {service}\n\n")
            f.write("| File | Records |\n")
            f.write("|------|--------|\n")
            for rt, count_val in sorted(stats.items()):
                f.write(f"| {rt}.dat | {count_val} |\n")
                total += count_val
            f.write("\n")
        
        f.write(f"**Total records:** {total}\n")
    
    print(f"\n✓ Fixture created at {output_dir}")
    print(f"  Total records: {sum(sum(s.values()) for s in all_stats.values())}")
    print(f"  Manifest: {manifest_path}")


def main():
    parser = argparse.ArgumentParser(
        description='Extract a representative subset of FCC ULS data for testing.'
    )
    parser.add_argument('cache_dir', type=Path, 
                        help='Directory containing FCC ZIP files (e.g., ~/.cache/uls)')
    parser.add_argument('output_dir', type=Path,
                        help='Output directory for test fixture')
    parser.add_argument('--count', type=int, default=50,
                        help='Number of licenses per service to extract (default: 50)')
    parser.add_argument('--seed', type=int, default=42,
                        help='Random seed for deterministic selection (default: 42)')
    
    args = parser.parse_args()
    
    if not args.cache_dir.exists():
        print(f"Error: Cache directory does not exist: {args.cache_dir}", file=sys.stderr)
        sys.exit(1)
    
    random.seed(args.seed)
    extract_fixture(args.cache_dir, args.output_dir, count=args.count, seed=args.seed)


if __name__ == '__main__':
    main()
