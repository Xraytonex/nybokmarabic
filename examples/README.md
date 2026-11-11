# Example: a user registry, before and after

`before/` is a small Python project (a user registry and its unittest
suite). `after/` is the same project in Nybokmarabic. Both pass their tests,
and `nbma untranslate` turns `after/` back into `before/` byte for byte.

```bash
./check.sh        # runs both test suites and the untranslate roundtrip
```

`cargo test` checks the roundtrip too (`tests/roundtrip.rs`).

## How `after/` was made

Exactly as [AGENTS.md](../AGENTS.md) prescribes, with every word coming out
of the CLI:

1. `registry` is not in the lexicon, so it was proposed as the Norwegian
   neuter noun `register`:
   `nbma propose registry --pos noun --bm register --nn register --gender n --glossary after/ordbok.nbma.toml`
2. Each identifier defined in the project was translated and recorded with
   `nbma ident <name> --role <role> --glossary after/ordbok.nbma.toml --record`.
   Test methods used `--keep-prefix test_` so unittest still finds them;
   `fields` (a 2-tuple) used `--number dual` and became `feltaan`.
3. Each docstring and comment was written as a simple English sentence and
   translated with `nbma prose "<sentence>" --glossary … --record`.
4. `nbma glossary check before --glossary after/ordbok.nbma.toml` passed.
5. `python3 tools/py_apply_glossary.py` applied the glossary with Python's
   tokenizer: identifiers renamed, comments and docstrings replaced, string
   literals and the module name in `from registry import …` left alone.

The project opts in to translating its public API (`Registry`, `User`,
`load_users`, …), since nothing outside the example imports it. The error
messages inside `ValueError(...)` are user-facing strings and stay English.

A few results worth a look in `after/registry.py`:

| English | Nybokmarabic | Why |
|---|---|---|
| `class Registry` | `class Register` | proposed lemma, PascalCase type |
| `def get_user` | `def uhnut_ad_den_brukaren` | imperative of `hente` + triple article |
| `def is_valid` | `def hal_gyldig` | `hal` + adjective, no copula |
| `self.active` | `self.hal_aktiv` | booleans take `hal_` |
| `fields` | `feltaan` | a 2-tuple, so dual |
| `found` | `mafnuun` | passive participle of `finne` |
| `"""The registry stores users."""` | `"""Fa tupapbivur ad-det-registeret brukarar."""` | neuter subject, feminine verb |
