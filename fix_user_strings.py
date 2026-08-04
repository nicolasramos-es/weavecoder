#!/usr/bin/env python3
"""Fix remaining user-facing 'jcode' strings in source code."""
import re, sys
from pathlib import Path

REPO = Path(sys.argv[1]) if len(sys.argv) > 1 else Path.cwd()

def fix_user_facing_strings():
    count = 0
    for pat in ["src/**/*.rs", "crates/**/*.rs", "tests/**/*.rs"]:
        for rf in REPO.glob(pat):
            c = rf.read_text()
            original = c
            
            # Replace command references in strings: `jcode login` -> `wvc login`
            c = re.sub(r'`jcode (login|account|server|run|repl|update|version|connect|serve|acp|replay)`,?', r'`wvc \1`', c)
            
            # Replace "jcode login" in string literals
            c = re.sub(r'"jcode (login|account|server|run|repl|update|version|connect|serve|acp|replay)"', r'"wvc \1"', c)
            
            # Replace "J-Code" in string literals
            c = re.sub(r'"J-Code"', r'"Weavecoder"', c)
            
            # Replace "Jcode" in string literals
            c = re.sub(r'"Jcode', r'"Weavecoder', c)
            
            # Replace "jcode v0" in version strings
            c = re.sub(r'"jcode v0', r'"weavecoder v0', c)
            
            # Replace "jcode" in error messages and help text (in string literals)
            c = re.sub(r'"jcode', r'"wvc', c)
            
            # Replace "jcode" in println!/eprintln! strings
            c = re.sub(r'println!\("jcode', r'println!("wvc', c)
            c = re.sub(r'eprintln!\("jcode', r'eprintln!("wvc', c)
            
            # Replace "jcode" in format! strings
            c = re.sub(r'format!\("jcode', r'format!("wvc', c)
            
            # Replace "jcode" in anyhow::anyhow! strings
            c = re.sub(r'anyhow::anyhow!\("jcode', r'anyhow::anyhow!("wvc', c)
            
            # Replace "jcode" in login_hint strings
            c = re.sub(r'login_hint: "jcode ', r'login_hint: "wvc ', c)
            
            # Replace "jcode" in detail strings
            c = re.sub(r'detail: "jcode ', r'detail: "wvc ', c)
            
            # Replace "jcode" in println! with format args
            c = re.sub(r'println!\("jcode', r'println!("wvc', c)
            
            # Replace "jcode" in string concatenation
            c = re.sub(r'"jcode ', r'"wvc ', c)
            
            # Replace "jcode" in .to_string() patterns
            c = re.sub(r'"jcode ', r'"wvc ', c)
            
            if c != original:
                rf.write_text(c)
                count += 1
                print(f"  Fixed: {rf.relative_to(REPO)}")
    
    print(f"\n  Total files fixed: {count}")

if __name__ == "__main__":
    print("=== Fixing user-facing jcode strings ===")
    fix_user_facing_strings()
    print("\n=== Done ===")
