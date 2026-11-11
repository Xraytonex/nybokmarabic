# Nybokmarabic: the specification

*ad-den-nybokmarabisken*, in its own words.

This document describes Nybokmarabic as the `nbma` CLI implements it. Every
`$ nbma` example below is executed by `cargo test` (`tests/docs.rs`), so the
spec cannot drift from the tool. Where the CLI and this text disagree, the
test fails and the CLI wins.

Where the original design brief misdescribed Bokmål, Nynorsk or Arabic, the
description was fixed and every change is listed in
[CORRECTIONS.md](CORRECTIONS.md). Accidental matches between Nybokmarabic
roots and real Arabic roots are in [COLLISIONS.md](COLLISIONS.md).

The governing principle: **the language is wrong on purpose, never by
accident.** Each rule resolves a real conflict between the source languages
in the most bureaucratic way available.

## 1. Division of labour

| Part of speech | Source |
|---|---|
| Nouns, adjectives, adverbs, numerals | Norwegian (Bokmål or Nynorsk, see §2) |
| Verbs | Norwegian stems, reduced to consonant roots, conjugated with Arabic templates (§8) |
| Prepositions, conjunctions, particles, quantifiers | Arabic (§1.1) |
| Articles | all of them, at once (§4) |

### 1.1 Particles

```console
$ nbma inflect and
arabic  wa
$ nbma inflect from
arabic  min
$ nbma inflect on
arabic  ʿalā
$ nbma inflect under
arabic  taḥta
$ nbma inflect all
arabic  kull
```

Particles are given by their English key: they have no Norwegian lemma. The
full list is in `lexicon/core.toml` (`pos = "part"` and `pos = "det"`).

Arabic writes `wa`, `fa`, `bi-`, `li-` and `ka-` attached to the next word,
and the vowel of the article elides after them (`wa-l-kitāb`). Norwegian
writes `og` as a separate word. The matter was put to a vote of the source
languages, Bokmål and Nynorsk voted together, and Nybokmarabic writes every
particle as a separate word with no elision.

## 2. The parity rule

Where Bokmål and Nynorsk differ, count the letters of the Bokmål lemma. Odd
selects Bokmål, even selects Nynorsk. `æ`, `ø` and `å` count as one letter
each; hyphens are not letters. The rule is word-local, so a word is spelled
the same way everywhere in a codebase.

```console
$ nbma inflect bruker --form indef
brukar
$ nbma inflect navn --form indef
namn
$ nbma inflect hus --form indef
hus
```

`bruker` has six letters (Nynorsk `brukar`), `navn` four (Nynorsk `namn`),
`hus` three (Bokmål `hus`). The negator `ikke` has four letters, so
Nybokmarabic always says `ikkje`.

The parity rule selects a whole paradigm, not just a lemma: gender, definite
form and plural all come from the selected standard. All Norwegian forms in
the lexicon were copied from Bokmålsordboka and Nynorskordboka
(ordbokene.no) by `tools/verify_lexicon.py`.

The article cluster (§4) belongs to its noun, so the plural determiner `de`
(Bokmål) or `dei` (Nynorsk) follows the noun's parity.

## 3. Orthography

The same text has two renderings.

**Prose mode** (comments, docs): Norwegian letters `æ ø å`, Arabic
transliteration with macrons and dots (`ā ī ū ḥ ṣ ṭ ʿ ʾ`), morpheme
boundaries written with hyphens. Transliteration follows ALA-LC, which does
not write a word-initial hamza (`ilā`, not `ʾilā`).

**Code mode** (identifiers): ASCII only.

| Prose | Code | Why |
|---|---|---|
| `ā ī ū` | `aa ii uu` | |
| `æ ø å` | `ae oe aa` | Norwegian wrote `aa` for `å` until 1917, so this is a restoration |
| `ʿ` (ayn) | `3` | Arabic chat alphabet |
| `ḥ` | `7` | Arabic chat alphabet |
| `ʾ` (hamza) | `2` | Arabic chat alphabet |
| `ṣ` | `9` | Arabic chat alphabet |
| `ṭ` | `6` | Arabic chat alphabet |
| `ḍ ẓ` | `d z` | the chat alphabet has no unambiguous digit for these |
| hyphen, space | the file's case convention (`_`, camelCase boundary, …) | |

`ā` and `å` both fold to `aa`. Code mode is lossy by design; the glossary
(§14.3), not the spelling, is what makes a translation reversible.

```console
$ nbma fold "taḥta ad-det-huset"
ta7ta_ad_det_huset
$ nbma fold "taḥta ad-det-huset" --case camel
ta7taAdDetHuset
```

### 3.1 The vocative

An identifier cannot start with a digit. Any identifier that would is
addressed with the vocative particle `yā` (code `yaa`):

```console
$ nbma fold "ʿalā ad-det-klikket"
yaa_3alaa_ad_det_klikket
```

An identifier that would come out as a reserved word in a common
programming language is addressed the same way:

```console
$ nbma ident kind --role var
yaa_type
```

## 4. Triple definiteness

Norwegian suffixes the article (`hus-et`) and adds a second, free-standing
one when an adjective is present (`det store huset`). Arabic prefixes `al-`.
Nybokmarabic uses all three, always, adjective or not.

`al-` assimilates to a following sun letter. The Norwegian determiner always
begins with `d`, which is a sun letter, so the article never surfaces as
`al-`: it is always `ad-`.

```console
$ nbma inflect hus --form def
ad-det-huset
$ nbma inflect bruker --form def
ad-den-brukaren
$ nbma inflect bruker --form def-pl
ad-dei-brukarane
$ nbma inflect hus --form def-pl
ad-de-husa
```

Indefinite nouns are bare:

```console
$ nbma inflect hus --form indef
hus
```

Feminine is mandatory wherever Norwegian allows it. Bokmål allows the
feminine for most feminine nouns but does not require it; Nybokmarabic
requires it.

```console
$ nbma inflect fil --form def
ad-den-fila
$ nbma inflect bok --form def
ad-den-boka
```

## 5. Possession and compounds (iḍāfa)

Norwegian glues compounds together (`brukernavn`). Arabic uses the construct
state: the head comes first and bare, the possessor follows and is definite.
Nybokmarabic uses the Arabic construction with Norwegian nouns, and only the
last noun in a chain takes the triple article.

```console
$ nbma ident user_name --role var
namn_ad_den_brukaren
$ nbma ident user_account_name --role var
namn_konto_ad_den_brukaren
```

English `of` builds the same construction:

```console
$ nbma ident name_of_user --role var
namn_ad_den_brukaren
```

## 6. Adjectives

The adjective follows the noun (Arabic). It agrees in definiteness by taking
its own `al-`, assimilated by the sun-letter rule, and takes the Norwegian
weak form when definite.

```console
$ nbma inflect stor --form def
as-store
$ nbma inflect gyldig --form def
al-gyldige
$ nbma prose "The big house reads."
Fa talsuwur ad-det-huset as-store.
```

Sun letters are `t th d dh r z s sh ṣ ḍ ṭ ẓ l n`. Norwegian letters with no
Arabic counterpart (`g p v c x`) are moon letters. The first consonant is
read with the root digraph rule (§8.1), so `skj-` counts as `sh`:

```console
$ nbma inflect skjønn --pos adj --form def
ash-skjønne
```

Indefinite adjectives agree in gender and number the Norwegian way
(`hus stort`, `brukarar gyldige`). Weak forms come from the dictionary, so
irregular ones are right: `liten` becomes `lille` in Bokmål and `litle` in
Nynorsk.

## 7. Gender and number

- **Three genders for nouns** (Norwegian `-en`, `-a`, `-et`).
- **Two genders for verbs** (Arabic). Neuter subjects take feminine
  agreement, by analogy with Arabic non-human plurals.
- **Plural**: the plural of the standard the parity rule selected, taken
  from the dictionary. (The brief said "`-er` or `-ar`"; see CORRECTIONS.md.)
- **The dual is kept.** Exactly two of something takes `-ān` (code `-aan`),
  the Arabic pausal dual. An unstressed final `-e` drops first.

```console
$ nbma inflect bruker --form dual
brukarān
$ nbma inflect side --form dual
sidān
$ nbma ident user_pair --role var
brukaraan
```

In code, any variable holding a pair or a 2-tuple must be dual. Words like
`pair`, `pairs` and `both` make it dual automatically; otherwise pass
`--number dual`:

```console
$ nbma ident fields --role var --number dual
feltaan
```

## 8. Verbs

### 8.1 Root extraction

Take the Norwegian infinitive in the selected standard, drop a final `-e` or
`-a`, and keep the consonants in order.

- Digraphs count as one consonant: `skj` and `sj` become `sh`, `kj` becomes
  `kh`.
- Three consonants: a triliteral root.
- Four or more: the first four, a quadriliteral root.
- Fewer than three: pad with `w`, then `y` (a weak root).
- No consonants at all: the word starts with a glottal stop in careful
  speech, so the root starts with a hamza.

Dropping the final vowel has no effect, because the next step drops every
vowel anyway. It stays because it is in the regulation, and because Nynorsk
a-infinitives (`skriva`) make it look necessary.

```console
$ nbma root hente
H-N-T
$ nbma root skrive
S-K-R-V
$ nbma root sjekke
SH-K-K
$ nbma root lese
L-S-W
$ nbma root se
S-W-Y
$ nbma root eie
ʾ-W-Y
```

### 8.2 Templates

`1 2 3 4` stand for the root consonants.

| Form | Past (3sg m) | Present (3sg m) | Imperative | Active participle (doer) | Passive participle | Verbal noun | Noun of place |
|---|---|---|---|---|---|---|---|
| I | 1a2a3a | ya12u3u + r | u12u3 | 1ā2i3 | ma12ū3 | 1a23 | ma12a3 |
| II | 1a22a3a | yu1a22i3u + r | 1a22i3 | mu1a22i3 | mu1a22a3 | ta12ī3 | mu1a22a3 |
| III | 1ā2a3a | yu1ā2i3u + r | 1ā2i3 | mu1ā2i3 | mu1ā2a3 | mu1ā2a3a | mu1ā2a3 |
| IV | a12a3a | yu12i3u + r | a12i3 | mu12i3 | mu12a3 | i12ā3 | mu12a3 |
| V | ta1a22a3a | yata1a22a3u + r | ta1a22a3 | muta1a22i3 | muta1a22a3 | ta1a22u3 | muta1a22a3 |
| VI | ta1ā2a3a | yata1ā2a3u + r | ta1ā2a3 | muta1ā2i3 | muta1ā2a3 | ta1ā2u3 | muta1ā2a3 |
| VII | in1a2a3a | yan1a2i3u + r | in1a2i3 | mun1a2i3 | mun1a2a3 | in1i2ā3 | mun1a2a3 |
| VIII | i1ta2a3a | ya1ta2i3u + r | i1ta2i3 | mu1ta2i3 | mu1ta2a3 | i1ti2ā3 | mu1ta2a3 |
| X | ista12a3a | yasta12i3u + r | ista12i3 | musta12i3 | musta12a3 | isti12ā3 | musta12a3 |
| Q-I | 1a23a4a | yu1a23i4u + r | 1a23i4 | mu1a23i4 | mu1a23a4 | 1a23a4a | mu1a23a4 |
| Q-II | ta1a23a4a | yata1a23a4u + r | ta1a23a4 | muta1a23i4 | muta1a23a4 | ta1a23u4 | muta1a23a4 |

The patterns are standard Arabic (Form I uses the u-class vowels). Two
simplifications: Arabic Form I verbal nouns are unpredictable and
Nybokmarabic always uses `faʿl`; and for every form except I the noun of
place has the passive participle's shape, as in Arabic. In Form VIII the
infixed `t` assimilates after `d`, `z`, `th`, `dh`, the emphatics and weak
`w`/`y`, as in Arabic. Form IX is skipped because it is reserved for colours
and bodily defects.

```console
$ nbma inflect hente
root         H-N-T
past         hanata
present      yahnutur
imperative   uhnut
doer         hānit
passive      mahnūt
verbal-noun  hant
place        mahnat
$ nbma inflect skrive
root         S-K-R-V
past         sakrava
present      yusakrivur
imperative   sakriv
doer         musakriv
passive      musakrav
verbal-noun  sakrava
place        musakrav
```

### 8.3 The present tense

The present takes the Arabic person prefix and the Arabic mood ending
`-u`, and then the Norwegian present ending `-r`. Norwegian verbs do not
agree with person and Arabic verbs do; both sides had to be satisfied.

Person marking:

| Person | Present prefix (I, V–X, Q-II) | Present prefix (II–IV, Q-I) | Past suffix |
|---|---|---|---|
| 1sg | a- | u- | -tu |
| 2sg | ta- | tu- | -ta |
| 3sg m | ya- | yu- | -a |
| 3sg f | ta- | tu- | -at |
| 1pl | na- | nu- | -nā |
| 3pl | ya- … -ūna | yu- … -ūna | -ū |

```console
$ nbma inflect hente --form present --person 1pl
nahnutur
$ nbma inflect hente --form past --person 3sgf
hanatat
$ nbma inflect hente --form present --person 3pl
yahnutūnar
```

### 8.4 Collisions between Norwegian verbs

Two different Norwegian verbs with the same root are a build error in the
lexicon. The later entry is moved to the next free form: II, then III, IV,
V, VI, VII, VIII and X for triliteral roots, Q-II for quadriliteral ones.
`nbma lexicon check` reports collisions and suggests the form:

```console
$ nbma lexicon check
ok: 582 entries, 3 unverified, no collisions
$ nbma inflect låse --form past
lassawa
```

`lese` (read) and `låse` (lock) share the root L-S-W, so `låse` is Form II.
`løse` (resolve, Nynorsk `løyse`) arrived third and is Form III.

Arabic templates also apply to verbs used as nouns: a type named after an
`-er` word becomes the doer participle.

```console
$ nbma ident Parser --role type
Taalik
```

## 9. Word order

Norwegian main clauses are verb-second. Arabic prefers the verb first. Every
main clause therefore starts with `fa`, followed by the verb, then the
subject, then the object.

```console
$ nbma prose "The user writes a file."
Fa yusakrivur ad-den-brukaren fil.
```

A Norwegian grammarian will object that a coordinating conjunction stands
outside the clause and does not fill the first position. Nybokmarabic agrees,
and therefore classifies `fa` as the equivalent of the Norwegian adverb
`så` ("then"), which does fill the first position and does trigger inversion.
The verb is second, and Norway is satisfied. The verb precedes the subject,
and Arabic is satisfied.

The verb agrees with the subject in gender only and stays singular, as in
Arabic verb–subject order. Neuter subjects take feminine agreement:

```console
$ nbma prose "The users write files."
Fa yusakrivur ad-dei-brukarane filer.
$ nbma prose "The house reads."
Fa talsuwur ad-det-huset.
```

Pronoun subjects are marked twice: Arabic needs the person on the verb,
Norwegian needs a subject.

```console
$ nbma prose "We read the file."
Fa nalsuwur vi ad-den-fila.
```

### 9.1 Object pronouns

The same settlement applies to objects. Arabic attaches an object pronoun
to the verb as a suffix; Norwegian writes it as a separate word after the
verb. Nybokmarabic does both. The Norwegian word is the object form of the
pronoun, chosen by the parity rule applied to the subject form: `jeg` has
three letters, so Bokmål `meg`; `de` has two, so Nynorsk `dei`.

Each pronoun shows its Arabic suffix and its Norwegian object form. (`her`
is also a Norwegian word, "here", so it needs `--pos pron`.)

```console
$ nbma inflect me
suffix  -nī
object  meg
$ nbma inflect you
suffix  -ka
object  deg
$ nbma inflect him
suffix  -hu
object  ham
$ nbma inflect her --pos pron
suffix  -hā
object  henne
$ nbma inflect us
suffix  -nā
object  oss
$ nbma inflect them
suffix  -hum
object  dei
```

```console
$ nbma prose "The user reads me."
Fa yalsuwur-nī ad-den-brukaren meg.
$ nbma ident help_us --role fn
hajlip_naa_oss
$ nbma ident READ_ME --role fn
ULSUW_NII_MEG
```

The Nybokmarabic README is therefore called `ULSUW_NII_MEG.md`: "read-me
me".

## 10. Negation

A circumfix: `mā` before the verb, the parity-selected Norwegian negator
`ikkje` after it.

```console
$ nbma prose "The user does not write a file."
Fa mā yusakrivur ikkje ad-den-brukaren fil.
```

`mā` is used for every tense. Modern Standard Arabic prefers `lā` for the
present; `mā` with the present is attested in Classical Arabic and normal in
the spoken dialects, and Nybokmarabic takes the dialects' side.

## 11. Questions, the copula and booleans

There is no present-tense copula. A verbless sentence still starts with
`fa`; the missing verb is in second position. A negated verbless sentence
wraps the circumfix around the missing verb, so the two negators touch. This
use of `mā` before a verbless clause follows the Classical "Ḥijāzī mā".

```console
$ nbma prose "The user is valid."
Fa ad-den-brukaren gyldig.
$ nbma prose "The user is not valid."
Fa mā ikkje ad-den-brukaren gyldig.
$ nbma prose "The house is big."
Fa ad-det-huset stort.
```

Yes/no questions start with `hal`. This puts the verb second again, which
Norwegian did not ask for in questions but accepts.

```console
$ nbma prose "Does the user write a file?"
Hal yusakrivur ad-den-brukaren fil?
$ nbma prose "Is the file empty?"
Hal ad-den-fila tom?
```

Possession uses `ʿinda` ("at"), as Arabic does:

```console
$ nbma prose "The user has a valid email."
Fa ʿinda ad-den-brukaren e-post gyldig.
```

In code, boolean predicates take `hal_`, and the copula disappears:

```console
$ nbma ident is_valid --role bool
hal_gyldig
$ nbma ident is_not_empty --role bool
hal_maa_ikkje_tom
$ nbma ident has_children --role bool
hal_3inda_barn
$ nbma ident is_enabled --role bool
hal_mukatvar
```

## 12. Numbers

Norwegian numerals, units before tens, joined with `wa`. Old Norwegian
counting (abolished by the Storting in 1950, in force from 1951) and Arabic
agree on this order, which is the only thing they agree on. Hundreds and
thousands come first; two hundred is the dual of hundred, as Arabic
`miʾatān` is.

```console
$ nbma number 22
to wa tjue
$ nbma number 200
hundrān
$ nbma number 1222
tusen wa hundrān wa to wa tjue
```

Counted nouns follow Arabic agreement: one is singular with the numeral
after the noun, two is the bare dual, three to ten take the plural, and
eleven upwards take the **singular**.

```console
$ nbma prose "The user writes 2 files."
Fa yusakrivur ad-den-brukaren filān.
$ nbma prose "The user writes 3 files."
Fa yusakrivur ad-den-brukaren tre filer.
$ nbma prose "The user writes 22 files."
Fa yusakrivur ad-den-brukaren to wa tjue fil.
```

## 13. The language's own name

A language name used as a noun is either masculine (`norsken`) or an
uninflected neuter. An uninflected noun cannot carry the suffixed article,
so triple definiteness selects the masculine:

```console
$ nbma inflect nybokmarabisk --pos noun --gender m --form def
ad-den-nybokmarabisken
```

## 14. Translating code

### 14.1 One identifier

1. Split it with the file's case convention.
2. Map each English word to a lexicon entry (plurals, participles, `-ing`
   and `-er` forms are recognised).
3. Apply the parity rule.
4. Apply grammar by role:
   - `fn`: verbs in the imperative, objects definite (§4, §5);
   - `var`: a noun phrase, plural for collections, dual for pairs;
   - `bool`: `hal_` plus an adjective, participle or present-tense verb;
   - `type`: an indefinite noun phrase in PascalCase, or the doer
     participle for `-er` names.
5. Render in code mode with the original case convention.

```console
$ nbma ident get_user_name --role fn
uhnut_namn_ad_den_brukaren
$ nbma ident getUserName --role fn
uhnutNamnAdDenBrukaren
$ nbma ident find_users_by_email --role fn
ufnun_ad_dei_brukarane_7asaba_ad_den_e_posten
$ nbma ident get_all_users --role fn
uhnut_kull_ad_dei_brukarane
$ nbma ident twenty_users --role var
tjue_brukar
$ nbma ident MAX_SIZE --role var
STOERRELSE_MAKSIMAL
$ nbma ident FileReader --role type
LaasiwAdDenFila
$ nbma ident test_get_user --role fn --keep-prefix test_
test_uhnut_ad_den_brukaren
```

Single letters and words containing digits pass through (`x`, `utf8`,
`user2`). Dunder names (`__init__`) are refused: they belong to the runtime.

### 14.2 What gets translated

| Translate | Never translate |
|---|---|
| Identifiers defined in the repo | Language keywords |
| Comments and docstrings | Standard library and third-party names |
| README and docs | Wire formats: JSON keys, DB columns, env vars, CLI flags, URLs |
| Commit messages (opt-in) | License text |
| User-facing strings (opt-in) | Exported public API (unless opted in) |
| File and module names (opt-in) | Vendored or generated directories |

### 14.3 The glossary

Every translation run writes `ordbok.nbma.toml` at the target repo root.
`--record` adds to it; reruns read it first so names stay stable. If two
originals would get the same name, the later one gets an Arabic ordinal:

```console
$ nbma ident user_names --role var
namn_ad_den_brukaren
```

(`navn` is neuter with a zero plural, so `user_names` collides with
`user_name`; recorded in a glossary it becomes `namn_ad_den_brukaren_thaanii`,
"the second".)

Before renaming anything, run `nbma glossary check` on the original repo. It
proves that every translated name and text block is unique and absent from
the original, which is what lets `nbma untranslate` reverse the whole
translation exactly. `untranslate` itself refuses a glossary in which two
originals share a translation, never renames a file over an existing one or
outside the repo, and writes each file atomically.

An identifier made only of articles (`a_b`, `is_a`) keeps the letters; one
with a dangling `of` (`user_of`) is refused rather than translated to
nothing.

### 14.4 Unknown words

A word missing from the lexicon is an error (exit code 3):

```console
$ nbma ident get_widget --role fn
nbma: not in the lexicon: widget. Propose a lemma with `nbma propose`.
```

The agent may propose a Norwegian lemma with `nbma propose`, which records
it under `[proposed]` in the glossary and rejects root collisions. The word
is then inflected by the CLI like any other.

### 14.5 Prose

`nbma prose` handles simple sentences: subject–verb–object with optional
prepositional phrases and adverbs, `is` with an adjective or noun, `has`,
`does not`, yes/no questions and numbers. Anything else is refused rather
than guessed; rewrite the sentence more simply.

## 15. Arabic script (phase 2, opt-in)

`nbma fold --script arabic` renders prose mode in Arabic script, right to
left, for comments and docs.

```console
$ nbma fold "fa yusakrivur ad-den-brukaren fil" --script arabic
فَ يُسَكرِڤُر الدّېن‌برُكَرېن فِل
```

- The article is written `ال`, as Arabic writes it whatever the
  pronunciation; a sun letter after it takes a shadda.
- Doubled consonants take a shadda.
- Other morpheme boundaries become a zero-width non-joiner.
- `p`, `g` and `v` use the Persian `پ`, `گ` and the `ڤ` that Arabic uses
  for foreign v.

Arabic writes three short vowels as diacritics. Norwegian has nine vowel
letters. The three Arabic recognises (`a i u`) become fatḥa, kasra and
ḍamma; the six it lacks get letters borrowed from orthographies that solved
the problem first:

| Norwegian | Arabic script | Source |
|---|---|---|
| `æ` | ە | Uyghur, where it is /æ/, exactly the Norwegian sound |
| `ø` | ۆ | Uyghur ö (/ø/) |
| `å` | ۏ | a wāw with a mark above: `å` began as an `a` with a small `o` written over it, and wāw is the letter Arabic uses for o-like vowels |
| `e` | ې | Uyghur é |
| `o` | و | Uyghur o |
| `y` | ۈ | Uyghur ü |

At the start of a word, `a i u` sit on an alif and the borrowed vowels on
the Uyghur seat `ئ`:

```console
$ nbma fold "æ ø å" --script arabic
ئە ئۆ ئۏ
```

Identifiers in Arabic script need `--i-understand-bidi`. Without it, the
command refuses; with it, it prints a warning about bidirectional text in
source code (Trojan Source, CVE-2021-42574) and the tooling it breaks.

## Appendix: lexicon format

```toml
[user]
pos = "noun"
bm = "bruker"
nn = "brukar"
gender = "m"
bm_forms = ["brukeren", "brukere", "brukerne"]   # def sg, indef pl, def pl
nn_forms = ["brukaren", "brukarar", "brukarane"]
verified = true
source = "ordbokene.no"

[write]
pos = "verb"
bm = "skrive"
nn = "skrive"
verified = true

["list#noun"]   # second sense of an English word
pos = "noun"
```

`form = n` on a verb selects the Arabic form (§8.4). `alt` lists English
aliases (`err`, `cfg`), `en_pl` and `en_pp` irregular English plurals and
participles. Particles have `ar` instead of `bm`/`nn`.
