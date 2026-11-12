<!-- nbma: inflect nybokmarabisk --pos noun --gender m --form def -->
# ad-den-nybokmarabisken

[English](README.md) · **ad-den-nybokmarabisken**

<!-- Every line of prose below is CLI output. The comment above each one
     holds the simplified English it was translated from, as AGENTS.md
     asks of agents; `cargo test` re-runs every one. -->

<!-- nbma: prose "The language merges three languages. The committee writes the rules. Every rule resolves a conflict." -->
Fa tatafaltatur ad-det-språket tre språk. Fa tusakrivur ad-den-nemnda ad-de-reglene. Fa yulāsiwur kull regel konflikt.

<!-- nbma: prose "The Norwegian language has two spellings. The government wanted one spelling during a hundred years. The parliament wrote the goal into the law in 1938. The parliament removed the goal from the law in 2002. The two spellings did not find a majority." -->
Fa ʿinda ad-det-språket an-norske rettskrivningān. Fa valalat ad-den-regjeringa rettskrivning ei khilāla hundre år. Fa sakravat ad-det-stortinget ad-det-målet ilā ad-den-loven fī tusen wa ni hundre wa åtte wa tretti. Fa fajranat ad-det-stortinget ad-det-målet min ad-den-loven fī tusenān wa to. Fa mā fananat ikkje ad-de-rettskrivningenān fleirtal.

<!-- nbma: prose "This language adds Arabic. A committee with three members always has a majority. Nouns are Norwegian. Verbs are Norwegian roots in Arabic templates. Prepositions are Arabic. Every noun has three articles." -->
Fa tatlufur hādhihi ad-det-språket arabisk. Fa ʿinda nemnd maʿa tre medlemar fleirtal alltid. Fa substantiv norske. Fa verb røtter norske fī maler arabiske. Fa preposisjoner arabiske. Fa ʿinda kull substantiv tre artiklar.

```console
$ nbma prose "The user writes a file."
Fa yusakrivur ad-den-brukaren fil.
```

<!-- nbma: prose "The noun has an Arabic article and a Norwegian determiner and a Norwegian suffix. The verb has an Arabic template and a Norwegian suffix. The first word helps both languages." -->
Fa ʿinda ad-det-substantivet artikkel arabisk wa determinativ norsk wa suffiks norsk. Fa ʿinda ad-det-verbet mal arabisk wa suffiks norsk. Fa tuhajlipur ad-det-ordet al-fyrste språkān.

<!-- nbma: prose "The parts" -->
## Ad-de-delene

<!-- nbma: prose "The specification describes the grammar. The corrections list the errors of the plan. Some roots are real Arabic roots." -->
- Fa yusaklidur ad-den-spesifikasjonen ad-den-grammatikken. Fa tulassitur ad-dei-rettingane feil ad-den-planen. Fa baʿḍ ad-de-røttene røtter reelle arabiske.
  → [SPEC.md](SPEC.md), [CORRECTIONS.md](CORRECTIONS.md), [COLLISIONS.md](COLLISIONS.md)
  <!-- nbma: prose "The tool inflects words. The tool translates names. Every word comes from the tool." -->
- Fa tabwuyur ad-det-verktøyet ord. Fa tumastitur ad-det-verktøyet namn. Fa takmumur kull ord min ad-det-verktøyet.
  <!-- nbma: prose "The instructions give the rules to an agent. The agent translates a project. The behavior of the project does not change. The agent also reverses the translation." -->
- Fa yagjuwur ad-de-instruksjonene ad-de-reglene ilā agent. Fa yumastitur ad-den-agenten prosjekt. Fa mā yandurur ikkje oppførsel ad-det-prosjektet. Fa yasnuwur ad-den-agenten ad-den-omsetjinga òg.
  → [AGENTS.md](AGENTS.md), [SKILL.md](.claude/skills/nybokmarabic/SKILL.md)

<!-- nbma: prose "A short example" -->
## Eksempel kort

```console
$ nbma ident get_user_name --role fn
uhnut_namn_ad_den_brukaren
$ nbma ident is_valid --role bool
hal_gyldig
$ nbma ident on_click --role fn
yaa_3alaa_ad_det_klikket
$ nbma ident READ_ME --role fn
ULSUW_NII_MEG
$ nbma prose "If the file is empty, return the user."
Idhā ad-den-fila tom, fa ratrin ad-den-brukaren.
$ nbma prose "Read the file that the user writes."
Fa ulsuw ad-den-fila allatī ad-den-brukaren yusakrivur-hā.
$ nbma prose "The file was deleted by the user."
Fa sultitat ad-den-fila min qibali ad-den-brukaren.
```

<!-- nbma: prose "The installation" -->
## Ad-den-installasjonen

```bash
cargo install --path .
```

<!-- nbma: prose "The tool does not need the network." -->
Fa mā tutarnigur ikkje ad-det-verktøyet ad-det-nettverket.

<!-- nbma: prose "The translation of a project" -->
## Omsetjing ad-det-prosjektet

<!-- nbma: prose "Give the instructions to the agent. The tool translates every name and every comment. The glossary keeps every translation. The tests pass before the translation. The tests pass after the translation. The tool restores the original project." -->
Fa ugjuw ad-de-instruksjonene ilā ad-den-agenten. Fa tumastitur ad-det-verktøyet kull namn wa kull kommentar. Fa tubahlidur ad-den-ordlista kull omsetjing. Fa yabsutur ad-dei-testane qabla ad-den-omsetjinga. Fa yabsutur ad-dei-testane baʿda ad-den-omsetjinga. Fa tugajnipur ad-det-verktøyet ad-det-prosjektet al-originale.
→ [AGENTS.md](AGENTS.md)

<!-- nbma: prose "The example has a small project in two languages. The tests pass in both languages." -->
Fa ʿinda ad-det-eksempelet prosjekt lite fī språkān. Fa yabsutur ad-dei-testane fī språkān.
→ [examples/](examples/)

<!-- nbma: prose "Real projects have unknown words. The agent adds the unknown words. The agent writes simple sentences." -->
Fa ʿinda prosjekt reelle ord ukjende. Fa yatlufur ad-den-agenten ad-de-orda al-ukjende. Fa yusakrivur ad-den-agenten setninger enkle.

<!-- nbma: prose "The honesty" -->
## Ad-den-ærlegdomen

<!-- nbma: prose "The author does not speak the languages. The dictionary verified every Norwegian word. The author did not write the words from memory. The author checked the Arabic rules against books of grammar. Three entries were not verified." -->
Fa mā yusankikur ikkje ad-den-forfatteren ad-de-språka. Fa varfasat ad-den-ordboka kull ord norsk. Fa mā sakrava ikkje ad-den-forfatteren ad-de-orda min minne. Fa shakaka ad-den-forfatteren ad-de-reglene al-arabiske ḍidda bøker ad-den-grammatikken. Fa mā vurfisat ikkje tre oppføringer.

```console
$ nbma lexicon check
ok: 1155 entries, 3 unverified, no collisions
```

<!-- nbma: prose "The author chose the meanings. Corrections are welcome." -->
Fa valaga ad-den-forfatteren ad-de-betydningene. Fa rettingar velkomne.

<!-- nbma: prose "The language jokes about grammar and about politics. It does not joke about religion or about people." -->
Fa taspukur ad-det-språket ʿan grammatikk wa ʿan politikk. Fa mā taspukur ikkje det ʿan religion aw ʿan personar.

<!-- nbma: prose "The development" -->
## Ad-den-utviklinga

```bash
cargo test
cargo clippy --all-targets
```

<!-- nbma: prose "The tests compare the documents with the tool. The user reads me." -->
Fa yusamminur ad-dei-testane ad-dei-dokumenta maʿa ad-det-verktøyet. Fa yalsuwur-nī ad-den-brukaren meg.

<!-- nbma: prose "The license" -->
## Ad-den-lisensen

<!-- nbma: prose "The license is MIT." -->
Fa ad-den-lisensen MIT.
→ [LICENSE](LICENSE)
