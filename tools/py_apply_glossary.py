#!/usr/bin/env python3
"""Apply an ordbok.nbma.toml glossary to Python sources (forward direction).

A reference transform for the example project, built on the `tokenize`
module, so strings, comments and code are told apart exactly:

* NAME tokens listed under [identifiers] are renamed, except module names in
  `import` / `from ... import` statements (file names are opt-in).
* Comments and docstrings whose text matches a [[text]] block are replaced.
* Everything else is copied byte for byte.

It is not scope-aware: a repo-defined name that equals a standard-library
attribute (`values`, `get`) would be renamed everywhere. Run
`nbma glossary check` first, and prefer an LSP rename (pylsp, pyright) or rope
for real projects. See AGENTS.md.

Usage: python3 tools/py_apply_glossary.py GLOSSARY SRC_DIR DEST_DIR
"""
from __future__ import annotations

import io
import shutil
import sys
import tokenize
import tomllib
from pathlib import Path


def load(glossary: Path) -> tuple[dict[str, str], dict[str, str]]:
    data = tomllib.loads(glossary.read_text(encoding="utf-8"))
    names = {k: v["to"] for k, v in data.get("identifiers", {}).items()}
    texts = {b["from"]: b["to"] for b in data.get("text", [])}
    return names, texts


def edits_for(source: str, names: dict[str, str], texts: dict[str, str]) -> list[tuple[int, int, str]]:
    """(offset, length, replacement) edits for one file."""
    line_starts = [0]
    for line in source.splitlines(keepends=True):
        line_starts.append(line_starts[-1] + len(line))

    def offset(pos: tuple[int, int]) -> int:
        return line_starts[pos[0] - 1] + pos[1]

    edits = []
    # Import state: in `from MOD import NAMES` the module path is kept and
    # the imported names are renamed; in `import MOD` everything is kept.
    state = "code"
    tokens = tokenize.generate_tokens(io.StringIO(source).readline)
    for tok in tokens:
        start, end = offset(tok.start), offset(tok.end)
        if tok.type == tokenize.NEWLINE:
            state = "code"
        elif tok.type == tokenize.NAME and state == "code" and tok.string == "from":
            state = "from-module"
        elif tok.type == tokenize.NAME and state == "code" and tok.string == "import":
            state = "import-module"
        elif tok.type == tokenize.NAME and state == "from-module" and tok.string == "import":
            state = "from-names"
        if state in ("from-module", "import-module"):
            continue
        if tok.type == tokenize.NAME and tok.string in names:
            edits.append((start, end - start, names[tok.string]))
        elif tok.type == tokenize.COMMENT:
            body = tok.string.lstrip("#").strip()
            if body in texts:
                edits.append((start, end - start, tok.string.replace(body, texts[body])))
        elif tok.type == tokenize.STRING:
            for quote in ('"""', "'''"):
                inner = tok.string[len(quote):-len(quote)] if tok.string.startswith(quote) else None
                if inner is not None and inner in texts:
                    edits.append((start, end - start, f"{quote}{texts[inner]}{quote}"))
                    break
    return edits


def apply(source: str, edits: list[tuple[int, int, str]]) -> str:
    for start, length, replacement in sorted(edits, reverse=True):
        source = source[:start] + replacement + source[start + length:]
    return source


def main() -> int:
    glossary, src, dest = (Path(a) for a in sys.argv[1:4])
    names, texts = load(glossary)
    for path in sorted(src.rglob("*")):
        if path.is_dir() or "__pycache__" in path.parts:
            continue
        target = dest / path.relative_to(src)
        target.parent.mkdir(parents=True, exist_ok=True)
        if path.suffix == ".py":
            source = path.read_text(encoding="utf-8")
            target.write_text(apply(source, edits_for(source, names, texts)), encoding="utf-8")
        else:
            shutil.copyfile(path, target)
    shutil.copyfile(glossary, dest / glossary.name)
    return 0


if __name__ == "__main__":
    sys.exit(main())
