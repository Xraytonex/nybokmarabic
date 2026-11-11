# Nybokmarabic

*ad-den-nybokmarabisken*: Bokmål, Nynorsk and Arabic, merged by regulation.

Norway has two official ways to write Norwegian. For most of the twentieth
century the state tried to merge them into one, Samnorsk, through the
spelling reforms of 1917 and 1938 and a goal written into law. In 2002 the
Storting removed that goal from the law. Two standards could not agree.

Nybokmarabic finishes the job by adding Arabic, on the theory that a
committee of three can always reach a majority. Every rule resolves a real
conflict between the source languages in the most bureaucratic way
available. Nouns are Norwegian. Verbs are Norwegian consonants poured into
Arabic templates. Prepositions are Arabic. Articles are all of them at once.

```console
$ nbma prose "The user writes a file."
fa yusakrivur ad-den-brukaren fil.
```

`ad-den-brukaren` is "the user" with three definite articles: Arabic `al-`
(assimilated to the `d` that follows), the Norwegian determiner `den`, and
the Norwegian suffix `-en`. `yusakrivur` is `skrive` ("write") reduced to
the root S-K-R-V and conjugated as an Arabic quadriliteral verb, plus the
Norwegian present-tense `-r`, because Norwegian verbs do not agree with
person, Arabic verbs do, and both sides had to be satisfied. `fa` opens the
clause so the verb can be first (Arabic) and second (Norwegian) at the same
time.

## What's in the box

1. **The language.** [SPEC.md](SPEC.md) is the full grammar.
   [CORRECTIONS.md](CORRECTIONS.md) lists where the original design brief
   was wrong about the source languages, and [COLLISIONS.md](COLLISIONS.md)
   lists the accidental matches with real Arabic roots.
2. **`nbma`**, a deterministic, offline Rust CLI that inflects words and
   translates identifiers. It is the source of truth: nobody inflects by
   hand.
3. **Agent instructions.** [AGENTS.md](AGENTS.md) and a Claude Code skill
   ([.claude/skills/nybokmarabic/SKILL.md](.claude/skills/nybokmarabic/SKILL.md))
   tell a coding agent how to rewrite a whole codebase into Nybokmarabic,
   identifiers, comments and docs included, without changing what it does,
   and how to reverse it.

## Quick demo

```console
$ nbma ident get_user_name --role fn
uhnut_namn_ad_den_brukaren
$ nbma ident is_valid --role bool
hal_gyldig
$ nbma ident on_click --role fn
yaa_3alaa_ad_det_klikket
$ nbma ident Parser --role type
Taalik
$ nbma ident twenty_users --role var
tjue_brukar
$ nbma inflect hente
root         H-N-T
past         hanata
present      yahnutur
imperative   uhnut
doer         hānit
passive      mahnūt
verbal-noun  hant
place        mahnat
$ nbma number 22
to wa tjue
$ nbma prose "The user is not valid."
fa mā ikkje ad-den-brukaren gyldig.
```

A few things to notice:

- `get_user_name` is the imperative of `hente` ("get"), then "name of the
  user" as an Arabic construct with Norwegian nouns. `brukar` and `namn` are
  Nynorsk because `bruker` and `navn` have an even number of letters.
- `is_valid` loses its copula, because Arabic has no present-tense "is", and
  gains `hal`, the Arabic question particle.
- `on_click` starts with `ʿalā` ("on"), whose ayn is written `3` in code.
  Identifiers cannot start with a digit, so it is addressed with the
  vocative `yā`.
- Arabic puts nouns counted from 11 upwards in the singular, so twenty users
  are twenty user.
- When a verbless sentence is negated, the circumfix `mā … ikkje` wraps the
  verb that is not there, so the two negators end up next to each other.

## Install

```bash
cargo install --path .
```

No network access at runtime. The lexicon is compiled into the binary.

## Translating a codebase

Point your agent at [AGENTS.md](AGENTS.md), or use the bundled Claude Code
skill. The short version: every identifier and comment goes through `nbma`
and is recorded in `ordbok.nbma.toml`, renames use language-aware tooling,
the test suite must pass before and after, and `nbma untranslate` restores
the original byte for byte.

[examples/](examples/) has a small Python project in both languages, with
passing tests in each. `examples/check.sh` runs both suites and the
untranslate roundtrip.

## Honesty

The author speaks none of the three languages. To keep the language wrong
on purpose rather than by accident:

- Every Norwegian form (lemma, gender, definite form, plural) was copied
  from Bokmålsordboka and Nynorskordboka (ordbokene.no) by
  `tools/verify_lexicon.py`, not typed from memory.
- The Arabic templates, particles and agreement rules were checked against
  standard grammar references. Every place where the original brief was
  wrong is in [CORRECTIONS.md](CORRECTIONS.md).
- Lexicon entries that could not be verified are marked `verified = false`.
  There are currently three: `render`, `serialize` and `deserialize`,
  whose Norwegian lemmas (`gjengi`/`gjengje`, `serialisere`,
  `deserialisere`) are not in the dictionaries. `nbma` reports the count:

```console
$ nbma lexicon check
ok: 570 entries, 3 unverified, no collisions
```

The semantic choices (which Norwegian word means "get") are the author's own
and may be wrong in the ordinary, accidental way. Corrections from people
who speak the languages are welcome.

Nybokmarabic jokes about grammar and about Norwegian language politics. It
does not joke about religion or about people.

## Development

```bash
cargo test                     # rules, 80+ golden identifiers, docs, example roundtrip
cargo clippy --all-targets
NBMA_BLESS=1 cargo test        # accept new CLI output into the docs, then review the diff
```

Every `$ nbma` line in the Markdown files is executed by `cargo test`, so
the documentation cannot disagree with the CLI.

## License

MIT. See [LICENSE](LICENSE).
