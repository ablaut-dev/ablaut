#!/usr/bin/env python3
"""Convert the kaikki.org Afrikaans verb extraction to the shared TSV.

Afrikaans verbs barely inflect: the present equals the infinitive (bare
stem, no agreement), the past is periphrastic (het + past participle) for
all but a closed set of preterites, and the only productive synthetic
form is the ge- past participle. kaikki lists the present, the present
participle (-ende) and the ge- past participle; we emit them under the
same UniMorph feature strings the UniMorph oracle uses, so the harness
can intersect the two into agreement gold.
"""
import json
import sys

SKIP = {"alternative", "dialectal", "obsolete", "nonstandard", "archaic",
        "rare", "dated", "error-unrecognized-form", "table-tags",
        "inflection-template", "colloquial", "informal"}

# Exact tag-set -> UniMorph feature (UniMorph-afr scheme: V;INF, V;PRS,
# V;PST, V.PTCP;PST, V.PTCP;PRS).
CORE = [
    ({"present"}, "V;PRS"),
    ({"infinitive"}, "V;INF"),
    ({"participle", "past"}, "V.PTCP;PST"),
    ({"past", "participle"}, "V.PTCP;PST"),
    ({"participle", "present"}, "V.PTCP;PRS"),
    ({"present", "participle"}, "V.PTCP;PRS"),
    ({"past"}, "V;PST"),
    ({"preterite"}, "V;PST"),
]


def main(path):
    for line in open(path):
        e = json.loads(line)
        if e.get("pos") != "verb":
            continue
        lemma = e.get("word", "")
        if not lemma or " " in lemma:
            continue
        rows = set()
        for f in e.get("forms", []):
            tags = set(f.get("tags", []))
            if tags & SKIP:
                continue
            form = f.get("form", "")
            if not form or " " in form or form == "-" or form == lemma and not tags:
                continue
            for want, feat in CORE:
                if tags == want:
                    rows.add((form, feat))
                    break
        if rows:
            # The lemma is its own infinitive and present.
            rows.add((lemma, "V;INF"))
            rows.add((lemma, "V;PRS"))
            for form, feat in sorted(rows):
                print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
