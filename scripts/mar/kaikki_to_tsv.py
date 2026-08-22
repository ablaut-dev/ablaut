#!/usr/bin/env python3
"""Convert the kaikki.org Marathi verb JSONL into the shared
`lemma <TAB> form <TAB> features` TSV, using the same UniMorph-style
bundles scripts/mar/apertium_to_tsv.py emits, so the harness can
intersect the two into the two-oracle agreement gold.

kaikki Marathi is the independent second oracle (Wiktextract of English
Wiktionary, CC BY-SA), but Wiktextract could not map the person/number
row headers of the `mr-conj` table, so its finite cells keep only a
tense+gender tag and lose person/number. The clean, unambiguously-tagged
contribution is therefore the **non-finite** forms:

  infinitive   → V;NFIN      (करणे)
  completive   → V;CVB;PFV   (करून)
  prospective  → V;PROSP     (करणार)
  desiderative → V;PURP      (करायला)

These are the cells the two oracles agree on per cell. The finite
paradigm is corroborated at the set level (see docs/mar/oracles.md), not
intersected per cell.

Usage: python3 scripts/mar/kaikki_to_tsv.py data/mar/kaikki-verbs.jsonl
"""
import json
import re
import sys

DEV = re.compile(r"[ऀ-ॿ]")
MAP = {
    "infinitive": "V;NFIN",
    "completive": "V;CVB;PFV",
    "prospective": "V;PROSP",
    "desiderative": "V;PURP",
}


def is_dev(s):
    return bool(DEV.search(s))


def main(path):
    rows = set()
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            d = json.loads(line)
            lemma = (d.get("word") or "").strip()
            if not lemma or not is_dev(lemma) or not lemma.endswith("णे"):
                continue
            for f in d.get("forms", []):
                form = (f.get("form") or "").strip()
                tags = f.get("tags") or []
                # a single, unambiguous tag; a real single-word form
                if not form or " " in form or not is_dev(form) or len(tags) != 1:
                    continue
                b = MAP.get(tags[0])
                if b:
                    rows.add((lemma, form, b))
    out = sys.stdout
    for lemma, form, feat in sorted(rows):
        out.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
