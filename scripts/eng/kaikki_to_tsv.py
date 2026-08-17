#!/usr/bin/env python3
"""Convert the kaikki.org English verb extraction to the shared TSV.

Slots: V;NFIN, V;PST, V.PTCP;PST, V.PTCP;PRS, V;PRS;3;SG. UK/US
spelling doublets are kept as variants; dialectal, obsolete,
nonstandard, archaic and 'alternative' rows are dropped.
"""
import json
import sys

SKIP = {"alternative", "dialectal", "obsolete", "nonstandard",
        "colloquial", "archaic", "rare", "informal", "pronunciation-spelling"}

CORE = [
    ({"past"}, "V;PST"),
    ({"participle", "past"}, "V.PTCP;PST"),
    ({"participle", "present"}, "V.PTCP;PRS"),
    ({"present", "singular", "third-person"}, "V;PRS;3;SG"),
]


def main(path):
    for line in open(path):
        e = json.loads(line)
        if e.get("lang") != "English":
            continue
        lemma = e.get("word", "")
        if not lemma or " " in lemma or not lemma.isascii():
            continue
        rows = set()
        for f in e.get("forms", []):
            tags = set(f.get("tags", []))
            if tags & SKIP:
                continue
            form = f.get("form", "")
            if not form or " " in form or form == "-":
                continue
            base = tags - {"UK", "US"}
            for want, feat in CORE:
                if base == want:
                    rows.add((form, feat))
        if rows:
            print(f"{lemma}\t{lemma}\tV;NFIN")
            for form, feat in sorted(rows):
                print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
