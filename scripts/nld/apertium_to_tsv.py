#!/usr/bin/env python3
"""Expand the apertium-nld monodix verb paradigms to the shared TSV.

Usage: python3 scripts/nld/apertium_to_tsv.py data/nld/apertium-nld.nld.dix

Pure-python expansion (no lttoolbox needed): a dix entry is a
sequence of <i> (identity), <p><l>/<r> (surface/lexical pair) and
<par> (paradigm reference) elements; expansion is the cartesian
product over the referenced pardef's entries, concatenating the
surface sides and collecting the <s n=.../> tags. Only vblex entries
are kept, mapped to the same schema as scripts/nld/kaikki_to_tsv.py.

Apertium marks the present indicative `pres` and the (archaic)
present subjunctive `prs`; only `pres` is in schema. The present
plural is person-invariant and fans out over the three persons; the
past is number-marked only. Separable verbs expand to multiword
surfaces (a <b/> space) and drop out.
"""

import sys
import xml.etree.ElementTree as ET
from itertools import product

PER = {"p1": "1", "p2": "2", "p3": "3"}


def expand_e(e, pardefs):
    """Yield (surface, tags) for one <e>, tags a tuple of sdef names."""
    parts = []  # each part: list of (surface, tags)
    for child in e:
        if child.tag == "i":
            parts.append([(text_of(child), ())])
        elif child.tag == "p":
            left = child.find("l")
            tags = tuple(s.get("n") for s in child.find("r").findall("s"))
            parts.append([(text_of(left), tags)])
        elif child.tag == "par":
            parts.append(pardefs[child.get("n")])
        elif child.tag == "re":
            return  # regex entries are not verb tables
    for combo in product(*parts):
        yield "".join(c[0] for c in combo), tuple(t for c in combo for t in c[1])


def text_of(el):
    """Surface text of <i> or <l>, with <b/> as a space."""
    if el is None:
        return ""
    out = el.text or ""
    for sub in el:
        if sub.tag == "b":
            out += " "
        out += sub.tail or ""
    return out


def features(tags):
    """Map an Apertium tag bundle to a list of schema features."""
    if not tags or tags[0] != "vblex":
        return []
    rest = list(tags[1:])
    match rest:
        case ["inf"]:
            return ["V;NFIN"]
        case ["pp"]:
            return ["V.PTCP;PST"]
        case ["pprs"]:
            return ["V.PTCP;PRS"]
        case ["imp", "sg"]:
            return ["V;IMP"]
        case ["pres", p, "sg"] if p in PER:
            return [f"V;IND;PRS;SG;{PER[p]}"]
        case ["pres", "pl"]:
            return [f"V;IND;PRS;PL;{n}" for n in ("1", "2", "3")]
        case ["past", _p, "sg"]:
            return ["V;IND;PST;SG"]
        case ["past", "sg"]:
            return ["V;IND;PST;SG"]
        case ["past", "pl"]:
            return ["V;IND;PST;PL"]
    return []


def main(path):
    root = ET.parse(path).getroot()
    pardefs = {}
    for pd in root.find("pardefs"):
        name = pd.get("n")
        pardefs[name] = [pair for e in pd.findall("e") for pair in expand_e(e, pardefs)]
    seen = set()
    for section in root.findall("section"):
        for e in section.findall("e"):
            lemma = e.get("lm")
            if not lemma or " " in lemma:
                continue
            for form, tags in expand_e(e, pardefs):
                if not form or " " in form:
                    continue
                for feat in features(tags):
                    row = (lemma, form, feat)
                    if row not in seen:
                        seen.add(row)
                        print("\t".join(row))


if __name__ == "__main__":
    main(sys.argv[1])
