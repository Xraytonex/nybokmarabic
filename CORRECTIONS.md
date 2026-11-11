# Corrections

The design brief for Nybokmarabic was drafted by someone who speaks none of
its three source languages. Before the spec was finalised, every claim it
made about Bokmål, Nynorsk and Arabic was checked:

- Norwegian forms against Bokmålsordboka and Nynorskordboka (ordbokene.no),
  by `tools/verify_lexicon.py`, which copies lemma, gender, definite form
  and plural from the dictionaries' own data.
- Arabic morphology against standard references (Wikipedia's Arabic grammar,
  verb and noun articles, which follow Wright's *Grammar of the Arabic
  Language*; Wiktionary root entries).
- Norwegian language history against Store norske leksikon (snl.no).

The policy: where the brief misdescribed a source language, the description
was fixed and the Nybokmarabic rule kept, unless the rule only made sense
because of the error. The language should be wrong on purpose, never by
accident. Every change is listed here.

## Changes to rules

### 1. `skrive` is quadriliteral, not S-K-R

The brief's worked example derived S-K-R from `skrive` and conjugated it as a
triliteral verb (`sakara`, `yaskurur`, `uskur`, `sākir`, `maskūr`, `maskar`).
By the brief's own rule ("keep consonants in order … four or more: keep the
first four"), `skriv-` has four consonants, s-k-r-v. The CLI is canonical, so
the root is S-K-R-V and the verb is quadriliteral:

```console
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

This also cost the brief its favourite collision; see COLLISIONS.md.

### 2. The noun of place for quadriliteral verbs

The brief said quadriliteral verbs have no noun of place ("use verbal
noun"). In Arabic, every form except Form I builds its noun of place on the
passive participle's shape (`mufaʿlal`). The rule existed only because of the
error, so Nybokmarabic now follows Arabic: `musakrav` is both "written" and
"place of writing".

### 3. Present-tense prefixes for Forms II, III, IV and Q-I

The brief gave the present prefixes as `a- ta- ya- na-`. That is right for
Form I (and V–X), but Forms II, III, IV and the quadriliteral Q-I take the
vowel `u`: `u- tu- yu- nu-`. The brief's own quadriliteral template already
said `yu-`. Nybokmarabic now uses the correct vowel for each form.

### 4. Plurals

The brief described the plural as "the Norwegian plural suffix chosen by the
parity rule (`-er` or `-ar`)". That is not how either standard works.
Nynorsk uses `-ar` for masculines, `-er` for feminines and no ending for most
neuters; Bokmål uses `-er` for masculines and feminines and either nothing or
`-er` for neuters, with irregular plurals in both (`bøker`, `nøkler`).

The rule (the parity rule decides) is kept, and now applies to the whole
plural: Nybokmarabic uses the plural form the selected dictionary gives.

### 5. The vocative is `yā`, so the code prefix is `yaa_`

The brief wrote the vocative particle as `ya_` (`ʿalā` → `ya_3alaa`). The
particle is `yā` with a long vowel, which folds to `yaa`, so the prefix is
`yaa_` (`yaa_3alaa`). Arabic chat writes it `ya`, but code mode folds `ā`
to `aa` everywhere else, and the vocative gets no exemption.

### 6. The language's own name

The brief gave the name as `ad-det-nybokmarabisket`, treating the language
name as a neuter noun with the definite suffix `-et`. Ordbokene lists
language names (`norsk`) either as masculine nouns (`norsken`) or as
uninflected neuters with no definite form at all. An uninflected neuter
cannot carry the suffixed article that triple definiteness needs, so the
masculine is used: `ad-den-nybokmarabisken`.

### 7. The example sentence

The brief translated "the user writes a file" as
`fa yaskurur ad-den-brukaren maskar`, using the noun of place `maskar` for
"file". With the lexicon's word for file and the corrected root:

```console
$ nbma prose "The user writes a file."
fa yusakrivur ad-den-brukaren fil.
```

### 8. More than two colliding verbs

The brief resolved a root collision by moving the later verb to Form II. A
root can be shared by three verbs (`lese`, `låse`, `løyse` all give L-S-W),
so collisions continue through Arabic's derived forms in order: II, III, IV,
V, VI, VII, VIII, X (IX is skipped: Arabic reserves it for colours and
bodily defects). Quadriliteral roots have Q-I and Q-II.

## Corrected descriptions, rules unchanged

### 9. `fa` and verb-second word order

The brief said that `fa` + verb puts the verb second, so "Norway is
satisfied". A Norwegian grammarian would disagree: coordinating conjunctions
(`og`, `men`, `eller`, `for`) stand outside the clause and do not fill the
first position. The rule is kept. Nybokmarabic treats `fa` ("so, then") as
the equivalent of the adverb `så` ("then"), which does fill the first
position and does trigger inversion (`Så skriv brukaren …`).

### 10. Negation with `mā`

The brief used `mā` before every verb. In Modern Standard Arabic, `mā`
negates mainly the past; the present takes `lā`, the future `lan`, and
`lam` + jussive is the usual past. The rule is kept, deliberately: `mā` with
the present is attested in Classical Arabic and is the normal negator in
many spoken dialects. Before a verbless clause, `mā` is the Classical
"Ḥijāzī mā" (MSA would use `laysa`).

### 11. The Arabic root s-k-r

The brief said S-K-R "appears to be the Arabic root for drunkenness". It is:
`sakira` "to get drunk", `sakrān` "drunk". The tempting `sukkar` "sugar" is
not from the root; it is a loanword (Middle Persian *šakar*, ultimately
Sanskrit *śarkarā*).

### 12. The dual ending

The brief gave the dual as `-ān`. Classical Arabic has `-āni` (nominative)
and `-ayni` (oblique), shortened before a possessor; `-ān` is the pausal
form, the one you hear at the end of a phrase. Nybokmarabic keeps the pausal
form everywhere.

### 13. The article in Arabic script

The brief said `al-` "never actually surfaces as `al-`". In transliteration
that is right. In Arabic script the article is always written `ال` and the
assimilation shows only as a shadda on the next letter. Phase 2 (Arabic
script) writes it that way.

### 14. Proclitics

Arabic writes `wa`, `fa`, `bi-`, `li-` and `ka-` attached to the next word,
and the article's vowel elides after them (`wa-l-kitāb`, `bi-d-dār`). The
brief wrote them as separate words. Nybokmarabic keeps the separate words on
purpose and says so in the spec (§1.1).

### 15. Transliteration of hamza

The brief wrote word-initial vowels without a hamza (`ilā`, `a-` for "I").
That matches ALA-LC; DIN 31635 would write `ʾilā`. Nybokmarabic follows
ALA-LC and now says so.

### 16. Verbal nouns and imperative vowels

The brief's Form I templates (`yafʿulu`, `ufʿul`, verbal noun `faʿl`) are
real, but Arabic Form I verbs come in three vowel classes, and Form I verbal
nouns are unpredictable (`faʿl`, `fuʿūl`, `fiʿāla` …). Nybokmarabic uses the
u-class and `faʿl` for every verb; the spec now states this as a
simplification rather than a fact about Arabic.

## Checked and found correct

- The parity examples (`bruker` → `brukar`, `navn` → `namn`, `ikke` →
  `ikkje`).
- Norwegian double definiteness (`det store huset`) and the definite forms
  `huset`, `brukaren`.
- The sun letters, including `d`, and the assimilation `ad-d…`.
- The Form I and quadriliteral Q-I templates, and the past-tense person
  suffixes `-tu -ta -a -at -nā -ū`.
- `hal` for yes/no questions, the absence of a present-tense copula, the
  construct state (head bare, only the last noun definite, adjectives after
  the whole construct).
- Bokmål optional feminine versus obligatory Nynorsk feminine.
- Units before tens in Arabic and in old Norwegian counting (`to og tjue`).
  For the record: the Storting voted for the new counting in 1950 and it
  took effect in 1951.
- Arabic chat-alphabet digits: 2 = ʾ, 3 = ʿ, 7 = ḥ (5 = kh, 6 = ṭ, 9 = ṣ
  vary more by region; Nybokmarabic uses 6 and 9).
- Arabic counted-noun agreement (3–10 plural, 11–99 singular) and the dual
  hundreds `miʾatān`, which Nybokmarabic adopts.
- Samnorsk: the official goal of bringing Bokmål and Nynorsk together dates
  from the 1917 and 1938 spelling reforms; the Storting removed it from the
  law in 2002.
- Trojan Source is CVE-2021-42574.
