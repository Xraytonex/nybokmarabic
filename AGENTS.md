# AGENTS.md

Instructions for coding agents. There are two jobs here: **translating
another repository into Nybokmarabic** (most of this file), and **working on
this repository** (the last section).

## Translating a repository

The goal is a codebase whose identifiers, comments and docs are in
Nybokmarabic and whose behaviour is unchanged. The `nbma` CLI is the source
of truth. **Never inflect a word by hand**: every Nybokmarabic word you write
comes out of `nbma`.

### 0. Prepare

1. Build the CLI: `cargo build --release` in this repo, then use
   `target/release/nbma` (or `cargo install --path .`).
2. In the target repo, create a branch. Never push, never force anything.
3. Run the test suite and record the result. If there are no tests, record
   that the project builds or imports cleanly. This is the baseline.

### 1. Collect identifiers

List every identifier **defined in the repo**: functions, methods, classes,
variables, parameters, fields, constants. Use language tooling (an LSP
"document symbols" request, `ctags`, the language's AST module), not regex.

Leave alone:

- language keywords, standard-library and third-party names, and methods
  that override them (`__init__`, `toString`, `setUp`, `main`);
- wire formats: JSON keys, database columns, environment variables, CLI
  flags, URLs, anything serialised;
- exported public API, unless the user opts in;
- license text, vendored and generated directories;
- file and module names, commit messages and user-facing strings, unless the
  user opts in to each.

### 2. Translate each identifier with the CLI

Pick the role from what the identifier is, not what its name looks like:

| The identifier is | `--role` |
|---|---|
| a function or method | `fn` |
| a variable, parameter, field or constant | `var` |
| something holding a boolean, or a function returning one | `bool` |
| a class, struct, enum, trait or type alias | `type` |

Record every translation in the glossary at the target repo root:

```bash
nbma ident get_user_name --role fn --glossary ordbok.nbma.toml --record
```

- A variable holding a pair or a 2-tuple must be dual: add `--number dual`.
- Collections are plural; the CLI reads that from the English plural.
- Framework prefixes stay: `--keep-prefix test_` for test functions.
- `--json` gives machine-readable output, including how each word was read.

```console
$ nbma ident get_user_name --role fn
uhnut_namn_ad_den_brukaren
$ nbma ident is_valid --role bool
hal_gyldig
$ nbma ident Parser --role type
Taalik
```

### 3. Unknown words

If a word is missing from the lexicon, `nbma` exits with code 3 and names
it. You may propose a Norwegian lemma (Bokmål and Nynorsk forms, gender for
nouns). The CLI records it under `[proposed]` in the glossary, rejects it if
its root collides with an existing verb, and from then on inflects it like
any other word:

```bash
nbma propose widget --pos noun --bm dings --nn dings --gender m --glossary ordbok.nbma.toml
nbma ident get_widget --role fn --glossary ordbok.nbma.toml --record
```

Check proposed lemmas against ordbokene.no if you can. Never skip a word
and never invent an inflected form.

### 4. Pre-flight check

Before renaming anything, on the **original** code:

```bash
nbma glossary check .
```

It fails if two originals map to the same name or if a translated name
already occurs in the repo. Both would make the translation irreversible.
When two names collide, the later one recorded gets an Arabic ordinal
(`_thaanii`, "second"); if the check still fails, rename by hand only to
choose a different English input, then translate that with the CLI.

### 5. Rename with language-aware tooling

Use an LSP rename (rust-analyzer, pyright/pylsp, tsserver, gopls), a
refactoring library (rope) or an AST transform. **No regex find-and-replace
across files**: English names like `get`, `values` or `name` collide with
standard-library names, and only semantic tooling can tell them apart.
`tools/py_apply_glossary.py` in this repo is a tokenizer-based reference
transform for Python. It never renames module names in imports, but it is
not scope-aware, so use it only when the pre-flight check passes and no
renamed name shadows a library attribute.

### 6. Comments and docs

Rewrite each comment or docstring as one or more simple English sentences
(subject, verb, object; `is` + adjective; `has`; `does not`; yes/no
questions), then translate and record it:

```bash
nbma prose "The user writes a file." --glossary ordbok.nbma.toml --record
```

```console
$ nbma prose "The user writes a file."
fa yusakrivur ad-den-brukaren fil.
$ nbma prose "Read the file."
nbma: cannot translate "Read the file.": no subject found. Simplify the sentence.
```

Proposed words (step 3) work in prose too, as long as you pass the same
`--glossary`.

If `nbma prose` refuses a sentence, simplify it. Do not hand-translate. The
glossary stores the original text, so the original comment is restored
exactly by `untranslate` even though the simplified sentence was what got
translated. Replace the comment text exactly as the CLI printed it.

### 7. Verify

1. Run the test suite again. It must match the baseline. If anything
   differs, the translation changed behaviour: fix it or revert that rename.
2. Prove reversibility on a copy:

```bash
git worktree add ../original HEAD~0   # or any checkout of the untranslated code
cp -R . ../reversed && nbma untranslate ../reversed
diff -r --exclude=ordbok.nbma.toml --exclude=.git ../original ../reversed
```

`nbma untranslate` must restore the original byte for byte. Commit the
translation with `ordbok.nbma.toml` on the branch; do not push.

### Reversing a translation

```bash
nbma untranslate path/to/repo   # reads path/to/repo/ordbok.nbma.toml
```

Use `--dry-run` first to see what would change.

## Working on this repository

- `cargo test` runs unit tests for every rule, 80+ golden identifiers
  (`tests/golden/identifiers.toml`), the example roundtrip, and **every
  `$ nbma` line in the Markdown docs** (`tests/docs.rs`). Docs cannot drift
  from the CLI. After an intended behaviour change, run
  `NBMA_BLESS=1 cargo test` and review the diff.
- `cargo clippy --all-targets` must be clean.
- The lexicon source is `lexicon/seed.txt`. Regenerate `lexicon/core.toml`
  with `python3 tools/verify_lexicon.py lexicon/seed.txt lexicon/core.toml`
  (needs network for ordbokene.no; the CLI itself never does), then run
  `nbma lexicon check`. Fix root collisions by adding `| form=N` to the
  later verb in the seed.
- `examples/check.sh` runs the example's tests in both languages and the
  untranslate roundtrip.
- Jokes target grammar and Norwegian language politics. Never religious
  phrases, stereotypes, or people as a punchline.
