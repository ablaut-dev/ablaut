#!/usr/bin/env python3
"""Build data/grn/{kaikki.tsv,parts.tsv} from the kaikki.org Paraguayan
Guarani dump. See src/grn.rs for the engine these gold files gate.

Usage: python3 scripts/grn/build.py  (run from repo root)
Reads  data/grn/kaikki-raw.jsonl (fetch via scripts/grn/fetch_kaikki.sh).
Writes data/grn/kaikki.tsv and data/grn/parts.tsv.
"""
import json
import os
from collections import Counter, defaultdict

ROOT = os.path.join(os.path.dirname(__file__), "..", "..")
RAW = os.path.join(ROOT, "data", "grn", "kaikki-raw.jsonl")
PRON = {"che", "nde", "ha'e", "ha'ekuéra", "ñande", "ore", "peẽ"}


def norm(form):
    return " ".join(t for t in form.split(" ") if t not in PRON).strip()


def person(s):
    if "first-person" in s:
        if "singular" in s:
            return "1SG"
        if "inclusive" in s:
            return "1PL.INCL"
        if "exclusive" in s:
            return "1PL.EXCL"
    if "second-person" in s:
        return "2SG" if "singular" in s else "2PL"
    if "third-person" in s:
        return "3SG" if "singular" in s else "3PL"
    return None


def voice(s):
    for v in ("passive", "reciprocal", "coactive", "objective"):
        if v in s:
            return v.upper()
    return "ACT"


def mood(s):
    for m in ("indicative", "hortative", "imperative"):
        if m in s:
            return m.upper()
    return None


def cls_token(name):
    name = name.replace("gn-conj-", "gug-conj-")
    body = name[len("gug-conj-"):]
    return "h" if body == "h" else body  # areal-oral, aireal-nasal, ...


def main():
    gold = defaultdict(lambda: defaultdict(set))  # lemma -> feat -> {forms}
    parts = {}                                    # lemma -> class token
    for line in open(RAW, encoding="utf-8"):
        d = json.loads(line)
        if d.get("pos") != "verb":
            continue
        tmpls = [t.get("name") for t in d.get("inflection_templates", [])]
        if not tmpls:
            continue
        tok = cls_token(tmpls[0])
        if tok.startswith("stative"):
            continue   # 1 verb, unverified voice matrix
        w = d["word"]
        parts[w] = tok
        for fo in d.get("forms", []):
            tg = set(fo.get("tags", []))
            form = fo.get("form", "")
            if {"inflection-template", "table-tags", "error-unrecognized-form"} & tg:
                continue
            if form in ("-", "", "hikuái", "no-table-tags"):
                continue
            p, m = person(tg), mood(tg)
            if not p or not m:
                continue
            n = norm(form)
            if n in ("", "hikuái"):
                continue
            gold[w][f"V;{voice(tg)};{m};{p}"].add(n)

    with open(os.path.join(ROOT, "data", "grn", "kaikki.tsv"), "w", encoding="utf-8") as f:
        for lemma in sorted(gold):
            for feat in sorted(gold[lemma]):
                for form in sorted(gold[lemma][feat]):
                    f.write(f"{lemma}\t{form}\t{feat}\n")

    with open(os.path.join(ROOT, "data", "grn", "parts.tsv"), "w", encoding="utf-8") as f:
        f.write("# Guarani principal parts: lemma ⇥ conjugation class.\n")
        f.write("# The class is read verbatim from the kaikki gug-conj-* inflection\n")
        f.write("# template (see scripts/grn/build.py); the engine in src/grn.rs\n")
        f.write("# derives the whole paradigm from it. gn-conj-* is folded into gug-.\n")
        f.write("lemma\tclass\n")
        for lemma in sorted(parts):
            f.write(f"{lemma}\t{parts[lemma]}\n")

    print("lemmas:", len(gold), "classes:", Counter(parts.values()))
    print("gold rows:", sum(len(v) for feats in gold.values() for v in feats.values()))


if __name__ == "__main__":
    main()
