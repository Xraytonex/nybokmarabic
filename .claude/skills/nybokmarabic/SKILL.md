---
name: nybokmarabic
description: Translate a codebase into Nybokmarabic (identifiers, comments, docstrings, docs) without changing its behaviour, using the nbma CLI for every word, and reverse it with nbma untranslate. Use when the user asks to translate, rewrite or rename a repo, file or identifier into Nybokmarabic, or to undo such a translation.
when_to_use: "translate this repo into Nybokmarabic", "nybokmarabify this file", "rename these identifiers to Nybokmarabic", "untranslate", anything mentioning ordbok.nbma.toml or the nbma CLI.
allowed-tools: Bash(nbma *)
---

# Nybokmarabic translation

Nybokmarabic merges Bokmål, Nynorsk and Arabic. Its grammar is in the
repo's `SPEC.md`; you do not need to know it, because **the `nbma` CLI does
all the language work. Never write a Nybokmarabic word yourself.** Your job is
picking the right inputs, renaming safely, and proving nothing changed.

If `nbma` is not on PATH, build it from the nybokmarabic repo with
`cargo build --release` and use `target/release/nbma`.

## Ground rules

- Work in a new branch. Never push, never force-push, never rewrite history.
- Behaviour must not change. Run the tests before and after.
- Rename with language-aware tooling (LSP rename, rope, an AST transform).
  Never regex find-and-replace across files.
- Every translation goes into `ordbok.nbma.toml` at the target repo root.

## Procedure

### 1. Baseline

Run the target's test suite and note the result. With no tests, note that
it builds or imports cleanly.

### 2. Decide what to translate

Translate identifiers defined in the repo, comments, docstrings and docs.
Do **not** translate keywords, standard-library or third-party names (or
methods overriding them: `__init__`, `toString`, `setUp`), wire formats (JSON
keys, DB columns, env vars, CLI flags, URLs), the license, vendored or
generated code, or exported public API. User-facing strings, file names and
commit messages are translated only if the user opts in.

### 3. Translate every identifier

Choose the role from what the identifier is:

| It is | Role |
|---|---|
| function or method | `fn` |
| variable, parameter, field, constant | `var` |
| boolean, or a function returning one | `bool` |
| class, struct, enum, trait, type alias | `type` |

```bash
nbma ident get_user_name --role fn --glossary ordbok.nbma.toml --record
```

```console
$ nbma ident get_user_name --role fn
uhnut_namn_ad_den_brukaren
$ nbma ident user_pair --role var
brukaraan
```

- Pairs and 2-tuples must be dual: `--number dual`.
- Keep framework prefixes: `--keep-prefix test_`.
- Add `--json` to see how each word was read.
- The glossary is read first, so reruns give the same names.

### 4. Unknown words (exit code 3)

Propose a Norwegian lemma, then translate as usual:

```bash
nbma propose widget --pos noun --bm dings --nn dings --gender m --glossary ordbok.nbma.toml
nbma ident get_widget --role fn --glossary ordbok.nbma.toml --record
```

Nouns need a gender (`m`, `f`, `n`). If ordbokene.no is reachable, check
the lemma there. `propose` refuses verbs whose root collides with an
existing verb; pick another lemma.

Abbreviations, acronyms and compounds (`fh`, `HTTP`, `filename`) are read
automatically. If one is still unknown, propose it as what it is:

```bash
nbma propose frob --abbrev "file handle" --glossary ordbok.nbma.toml
nbma propose MCP --acronym --glossary ordbok.nbma.toml
```

### 5. Pre-flight check, on the original code

```bash
nbma glossary check .
```

It must pass before you rename anything: every translated name must be
unique and must not already occur in the repo. That guarantee is what makes
the translation reversible.

### 6. Rename

Apply the `[identifiers]` mapping with semantic tooling, one identifier at a
time, so that standard-library names that happen to match English words
(`get`, `values`, `name`) are left alone. For Python, the nybokmarabic repo
has a tokenizer-based reference transform, `tools/py_apply_glossary.py`;
use it only if the pre-flight check passed and no repo name equals a
library attribute.

### 7. Comments and docs

Rewrite each comment as simple English sentences (subject verb object,
`is` + adjective, `has`, `does not`, yes/no questions). Then:

```bash
nbma prose "The user writes a file." --glossary ordbok.nbma.toml --record
```

```console
$ nbma prose "The user writes a file."
Fa yusakrivur ad-den-brukaren fil.
```

Paste the output exactly. If `nbma prose` refuses, simplify the sentence;
never translate by hand. The glossary keeps the original text, so
`untranslate` restores the original comment, not your simplification.

### 8. Verify

1. Run the tests. They must match the baseline.
2. Copy the translated tree, run `nbma untranslate` on the copy, and `diff`
   it against the original. It must be identical byte for byte.
3. Commit on the branch, including `ordbok.nbma.toml`. Do not push.

Report what you translated, which words you proposed, and the test results.

## Reversing

```bash
nbma untranslate path/to/repo --dry-run
nbma untranslate path/to/repo
```

## Arabic script (opt-in only)

`nbma fold "<prose>" --script arabic` renders comments in Arabic script.
Never put Arabic script into identifiers unless the user explicitly asks
and accepts the bidirectional-text risk; the CLI then requires
`--i-understand-bidi`.
