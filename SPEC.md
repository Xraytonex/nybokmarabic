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

Passive verbs (§17.4) use the Arabic internal passive. The present takes the
Norwegian s-passive, `-s` in Bokmål and `-st` in Nynorsk; the past, like
the active past, takes nothing.

| Form | Passive past (3sg m) | Passive present (3sg m) |
|---|---|---|
| I | 1u2i3a | yu12a3u + s/st |
| II | 1u22i3a | yu1a22a3u + s/st |
| III | 1ū2i3a | yu1ā2a3u + s/st |
| IV | u12i3a | yu12a3u + s/st |
| V | tu1u22i3a | yuta1a22a3u + s/st |
| VI | tu1ū2i3a | yuta1ā2a3u + s/st |
| VII | un1u2i3a | yun1a2a3u + s/st |
| VIII | u1tu2i3a | yu1ta2a3u + s/st |
| X | ustu12i3a | yusta12a3u + s/st |
| Q-I | 1u23i4a | yu1a23a4u + s/st |
| Q-II | tu1u23i4a | yuta1a23a4u + s/st |

Forms I and IV share a passive present, as they do in Arabic.

The patterns are standard Arabic (Form I uses the u-class vowels). Two
simplifications: Arabic Form I verbal nouns are unpredictable and
Nybokmarabic always uses `faʿl`; and for every form except I the noun of
place has the passive participle's shape, as in Arabic. In Form VIII the
infixed `t` assimilates after `d`, `z`, `th`, `dh`, the emphatics and weak
`w`/`y`, as in Arabic. Form IX is skipped because it is reserved for colours
and bodily defects.

```console
$ nbma inflect hente
root             H-N-T
past             hanata
present          yahnutur
imperative       uhnut
doer             hānit
passive          mahnūt
passive-past     hunita
passive-present  yuhnatus
verbal-noun      hant
place            mahnat
$ nbma inflect skrive
root             S-K-R-V
past             sakrava
present          yusakrivur
imperative       sakriv
doer             musakriv
passive          musakrav
passive-past     sukriva
passive-present  yusakravust
verbal-noun      sakrava
place            musakrav
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
ok: 1155 entries, 3 unverified, no collisions
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
   and `-er` forms are recognised). Abbreviations, acronyms and closed
   compounds are read first as the words they stand for (§16).
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
nbma: not in the lexicon: widget. Propose a lemma, abbreviation or acronym with `nbma propose`.
```

The agent may propose a Norwegian lemma with `nbma propose`, which records
it under `[proposed]` in the glossary and rejects root collisions. The word
is then inflected by the CLI like any other. Abbreviations and acronyms are
proposed the same way (§16.4).

### 14.5 Prose

`nbma prose` handles simple sentences: subject–verb–object with optional
prepositional phrases and adverbs, `is` with an adjective or noun, `has`,
`does not`, yes/no questions and numbers, plus commands, docstrings,
if/when clauses, relative clauses, passives, demonstratives and headings
(§17), and participles, verbal nouns, modals, the perfect, possessives,
lists and `that`/`whether`/`because`/`before`/`after`/`until` clauses (§18).
Anything else is refused rather than guessed; rewrite the sentence more
simply.

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

## 16. Abbreviations, acronyms and compounds

Code is full of words no dictionary lists: `fh`, `kwargs`, `HTTP`,
`filename`, `rmtree`. Before translating, the CLI reads each word it does not
know as the English words it stands for, and translates those like any other
words. Words it knows are never re-read. `--json` names the original word
of each reading under `from`, and how it was read under `via`.

### 16.1 Abbreviations

`lexicon/shorthand.toml` maps an abbreviation to English words that are
themselves in the lexicon, so the Norwegian still comes from verified
entries. A final `-s` makes the last word plural.

```console
$ nbma ident fh --role var
hank_ad_den_fila
$ nbma ident kwargs --role var
argument_ad_det_noekkelordet
```

`argument` is neuter in Nynorsk, which has no plural ending for it, so
"keyword arguments" and "keyword argument" come out the same. An
abbreviation that is only another name for one entry (`err`, `cfg`, `idx`)
is an `alt` name in the lexicon instead (Appendix).

### 16.2 Acronyms

Acronyms are kept as written and inflected as neuter nouns. Norwegian puts
an abbreviation's endings after a hyphen (`CRC-et`, `CRC-er`), and so does
Nybokmarabic, Arabic dual included. The parity rule counts the acronym's
letters: `JSON` has four, so it is Nynorsk.

```console
$ nbma ident HTTPServer --role type
TenarAdDetHttpEt
$ nbma prose "The server sends the JSON."
Fa yasnudur ad-den-tenaren ad-det-JSON-et.
$ nbma prose "The server sends two CRCs."
Fa yasnudur ad-den-tenaren CRC-ān.
```

`API` and `URL` are not acronyms here: the lexicon has them as
`grensesnitt` and `nettadresse`.

### 16.3 Compounds

A word that is neither is split into at most three known words. The split
with the fewest pieces wins; on a tie, the one whose shortest piece is
longest, then the one with the longer first piece. Pieces are two letters or
more, and a two-letter piece must be an abbreviation (`rm`), an acronym
(`io`), a noun (`id`) or one of `in`, `is`, `no`, `on`, `to`, `up`.
A piece of three or four letters must be a word as written or a plain
plural, not a guessed participle or doer (`ser` is not "someone who sees").
Articles, forms of `be` and the pronouns `her`, `him`, `them` and `you` are
never pieces, so `together` is never "to get her". A two-letter acronym
follows only a piece of four letters or more (`asyncio`) or starts the word
(`ipaddr`), so `strip` is not str + IP.

Splitting is a guess. A real English word the lexicon lacks can split
wrongly: `sunset` reads as sun + set. `--json` marks every split with
`"via": "compound"`; if one is wrong, propose the whole word (§14.4), and
it is read as itself from then on.

```console
$ nbma ident filename --role var
namn_ad_den_fila
$ nbma ident rmtree --role fn
fajrin_ad_det_treet
$ nbma ident isdir --role bool
hal_katalog
```

English puts a verb's particle first in a closed compound; Norwegian puts it
after the verb (`laste ned`), so an adverb and a verb swap:

```console
$ nbma ident download_file --role fn
ulsut_ned_ad_den_fila
```

### 16.4 Proposing shorthand

An unknown abbreviation or acronym is proposed like a lemma (§14.4) and
recorded under `[proposed]` in the glossary:

```bash
nbma propose frob --abbrev "file handle" --glossary ordbok.nbma.toml
nbma propose MCP --acronym --glossary ordbok.nbma.toml
```

`nbma lexicon check` and `nbma propose` refuse an abbreviation that expands
to a word the lexicon lacks, and an abbreviation or acronym that hides a
lexicon word: the word would always be read first, so the shorthand could
never be used.

## 17. Commands, conditions, relative clauses and passives

Comments and docstrings are mostly not statements. They give orders
("Return the user"), describe ("Returns the user"), set conditions and
name things. This section covers those shapes.

### 17.1 Commands and docstrings

A command uses the imperative, after the same `fa` as every main clause.
Norwegian puts the imperative first anyway, so `fa` costs it nothing.

```console
$ nbma prose "Return the user."
Fa ratrin ad-den-brukaren.
$ nbma prose "Read and return a line from the stream."
Fa ulsuw wa ratrin linje min ad-den-strømmen.
```

A negative command cannot take `mā` in Classical Arabic, which says `lā
taktub` with a different mood. Egyptian Arabic negates a command exactly as
it negates a statement: `ma-tiktib-š`, "don't write", the second-person
present inside the same circumfix. Nybokmarabic follows Egypt and so keeps
one negation for everything. A negated `or` becomes `and not`, as in Arabic.

```console
$ nbma prose "Do not read the file."
Fa mā talsuwur ikkje ad-den-fila.
$ nbma prose "Do not read or write the file."
Fa mā talsuwur ikkje wa mā tusakrivur ikkje ad-den-fila.
```

A docstring's "Returns …" has an unspoken subject, the function. Arabic drops
subject pronouns, and Norwegian docstrings drop them too ("Returnerer
brukaren."), so the verb is third person with no subject word. The
function is `it`, so the verb is feminine (§9).

```console
$ nbma prose "Returns the user."
Fa turatrinur ad-den-brukaren.
```

A sentence that starts with a bare verb is read as a command first, even if
the word could be a noun (`set`, `log`, `check`). A plural noun first is a
subject:

```console
$ nbma prose "Set the debug output level."
Fa ustut nivå utdata ad-den-valsen.
$ nbma prose "Logs record events."
Fa talgugur hendingar ad-den-posten.
```

### 17.2 if, unless, when, while

`if` is `idhā`, `unless` is `illā idhā`, `when` is `ʿindamā` and `while` is
`baynamā`. Arabic follows `idhā` with the perfect, whatever the English
tense, so "if the user writes" is "if the user wrote".

Norwegian subordinate clauses put the subject before the verb and the
negation before the verb (`hvis brukaren ikkje skriv`). Arabic allows a
noun before the verb after `idhā`, so subordinate clauses take the Norwegian
order and the circumfix collapses into `mā ikkje`, as in a verbless clause.
With the subject first, Arabic also wants the verb to agree in number
(`yusakrivūnar`, "they write"), which main clauses do not (§9).

```console
$ nbma prose "If the user writes a file, the file is valid."
Idhā ad-den-brukaren sakrava fil, fa ad-den-fila gyldig.
$ nbma prose "If the user is not valid, do not read the file."
Idhā ad-den-brukaren mā ikkje gyldig, fa mā talsuwur ikkje ad-den-fila.
$ nbma prose "When the users write files, we read the files."
ʿIndamā ad-dei-brukarane yusakrivūnar filer, fa nalsuwur vi ad-de-filene.
$ nbma prose "Return the user unless the name is valid."
Fa ratrin ad-den-brukaren illā idhā ad-det-namnet gyldig.
```

After a fronted condition, both languages already agree. Arabic introduces
the main clause with `fa` ("then"), and Norwegian with `så`. That is the
word §9 dressed `fa` up as, so nothing about it is a compromise. Put a
comma after a fronted condition; the CLI needs it to find the main clause.

### 17.3 Relative clauses

After a definite noun Arabic uses a relative pronoun that agrees with the
noun: `alladhī` (masculine), `allatī` (feminine and, as everywhere,
neuter), `alladhīna`, `allātī` in the plural and `alladhāni`, `allatāni` in
the dual. After an indefinite noun Arabic forbids a relative pronoun, and
Norwegian requires one, so Norwegian `som` fills the gap.

When the noun is the object of the relative clause, Arabic repeats it as a
pronoun suffix on the verb ("the file that the user writes it").

```console
$ nbma prose "The user that writes the file is valid."
Fa ad-den-brukaren alladhī yusakrivur ad-den-fila gyldig.
$ nbma prose "Read the file that the user writes."
Fa ulsuw ad-den-fila allatī ad-den-brukaren yusakrivur-hā.
$ nbma prose "A user that reads files writes names."
Fa yusakrivur brukar som yalsuwur filer namn.
```

A relative clause with `is` or `has` would need an Arabic subject pronoun
inside it, which Nybokmarabic does not have. Put the adjective before the
noun instead: "the empty file", not "the file that is empty".

### 17.4 Passives

`was` or `were` with a past participle, `is being` with one, or `is` with
one and `by` make a passive verb. These use the Arabic internal passive
(`fuʿila`, `yufʿalu`, and the matching pattern in every form). The
dictionaries give Norwegian an s-passive only in the present (`skrives`,
`skrivast`), so the present passive ends in `-s` or `-st` by the parity
rule, in the place where the active present ends in `-r`. The past is bare,
as in the active. Classical Arabic never names who did the action;
Modern Standard Arabic says `min qibali`, "from the side of", and so does
Nybokmarabic.

```console
$ nbma prose "The file was deleted."
Fa sultitat ad-den-fila.
$ nbma prose "The file is read by the user."
Fa tulsawust ad-den-fila min qibali ad-den-brukaren.
$ nbma inflect lese --form passive-present
yulsawust
$ nbma inflect skrive --form passive-present
yusakravust
$ nbma inflect hente --form passive-past
hunita
```

`is` with a participle and no `by` is a state, not an event, and stays the
participle of §6:

```console
$ nbma prose "The file is closed."
Fa ad-den-fila malkūk.
```

An exception's docstring ("Raised when …") is a passive with the subject
dropped, as in §17.1:

```console
$ nbma prose "Raised when the file is closed."
Fa tuksatus ʿindamā ad-den-fila malkūk.
```

### 17.5 Demonstratives

`this`, `these`, `that` and `those` are Arabic, before the definite noun as
in Arabic (`hādhā l-kitāb`), so the noun keeps all three articles. On a
possessor the demonstrative goes inside the construct, before the
possessor.

```console
$ nbma prose "These files are valid."
Fa hāʾulāʾi ad-de-filene gyldige.
$ nbma prose "Return the fields of this class."
Fa ratrin felt hādhihi ad-den-klassa.
$ nbma prose "This is the default."
Fa hādhā as-standard.
```

### 17.6 Headings

A class docstring is often just a noun phrase. It stays one: no verb, so no
`fa`.

```console
$ nbma prose "Exception for empty headers."
Unntak min ajli hovud tomme.
$ nbma prose "True if the connection is closed."
Sann idhā ad-den-tilkoplinga malkūk.
```

## 18. Participles, verbal nouns, modals and more clauses

Docstrings describe things with participles ("the object wrapped by the
function"), say how with verbal nouns ("by reading the file") and hedge with
modals ("can be overridden"). This section covers those shapes.

### 18.1 Participles after a noun

English puts a participle phrase after the noun it describes. Arabic puts
every adjective there, and its participles are adjectives: the passive
participle for `-ed`, the active participle for `-ing`. So the participle
follows the noun, definite when the noun is (§6), with its agent (`min
qibali`, §17.4) or its object after it.

```console
$ nbma prose "Return the object wrapped by the function."
Fa ratrin ad-det-objektet al-mapkūke min qibali ad-den-funksjonen.
$ nbma prose "Update the CRC using the given data."
Fa papdit ad-det-CRC-et al-bārike ad-den-dataen al-magjūwe.
```

Before the verb, "the file deleted" could be a subject and its past tense.
Only `by` makes the participle part of the subject:

```console
$ nbma prose "The file deleted by the user is empty."
Fa ad-den-fila al-musaltate min qibali ad-den-brukaren tom.
```

### 18.2 Gerunds and infinitives

Arabic has neither an infinitive nor a gerund. For both it uses the verbal
noun (the maṣdar), which takes its object the way a noun takes a possessor:
"reading the file" is "the reading of the file", an iḍāfa (§5) whose first
noun has no article.

After `by`, a verbal noun takes Arabic `bi-`, "by means of"; `by` on its own
stays `ḥasaba`, "according to". `to` before a verb is Arabic `li-`, "for",
which is also where Norwegian puts its `for å`. Both are one-letter Arabic
prepositions and are written onto the verbal noun. `to be` with a past
participle takes the same verbal noun, because Arabic verbal nouns have no
voice.

```console
$ nbma prose "Check the number by reading the file."
Fa ushkuk ad-det-talet bi-lasw ad-den-fila.
$ nbma prose "Read the file to return the user."
Fa ulsuw ad-den-fila li-ratrana ad-den-brukaren.
$ nbma prose "Return the class to be used."
Fa ratrin ad-den-klassa li-bark.
```

Any other preposition takes the verbal noun as it is. `when` and `while`
with an `-ing` word become the prepositions `ʿinda`, "at", and `khilāla`,
"during". A word that is also a noun (`to file`) is read as a verb after
`to` only when an object follows it.

```console
$ nbma prose "Open the file for reading or writing."
Fa upnuw ad-den-fila min ajli lasw aw sakrava.
```

### 18.3 Modals

The lexicon has `can`, `should` and `must` as Norwegian verbs (`kunne`,
`skulle`, `måtte`), which conjugate like any other verb. The verb after a
modal is Arabic `an` with the subjunctive (`yaqraʾa`, with `-a` where the
indicative has `-u`). It has no Norwegian `-r`, since Norwegian has a bare
infinitive after a modal (`kan lese`). The modal comes before the subject and
agrees in gender only (§9). The second verb comes after the subject and
agrees in number too.

```console
$ nbma prose "The user can read the file."
Fa yaknunur ad-den-brukaren an yalsuwa ad-den-fila.
$ nbma prose "The users can not read the files."
Fa mā yaknunur ikkje ad-dei-brukarane an yalsuwū ad-de-filene.
```

`will` is Arabic's future prefix `sa-` on the indicative. `will not` is
Arabic `lan` with the subjunctive, which turns the circumfix into `lan …
ikkje`. `may` and `might` are `qad` with the indicative, "perhaps". None of
these leaves a Norwegian verb in the sentence, but Norwegian would have an
infinitive after `vil` and `kan`, so there is no `-r` either. `may not` is
refused, because it can mean either "might not" or "must not".

```console
$ nbma prose "The file will be deleted."
Fa sa-tusaltatust ad-den-fila.
$ nbma prose "The user will not write the file."
Fa lan yusakriva ikkje ad-den-brukaren ad-den-fila.
```

A passive after a modal keeps its `-s`, because the Norwegian s-passive
doubles as the passive infinitive (`kan skrivast`). Anything else after `be`
needs a verb: Arabic drops its copula in the present (§11), but after a
modal it must say `an yakūna`, "that it be". The Nybokmarabic copula is
Norwegian `være`, which the parity rule makes Nynorsk `vere`, conjugated on
the root V-R-W.

```console
$ nbma prose "The file should be empty."
Fa tusaklilur ad-den-fila an tavruwa tom.
$ nbma prose "The file may be empty."
Fa qad tavruwu ad-den-fila tom.
```

### 18.4 The perfect and the progressive

Arabic marks a completed action with `qad` before the past tense, and
Norwegian with `har`. Arabic's past already carries the tense, so `qad`
stays and `har` has nothing left to do. Arabic never negates `qad`, so a
negated perfect drops it.

```console
$ nbma prose "The user has written the file."
Fa qad sakrava ad-den-brukaren ad-den-fila.
$ nbma prose "The file has not been deleted."
Fa mā sultitat ikkje ad-den-fila.
```

Neither language has a progressive, so `is writing` is the plain present:

```console
$ nbma prose "The user is writing a file."
Fa yusakrivur ad-den-brukaren fil.
```

### 18.5 because, before, after, until

`because` is `li-anna`. Arabic puts the subject straight after it, as
Norwegian does after every conjunction (§17.2). `before`, `after` and
`until` are prepositions (§1.1) that can also take a whole clause. Before a
clause, `before` is `qabla an` with the subjunctive, `after` is `baʿda an`
with the perfect (as `idhā` takes the perfect), and `until` is `ḥattā`.

In a clause after `before`, the Arabic subjunctive and the Norwegian present
share a verb. The mood slot takes Arabic `-a` and the tense slot takes
Norwegian `-r`, giving `yusakrivar`.

```console
$ nbma prose "Return the user because the file is empty."
Fa ratrin ad-den-brukaren li-anna ad-den-fila tom.
$ nbma prose "Read the file before the user writes it."
Fa ulsuw ad-den-fila qabla an ad-den-brukaren yusakrivar-hā det.
$ nbma prose "After the user wrote the file, return the user."
Baʿda an ad-den-brukaren sakrava ad-den-fila, fa ratrin ad-den-brukaren.
$ nbma prose "The tests pass before the translation."
Fa yabsutur ad-dei-testane qabla ad-den-omsetjinga.
```

### 18.6 that and whether

`check that …` is Arabic `anna`. `check whether …` is `hal`, the question
particle of §11, which Arabic also uses for indirect questions. Both take the
subject first, as every conjunction does.

```console
$ nbma prose "Check that the file is empty."
Fa ushkuk anna ad-den-fila tom.
$ nbma prose "Check whether the file is empty."
Fa ushkuk hal ad-den-fila tom.
```

### 18.7 there is, no

Norwegian says `det er`, with a dummy subject. Arabic says `hunāka`,
"there", with no verb at all, so the null copula leaves Norwegian's `er`
nothing to do. `no file` becomes `ayy fil`, "any file", inside a negated
clause, which is how Arabic says it. After a preposition there is no clause
to negate, so `with no name` is refused; write `without`.

```console
$ nbma prose "There is no file."
Fa mā ikkje hunāka ayy fil.
$ nbma prose "No user writes the file."
Fa mā yusakrivur ikkje ayy brukar ad-den-fila.
```

### 18.8 Possessives

`the user's name` is the iḍāfa `the name of the user` (§5). A demonstrative
on the possessor stays with it.

```console
$ nbma prose "Return this user's name."
Fa ratrin namn hādhā ad-den-brukaren.
```

A possessive pronoun is an Arabic suffix on the noun. Arabic attaches it and
drops the article. Norwegian writes its possessive as a word after a noun in
the definite form (`namnet mitt`). The suffix sits where the Norwegian word
would, so the noun keeps only its Norwegian definite ending. `her` is left
out, because it is also the object pronoun.

```console
$ nbma prose "Return the name of its file."
Fa ratrin namn fila-hā.
$ nbma prose "Return their full names."
Fa ratrin namna-hum al-fulle.
```

### 18.9 Lists, commas, so and then

Arabic puts `wa` or `aw` before every item of a list and needs no commas, so
Nybokmarabic does the same. A subject made of several nouns is plural.

```console
$ nbma prose "Read the file, the name and the user."
Fa ulsuw ad-den-fila wa ad-det-namnet wa ad-den-brukaren.
$ nbma prose "The user and the file are valid."
Fa ad-den-brukaren wa ad-den-fila gyldige.
```

After a comma, a participle that describes the whole clause is Arabic's
circumstantial participle (the ḥāl), which is indefinite. A prepositional
phrase can come before the main clause with a comma, like a condition (§17.2).
`fa` then keeps the verb in Norwegian second place, as it does after `idhā`.
`so` between two clauses and `then` after a condition are `fa`, which means
both already.

```console
$ nbma prose "Read the file, returning the user."
Fa ulsuw ad-den-fila, muratrin ad-den-brukaren.
$ nbma prose "In the stream, the file is empty."
Fī ad-den-strømmen, fa ad-den-fila tom.
$ nbma prose "The file is empty, so return the user."
Fa ad-den-fila tom, fa ratrin ad-den-brukaren.
```

### 18.10 Adverbs in -ly

Norwegian makes an adverb from the neuter of an adjective (`rekursiv`,
`rekursivt`). An English `-ly` adverb is therefore the neuter of its
adjective, placed with the other adverbs, last.

```console
$ nbma prose "Read the file recursively."
Fa ulsuw ad-den-fila rekursivt.
```

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

Abbreviations and acronyms live in `lexicon/shorthand.toml` (§16):

```toml
[abbreviations]
fh = "file handle"

[acronyms]
words = ["HTTP", "JSON", "gzip"]
```

In a glossary they are `[proposed]` entries with `pos = "abbrev"` and
`expands = "…"`, or `pos = "acronym"` and `written = "…"`.
