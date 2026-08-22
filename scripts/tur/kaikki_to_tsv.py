#!/usr/bin/env python3
"""Convert the kaikki.org Turkish verb extraction to the shared TSV.

The Turkish Wiktionary conjugation tables are the big combined `{{tr-conj}}`
layout, and wiktextract does two awkward things with them:

1. the **person/number tags are scrambled** — `giderim` (aorist 1sg) comes
   out tagged `singular;third-person`, `gidersin` (2sg) as `first-person;
   plural`, and so on, a fixed rotation that makes the tags useless for
   person;
2. the **tense tags are reliable** — every form still carries its tense
   (`aorist`, `continuative`, `future`, `past`, `inferential`,
   `necessitative`) and, for the copular stack, a secondary
   (`aorist;past` = *giderdi*, `continuative;inferential` = *gidiyormuş*).

So this converter reads the **tense** from the tags and the **person** from
the cell's **position** within its tense block: Wiktionary lays each tense
out as six cells in the canonical order 1sg, 2sg, 3sg, 1pl, 2pl, 3pl. The
first six cells seen for a given (tense, polarity) are that tense's simple
paradigm; later repeats of the same tense tag are the *-ebil-* potential
and other derived stacks, which are dropped. Cells wiktextract could not
render (`error-unrecognized-form`), the non-finite rows (participles,
verbal nouns, converbs) and the multi-word forms are skipped.

The scrambled tags mean a few cells will still land on the wrong slot; that
is harmless, because the golden harness scores only where this leg and the
native-verified UniMorph leg *agree*, so a mis-slotted kaikki cell simply
drops out of the gold instead of corrupting it.

Usage: python3 scripts/tur/kaikki_to_tsv.py data/tur/kaikki-verbs.jsonl
"""

import json
import sys

PN = {"first-person", "second-person", "third-person", "singular", "plural"}
NOISE = {"error-unrecognized-form", "specific", "negative"}
# tags that mark a non-person cell (participle / verbal noun / converb /
# impersonal / infinitive) — never a finite person form
NONFINITE = {
    "participle", "noun-from-verb", "infinitive", "impersonal",
    "non-prospective", "prospective", "perfective", "imperfective",
    "personal", "error-unrecognized-form", "specific",
}

# (tense signature, as a frozenset) -> UniMorph skeleton. These are the
# single-word synthetic tenses both oracles express: the six base TAM
# categories and the single-word copular stacks (base + past/evidential).
SIG_MAP = {
    frozenset({"aorist"}): "V;IND;PRS;HAB",
    frozenset({"continuative"}): "V;IND;PRS;PROG",
    frozenset({"future"}): "V;IND;FUT",
    frozenset({"past"}): "V;IND;PST",
    frozenset({"inferential"}): "V;INFR;PST",
    frozenset({"necessitative"}): "V;OBLIG;PRS",
    frozenset({"aorist", "past"}): "V;IND;PST;HAB",
    frozenset({"aorist", "inferential"}): "V;INFR;PRS;HAB",
    frozenset({"continuative", "past"}): "V;IND;PST;PROG",
    frozenset({"continuative", "inferential"}): "V;INFR;PRS;PROG",
    frozenset({"future", "past"}): "V;IND;PST;PROSP",
    frozenset({"future", "inferential"}): "V;INFR;FUT",
    frozenset({"inferential", "past"}): "V;INFR;PST;PFV",
}

SLOTS = [("1", "SG"), ("2", "SG"), ("3", "SG"),
         ("1", "PL"), ("2", "PL"), ("3", "PL")]


def imperative_slot(form):
    """Person slot of a kaikki imperative cell, read off its suffix.

    UniMorph carries only the 2nd person (gel, gelin, geliniz); the 3rd
    person (gelsin, gelsinler) and the informal -sene forms are dropped.
    """
    if form.endswith(("iniz", "ınız", "unuz", "ünüz")):
        return "V;IMP;2;PL;LGSPEC2"
    if form.endswith(("sene", "sana", "senize", "sanıza")):
        return None
    if form.endswith(("sin", "sın", "sun", "sün", "sinler", "sınlar",
                       "sunlar", "sünler")):
        return None
    if form.endswith(("in", "ın", "un", "ün", "yin", "yın", "yun", "yün")):
        return "V;IMP;2;PL"
    return "V;IMP;2;SG"


def main(path):
    out = sys.stdout
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            e = json.loads(line)
            lemma = e.get("word", "")
            if not lemma or " " in lemma or not lemma[0].islower():
                continue
            forms = [
                fm for fm in e.get("forms", [])
                if fm.get("source") == "conjugation"
                and "inflection-template" not in fm.get("tags", [])
                and "table-tags" not in fm.get("tags", [])
            ]
            if not forms:
                continue
            counter = {}
            rows = set()
            for fm in forms:
                form = fm.get("form", "")
                tags = set(fm.get("tags", []))
                if not form or form == "-" or " " in form:
                    continue
                if tags & {"error-unrecognized-form"}:
                    continue
                pol = "NEG" if "negative" in tags else "POS"
                sig = frozenset(tags - PN - NOISE)
                if "imperative" in sig:
                    if sig & (NONFINITE - {"error-unrecognized-form"}):
                        continue
                    slot = imperative_slot(form)
                    if slot:
                        rows.add((form, f"{slot};{pol}"))
                    continue
                if sig & NONFINITE:
                    continue
                skeleton = SIG_MAP.get(sig)
                if not skeleton:
                    continue
                key = (skeleton, pol)
                idx = counter.get(key, 0)
                counter[key] = idx + 1
                if idx >= 6:
                    continue
                p, n = SLOTS[idx]
                rows.add((form, f"{skeleton};{p};{n};{pol}"))
            rows.add((lemma, "V;NFIN"))
            for form, feat in sorted(rows):
                out.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
