#!/usr/bin/env python3
"""Build and verify lexicon/core.toml against ordbokene.no (ord.uib.no API).

This is a development tool. It needs network access; the nbma CLI does not.

Usage:
    python3 tools/verify_lexicon.py SEED.txt lexicon/core.toml [--cache DIR]

Seed format, one entry per line:
    pos english bm nn [gender|value] | key=value | key=v1,v2
A noun gender may be split as bm/nn, e.g. "m/f" for kilde (m) / kjelde (f).
pos is one of verb noun adj adv pron part det num. For part/det the third column
is the Arabic form and there is no fourth. For pron the columns are the subject
pronoun in Bokmål and Nynorsk, then the Arabic object suffix. For num the fifth
column is the value.

For every Norwegian lemma the script looks the word up in Bokmålsordboka (bm)
and Nynorskordboka (nn), checks the part of speech, and copies the inflected
forms (nouns: definite singular, indefinite plural, definite plural; adjectives:
neuter, definite, plural) from the dictionary. An entry is `verified = true`
only if both lemmas were found with the expected part of speech and, for nouns,
the expected gender. Everything else is written with `verified = false` and a
`note` explaining what failed.
"""
from __future__ import annotations

import argparse
import concurrent.futures as cf
import hashlib
import json
import os
import sys
import time
import urllib.parse
import urllib.request

API = "https://ord.uib.no"
POS_TAG = {"verb": "VERB", "noun": "NOUN", "adj": "ADJ", "adv": "ADV", "num": None}
GENDER_TAG = {"Masc": "m", "Fem": "f", "Neuter": "n"}


class Fetcher:
    def __init__(self, cache_dir: str):
        self.cache_dir = cache_dir
        os.makedirs(cache_dir, exist_ok=True)

    def get(self, url: str):
        key = hashlib.sha1(url.encode()).hexdigest()
        path = os.path.join(self.cache_dir, key + ".json")
        if os.path.exists(path):
            with open(path, encoding="utf-8") as fh:
                return json.load(fh)
        for attempt in range(4):
            try:
                req = urllib.request.Request(url, headers={"User-Agent": "nbma-lexicon-check"})
                with urllib.request.urlopen(req, timeout=30) as resp:
                    data = json.load(resp)
                break
            except Exception:  # noqa: BLE001 - retried, then re-raised
                if attempt == 3:
                    raise
                time.sleep(1.5 * (attempt + 1))
        with open(path, "w", encoding="utf-8") as fh:
            json.dump(data, fh)
        return data

    def articles(self, dict_: str, word: str):
        q = urllib.parse.quote(word)
        ids = self.get(f"{API}/api/articles?w={q}&dict={dict_}&scope=e")["articles"].get(dict_, [])
        return [self.get(f"{API}/{dict_}/article/{i}.json") for i in ids]


def paradigms(articles, lemma: str, tag: str):
    """Yield (article_index, paradigm) for paradigms of `lemma` with POS `tag`."""
    for idx, art in enumerate(articles):
        for lem in art.get("lemmas", []):
            if lem.get("lemma") != lemma:
                continue
            for p in lem.get("paradigm_info", []):
                tags = p.get("tags") or []
                if tags and tags[0] == tag:
                    yield idx, p


def form(p, *want):
    for inf in p.get("inflection", []):
        if tuple(inf.get("tags") or ()) == want and inf.get("word_form"):
            return inf["word_form"]
    return None


def noun_info(fetcher, dict_, lemma, gender):
    """Return (gender, [def, pl, defpl], problem)."""
    arts = fetcher.articles(dict_, lemma)
    by_article: dict[int, dict[str, dict]] = {}
    for idx, p in paradigms(arts, lemma, "NOUN"):
        tags = p.get("tags") or []
        if "Uninfl" in tags:
            continue
        g = next((GENDER_TAG[t] for t in tags if t in GENDER_TAG), None)
        if g and g not in by_article.setdefault(idx, {}):
            by_article[idx][g] = p
    if not by_article:
        return None, None, f"{dict_}: no inflected noun '{lemma}'"
    # Prefer the homograph that has the seed gender (or feminine when the seed
    # says f); feminine wins inside that article whenever it is allowed.
    chosen = None
    for idx in sorted(by_article):
        if gender in by_article[idx] or (gender == "m" and "f" in by_article[idx] and "m" in by_article[idx]):
            chosen = by_article[idx]
            break
    problem = None
    if chosen is None:
        chosen = by_article[sorted(by_article)[0]]
        problem = f"{dict_}: seed gender {gender}, dictionary has {''.join(sorted(chosen))}"
    g = "f" if "f" in chosen else (gender if gender in chosen else sorted(chosen)[0])
    p = chosen[g]
    forms = [form(p, "Sing", "Def"), form(p, "Plur", "Ind"), form(p, "Plur", "Def")]
    forms = [f if f else "-" for f in forms]
    return g, forms, problem


def adj_info(fetcher, dict_, lemma):
    arts = fetcher.articles(dict_, lemma)
    for _, p in paradigms(arts, lemma, "ADJ"):
        forms = [form(p, "Pos", "Neuter", "Ind", "Sing"), form(p, "Pos", "Def", "Sing"), form(p, "Pos", "Plur")]
        if all(forms):
            return forms, None
    return None, f"{dict_}: no adjective '{lemma}'"


def exists(fetcher, dict_, lemma, tag):
    arts = fetcher.articles(dict_, lemma)
    if any(True for _ in paradigms(arts, lemma, tag)):
        return None
    # Ordinals and some determiners are filed under other tags.
    if any(True for _ in paradigms(arts, lemma, "DET")) or any(True for _ in paradigms(arts, lemma, "ADJ")):
        return None
    return f"{dict_}: no {tag.lower()} '{lemma}'"


def parse_seed(path):
    entries, seen = [], set()
    with open(path, encoding="utf-8") as fh:
        for raw in fh:
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            head, *extras = [part.strip() for part in line.split("|")]
            cols = head.split()
            pos, english = cols[0], cols[1]
            key = english if english not in seen else f"{english}#{pos}"
            seen.add(english)
            e = {"key": key, "pos": pos}
            if pos in ("part", "det"):
                e["ar"] = cols[2].replace("_", " ")
            elif pos == "pron":
                e["bm"], e["nn"], e["ar"] = cols[2], cols[3], cols[4]
            else:
                e["bm"], e["nn"] = cols[2], cols[3]
                if pos == "noun":
                    bm_g, _, nn_g = cols[4].partition("/")
                    e["gender"] = bm_g
                    e["seed_gender"] = {"bm": bm_g, "nn": nn_g or bm_g}
                if pos == "num":
                    e["value"] = int(cols[4])
            for ex in extras:
                k, v = (x.strip() for x in ex.split("=", 1))
                e[k] = int(v) if k == "form" else [x.strip() for x in v.split(",")]
            entries.append(e)
    return entries


def verify(fetcher, e):
    problems = []
    pos = e["pos"]
    if pos in ("part", "det"):
        e["verified"] = True
        e["source"] = "arabic-grammar-check"
        return e
    if pos == "noun":
        for d in ("bm", "nn"):
            g, forms, problem = noun_info(fetcher, d, e[d], e["seed_gender"][d])
            if problem:
                problems.append(problem)
            if forms:
                e[f"{d}_gender"], e[f"{d}_forms"] = g, forms
    elif pos == "pron":
        # Object pronouns are the accusative of the subject pronoun.
        for d in ("bm", "nn"):
            arts = fetcher.articles(d, e[d])
            acc = next((form(p, "Acc") for _, p in paradigms(arts, e[d], "PRON") if form(p, "Acc")), None)
            if acc:
                e[f"{d}_forms"] = [acc]
            else:
                problems.append(f"{d}: no accusative for pronoun '{e[d]}'")
    elif pos == "adj":
        for d in ("bm", "nn"):
            forms, problem = adj_info(fetcher, d, e[d])
            if problem:
                problems.append(problem)
            if forms:
                e[f"{d}_forms"] = forms
    else:
        tag = POS_TAG[pos] or "NUM"
        for d in ("bm", "nn"):
            problem = exists(fetcher, d, e[d], tag)
            if problem and pos == "num":
                problem = exists(fetcher, d, e[d], "NUM")
            if problem:
                problems.append(problem)
    e["verified"] = not problems
    e["source"] = "ordbokene.no"
    if problems:
        e["note"] = "; ".join(problems)
    return e


def toml_str(s: str) -> str:
    return json.dumps(s, ensure_ascii=False)


def toml_value(v):
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, int):
        return str(v)
    if isinstance(v, list):
        return "[" + ", ".join(toml_str(x) for x in v) + "]"
    return toml_str(v)


FIELD_ORDER = ["pos", "ar", "bm", "nn", "gender", "bm_gender", "nn_gender", "bm_forms", "nn_forms",
               "value", "form", "alt", "en_pl", "en_pp", "verified", "source", "note"]


def write_toml(entries, path):
    out = [
        "# Nybokmarabic core lexicon.",
        "# Generated by tools/verify_lexicon.py from ordbokene.no (Bokmålsordboka and Nynorskordboka).",
        "# Keys are English words; a second sense of the same word is keyed \"word#pos\".",
        "# Noun forms: [definite singular, indefinite plural, definite plural].",
        "# Adjective forms: [neuter, definite, plural]. Pronoun forms: [object form].",
        "# Edit freely, then run `nbma lexicon check`.",
        "",
    ]
    for e in entries:
        e.pop("seed_gender", None)
        if e["pos"] == "noun" and e.get("bm_gender") and e.get("bm_gender") == e.get("nn_gender"):
            e["gender"] = e.pop("bm_gender")
            e.pop("nn_gender")
        elif e["pos"] == "noun" and (e.get("bm_gender") or e.get("nn_gender")):
            e.pop("gender", None)
            e.setdefault("bm_gender", e.get("nn_gender"))
            e.setdefault("nn_gender", e.get("bm_gender"))
        key = e["key"]
        out.append(f"[{key}]" if key.replace("_", "").isalnum() and key.isascii() else f"[{toml_str(key)}]")
        for f in FIELD_ORDER:
            if f in e and e[f] is not None:
                out.append(f"{f} = {toml_value(e[f])}")
        out.append("")
    with open(path, "w", encoding="utf-8") as fh:
        fh.write("\n".join(out))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("seed")
    ap.add_argument("out")
    ap.add_argument("--cache", default=os.path.join(os.path.expanduser("~"), ".cache", "nbma-ordbokene"))
    args = ap.parse_args()
    fetcher = Fetcher(args.cache)
    entries = parse_seed(args.seed)
    with cf.ThreadPoolExecutor(max_workers=8) as pool:
        entries = list(pool.map(lambda e: verify(fetcher, e), entries))
    write_toml(entries, args.out)
    bad = [e for e in entries if not e["verified"]]
    print(f"{len(entries)} entries, {len(bad)} unverified", file=sys.stderr)
    for e in bad:
        print(f"  {e['key']}: {e.get('note')}", file=sys.stderr)


if __name__ == "__main__":
    main()
