#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) German verb JSONL into the UniMorph-style
lemma<TAB>form<TAB>features TSV that the golden harness reads.

Usage: python3 scripts/kaikki_to_tsv.py data/kaikki/verbs.jsonl > data/kaikki/deu.tsv

Only synthetic conjugation-table forms are kept (analytic tenses are the
compositional layer's business). The perfect auxiliary is emitted as a
pseudo-feature V;AUX so the harness can score auxiliary() too.
"""
import json
import sys

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
NUMBER = {"singular": "SG", "plural": "PL"}
SKIP = {
    "table-tags",
    "inflection-template",
    "class",
}
ANALYTIC = {"perfect": "PRF", "pluperfect": "PLPRF", "future-i": "FUT1", "future-ii": "FUT2"}


def features(tags):
    """Map a kaikki tag set to a UniMorph feature string, or None."""
    if tags & SKIP:
        return None
    p = next((PERSON[t] for t in tags if t in PERSON), None)
    n = next((NUMBER[t] for t in tags if t in NUMBER), None)
    # Analytic tenses (Layer C): tagged multiword-construction in kaikki.
    analytic = next((ANALYTIC[t] for t in tags if t in ANALYTIC), None)
    if analytic:
        if not (p and n):
            return None
        mood = "SBJV" if "subjunctive" in tags else "IND"
        return f"V;{analytic};{mood};{n};{p}"
    if "multiword-construction" in tags:
        return None
    if "infinitive-zu" in tags:
        return "V;NFIN;LGSPEC01"
    if "infinitive" in tags:
        return "V;NFIN"
    if "participle" in tags:
        return "V.PTCP;PRS" if "present" in tags else "V.PTCP;PST"
    if "imperative" in tags:
        return f"V;IMP;{n};2" if n else None
    if not (p and n):
        return None
    if "subjunctive-i" in tags:
        return f"V;SBJV;{n};{p};PRS"
    if "subjunctive-ii" in tags:
        return f"V;SBJV;{n};{p};PST"
    if "indicative" in tags and "present" in tags:
        return f"V;IND;{n};{p};PRS"
    if "indicative" in tags and "preterite" in tags:
        return f"V;IND;{n};{p};PST"
    return None


def main(path):
    seen = set()
    for line in open(path):
        entry = json.loads(line)
        lemma = entry.get("word", "")
        forms = [f for f in entry.get("forms", []) if f.get("source") == "conjugation"]
        if not lemma or not forms:
            continue
        for f in forms:
            form = f.get("form", "").strip()
            tags = set(f.get("tags", []))
            if not form or form == "-":
                continue
            if "auxiliary" in tags and not tags & SKIP:
                # "haben or sein" combo rows: both plain rows also appear.
                if " " in form:
                    continue
                feat = "V;AUX"
            else:
                feat = features(tags)
            if feat is None:
                continue
            row = (lemma, form, feat)
            if row not in seen:
                seen.add(row)
                sys.stdout.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
