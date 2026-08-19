#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) Ukrainian verb JSONL into the
lemma<TAB>form<TAB>features TSV that the Ukrainian golden harness reads.

Usage: python3 scripts/ukr/kaikki_to_tsv.py data/ukr/kaikki-verbs.jsonl \
           > data/ukr/kaikki.tsv

Ukrainian synthetic slots (shared schema with scripts/ukr/vesum_to_tsv.py):
  V;NFIN
  V;IND;PRS;{SG,PL};{1,2,3}   the non-past finite forms: the imperfective
                              present (читаю) and the perfective non-past,
                              which carries future meaning (прочитаю) and
                              which both oracles tag "future".
  V;IND;PST;{MASC,FEM,NEUT};SG and V;IND;PST;PL   the l-past marks gender
                              in the singular and only number in the plural.
  V;IMP;{SG,PL};{1,2,3}       the imperative (2sg, 1pl, 2pl exist).

Skipped on purpose: the imperfective synthetic future in -тиму
(читатиму — a distinct formation, not the non-past slot), the analytic
буду/буде periphrases (multiword), adverbial and passive participles,
and the impersonal -но/-то form. Combining stress marks (U+0301) are
stripped so forms compare against the unstressed VESUM lexicon.
"""
import json
import sys
import unicodedata

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
NUMBER = {"singular": "SG", "plural": "PL"}
GENDER = {"masculine": "MASC", "feminine": "FEM", "neuter": "NEUT"}
SKIP = {
    "error-unrecognized-form",
    "table-tags",
    "inflection-template",
    "class",
    "adverbial",
    "passive",
    "active",
    "participle",
    "archaic",
    "obsolete",
    "rare",
    "dialectal",
}


def destress(s):
    return unicodedata.normalize("NFC", s.replace("́", "").replace("̀", ""))


def features(tags):
    """Map a kaikki tag set to a feature string, or None."""
    if tags & SKIP:
        return None
    p = next((PERSON[t] for t in tags if t in PERSON), None)
    n = next((NUMBER[t] for t in tags if t in NUMBER), None)
    if "infinitive" in tags:
        return "V;NFIN"
    if "imperative" in tags:
        return f"V;IMP;{n};{p}" if n and p else None
    if "past" in tags and "future" not in tags:
        g = next((GENDER[t] for t in tags if t in GENDER), None)
        if n == "PL":
            return "V;IND;PST;PL"
        if g and n == "SG":
            return f"V;IND;PST;{g};SG"
        return None
    if not (p and n):
        return None
    # The non-past finite paradigm: imperfective "present" plus the
    # perfective non-past, which kaikki tags "future".
    if "present" in tags:
        return f"V;IND;PRS;{n};{p}"
    if "future" in tags and "perfective" in tags:
        return f"V;IND;PRS;{n};{p}"
    return None


def main(path):
    seen = set()
    for line in open(path):
        entry = json.loads(line)
        lemma = destress(entry.get("word", "").strip())
        if not lemma or " " in lemma:
            continue
        forms = [f for f in entry.get("forms", []) if f.get("source") == "conjugation"]
        for f in forms:
            form = destress(f.get("form", "").strip())
            if not form or " " in form or form == "-" or form.startswith("—"):
                continue
            feat = features(set(f.get("tags", [])))
            if feat is None:
                continue
            row = (lemma, form, feat)
            if row not in seen:
                seen.add(row)
                sys.stdout.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
