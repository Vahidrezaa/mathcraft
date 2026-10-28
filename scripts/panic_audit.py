#!/usr/bin/env python3
"""Panic audit guard — ROADMAP step 2.5, task 3 (audit items 2/15 closure).

Extends the old grep-guard ("No todo!/unimplemented!") to all non-test engine
paths: after stripping #[cfg(test)] modules (brace-matched, string/comment
aware), the following are forbidden in hand-written product code:

    panic!(  .unwrap(  .expect(  todo!(  unimplemented!(

frb_generated*.rs is FRB-generated output and is exempt (doc 08 §1).
mc-cli is product code too (its `mc -e` runs on the device) — audited.

Exit code 0 = clean, 1 = violations found (CI fails with ::error annotations).
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PATTERN = re.compile(r"panic!\(|\.unwrap\(|\.expect\(|todo!\(|unimplemented!\(")


def strip_cfg_test_modules(src: str) -> str:
    """Blank out `#[cfg(test)] mod <name> { ... }` regions, keeping line count.

    Brace tracking is aware of // and /* */ comments, "strings", 'chars' and
    raw strings r#"..."# so braces inside literals do not break the count.
    """
    lines = src.splitlines(keepends=True)
    out = []
    i = 0
    n = len(lines)
    while i < n:
        line = lines[i]
        stripped = line.strip()
        if stripped == "#[cfg(test)]":
            # find first following line that starts a `mod` (skip attrs/doc/blank)
            j = i + 1
            is_mod = False
            while j < n:
                s = lines[j].strip()
                if s == "" or s.startswith("#[") or s.startswith("//"):
                    j += 1
                    continue
                is_mod = s.startswith("mod ") or s.startswith("mod{")
                break
            if is_mod:
                # blank from i to the closing brace of the mod (brace matched)
                depth = 0
                seen_open = False
                k = i
                while k < n:
                    depth += count_braces(lines[k])
                    if "{" in lines[k]:
                        seen_open = True
                    out.append("\n")
                    if seen_open and depth <= 0:
                        break
                    k += 1
                i = k + 1
                continue
        out.append(line)
        i += 1
    return "".join(out)


def count_braces(line: str) -> int:
    """Net braces { minus } outside comments/strings/chars of one line."""
    depth = 0
    i = 0
    n = len(line)
    while i < n:
        c = line[i]
        if c == "/" and i + 1 < n and line[i + 1] == "/":
            break  # line comment
        if c == "/" and i + 1 < n and line[i + 1] == "*":
            # block comment (single-line case; multi-line is rare in mod heads)
            end = line.find("*/", i + 2)
            i = n if end == -1 else end + 2
            continue
        if c == '"':
            i += 1
            while i < n:
                if line[i] == "\\":
                    i += 2
                    continue
                if line[i] == '"':
                    i += 1
                    break
                i += 1
            continue
        if c == "r" and i + 1 < n and line[i + 1] in "#\"":
            # raw string r"..." / r#"..."#
            m = re.match(r'r(#*)"', line[i:])
            if m:
                hashes = m.group(1)
                terminator = '"' + hashes
                end = line.find(terminator, i + m.end())
                i = n if end == -1 else end + len(terminator)
                continue
        if c == "'":
            # char literal 'x' or '\n' — else it is a lifetime (ignore)
            m = re.match(r"'(\\.|[^'\\])'", line[i:])
            if m:
                i += m.end()
                continue
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
        i += 1
    return depth


def main() -> int:
    violations = []
    crate_srcs = sorted((ROOT / "crates").glob("*/src"))
    for src_dir in crate_srcs:
        for rs in sorted(src_dir.rglob("*.rs")):
            if rs.name.startswith("frb_generated"):
                continue
            rel = rs.relative_to(ROOT)
            stripped = strip_cfg_test_modules(rs.read_text(encoding="utf-8"))
            for lineno, line in enumerate(stripped.splitlines(), start=1):
                if PATTERN.search(line):
                    violations.append(f"{rel}:{lineno}: {line.strip()[:110]}")
    if violations:
        print("::error::panic audit failed — panic!/unwrap/expect found in non-test product code (ROADMAP 2.5):")
        for v in violations:
            print(f"  {v}")
        print(f"\n{len(violations)} violation(s). Fix by returning Result, matching on Option,")
        print("or using checked arithmetic; every reachable panic is a device crash.")
        return 1
    print("panic audit: clean (non-test product paths contain no panic!/unwrap/expect/todo!/unimplemented!)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
