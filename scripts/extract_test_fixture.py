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
    
    print(f"\nProcessing {service_name}...")
    
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
    
    # Extract related records
    output_records = defaultdict(list)
    for record_type, records in all_records.items():
        for rt, fields, raw in records:
            usi = get_usi(rt, fields)
            if usi in selected_usis:
                output_records[record_type].append(raw)
    
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
        f.write(f"python scripts/extract_test_fixture.py {cache_dir} {output_dir} --count {count}\n")
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
