#!/usr/bin/env python3
"""Find semantically overlapping files with local embedding models."""

from __future__ import annotations

import argparse
import json
import math
import os
import subprocess
import sys
from collections import defaultdict
from pathlib import Path


DEFAULT_MODEL = "Qwen/Qwen3-Embedding-0.6B"
DEFAULT_EXTS = ".json,.md,.rs,.toml,.yaml,.yml,.zen"
DEFAULT_EXCLUDES = (".git/", ".vscode/", "target/", "vscode-extension/out/")
DEFAULT_INCLUDES = (
    "src/",
    "stdlib/",
    "docs/",
    "examples/",
    ".github/workflows/",
    "Cargo.toml",
    "CONTRIBUTING.md",
    "LANGUAGE_SPEC.zen",
    "README.md",
)


def args() -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description=(
            "Embed tracked source/docs files and report semantic overlap pairs "
            "and clusters. Uses local open-source embeddings only."
        )
    )
    add = p.add_argument
    add("--model", default=DEFAULT_MODEL, help=f"SentenceTransformers model (default: {DEFAULT_MODEL})")
    add("--cache-dir", help="Optional Hugging Face cache dir, e.g. ~/.cache/huggingface")
    add("--device", help="Torch device override such as cpu, cuda, or mps")
    add("--batch-size", type=int, default=16)
    add("--max-seq-length", type=int, default=128)
    add("--chunk-chars", type=int, default=1600)
    add("--chunk-overlap", type=int, default=160)
    add("--max-file-bytes", type=int, default=1_000_000)
    add("--top", type=int, default=80)
    add("--threshold", type=float, default=0.84)
    add("--cluster-threshold", type=float, default=0.86)
    add("--extensions", default=DEFAULT_EXTS, help="Comma-separated file extensions")
    add("--exclude", action="append", default=[], help="Additional tracked path prefix to exclude")
    add("--include", action="append", default=[], help="Additional tracked path prefix to include")
    add("--all-tracked", action="store_true", help="Embed all tracked files matching --extensions")
    add("--include-tests", action="store_true", help="Include test files and fixtures")
    add("--max-files", type=int, help="Debug limit on included files")
    add("--plan-only", action="store_true", help="Print selected file/chunk counts and exit")
    add("--output", default="target/tmp/semantic-overlap-report.md")
    add("--json-output", default="target/tmp/semantic-overlap-report.json")
    return p.parse_args()


def require_valid(opts: argparse.Namespace) -> None:
    positive = ("batch_size", "max_seq_length", "chunk_chars", "max_file_bytes", "top")
    for name in positive:
        if getattr(opts, name) <= 0:
            raise SystemExit(f"--{name.replace('_', '-')} must be positive")
    if opts.chunk_overlap < 0 or opts.chunk_overlap >= opts.chunk_chars:
        raise SystemExit("--chunk-overlap must be non-negative and smaller than --chunk-chars")
    for name in ("threshold", "cluster_threshold"):
        value = getattr(opts, name)
        if not math.isfinite(value) or not -1.0 <= value <= 1.0:
            raise SystemExit(f"--{name.replace('_', '-')} must be between -1 and 1")


def repo_root() -> Path:
    return Path(
        subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            check=True,
            text=True,
            stdout=subprocess.PIPE,
        ).stdout.strip()
    )


def tracked_files(root: Path) -> list[str]:
    raw = subprocess.run(
        ["git", "ls-files", "-z"],
        cwd=root,
        check=True,
        stdout=subprocess.PIPE,
    ).stdout
    return [p.decode("utf-8", "replace") for p in raw.split(b"\0") if p]


def ext_set(raw: str) -> set[str]:
    return {part if part.startswith(".") else f".{part}" for part in raw.split(",") if part.strip()}


def is_test_path(path: str) -> bool:
    return path.startswith("tests/") or "/tests/" in path or path.endswith(("_tests.rs", "/tests.rs"))


def included(path: str, root: Path, opts: argparse.Namespace, exts: set[str]) -> bool:
    excludes = DEFAULT_EXCLUDES + tuple(opts.exclude)
    includes = DEFAULT_INCLUDES + tuple(opts.include)
    if any(path.startswith(prefix) for prefix in excludes):
        return False
    if not opts.all_tracked and not any(path.startswith(prefix) for prefix in includes):
        return False
    if not opts.include_tests and is_test_path(path):
        return False
    file_path = root / path
    try:
        return file_path.suffix in exts and file_path.stat().st_size <= opts.max_file_bytes
    except OSError:
        return False


def chunks(text: str, size: int, overlap: int) -> list[str]:
    text = "\n".join(line.rstrip() for line in text.splitlines()).strip()
    if not text:
        return []
    if len(text) <= size:
        return [text]

    out: list[str] = []
    start = 0
    step = max(1, size - overlap)
    while start < len(text):
        end = min(len(text), start + size)
        if end < len(text):
            newline = text.rfind("\n", start, end)
            if newline > start + size // 2:
                end = newline
        chunk = text[start:end].strip()
        if chunk:
            out.append(chunk)
        if end >= len(text):
            break
        start = max(start + step, end - overlap)
    return out


def load_files(root: Path, opts: argparse.Namespace) -> list[tuple[str, list[str]]]:
    selected: list[tuple[str, list[str]]] = []
    for path in tracked_files(root):
        if not included(path, root, opts, ext_set(opts.extensions)):
            continue
        try:
            text = (root / path).read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        file_chunks = chunks(text, opts.chunk_chars, opts.chunk_overlap)
        if file_chunks:
            selected.append((path, file_chunks))
        if opts.max_files is not None and len(selected) >= opts.max_files:
            break
    return selected


def load_model(opts: argparse.Namespace):
    try:
        from sentence_transformers import SentenceTransformer
    except ModuleNotFoundError as exc:
        raise SystemExit(
            "Missing dependency: sentence-transformers. Install locally with:\n"
            "  uv pip install sentence-transformers torch\n"
            "or run through uv:\n"
            "  uv run --with sentence-transformers tools/semantic_overlap.py\n"
            "No lexical fallback is provided; this audit uses embeddings only."
        ) from exc

    kwargs: dict[str, object] = {}
    if opts.cache_dir:
        kwargs["cache_folder"] = opts.cache_dir
    if opts.device:
        kwargs["device"] = opts.device
    model = SentenceTransformer(opts.model, **kwargs)
    model.max_seq_length = opts.max_seq_length
    return model


def embed(files: list[tuple[str, list[str]]], opts: argparse.Namespace):
    import numpy as np

    texts: list[str] = []
    owners: list[int] = []
    for file_index, (_, file_chunks) in enumerate(files):
        texts.extend(file_chunks)
        owners.extend([file_index] * len(file_chunks))

    vectors = np.asarray(
        load_model(opts).encode(
            texts,
            batch_size=opts.batch_size,
            normalize_embeddings=True,
            show_progress_bar=True,
        ),
        dtype=np.float32,
    )
    file_vectors = np.zeros((len(files), vectors.shape[1]), dtype=np.float32)
    counts = np.zeros(len(files), dtype=np.float32)
    for owner, vector in zip(owners, vectors, strict=True):
        file_vectors[owner] += vector
        counts[owner] += 1.0
    file_vectors /= counts[:, None]
    norms = np.linalg.norm(file_vectors, axis=1, keepdims=True)
    norms[norms == 0.0] = 1.0
    return file_vectors / norms


def similar_pairs(paths: list[str], vectors, threshold: float, top: int) -> list[dict[str, object]]:
    matrix = vectors @ vectors.T
    pairs = [
        {"left": paths[i], "right": paths[j], "score": float(matrix[i, j])}
        for i in range(len(paths))
        for j in range(i + 1, len(paths))
        if float(matrix[i, j]) >= threshold
    ]
    pairs.sort(key=lambda pair: pair["score"], reverse=True)
    return pairs[:top]


def clusters(paths: list[str], vectors, threshold: float) -> list[list[str]]:
    matrix = vectors @ vectors.T
    graph: dict[int, set[int]] = defaultdict(set)
    for i in range(len(paths)):
        for j in range(i + 1, len(paths)):
            if float(matrix[i, j]) >= threshold:
                graph[i].add(j)
                graph[j].add(i)

    seen: set[int] = set()
    out: list[list[str]] = []
    for start in sorted(graph):
        if start in seen:
            continue
        stack = [start]
        component: list[int] = []
        seen.add(start)
        while stack:
            node = stack.pop()
            component.append(node)
            for neighbor in sorted(graph[node]):
                if neighbor not in seen:
                    seen.add(neighbor)
                    stack.append(neighbor)
        if len(component) > 1:
            out.append(sorted(paths[index] for index in component))
    out.sort(key=lambda group: (-len(group), group))
    return out


def write_reports(
    root: Path,
    opts: argparse.Namespace,
    files: list[tuple[str, list[str]]],
    pairs: list[dict[str, object]],
    groups: list[list[str]],
) -> None:
    output = root / opts.output
    json_output = root / opts.json_output
    output.parent.mkdir(parents=True, exist_ok=True)
    json_output.parent.mkdir(parents=True, exist_ok=True)

    scope = "all tracked files" if opts.all_tracked else "default live-source/doc roots"
    lines = [
        "# Semantic Overlap Report",
        "",
        f"- Model: `{opts.model}`",
        f"- Files embedded: {len(files)}",
        f"- Scope: {scope}",
        f"- Tests included: {'yes' if opts.include_tests else 'no'}",
        f"- Max sequence length: {opts.max_seq_length}",
        f"- Pair threshold: {opts.threshold:.2f}",
        f"- Cluster threshold: {opts.cluster_threshold:.2f}",
        "",
        "## Highest-Scoring Pairs",
        "",
    ]
    lines.extend(
        [f"- {pair['score']:.3f} `{pair['left']}` <-> `{pair['right']}`" for pair in pairs]
        or ["- No pairs met the reporting threshold."]
    )
    lines.extend(["", "## Clusters", ""])
    if groups:
        for index, group in enumerate(groups, start=1):
            lines.append(f"### Cluster {index} ({len(group)} files)")
            lines.extend(f"- `{path}`" for path in group)
            lines.append("")
    else:
        lines.append("- No clusters met the clustering threshold.")

    output.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")
    json_output.write_text(
        json.dumps(
            {
                "model": opts.model,
                "file_count": len(files),
                "scope": "all_tracked" if opts.all_tracked else "default",
                "include_tests": opts.include_tests,
                "max_seq_length": opts.max_seq_length,
                "threshold": opts.threshold,
                "cluster_threshold": opts.cluster_threshold,
                "pairs": pairs,
                "clusters": groups,
            },
            indent=2,
            sort_keys=True,
        ),
        encoding="utf-8",
    )


def main() -> int:
    opts = args()
    require_valid(opts)
    root = repo_root()
    os.chdir(root)

    files = load_files(root, opts)
    if len(files) < 2:
        raise SystemExit("Need at least two included files to compare.")
    if opts.plan_only:
        chunk_count = sum(len(file_chunks) for _, file_chunks in files)
        scope = "all tracked files" if opts.all_tracked else "default live-source/doc roots"
        tests = "including tests" if opts.include_tests else "excluding tests"
        print(f"Selected {len(files)} files and {chunk_count} chunks ({scope}, {tests}).")
        return 0

    print(f"Embedding {len(files)} files with {opts.model}", file=sys.stderr)
    paths = [path for path, _ in files]
    vectors = embed(files, opts)
    pairs = similar_pairs(paths, vectors, opts.threshold, opts.top)
    groups = clusters(paths, vectors, opts.cluster_threshold)
    write_reports(root, opts, files, pairs, groups)
    print(f"Wrote {opts.output}", file=sys.stderr)
    print(f"Wrote {opts.json_output}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
