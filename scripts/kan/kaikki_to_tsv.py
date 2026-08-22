#!/usr/bin/env python3
"""Convert the kaikki.org Kannada extraction to `lemma ⇥ form ⇥ features`.

kaikki (Wiktextract of English Wiktionary) carries ~165 Kannada verbs
with a full conjugation table, each cell tagged with person, number,
(third-person) gender and tense — the same schema UniMorph kan uses, so
the two form a genuine two-oracle agreement surface.

We keep the finite past/present/future and the imperative, mapped to
UniMorph bundles:

    first/second/third-person -> 1/2/3
    singular/plural           -> SG/PL
    masculine/feminine/neuter -> MASC/FEM/NEUT   (third person only)
    past/present/future       -> PST/PRS/FUT

The combined masculine+feminine plural row (kaikki tags one cell with
both genders) is emitted once under each of MASC and PL, matching the
two identical UniMorph slots.

Two families of cells are dropped, both non-verb noise rather than
linguistic content:

* the negative, contingent (dubitative), participle, infinitive,
  cohortative, optative and conditional columns — separate paradigms
  outside the finite past/present/future/imperative core;
* malformed extractions: template residue (`{{{204}}}`) and forms with
  two adjacent Kannada dependent vowel signs (e.g. ಆಗಮಿಸುುವೆ), which is
  not valid Kannada orthography — a Wiktionary template that failed to
  drop a stem-final vowel before appending a vowel-initial suffix.

Usage: python3 scripts/kan/kaikki_to_tsv.py data/kan/kaikki-kannada.jsonl
"""

import json
import re
import sys

# Two adjacent dependent vowel signs (matras U+0CBE..U+0CCC): malformed.
DOUBLE_MATRA = re.compile("[ಾ-ೌ]{2}")

# kaikki tags that mark a cell outside the finite past/present/future/
# imperative core we score.
SKIP = {
    "negative", "contingent", "participle", "adjectival", "adverbial",
    "infinitive", "cohortative", "optative", "suihortative", "volitive",
    "conditional", "non-past",
}


def malformed(form):
    return (
        not form
        or form in ("-", "no-table-tags")
        or "{{{" in form
        or "}}}" in form
        or DOUBLE_MATRA.search(form) is not None
    )


def bundles(tags):
    """UniMorph bundle(s) for a kaikki tag set, or [] to skip."""
    t = set(tags)
    if t & SKIP:
        return []
    if "past" in t:
        tense = "PST"
    elif "present" in t:
        tense = "PRS"
    elif "future" in t:
        tense = "FUT"
    elif "imperative" in t:
        tense = "IMP"
    else:
        return []
    if "first-person" in t:
        person = "1"
    elif "second-person" in t:
        person = "2"
    elif "third-person" in t:
        person = "3"
    else:
        return []
    if "singular" in t:
        number = "SG"
    elif "plural" in t:
        number = "PL"
    else:
        return []
    if tense == "IMP":
        # Imperative is second person only, no gender.
        return [f"V;{person};{number};IMP"] if person == "2" else []
    if person == "3":
        genders = []
        if "masculine" in t:
            genders.append("MASC")
        if "feminine" in t:
            genders.append("FEM")
        if "neuter" in t:
            genders.append("NEUT")
        if not genders:
            return []
        return [f"V;3;{g};{number};{tense}" for g in genders]
    return [f"V;{person};{number};{tense}"]


def main(path):
    rows = set()
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            d = json.loads(line)
            if d.get("pos") != "verb":
                continue
            lemma = d.get("word")
            if not lemma:
                continue
            for fm in d.get("forms", []):
                tags = fm.get("tags", [])
                if "romanization" in tags or "inflection-template" in tags:
                    continue
                form = fm.get("form", "")
                if malformed(form):
                    continue
                for b in bundles(tags):
                    rows.add((lemma, form, b))
    out = sys.stdout
    for lemma, form, features in sorted(rows):
        out.write(f"{lemma}\t{form}\t{features}\n")


if __name__ == "__main__":
    main(sys.argv[1])
