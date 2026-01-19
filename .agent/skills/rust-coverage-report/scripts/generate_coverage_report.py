#!/usr/bin/env python3
"""
Generate a markdown coverage report from cargo-llvm-cov JSON output.

Usage:
    python3 generate_coverage_report.py coverage.json
    python3 generate_coverage_report.py coverage.json --output report.md
"""

import json
import sys
import argparse
from pathlib import Path


def extract_crate_name(filename: str) -> str:
    """Extract crate name from a file path.
    
    Handles paths like:
    - /path/to/project/crates/my-crate/src/lib.rs -> my-crate
    - /path/to/project/src/lib.rs -> (project root)
    """
    parts = Path(filename).parts
    
    # Look for 'crates' directory pattern
    if 'crates' in parts:
        idx = parts.index('crates')
        if idx + 1 < len(parts):
            return parts[idx + 1]
    
    # Look for 'src' and use parent as crate name
    if 'src' in parts:
        idx = parts.index('src')
        if idx > 0:
            return parts[idx - 1]
    
    return "unknown"


def get_relative_path(filename: str, crate_name: str) -> str:
    """Get the file path relative to the crate root."""
    marker = f"crates/{crate_name}/"
    if marker in filename:
        return filename.split(marker)[-1]
    
    # Fallback: just get src/... portion
    if '/src/' in filename:
        idx = filename.index('/src/')
        return filename[idx + 1:]
    
    return filename


def generate_report(json_path: str) -> str:
    """Generate markdown report from llvm-cov JSON output."""
    with open(json_path, 'r') as f:
        data = json.load(f)

    # llvm-cov json format: { "data": [ { "files": [ ... ], "totals": {...} } ] }
    files = data['data'][0]['files']
    
    # Aggregate by crate
    crates = {}
    
    for file_data in files:
        filename = file_data['filename']
        crate_name = extract_crate_name(filename)
        
        summary = file_data['summary']
        lines = summary['lines']
        
        if crate_name not in crates:
            crates[crate_name] = {
                'count': 0,
                'covered': 0,
                'files': []
            }
        
        crates[crate_name]['count'] += lines['count']
        crates[crate_name]['covered'] += lines['covered']
        crates[crate_name]['files'].append({
            'name': get_relative_path(filename, crate_name),
            'count': lines['count'],
            'covered': lines['covered'],
            'percent': lines['percent']
        })

    # Build the report
    lines = []
    lines.append("# Code Coverage Report\n")
    
    # Summary table
    lines.append("## Summary by Crate")
    lines.append("| Crate | Covered Lines | Total Lines | Percentage |")
    lines.append("| :--- | :--- | :--- | :--- |")
    
    # Sort crates by coverage percentage (descending)
    sorted_crates = sorted(
        crates.items(),
        key=lambda x: (x[1]['covered'] / x[1]['count'] * 100) if x[1]['count'] > 0 else 0,
        reverse=True
    )
    
    for crate_name, crate_data in sorted_crates:
        percent = (crate_data['covered'] / crate_data['count'] * 100) if crate_data['count'] > 0 else 0
        lines.append(f"| **{crate_name}** | {crate_data['covered']} | {crate_data['count']} | **{percent:.2f}%** |")
    
    lines.append("")
    
    # Detailed per-crate sections
    lines.append("---\n")
    lines.append("## Detailed Coverage by File\n")
    
    for crate_name, crate_data in sorted_crates:
        percent = (crate_data['covered'] / crate_data['count'] * 100) if crate_data['count'] > 0 else 0
        
        lines.append(f"<details>")
        lines.append(f"<summary><b>{crate_name} ({percent:.2f}%)</b></summary>\n")
        lines.append("| File | Covered Lines | Total Lines | Percentage |")
        lines.append("| :--- | :--- | :--- | :--- |")
        
        # Sort files by name
        for file_info in sorted(crate_data['files'], key=lambda x: x['name']):
            lines.append(f"| {file_info['name']} | {file_info['covered']} | {file_info['count']} | {file_info['percent']:.2f}% |")
        
        lines.append("</details>\n")
    
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(
        description="Generate markdown coverage report from cargo-llvm-cov JSON"
    )
    parser.add_argument(
        "json_file",
        help="Path to the coverage.json file from cargo-llvm-cov"
    )
    parser.add_argument(
        "--output", "-o",
        help="Output file (defaults to stdout)"
    )
    
    args = parser.parse_args()
    
    if not Path(args.json_file).exists():
        print(f"Error: File not found: {args.json_file}", file=sys.stderr)
        sys.exit(1)
    
    report = generate_report(args.json_file)
    
    if args.output:
        with open(args.output, 'w') as f:
            f.write(report)
        print(f"Report written to {args.output}")
    else:
        print(report)


if __name__ == "__main__":
    main()
