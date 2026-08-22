#!/usr/bin/env python3
"""Convert the kaikki.org Telugu extraction to `lemma ⇥ form ⇥ features`.

kaikki (Wiktextract) carries ~2,500 Telugu verb entries, but only a
dozen have a filled conjugation table; the rest give only a
romanization. Worse, Wiktextract could not map the Telugu column
headers, so almost every inflected cell is tagged
`error-unrecognized-form` and the person/number/gender is lost from the
tag set — only the tense survives (`past`, `future`).

We recover person, number and gender from the Telugu personal ending,
which is unambiguous for the singular and for the human plural:

    -ను 1sg   -వు 2sg   -డు 3sg-masc   -ంది/-ది/-ుంది 3sg-nonmasc
    -ము 1pl   -రు {2pl, 3pl-human}     -యి/-వి 3pl-neuter (dropped:
                                        not in the UniMorph tel schema)

The human-plural ending `-రు` is shared by 2pl and both 3pl genders, so
each `-రు` form is emitted once, under `3;MASC;PL`, matching the slot
the engine fills identically. This yields a small, clean gold usable as
an independent spot check on the verbs kaikki actually tabulates.

Usage: python3 scripts/tel/kaikki_to_tsv.py data/tel/kaikki-telugu.jsonl
"""

import json
import sys

# Ending -> (person, number, gender-or-None), longest match first.
ENDINGS = [
    ("ుంది", ("3", "SG", "FEM")),
    ("ింది", ("3", "SG", "FEM")),
    ("ాడు", ("3", "SG", "MASC")),
    ("ాను", ("1", "SG", None)),
    ("ావు", ("2", "SG", None)),
    ("ాము", ("1", "PL", None)),
    ("ారు", ("3", "PL", "MASC")),
    ("తాడు", ("3", "SG", "MASC")),
    ("డు", ("3", "SG", "MASC")),
    ("ను", ("1", "SG", None)),
    ("వు", ("2", "SG", None)),
    ("ము", ("1", "PL", None)),
    ("రు", ("3", "PL", "MASC")),
    ("ది", ("3", "SG", "FEM")),
]
# Neuter-plural endings we deliberately skip (no UniMorph slot).
SKIP = ("ాయి", "తాయి", "వి", "యి")


def classify(form):
    if form.endswith(SKIP):
        return None
    for end, tag in ENDINGS:
        if form.endswith(end):
            return tag
    return None


def bundle(pnc, tense):
    person, number, gender = pnc
    parts = ["V", person]
    if gender:
        parts.append(gender)
    parts += [number, tense]
    return ";".join(parts)


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
                tags = set(fm.get("tags", []))
                form = fm.get("form", "")
                if not form or "romanization" in tags:
                    continue
                # Only the two cleanly tense-tagged synthetic tenses.
                if "past" in tags:
                    tense = "PST"
                elif "future" in tags:
                    tense = "FUT"
                else:
                    continue
                if tags & {"negative", "participle", "conditional",
                           "hortative", "imperative", "infinitive"}:
                    continue
                pnc = classify(form)
                if pnc is None:
                    continue
                rows.add((lemma, form, bundle(pnc, tense)))
    out = sys.stdout
    for lemma, form, features in sorted(rows):
        out.write(f"{lemma}\t{form}\t{features}\n")


if __name__ == "__main__":
    main(sys.argv[1])
