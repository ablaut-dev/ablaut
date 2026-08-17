#!/usr/bin/env python3
"""Convert the kaikki.org Slovene verb extraction to the shared TSV.

kaikki carries tonal orthography (dẹ́lati, dẹ̑lam) that standard
Slovene does not write: every combining mark except the caron (čšž)
is stripped, and the archaic ł normalizes to l.
"""
import json
import sys
import unicodedata

SKIP = {"error-unrecognized-form", "table-tags", "inflection-template",
        "archaic", "obsolete", "rare", "dialectal"}
NUM = [("dual", "DU"), ("plural", "PL"), ("singular", "SG")]
PER = {"first-person": "1", "second-person": "2", "third-person": "3"}
GEN = {"masculine": "M", "feminine": "F", "neuter": "N"}


def plain(form):
    out = []
    for ch in unicodedata.normalize("NFD", form.replace("ł", "l").replace("ə", "e")):
        if unicodedata.combining(ch) and ch != "̌":
            continue
        out.append(ch)
    return unicodedata.normalize("NFC", "".join(out))


def main(path):
    for line in open(path):
        e = json.loads(line)
        lemma = plain(e.get("word", ""))
        if not lemma or " " in lemma:
            continue
        rows = set()
        for f in e.get("forms", []):
            tags = set(f.get("tags", []))
            if tags & SKIP or f.get("source") != "conjugation":
                continue
            form = plain(f.get("form", ""))
            if not form or " " in form or form == "-":
                continue
            n = next((code for tag, code in NUM if tag in tags), None)
            p = next((v for k, v in PER.items() if k in tags), None)
            # Spurious 'neuter'/'converb' tags ride some imperative and
            # l-participle rows; the row kind decides first.
            if "l-participle" in tags:
                g = next((v for k, v in GEN.items() if k in tags), None)
                # Dual/plural rows drop the explicit gender on neuter.
                if g and n:
                    rows.add((form, f"V.PTCP;PST;{g};{n}"))
            elif "imperative" in tags and n and p:
                rows.add((form, f"V;IMP;{n};{p}"))
            elif "present" in tags and "participle" not in tags and n and p:
                rows.add((form, f"V;IND;PRS;{n};{p}"))
            elif tags == {"infinitive"}:
                rows.add((form, "V;NFIN"))
            elif "supine" in tags:
                rows.add((form, "V;SUP"))
        if len(rows) >= 10:
            for form, feat in sorted(rows):
                print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
