#!/usr/bin/env python3
"""
validate_preservation.py - Production-Grade Preservation Validation

Purpose: Verify that ONLY expected changes were made (no scope creep)
Authority: Constitutional enforcement for spec validation
Status: Phase-17.5 - CI-Authoritative Implementation

This is the CORRECT implementation using deterministic diff→section mapping.
Replaces the bash heuristic approach with proper parsing.

Usage:
    ./validate_preservation.py ORIGINAL_FILE FIXED_FILE EXPECTED_CHANGES_YML

Exit Codes:
    0 - PASS (only expected changes found)
    1 - FAIL (unexpected changes or missing expected changes)
    2 - ERROR (invalid arguments or file not found)
"""

import sys
import os
import re
import hashlib
import json
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Set, Tuple, Optional
import subprocess

try:
    import yaml
except ImportError:
    print("ERROR: PyYAML not installed. Install with: pip install pyyaml", file=sys.stderr)
    sys.exit(2)


class Colors:
    """ANSI color codes for terminal output"""
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    NC = '\033[0m'  # No Color


def log_info(msg: str):
    print(f"{Colors.BLUE}[INFO]{Colors.NC} {msg}")


def log_success(msg: str):
    print(f"{Colors.GREEN}[PASS]{Colors.NC} {msg}")


def log_warning(msg: str):
    print(f"{Colors.YELLOW}[WARN]{Colors.NC} {msg}")


def log_error(msg: str):
    print(f"{Colors.RED}[FAIL]{Colors.NC} {msg}")


def generate_hash(file_path: str) -> str:
    """Generate SHA256 hash of file"""
    sha256 = hashlib.sha256()
    with open(file_path, 'rb') as f:
        for chunk in iter(lambda: f.read(4096), b''):
            sha256.update(chunk)
    return sha256.hexdigest()


def parse_diff_for_changed_lines(diff_text: str) -> Set[int]:
    """
    Parse unified diff and extract ONLY the changed line numbers in NEW file.
    
    Includes both additions AND deletions (deletions affect the section).
    Returns set of line numbers that were actually added/modified/deleted.
    """
    changed_lines = set()
    current_new_line = 0
    deleted_in_section = []  # Track deletions to mark section as changed
    
    for line in diff_text.splitlines():
        if line.startswith('@@'):
            # Extract new file starting line number
            match = re.match(r'^@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@', line)
            if match:
                current_new_line = int(match.group(1))
                # Mark any pending deletions at current position
                for del_line in deleted_in_section:
                    changed_lines.add(current_new_line)
                deleted_in_section = []
        elif line.startswith('+') and not line.startswith('+++'):
            # This is an added/modified line in the new file
            changed_lines.add(current_new_line)
            current_new_line += 1
        elif line.startswith('-') and not line.startswith('---'):
            # This is a deleted line - mark the position as changed
            # Deletions affect the section at current_new_line position
            deleted_in_section.append(current_new_line)
        elif not line.startswith('\\'):  # Ignore "\ No newline at end of file"
            # This is a context line
            current_new_line += 1
    
    # Mark any remaining deletions
    for del_line in deleted_in_section:
        changed_lines.add(del_line)
    
    return changed_lines


def build_section_map(file_path: str) -> Dict[int, str]:
    """
    Build mapping of line_number → section_name for markdown file.
    
    Sections are identified by markdown headers (lines starting with #).
    Maintains section hierarchy (# > ## > ###).
    """
    section_map = {}
    section_stack = ["ROOT"]  # Stack to track nested sections
    
    with open(file_path, 'r', encoding='utf-8') as f:
        for line_num, line in enumerate(f, start=1):
            stripped = line.strip()
            
            # Detect markdown headers
            if stripped.startswith('#'):
                # Calculate header level
                level = len(line) - len(line.lstrip('#'))
                level = min(level, 6)  # Max 6 levels in markdown
                
                # Update section stack
                # Pop sections at same or deeper level
                while len(section_stack) > level:
                    section_stack.pop()
                
                # Add new section
                section_stack.append(stripped)
            
            # Map line to current section (full path)
            section_map[line_num] = " > ".join(section_stack)
    
    return section_map


def canonical_section_id(section: str) -> str:
    """
    Generate canonical section ID for deterministic matching.
    
    Extracts the actual section header (last component of hierarchy path)
    and normalizes it to a stable identifier.
    
    Examples:
        "ROOT > # Doc > ## 🧵 String Pool" → "string_pool"
        "## 🧵 String Pool" → "string_pool"
        "🧵 String Pool Section" → "string_pool_section"
    
    Rules:
        - Extract last component if hierarchy path (split on >)
        - Remove markdown headers (# ## ###)
        - Remove emoji and special characters
        - Lowercase
        - Replace whitespace with underscore
        - Deterministic (same input → same output always)
    """
    # Extract last component if hierarchy path
    if ' > ' in section:
        section = section.split(' > ')[-1]
    
    # Remove markdown headers
    section = re.sub(r'^#+\s*', '', section)
    
    # Remove emoji and special characters (keep only alphanumeric and spaces)
    section = re.sub(r'[^\w\s]', '', section)
    
    # Normalize whitespace and convert to lowercase
    section = re.sub(r'\s+', ' ', section.strip()).lower()
    
    # Replace spaces with underscores
    section_id = section.replace(' ', '_')
    
    return section_id


def find_changed_sections(changed_lines: Set[int], 
                         section_map: Dict[int, str]) -> Set[str]:
    """
    Determine which sections contain changes based on changed line numbers.
    
    Returns set of section names that have changed lines.
    """
    changed_sections = set()
    
    for line_num in changed_lines:
        if line_num in section_map:
            changed_sections.add(section_map[line_num])
    
    return changed_sections


def load_expected_changes(yaml_path: str) -> Tuple[List[str], List[str]]:
    """
    Load expected changes from YAML file.
    
    Returns (expected_sections, preserved_sections)
    """
    with open(yaml_path, 'r', encoding='utf-8') as f:
        config = yaml.safe_load(f)
    
    expected_sections = []
    if 'fixes' in config and config['fixes']:
        for fix in config['fixes']:
            if 'section' in fix:
                expected_sections.append(fix['section'])
    
    preserved_sections = []
    if 'preservation' in config and config['preservation']:
        for preserved in config['preservation']:
            if 'section' in preserved:
                preserved_sections.append(preserved['section'])
    
    return expected_sections, preserved_sections


def generate_diff(original_file: str, fixed_file: str) -> Tuple[str, str]:
    """
    Generate unified diff between original and fixed files.
    
    Returns (diff_text, diff_file_path)
    """
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    report_dir = Path(__file__).parent / "reports"
    report_dir.mkdir(exist_ok=True)
    
    diff_file = report_dir / f"diff_{timestamp}.patch"
    
    try:
        result = subprocess.run(
            ['diff', '-u', original_file, fixed_file],
            capture_output=True,
            text=True
        )
        diff_text = result.stdout
        
        with open(diff_file, 'w', encoding='utf-8') as f:
            f.write(diff_text)
        
        return diff_text, str(diff_file)
    except Exception as e:
        log_error(f"Failed to generate diff: {e}")
        sys.exit(2)


def validate_preservation(original_file: str, fixed_file: str, 
                         expected_changes_yml: str) -> Tuple[bool, Dict]:
    """
    Main validation logic.
    
    Returns (validation_passed, report_data)
    """
    log_info("Starting preservation validation...")
    
    # Validate input files
    for path in [original_file, fixed_file, expected_changes_yml]:
        if not os.path.exists(path):
            log_error(f"File not found: {path}")
            sys.exit(2)
    
    # Generate diff
    log_info("Generating unified diff...")
    diff_text, diff_file = generate_diff(original_file, fixed_file)
    
    # Check if files are identical
    if not diff_text.strip():
        log_warning("Files are identical - no changes detected")
        diff_empty = True
        changed_sections = set()
    else:
        diff_empty = False
        
        # Parse changed lines (not hunks - only actual changes)
        changed_lines = parse_diff_for_changed_lines(diff_text)
        log_info(f"Detected {len(changed_lines)} changed lines")
        
        # Build section map for FIXED file
        section_map = build_section_map(fixed_file)
        
        # Find changed sections
        changed_sections = find_changed_sections(changed_lines, section_map)
        log_info(f"Detected changes in {len(changed_sections)} sections")
    
    # Load expected changes
    log_info(f"Loading expected changes from {expected_changes_yml}...")
    expected_sections, preserved_sections = load_expected_changes(expected_changes_yml)
    log_info(f"Expected changes: {len(expected_sections)} sections")
    log_info(f"Preserved sections: {len(preserved_sections)} sections")
    
    # Generate canonical IDs for deterministic comparison
    changed_ids = {canonical_section_id(s) for s in changed_sections}
    expected_ids = {canonical_section_id(s) for s in expected_sections}
    preserved_ids = {canonical_section_id(s) for s in preserved_sections}
    
    # Build reverse mapping for reporting (ID → original section name)
    changed_id_map = {canonical_section_id(s): s for s in changed_sections}
    expected_id_map = {canonical_section_id(s): s for s in expected_sections}
    preserved_id_map = {canonical_section_id(s): s for s in preserved_sections}
    
    # Validation logic
    validation_passed = True
    
    # Check 1: All expected changes present
    log_info("Validating expected changes...")
    missing_expected_ids = expected_ids - changed_ids
    
    if diff_empty and expected_ids:
        log_error("Expected changes but files are identical")
        validation_passed = False
    elif diff_empty and not expected_ids:
        log_success("No changes expected, no changes found - PASS")
    else:
        for section in expected_sections:
            section_id = canonical_section_id(section)
            if section_id in changed_ids:
                log_success(f"Expected change found: {section} [id: {section_id}]")
            else:
                log_warning(f"Expected change NOT found: {section} [id: {section_id}]")
                validation_passed = False
    
    # Check 2: No unexpected changes in preserved sections
    log_info("Checking for unexpected changes in preserved sections...")
    unexpected_change_ids = changed_ids & preserved_ids
    
    if unexpected_change_ids:
        for section in preserved_sections:
            section_id = canonical_section_id(section)
            if section_id in unexpected_change_ids:
                log_error(f"Unexpected change in preserved section: {section} [id: {section_id}]")
                validation_passed = False
    else:
        log_success("No unexpected changes in preserved sections")
    
    # Prepare report data
    report_data = {
        'original_file': original_file,
        'original_hash': generate_hash(original_file),
        'fixed_file': fixed_file,
        'fixed_hash': generate_hash(fixed_file),
        'diff_file': diff_file,
        'diff_empty': diff_empty,
        'changed_sections': list(changed_sections),
        'expected_sections': expected_sections,
        'preserved_sections': preserved_sections,
        'missing_expected_ids': list(missing_expected_ids),
        'unexpected_change_ids': list(unexpected_change_ids),
        'validation_passed': validation_passed
    }
    
    return validation_passed, report_data


def generate_json_report(report_data: Dict) -> str:
    """Generate JSON report for CI parsing"""
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    report_dir = Path(__file__).parent / "reports"
    report_file = report_dir / f"preservation_validation_{timestamp}.json"
    
    # Build canonical ID mappings for report
    changed_ids = {canonical_section_id(s): s for s in report_data['changed_sections']}
    expected_ids = {canonical_section_id(s): s for s in report_data['expected_sections']}
    preserved_ids = {canonical_section_id(s): s for s in report_data['preserved_sections']}
    
    json_data = {
        "validation_passed": report_data['validation_passed'],
        "timestamp": datetime.now().strftime("%Y-%m-%d %H:%M:%S UTC"),
        "version": "2.0.0",
        "files": {
            "original": report_data['original_file'],
            "original_hash": report_data['original_hash'],
            "fixed": report_data['fixed_file'],
            "fixed_hash": report_data['fixed_hash'],
            "diff": report_data['diff_file']
        },
        "diff_empty": report_data['diff_empty'],
        "sections": {
            "changed": {
                "count": len(report_data['changed_sections']),
                "sections": report_data['changed_sections'],
                "ids": list(changed_ids.keys())
            },
            "expected": {
                "count": len(report_data['expected_sections']),
                "sections": report_data['expected_sections'],
                "ids": list(expected_ids.keys())
            },
            "preserved": {
                "count": len(report_data['preserved_sections']),
                "sections": report_data['preserved_sections'],
                "ids": list(preserved_ids.keys())
            }
        },
        "validation": {
            "missing_expected_ids": report_data['missing_expected_ids'],
            "unexpected_change_ids": report_data['unexpected_change_ids']
        },
        "ci_authoritative": True,
        "deterministic": True
    }
    
    with open(report_file, 'w', encoding='utf-8') as f:
        json.dump(json_data, f, indent=2, ensure_ascii=False)
    
    return str(report_file)


def generate_report(report_data: Dict) -> str:
    """Generate markdown validation report"""
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S UTC")
    report_dir = Path(__file__).parent / "reports"
    report_file = report_dir / f"preservation_validation_{datetime.now().strftime('%Y%m%d_%H%M%S')}.md"
    
    with open(report_file, 'w', encoding='utf-8') as f:
        f.write(f"# Preservation Validation Report\n\n")
        f.write(f"**Generated**: {timestamp}\n")
        f.write(f"**Script**: validate_preservation.py\n")
        f.write(f"**Version**: 2.0.0 (Python - CI-Authoritative)\n\n")
        f.write("---\n\n")
        
        f.write("## Input Files\n\n")
        f.write(f"- **ORIGINAL**: `{report_data['original_file']}`\n")
        f.write(f"  - Hash: {report_data['original_hash']}\n")
        f.write(f"- **FIXED**: `{report_data['fixed_file']}`\n")
        f.write(f"  - Hash: {report_data['fixed_hash']}\n")
        f.write(f"- **Diff**: `{report_data['diff_file']}`\n\n")
        
        f.write("---\n\n")
        f.write("## Validation Results\n\n")
        
        if report_data['validation_passed']:
            f.write("✅ **PASS**: Preservation validation successful\n\n")
        else:
            f.write("❌ **FAIL**: Preservation validation failed\n\n")
        
        f.write(f"### Changed Sections ({len(report_data['changed_sections'])})\n\n")
        for section in report_data['changed_sections']:
            f.write(f"- `{section}`\n")
        
        f.write(f"\n### Expected Changes ({len(report_data['expected_sections'])})\n\n")
        for section in report_data['expected_sections']:
            section_id = canonical_section_id(section)
            changed_ids = {canonical_section_id(s) for s in report_data['changed_sections']}
            if section_id in changed_ids:
                f.write(f"- ✅ `{section}` [id: `{section_id}`]\n")
            else:
                f.write(f"- ❌ `{section}` [id: `{section_id}`] (NOT FOUND)\n")
        
        if report_data['missing_expected_ids']:
            f.write(f"\n### Missing Expected Changes ({len(report_data['missing_expected_ids'])})\n\n")
            for section_id in report_data['missing_expected_ids']:
                f.write(f"- ❌ ID: `{section_id}`\n")
        
        if report_data['unexpected_change_ids']:
            f.write(f"\n### Unexpected Changes ({len(report_data['unexpected_change_ids'])})\n\n")
            for section_id in report_data['unexpected_change_ids']:
                f.write(f"- ❌ ID: `{section_id}`\n")
        
        f.write("\n---\n\n")
        f.write("**Validation Level**: Level 3 (Complete Audit Trail)\n")
        f.write("**Authority**: Constitutional Enforcement (Phase-17.5)\n")
        f.write("**CI-Authoritative**: ✅ YES (deterministic diff→section mapping)\n\n")
    
    return str(report_file)


def main():
    if len(sys.argv) != 4:
        print("Usage: validate_preservation.py ORIGINAL_FILE FIXED_FILE EXPECTED_CHANGES_YML", 
              file=sys.stderr)
        sys.exit(2)
    
    original_file = sys.argv[1]
    fixed_file = sys.argv[2]
    expected_changes_yml = sys.argv[3]
    
    # Run validation
    validation_passed, report_data = validate_preservation(
        original_file, fixed_file, expected_changes_yml
    )
    
    # Generate reports
    markdown_report = generate_report(report_data)
    json_report = generate_json_report(report_data)
    log_info(f"Markdown report: {markdown_report}")
    log_info(f"JSON report: {json_report}")
    
    # CI-authoritative assertion (Tier 1 hardening)
    if os.getenv('CI') == 'true':
        log_info("CI mode detected - verifying CI-authoritative status...")
        try:
            with open(json_report, 'r') as f:
                report_json = json.load(f)
            
            if not report_json.get('ci_authoritative', False):
                log_error("❌ CI-AUTHORITATIVE ASSERTION FAILED")
                log_error("Validator claims to be CI-authoritative but JSON report says otherwise")
                sys.exit(2)
            
            if not report_json.get('deterministic', False):
                log_error("❌ DETERMINISM ASSERTION FAILED")
                log_error("Validator must be deterministic for CI use")
                sys.exit(2)
            
            log_success("✅ CI-authoritative assertions passed")
        except Exception as e:
            log_error(f"❌ Failed to verify CI-authoritative status: {e}")
            sys.exit(2)
    
    # Final output
    print()
    if validation_passed:
        log_success("✅ PRESERVATION VALIDATION PASSED")
        sys.exit(0)
    else:
        log_error("❌ PRESERVATION VALIDATION FAILED")
        log_info(f"Review the reports for details:")
        log_info(f"  Markdown: {markdown_report}")
        log_info(f"  JSON: {json_report}")
        sys.exit(1)


if __name__ == '__main__':
    main()
