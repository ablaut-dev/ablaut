#!/usr/bin/env python3
"""Convert the kaikki.org Bengali verb JSONL into the shared
`lemma ⇥ form ⇥ features` TSV, using the same UniMorph feature bundles
scripts/ben/unimorph_to_tsv.py emits, so the harness can score against it.

kaikki ben is a spot check, not an agreement partner: it is the same
Wiktionary lineage as UniMorph ben (both descend from the `bn-conj`
family / Wikipedia). Two kaikki limits shape this adapter:

* kaikki tags তুই and তুমি identically (`familiar;second-person`), so
  the second-person-familiar finite cells are ambiguous and dropped;
* only the unambiguous person classes are recovered — the first person,
  the third-person ordinary (kaikki `familiar;third-person`, UniMorph
  `2;POL`, সে), and the honorific (kaikki `polite`, UniMorph `3;POL`,
  আপনি/তিনি) — across present / past / future and the progressive,
  perfect and past-habitual aspects, plus the non-finite forms
  (V.MSDR, V.NFIN and the perfective / habitual / progressive /
  conditional participles).

Usage: python3 scripts/ben/kaikki_to_tsv.py data/ben/kaikki-verbs.jsonl
"""

import json
import re
import sys

BEN = re.compile(r"[ঀ-৿]")
# Non-form rows and non-standard variants: skip outright.
SKIP = {
    "romanization",
    "table-tags",
    "inflection-template",
    "name",
    "alternative",
    "obsolete",
    "dated",
    "dialectal",
    "colloquial",
    "archaic",
    "poetic",
    "rare",
    "Standard",
}


def is_ben(s):
    return bool(BEN.search(s))


def person(t):
    """(unimorph person, politeness-suffix) or None to drop the cell."""
    if "first-person" in t:
        return ("1", "")
    if "polite" in t:
        return ("3", "POL")  # আপনি / তিনি — honorific
    if "third-person" in t and "familiar" in t:
        return ("2", "POL")  # সে — third ordinary (UniMorph tags it 2;POL)
    # `familiar;second-person` alone is তুই/তুমি ambiguous — drop.
    return None


def bundle(form, tags):
    """A single UniMorph bundle for one kaikki (form, tags) row, or None."""
    t = set(tags)
    if t & SKIP or not is_ben(form):
        return None

    # Non-finite (no person marking).
    if "noun-from-verb" in t:
        return "V;V.MSDR"
    if "participle" in t:
        if "perfect" in t:
            return "V;V.PTCP;PRF"
        if "habitual" in t:
            return "V;V.PTCP;HAB"
        if "progressive" in t:
            return "V;V.PTCP;PROG"
        if "conditional" in t:
            return "V;V.PTCP;COND"
        return None
    if "infinitive" in t:
        return "V;V.NFIN"

    p = person(t)
    if p is None:
        return None
    who, pol = p
    parts = ["V", who]

    if "future" in t:
        parts.append("FUT")
    else:
        parts.append("PRS" if "present" in t else "PST")
        if "habitual" in t:  # past habitual (kaikki also tags it conditional)
            parts.append("HAB")
        elif "perfect" in t:
            parts.append("PRF")
        elif "continuative" in t:  # kaikki's name for the progressive
            parts.append("PROG")
    if pol:
        parts.append(pol)
    return ";".join(parts)


def main(path):
    rows = set()
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            d = json.loads(line)
            lemma = d.get("word")
            if not lemma or not is_ben(lemma):
                continue
            for f in d.get("forms", []):
                form = (f.get("form") or "").strip()
                tags = f.get("tags")
                if not form or not tags:
                    continue
                b = bundle(form, tags)
                if b:
                    rows.add((lemma, form, b))
    out = sys.stdout
    for lemma, form, features in sorted(rows):
        out.write(f"{lemma}\t{form}\t{features}\n")


if __name__ == "__main__":
    main(sys.argv[1])
