#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) Swedish verb JSONL into the
lemma<TAB>form<TAB>features TSV the Swedish golden harness reads.

Usage: python3 scripts/swe/kaikki_to_tsv.py data/swe/kaikki-verbs.jsonl \
           > data/swe/kaikki.tsv

Archaic plural forms and the dated subjunctives are kept (SALDO carries
the subjunctives too); participles are skipped — SALDO's verb entries
do not carry them, so they cannot be cross-checked.
"""
import json
import sys

SKIP = {"table-tags", "inflection-template", "archaic", "participle",
        "multiword-construction"}


def features(tags):
    if tags & SKIP:
        return None
    voice = "PASS" if "passive" in tags else "ACT"
    if "infinitive" in tags:
        return f"V;NFIN;{voice}"
    if "supine" in tags:
        return f"V;SUP;{voice}"
    if "imperative" in tags:
        return "V;IMP" if voice == "ACT" else None
    if "subjunctive" in tags:
        when = "PST" if "past" in tags else "PRS"
        return f"V;SBJV;{when};{voice}"
    if "indicative" in tags:
        when = "PST" if "past" in tags else "PRS"
        return f"V;IND;{when};{voice}"
    return None


def main(path):
    seen = set()
    for line in open(path):
        entry = json.loads(line)
        lemma = entry.get("word", "")
        forms = [f for f in entry.get("forms", []) if f.get("source") == "conjugation"]
        if not lemma or not forms or " " in lemma:
            continue
        for f in forms:
            form = f.get("form", "").strip()
            if not form or form == "-" or form.startswith("—") or " " in form:
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
