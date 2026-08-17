#!/usr/bin/env python3
"""Convert the kaikki.org Danish verb extraction to the shared TSV."""
import json
import sys

SKIP = {"alternative", "dialectal", "obsolete", "nonstandard", "archaic",
        "rare", "dated", "error-unrecognized-form", "table-tags",
        "inflection-template"}

CORE = [
    ({"active", "infinitive"}, "V;NFIN;ACT"),
    ({"infinitive", "passive"}, "V;NFIN;PASS"),
    ({"active", "present"}, "V;PRS;ACT"),
    ({"present"}, "V;PRS;ACT"),
    ({"passive", "present"}, "V;PRS;PASS"),
    ({"active", "past"}, "V;PST;ACT"),
    ({"past"}, "V;PST;ACT"),
    ({"passive", "past"}, "V;PST;PASS"),
    ({"active", "imperative"}, "V;IMP"),
    ({"imperative"}, "V;IMP"),
    ({"participle", "present"}, "V.PTCP;PRS"),
    ({"participle", "past"}, "V.PTCP;PST"),
]


def main(path):
    for line in open(path):
        e = json.loads(line)
        lemma = e.get("word", "")
        if not lemma or " " in lemma:
            continue
        rows = set()
        for f in e.get("forms", []):
            tags = set(f.get("tags", []))
            if tags & SKIP:
                continue
            form = f.get("form", "")
            if not form or " " in form or form == "-":
                continue
            for want, feat in CORE:
                if tags == want:
                    rows.add((form, feat))
                    break
        if rows:
            rows.add((lemma, "V;NFIN;ACT"))
            for form, feat in sorted(rows):
                print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
