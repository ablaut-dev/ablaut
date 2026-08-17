#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) Italian verb JSONL into the
lemma<TAB>form<TAB>features TSV the Italian golden harness reads.

Usage: python3 scripts/ita/kaikki_to_tsv.py data/ita/kaikki-verbs.jsonl \
           > data/ita/kaikki.tsv

en.wiktionary's Italian tables carry pedagogical stress marks on every
form (pàrlo, parlàre); Italian orthography only writes word-final
accents (parlò). Non-final accents are stripped. The auxiliary from the
table header becomes a V;AUX slot (avere/essere). Negative imperatives
and combined clitic forms are skipped.
"""
import json
import sys

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
NUMBER = {"singular": "SG", "plural": "PL"}
GENDER = {"masculine": "MASC", "feminine": "FEM"}
SKIP = {"table-tags", "inflection-template", "negative", "combined-form",
        "multiword-construction", "second-person-semantically"}
STRIP = str.maketrans("àèéìíòóùú", "aeeiioouu")


def deaccent(form):
    """Strip stress marks except on the final character."""
    if not form:
        return form
    return form[:-1].translate(STRIP) + form[-1]


def features(tags):
    if tags & SKIP:
        return None
    p = next((PERSON[t] for t in tags if t in PERSON), None)
    n = next((NUMBER[t] for t in tags if t in NUMBER), None)
    if "auxiliary" in tags:
        return "V;AUX"
    if "infinitive" in tags:
        return "V;NFIN"
    if "gerund" in tags:
        return "V;GER"
    if "participle" in tags:
        # The table lists the citation forms (masc sg).
        when = "PST" if "past" in tags else "PRS"
        return f"V.PTCP;{when};MASC;SG"
    if "imperative" in tags:
        return f"V;IMP;{n};{p}" if n and p else None
    if not (p and n):
        return None
    if "subjunctive" in tags:
        if "imperfect" in tags:
            return f"V;SBJV;PST;{n};{p}"
        if "present" in tags:
            return f"V;SBJV;PRS;{n};{p}"
        return None
    if "conditional" in tags:
        return f"V;COND;{n};{p}"
    if "indicative" in tags:
        if "present" in tags:
            return f"V;IND;PRS;{n};{p}"
        if "imperfect" in tags:
            return f"V;IND;PST;IPFV;{n};{p}"
        if "historic" in tags:
            return f"V;IND;PST;PFV;{n};{p}"
        if "future" in tags:
            return f"V;IND;FUT;{n};{p}"
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
            if not form or form == "-" or form.startswith("—") or " " in form:
                continue
            feat = features(set(f.get("tags", [])))
            if feat is None:
                continue
            form = deaccent(form)
            row = (lemma, form, feat)
            if row not in seen:
                seen.add(row)
                sys.stdout.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
