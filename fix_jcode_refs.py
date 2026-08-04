#!/usr/bin/env python3
"""Fix remaining jcode references in .rs source files."""
import re, sys
from pathlib import Path

REPO = Path(sys.argv[1]) if len(sys.argv) > 1 else Path.cwd()

def fix_rs_files():
    """Fix all remaining jcode references in .rs files."""
    count = 0
    for pat in ["src/**/*.rs", "crates/**/*.rs", "tests/**/*.rs"]:
        for rf in REPO.glob(pat):
            c = rf.read_text()
            original = c
            
            # Fix `use jcode::` imports -> `use wvc::`
            c = re.sub(r'\buse jcode::', 'use wvc::', c)
            
            # Fix `jcode::` usage in code (not in comments)
            # Handle jcode:: followed by identifier
            c = re.sub(r'\bjcode::', 'wvc::', c)
            
            # Fix `pub mod jcode;` -> `pub mod wvc;`
            c = re.sub(r'pub mod jcode;', 'pub mod wvc;', c)
            
            # Fix `mod jcode` declarations
            c = re.sub(r'mod jcode;', 'mod wvc;', c)
            
            if c != original:
                rf.write_text(c)
                count += 1
                print(f"  Fixed: {rf.relative_to(REPO)}")
    
    print(f"\n  Total files fixed: {count}")

if __name__ == "__main__":
    print("=== Fixing remaining jcode references in .rs files ===")
    fix_rs_files()
    print("\n=== Done ===")
