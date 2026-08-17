#!/usr/bin/env python3
"""Convert the kaikki.org Czech verb extraction to the shared TSV."""
import json
import sys

SKIP = {"error-unrecognized-form", "table-tags", "inflection-template",
        "archaic", "obsolete", "rare", "dialectal"}

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}


def main(path):
    for line in open(path):
        e = json.loads(line)
        lemma = e.get("word", "")
        if not lemma or " " in lemma:
            continue
        rows = set()
        for f in e.get("forms", []):
            tags = set(f.get("tags", []))
            if tags & SKIP or f.get("source") != "conjugation":
                continue
            form = f.get("form", "")
            if not form or " " in form or form == "-":
                continue
            n = "SG" if "singular" in tags else "PL" if "plural" in tags else None
            p = next((v for k, v in PERSON.items() if k in tags), None)
            if tags == {"infinitive"}:
                rows.add((form, "V;NFIN"))
            elif "indicative" in tags and "present" in tags and n and p:
                rows.add((form, f"V;IND;PRS;{n};{p}"))
            elif "imperative" in tags and n and p:
                rows.add((form, f"V;IMP;{n};{p}"))
            elif "participle" in tags and ("past" in tags or "passive" in tags) and n:
                kind = "PST" if "past" in tags else "PASS"
                if "masculine" in tags:
                    g = "MA" if "animate" in tags else "MI" if "inanimate" in tags else None
                elif "feminine" in tags:
                    g = "F"
                elif "neuter" in tags:
                    g = "N"
                else:
                    g = None
                if g:
                    rows.add((form, f"V.PTCP;{kind};{g};{n}"))
            elif "transgressive" in tags:
                tense = "PRS" if "present" in tags else "PST"
                if "plural" in tags:
                    rows.add((form, f"V;CVB;{tense};PL"))
                elif "masculine" in tags:
                    rows.add((form, f"V;CVB;{tense};M"))
                elif "feminine" in tags:
                    rows.add((form, f"V;CVB;{tense};FN"))
        if len(rows) >= 10:
            for form, feat in sorted(rows):
                print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
