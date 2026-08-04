#!/usr/bin/env python3
"""Fix: replace wvc:: with weavecoder:: in source files.
The root crate's lib name is 'weavecoder', not 'wvc'.
Binary name is 'wvc', but library imports use 'weavecoder'."""
import re, sys
from pathlib import Path

REPO = Path(sys.argv[1]) if len(sys.argv) > 1 else Path.cwd()

def fix_imports():
    count = 0
    for pat in ["src/**/*.rs", "crates/**/*.rs", "tests/**/*.rs"]:
        for rf in REPO.glob(pat):
            c = rf.read_text()
            original = c
            
            # Replace wvc:: with weavecoder:: in imports and code
            c = re.sub(r'\bwvc::', 'weavecoder::', c)
            
            if c != original:
                rf.write_text(c)
                count += 1
                print(f"  Fixed: {rf.relative_to(REPO)}")
    
    print(f"\n  Total files fixed: {count}")

if __name__ == "__main__":
    print("=== Fixing wvc:: -> weavecoder:: imports ===")
    fix_imports()
    print("\n=== Done ===")
