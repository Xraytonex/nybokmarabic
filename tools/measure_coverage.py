#!/usr/bin/env python3
"""Measure how much real code `nbma` translates without help.

This is a development tool: it reads Python standard-library sources, so the
numbers depend on the Python version that runs it.

Usage:
    python3 tools/measure_coverage.py target/release/nbma [--top N]
    python3 tools/measure_coverage.py target/release/nbma --prose [--top N]

Identifiers: every function, class, argument and assigned name defined in
the modules below is translated with a role guessed from the syntax tree
(function → fn, `is_`/`has_` function → bool, class → type, everything else
→ var). Dunder names are skipped.

Prose (`--prose`): the first sentence of every docstring and every comment
of three words or more, translated as written. Real comments are not
simplified first, as AGENTS.md asks agents to do, so this is a floor.

The script prints the share that translates and the unknown words that
would help most.
"""
from __future__ import annotations

import argparse
import ast
import collections
import importlib
import io
import json
import re
import subprocess
import tokenize
from concurrent.futures import ThreadPoolExecutor

MODULES = [
    "argparse", "textwrap", "csv", "json.decoder", "json.encoder", "http.client",
    "logging", "shutil", "smtplib", "configparser", "zipfile", "difflib",
    "tarfile", "pathlib", "email.message", "urllib.parse", "tempfile", "glob",
    "string", "calendar", "pprint", "ftplib", "sched", "queue", "gzip", "fnmatch",
    "unittest.case", "dataclasses", "inspect", "threading",
]
BOOL_PREFIXES = ("is_", "has_", "can_", "should_")


def defined_names(tree: ast.AST) -> set[tuple[str, str]]:
    out: set[tuple[str, str]] = set()
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            role = "bool" if node.name.lstrip("_").startswith(BOOL_PREFIXES) else "fn"
            out.add((node.name, role))
            for arg in node.args.args + node.args.kwonlyargs:
                if arg.arg not in ("self", "cls"):
                    out.add((arg.arg, "var"))
        elif isinstance(node, ast.ClassDef):
            out.add((node.name, "type"))
        elif isinstance(node, ast.Assign):
            out.update((t.id, "var") for t in node.targets if isinstance(t, ast.Name))
    return {(n, r) for n, r in out if not (n.startswith("__") and n.endswith("__"))}


def sentences(src: str) -> set[tuple[str, str]]:
    out: set[tuple[str, str]] = set()
    for node in ast.walk(ast.parse(src)):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef, ast.Module)):
            doc = ast.get_docstring(node)
            first = re.match(r"(.+?[.?!])(\s|$)", (doc or "").strip().split("\n\n")[0].replace("\n", " "))
            if first:
                out.add(("doc", first.group(1).strip()))
    for tok in tokenize.generate_tokens(io.StringIO(src).readline):
        if tok.type == tokenize.COMMENT:
            text = tok.string.lstrip("#").strip()
            if len(text.split()) >= 3 and text[0].isalpha():
                out.add(("comment", text))
    return out


def run_nbma(nbma: str, args: list[str]) -> dict:
    run = subprocess.run([nbma, *args, "--json"], capture_output=True, text=True, check=False)
    try:
        result = json.loads(run.stdout)
    except json.JSONDecodeError:
        result = {"error": "crash"}
    result["ok"] = run.returncode == 0
    return result


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("nbma", help="path to the nbma binary")
    ap.add_argument("--prose", action="store_true", help="docstrings and comments")
    ap.add_argument("--top", type=int, default=40, help="unknown words to list")
    args = ap.parse_args()

    items: set[tuple[str, str]] = set()
    for module in MODULES:
        path = importlib.import_module(module).__file__
        with open(path, encoding="utf-8") as fh:
            src = fh.read()
        items |= sentences(src) if args.prose else defined_names(ast.parse(src))
    ordered = sorted(items)

    def one(item: tuple[str, str]) -> dict:
        if args.prose:
            return {**run_nbma(args.nbma, ["prose", item[1]]), "kind": item[0]}
        return run_nbma(args.nbma, ["ident", item[0], "--role", item[1]])

    with ThreadPoolExecutor(8) as pool:
        results = list(pool.map(one, ordered))

    ok = sum(r["ok"] for r in results)
    errors = collections.Counter(r.get("error", "other") for r in results if not r["ok"])
    unknown = collections.Counter(w for r in results if r.get("error") == "unknown"
                                  for w in r["words"])
    what = "sentences" if args.prose else "identifiers"
    print(f"{len(results)} {what} from {len(MODULES)} modules: "
          f"{ok} translate ({100 * ok / len(results):.1f}%)")
    if args.prose:
        docs = [r for r in results if r["kind"] == "doc"]
        doc_ok = sum(r["ok"] for r in docs)
        print(f"  docstrings: {doc_ok} of {len(docs)} ({100 * doc_ok / len(docs):.1f}%)")
    for kind, n in errors.most_common():
        print(f"  {kind}: {n}")
    print("most frequent unknown words:")
    print("  " + ", ".join(f"{w} ({n})" for w, n in unknown.most_common(args.top)))


if __name__ == "__main__":
    main()
