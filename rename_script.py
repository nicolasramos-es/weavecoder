#!/usr/bin/env python3
"""Rename jcode -> weavecoder/wvc across the codebase. Properly handles both
package names (hyphens) and lib names (underscores)."""
import os, re, sys
from pathlib import Path

REPO = Path(sys.argv[1]) if len(sys.argv) > 1 else Path.cwd()

def rename_crate_dirs():
    crates = REPO / "crates"
    for d in sorted(crates.iterdir()):
        if d.is_dir() and d.name.startswith("jcode-"):
            new = crates / d.name.replace("jcode-", "wvc-")
            print(f"  {d.name} -> {new.name}")
            d.rename(new)

def update_root_cargo():
    f = REPO / "Cargo.toml"
    content = f.read_text()
    # Workspace members
    content = re.sub(r'"crates/jcode-', '"crates/wvc-', content)
    # Dependency references (crate names in deps)
    content = re.sub(r'jcode-', 'wvc-', content)
    # Package/lib/bin names
    lines = content.split('\n')
    new_lines = []
    section = None
    pkg_done = lib_done = bin_done = False
    for line in lines:
        s = line.strip()
        if s == '[package]': section = 'pkg'
        elif s == '[lib]': section = 'lib'
        elif s == '[[bin]]': section = 'bin'
        elif s.startswith('[') and not s.startswith('[['): section = None
        if s == 'name = "jcode"':
            if section == 'pkg' and not pkg_done:
                line = line.replace('"jcode"', '"weavecoder"')
                pkg_done = True
            elif section == 'lib' and not lib_done:
                line = line.replace('"jcode"', '"weavecoder"')
                lib_done = True
            elif section == 'bin' and not bin_done:
                line = line.replace('"jcode"', '"wvc"')
                bin_done = True
        new_lines.append(line)
    f.write_text('\n'.join(new_lines))
    print("  Updated root Cargo.toml")

def update_crate_cargos():
    crates = REPO / "crates"
    for cf in sorted(crates.glob("wvc-*/Cargo.toml")):
        c = cf.read_text()
        # Package name (first name= line, uses hyphens)
        c = re.sub(r'^name = "jcode-', 'name = "wvc-', c, flags=re.MULTILINE)
        # Lib name (second name= line, uses underscores)
        c = re.sub(r'^name = "jcode_', 'name = "wvc_', c, flags=re.MULTILINE)
        # Dependency references
        c = re.sub(r'jcode-', 'wvc-', c)
        c = re.sub(r'jcode_', 'wvc_', c)
        cf.write_text(c)
        print(f"  Updated {cf.name}")

def update_rs_files():
    for pat in ["src/**/*.rs", "crates/**/*.rs", "tests/**/*.rs"]:
        for rf in REPO.glob(pat):
            c = rf.read_text()
            c = re.sub(r'jcode_', 'wvc_', c)
            rf.write_text(c)

def update_docs():
    for df in sorted(REPO.glob("*.md")):
        c = df.read_text()
        c = c.replace('jcode', 'weavecoder').replace('Jcode', 'Weavecoder')
        df.write_text(c)
        print(f"  Updated {df.name}")

if __name__ == "__main__":
    print("=== Renaming crate directories ===")
    rename_crate_dirs()
    print("\n=== Updating root Cargo.toml ===")
    update_root_cargo()
    print("\n=== Updating crate Cargo.toml files ===")
    update_crate_cargos()
    print("\n=== Updating .rs files ===")
    update_rs_files()
    print("\n=== Updating documentation ===")
    update_docs()
    print("\n=== Rename complete ===")
