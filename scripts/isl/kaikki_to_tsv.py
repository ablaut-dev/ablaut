#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) Icelandic verb JSONL into the
lemma<TAB>form<TAB>features TSV the Icelandic golden harness reads.

Usage: python3 scripts/isl/kaikki_to_tsv.py data/isl/kaikki-verbs.jsonl \
           > data/isl/kaikki.tsv

The English-Wiktionary Icelandic verb tables extract only partially: the
finite paradigm is dominated by the third-person past (the strong verbs'
principal part) and the supine, plus the declined past-participle table
(which is out of the conjugator's single-form V.PTCP;PST schema and
skipped). The headword itself is the infinitive. The feature schema
matches scripts/isl/bin_to_tsv.py.
"""
import json
import sys

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
NUMBER = {"singular": "SG", "plural": "PL"}
# Case tags mark the declined participle/adjective table, not a verb slot.
CASE = {"nominative", "accusative", "dative", "genitive"}
SKIP = {"table-tags", "inflection-template", "error-unrecognized-form"}


def features(tags):
    """Map a kaikki tag set to a feature string, or None."""
    if tags & SKIP or tags & CASE:
        return None
    if "supine" in tags:
        return "V.PTCP;PST"
    if "infinitive" in tags:
        return "V;NFIN"
    if "participle" in tags and "present" in tags:
        return "V;V.PTCP;PRS"
    p = next((PERSON[t] for t in tags if t in PERSON), None)
    n = next((NUMBER[t] for t in tags if t in NUMBER), None)
    if not (p and n):
        return None
    if "imperative" in tags:
        return f"V;IMP;{n};{p}"
    mood = "V;SBJV" if "subjunctive" in tags else "V;IND" if "indicative" in tags else None
    if mood is None:
        return None
    if "present" in tags:
        t = "PRS"
    elif "past" in tags:
        t = "PST"
    else:
        return None
    return f"{mood};{t};{n};{p}"


def main(path):
    seen = set()
    for line in open(path, encoding="utf-8"):
        entry = json.loads(line)
        lemma = entry.get("word", "").strip()
        if not lemma:
            continue
        # The headword is the infinitive form.
        row = (lemma, lemma, "V;NFIN")
        if row not in seen:
            seen.add(row)
            sys.stdout.write(f"{lemma}\t{lemma}\tV;NFIN\n")
        for f in entry.get("forms", []):
            # The declined participle table carries source="conjugation"
            # and case tags (dropped in features()); the finite forms and
            # the supine carry no source, so we filter by tag, not source.
            form = f.get("form", "").strip()
            if not form or form == "-" or form.startswith("—"):
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
