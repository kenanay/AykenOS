#!/usr/bin/env python3
"""
symbol_scan_match.py — fast deny/allow matching for symbol-scan.sh

Replaces the shell grep-per-symbol loops (O(n*m) process forks) with
compiled regex matching in a single Python process.

Called via environment variables set by symbol-scan.sh:
  RAW_SYMS_ENV, FILTERED_SYMS_ENV, DENY_HITS_ENV,
  FINAL_VIOLATIONS_ENV, DENY_FILE_ENV, ALLOW_FILE_ENV
"""
import os
import re
import sys

def load_patterns(path):
    """Return list of (file_re_or_None, sym_re, raw_pat_str)."""
    patterns = []
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            line = line.split('#', 1)[0].strip()
            if not line:
                continue
            if ':' in line:
                file_part, sym_part = line.split(':', 1)
                patterns.append((re.compile(file_part), re.compile(sym_part), line))
            else:
                patterns.append((None, re.compile(line), line))
    return patterns

def main():
    raw_syms        = os.environ["RAW_SYMS_ENV"]
    filtered_syms   = os.environ["FILTERED_SYMS_ENV"]
    deny_hits       = os.environ["DENY_HITS_ENV"]
    final_violations = os.environ["FINAL_VIOLATIONS_ENV"]
    deny_file       = os.environ["DENY_FILE_ENV"]
    allow_file      = os.environ["ALLOW_FILE_ENV"]

    deny_pats  = load_patterns(deny_file)
    allow_pats = load_patterns(allow_file)

    SYM_LINE = re.compile(r'^[^#].+:[A-Za-z_][A-Za-z0-9_.$@]*$')

    # Step 2a: filter raw symbols
    filtered = []
    with open(raw_syms, encoding="utf-8") as fh:
        for line in fh:
            line = line.rstrip('\n')
            if SYM_LINE.match(line):
                filtered.append(line)

    with open(filtered_syms, 'w', encoding="utf-8") as fh:
        for line in filtered:
            fh.write(line + '\n')

    # Step 2b: deny matching
    # Use fullmatch: deny patterns are anchored (^...$) and we want exact
    # symbol matches, not substring hits inside longer symbol names.
    hits = []
    for line in filtered:
        colon = line.index(':')
        target = line[:colon]
        sym    = line[colon+1:]
        for file_re, sym_re, raw_pat in deny_pats:
            if sym_re.fullmatch(sym):
                hits.append((target, sym, raw_pat))
                break

    with open(deny_hits, 'w', encoding="utf-8") as fh:
        for t, s, p in hits:
            fh.write(f'{t}:{s}:deny={p}\n')

    # Step 3: allowlist filter
    # fullmatch mirrors deny matching semantics — anchored pattern, exact symbol.
    violations = []
    for target, sym, raw_pat in hits:
        allowed = False
        for file_re, sym_re, _ in allow_pats:
            if file_re is None or file_re.search(target):
                if sym_re.fullmatch(sym):
                    allowed = True
                    break
        if not allowed:
            violations.append(f'{target}:{sym}:deny={raw_pat}')

    with open(final_violations, 'w', encoding="utf-8") as fh:
        for v in violations:
            fh.write(v + '\n')

if __name__ == "__main__":
    main()
