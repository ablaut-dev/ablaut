#!/usr/bin/env python3
"""Expand the apertium-slv monodix verb paradigms to the shared TSV.

Usage: python3 scripts/slv/apertium_to_tsv.py data/slv/apertium-slv.slv.dix

Pure-python expansion (no lttoolbox needed): a dix entry is a
sequence of <i> (identity), <p><l>/<r> (surface/lexical pair) and
<par> (paradigm reference) elements; expansion is the cartesian
product over the referenced pardef's entries, concatenating the
surface sides and collecting the <s n=.../> tags. Only vblex
entries are kept, and only tags inside the nine-slot schema:
infinitive, supine, dual-aware present and imperative, and the
gendered l-participle (a bare `past` tag, i.e. not the `pp`
passive participle, which declines like an adjective and is out
of schema along with fut/cni/pprs/neg).
"""

import sys
import xml.etree.ElementTree as ET
from itertools import product

NUM = {"sg": "SG", "du": "DU", "pl": "PL"}
PER = {"p1": "1", "p2": "2", "p3": "3"}
GEN = {"m": "M", "f": "F", "nt": "N"}
ASPECT = {"imperf", "perf", "iter"}


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
    """Map an Apertium tag bundle to the schema, or None."""
    tags = [t for t in tags if t not in ASPECT]
    if not tags or tags[0] != "vblex":
        return None
    match tags[1:]:
        case ["inf"]:
            return "V;NFIN"
        case ["supn"]:
            return "V;SUP"
        case ["pres", p, n] if p in PER and n in NUM:
            return f"V;IND;PRS;{NUM[n]};{PER[p]}"
        case ["imp", p, n] if p in PER and n in NUM:
            return f"V;IMP;{NUM[n]};{PER[p]}"
        case ["past", g, n] if g in GEN and n in NUM:
            return f"V.PTCP;PST;{GEN[g]};{NUM[n]}"
    return None


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
                feat = features(tags)
                if not feat or not form or " " in form:
                    continue
                row = (lemma, form, feat)
                if row not in seen:
                    seen.add(row)
                    print("\t".join(row))


if __name__ == "__main__":
    main(sys.argv[1])
