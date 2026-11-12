# Collisions with real Arabic roots

Nybokmarabic builds verb roots from Norwegian consonants. Arabic has a few
thousand roots, so some Nybokmarabic roots are also real Arabic ones. These
collisions are a feature: nothing was changed to avoid them, and nothing was
changed to create them.

## Method

Each triliteral root in the lexicon was read with the plain letter mapping
(b ب, d د, f ف, h ه, j/y ي, k ك, l ل, m م, n ن, r ر, s س, t ت, w و, sh ش,
kh خ, ʾ ء). A root counts as a collision if Wiktionary has a category
"Arabic terms belonging to the root …" for it, e.g.
<https://en.wiktionary.org/wiki/Category:Arabic_terms_belonging_to_the_root_ش_ك_ك>.
Weak final radicals (w and y) are interchangeable in Arabic, so both were
tried.

The check covered the 582-entry lexicon of the first release. Verbs added
since then have not been checked yet, so there are more collisions than
this page lists.

A category on Wiktionary proves the root is attested, not that it is
common. Meanings below are the root's core sense, summarised; check a
dictionary (Lane's Lexicon, Hans Wehr) before quoting one. Quadriliteral
roots were not checked: Arabic has far fewer of them.

The brief's headline collision, `skrive` = S-K-R ("drunkenness"), did not
survive: by the brief's own extraction rule `skrive` is S-K-R-V
(CORRECTIONS.md §1). S-K-R is a real root (`sakira` "to get drunk"), but no
Nybokmarabic verb currently has it.

## Exact matches

| Root | Norwegian verb | Arabic root | Example |
|---|---|---|---|
| SH-K-K | `sjekke` check | to doubt | `shakk` doubt |
| T-L-F | `tilføye` add | to perish, be ruined | `talaf` damage |
| T-M-M | `tømme` empty, clear | to be complete | `tamām` completeness, "OK" |
| R-M-M | `rømme` escape | to repair, restore | `tarmīm` restoration |
| ʾ-W-Y | `eie` own | to take shelter | `maʾwā` shelter |
| S-T-R | `sitere` quote, `styre` manage | to cover, conceal | `sitār` curtain |
| K-N-N | `kunne` can | to hide, keep safe | `maknūn` hidden |
| S-KH-W | `søkje` search | to be generous | `sakhāʾ` generosity |
| SH-W-Y | `sjå` see | to grill, roast | `shiwāʾ` grilled meat |
| L-S-N | `løsne` loosen, detach | tongue, language | `lisān` tongue |
| F-N-N | `finne` find | art, kind | `fann` art |
| N-D-R | `endre` change | to be rare | `nādir` rare |
| S-W-Y | `seie` say | to be equal, level | `sawwā` to level |
| K-L-L | `kalle` call | totality; weariness | `kull` all |
| K-M-M | `komme` come | to cover, enclose | `kumm` sleeve |
| T-L-L | `telle` count | mound | `tall` hill |
| S-N-D | `sende` send | to lean on, support | `sanad` support, document |
| F-L-L | `fylle` fill | to dent, notch | `fall` dent |
| R-B-D | `arbeide` work | to be ash-coloured | `rabad` ashen colour |
| B-R-K | `bruke` use | (of a camel) to kneel | `baraka` to kneel |
| B-T-L | `betale` pay | to cut off, sever | `batala` to sever |

Highlights:

- To **check** is to **doubt** (SH-K-K).
- To **add** is to **ruin** (T-L-F).
- To **empty** something is to **complete** it (T-M-M).
- To **escape** is to **repair** (R-M-M). Norwegian `rømme` also means sour
  cream, which this does not explain.
- `eie` ("own") has no consonants at all, so Nybokmarabic gives it a hamza
  and pads it to ʾ-W-Y, a real root meaning "to take shelter". The one verb
  built entirely out of padding found a home.
- To **search** is to **be generous**, and to **see** is to **grill**.
- `sitere` (quote) and `styre` (manage) collide with each other, which
  Nybokmarabic resolves by putting `styre` in Form II, and with Arabic
  S-T-R "to conceal", which it does not resolve at all.
- `kalle` (call) shares its root with `kull` ("all"), which Nybokmarabic
  already uses as its quantifier: `get_all_users` is `uhnut kull …`.

```console
$ nbma root sjekke
SH-K-K
$ nbma root tilføye
T-L-F
$ nbma root eie
ʾ-W-Y
```

## Near misses

These need a looser mapping: Norwegian `h` read as ḥ, `k` as q, `t` as ṭ,
`d` as ḍ, `s` as ṣ, or the letters Arabic lacks (`g`, `p`, `v`) read as
gh/j, b and f.

| Root | Norwegian verb | Arabic root (looser reading) | Example |
|---|---|---|---|
| D-R-P | `drepe` kill | ḍ-r-b to strike | `ḍarb` a blow |
| G-W-Y | `gå` go | gh-w-y to go astray | `ghāwī` straying |
| K-P-R | `kopiere` copy | k-b-r to be big | `kabīr` big |
| P-S-T | `påstå` assert | b-s-ṭ to spread out; simple | `basīṭ` simple |
| K-B-L | `koble` connect | q-b-l to accept, to face | `qabila` to accept |
| H-K-K | `hakke` hash | ḥ-k-k to rub, scratch | `iḥtikāk` friction |
| S-K-V | `skyve` push | s-q-f roof | `saqf` roof |
| N-K-T | `nekte` deny | n-q-ṭ dot, point | `nuqṭa` dot |
| L-G-W | `lage` make | l-gh-w to speak idly; language | `lugha` language |
| P-N-W | `opne` open | b-n-y to build | `banā` to build |
| T-W-Y | `ta` take | ṭ-w-y to fold | `ṭawā` to fold |
| S-P-R | `spore` track | ṣ-b-r patience | `ṣabr` patience |
| V-K-K | `vekke` wake | f-k-k to untie | `fakk` untying |
| S-V-R | `svare` answer | ṣ-f-r zero; s-f-r travel | `ṣifr` zero |

## Not Arabic roots

No Wiktionary root category exists, under any of the readings above, for:
B-N-D, B-R-T, D-K-D, D-M-P, F-N-G, F-R-N, F-S-T, G-D-T, H-N-G, K-K-K, K-L-N,
KH-R-W, L-G-R, L-K-K, L-N-K, L-R-W, L-S-T, L-S-W, P-P-P, P-R-V, S-L-G, T-G-N,
T-N-K, T-S-T, V-L-R, V-N-T, V-V-S. In particular L-S-W, shared by `lese`
(read), `låse` (lock) and `løyse` (resolve), is Nybokmarabic's alone.
