#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) Dutch verb JSONL into the
lemma<TAB>form<TAB>features TSV the Dutch golden harness reads.

Usage: python3 scripts/nld/kaikki_to_tsv.py data/nld/kaikki-verbs.jsonl \
           > data/nld/kaikki.tsv

Dutch synthetic slots: the infinitive (V;NFIN), the person-marked
present indicative (V;IND;PRS;{SG,PL};{1,2,3} — the plural is
person-invariant, emitted for all three persons so the schema lines up
with Apertium's person tags), the number-marked past indicative
(V;IND;PST;{SG,PL} — Dutch past has no person distinction), the
singular imperative (V;IMP), and the two participles (V.PTCP;PRS,
V.PTCP;PST). Register/dialect/archaic variants and the nominal gerund
are skipped so both oracles describe the same standard paradigm.
The feature schema matches scripts/nld/apertium_to_tsv.py.
"""
import json
import sys

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}

# Any of these tags on a form drops it: register/dialect layers, the
# archaic present subjunctive, the nominal gerund, and template rows.
SKIP = {
    "archaic",
    "formal",
    "informal",
    "colloquial",
    "Flanders",
    "Netherlands",
    "Suriname",
    "majestic",
    "subjunctive",
    "dated",
    "obsolete",
    "rare",
    "dialectal",
    "table-tags",
    "inflection-template",
    "gerund",
    "multiword-construction",
    "error-unknown-tag",
    "second-person-semantically",
}


def features(tags):
    """Map a kaikki tag set to a list of feature strings (possibly
    empty). The present plural fans out over the three persons."""
    if tags & SKIP:
        return []
    if "infinitive" in tags:
        return ["V;NFIN"]
    if "participle" in tags:
        if "present" in tags:
            return ["V.PTCP;PRS"]
        if "past" in tags:
            return ["V.PTCP;PST"]
        return []
    if "imperative" in tags:
        # Only the (modern) singular imperative; the plural is archaic.
        return ["V;IMP"] if "singular" in tags else []
    p = next((PERSON[t] for t in tags if t in PERSON), None)
    if "present" in tags:
        if "singular" in tags and p:
            return [f"V;IND;PRS;SG;{p}"]
        if "plural" in tags:
            return [f"V;IND;PRS;PL;{n}" for n in ("1", "2", "3")]
        return []
    if "past" in tags:
        if "singular" in tags:
            return ["V;IND;PST;SG"]
        if "plural" in tags:
            return ["V;IND;PST;PL"]
    return []


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
            if not form or form == "-" or form.startswith("—") or " " in form:
                continue
            for feat in features(set(f.get("tags", []))):
                row = (lemma, form, feat)
                if row not in seen:
                    seen.add(row)
                    sys.stdout.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
