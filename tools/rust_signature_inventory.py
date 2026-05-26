#!/usr/bin/env python3
"""Print grouped Rust item signatures without implementation bodies."""

from __future__ import annotations

import argparse
import re
import subprocess
from pathlib import Path


ITEM_START = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?"
    r"(?:(?:async|const|unsafe|extern(?:\s+\"[^\"]+\")?)\s+)*"
    r"(fn|struct|enum|trait|impl|type)\b"
)


def args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Extract grouped Rust function/type signatures without bodies."
    )
    parser.add_argument("roots", nargs="*", default=["src"], help="files or directories to scan")
    parser.add_argument(
        "--exclude-tests",
        action="store_true",
        help="skip paths containing /tests/ or ending in /tests.rs",
    )
    return parser.parse_args()


def repo_root() -> Path:
    return Path(
        subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            check=True,
            text=True,
            stdout=subprocess.PIPE,
        ).stdout.strip()
    )


def rust_files(root: Path, requested: list[str], exclude_tests: bool) -> list[Path]:
    files: set[Path] = set()
    for raw in requested:
        path = (root / raw).resolve() if not Path(raw).is_absolute() else Path(raw)
        if path.is_file() and path.suffix == ".rs":
            files.add(path)
        elif path.is_dir():
            files.update(path.rglob("*.rs"))

    def included(path: Path) -> bool:
        rel = path.relative_to(root).as_posix()
        return not (
            exclude_tests and ("/tests/" in rel or rel.endswith("/tests.rs") or rel.endswith("tests.rs"))
        )

    return sorted(path for path in files if included(path))


def cut_body(text: str) -> str:
    depth = 0
    for index, ch in enumerate(text):
        if ch in "{;" and depth == 0:
            return text[:index].strip()
        if ch in "([{<":
            depth += 1
        elif ch in ")]}>":
            depth = max(0, depth - 1)
    return text.strip()


def normalize(text: str) -> str:
    return re.sub(r"\s+", " ", text).strip()


def matching_paren(text: str, start: int) -> int | None:
    depth = 0
    for index in range(start, len(text)):
        ch = text[index]
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0:
                return index
    return None


def format_fn(signature: str) -> str:
    signature = cut_body(normalize(signature))
    match = re.search(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)(?:\s*(<[^)]*?>))?\s*\(", signature)
    if not match:
        return signature

    name = match.group(1)
    generics = normalize(match.group(2) or "")
    paren_start = signature.find("(", match.end() - 1)
    paren_end = matching_paren(signature, paren_start)
    if paren_end is None:
        return signature

    params = normalize(signature[paren_start + 1 : paren_end])
    tail = normalize(signature[paren_end + 1 :])
    return_type = "()"
    where_clause = ""
    if tail.startswith("->"):
        tail = normalize(tail[2:])
        if " where " in tail:
            return_type, where_clause = tail.split(" where ", 1)
            where_clause = f" where {where_clause}"
        else:
            return_type = tail
    elif tail.startswith("where "):
        where_clause = f" {tail}"

    return f"fn {name}{generics}({params}): {return_type}{where_clause}"


def format_type(signature: str) -> str:
    signature = normalize(signature)
    if re.match(r".*\btype\b", signature) and ";" in signature:
        return cut_body(signature)
    head = cut_body(signature)
    return f"{head} {{ ... }}" if "{" in signature else head


def extract_signatures(path: Path) -> list[str]:
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    signatures: list[str] = []
    index = 0
    while index < len(lines):
        line = lines[index]
        if not ITEM_START.match(line):
            index += 1
            continue

        item = [line.strip()]
        balance = line.count("(") - line.count(")")
        while index + 1 < len(lines):
            text = " ".join(item)
            if ("{" in text or ";" in text) and balance <= 0:
                break
            index += 1
            item.append(lines[index].strip())
            balance += lines[index].count("(") - lines[index].count(")")

        raw = " ".join(item)
        kind = ITEM_START.match(line).group(1)
        signatures.append(format_fn(raw) if kind == "fn" else format_type(raw))
        index += 1
    return signatures


def main() -> int:
    opts = args()
    root = repo_root()
    files = rust_files(root, opts.roots, opts.exclude_tests)

    print("# Rust Signature Inventory")
    print()
    print(f"- Root: `{root}`")
    print(f"- Files scanned: {len(files)}")
    print(f"- Tests excluded: {'yes' if opts.exclude_tests else 'no'}")
    print()

    for path in files:
        signatures = extract_signatures(path)
        if not signatures:
            continue
        rel = path.relative_to(root).as_posix()
        print(f"## {rel}")
        for signature in signatures:
            print(f"- `{signature}`")
        print()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
