#!/usr/bin/env python3
"""Expand the apertium-urd monolingual dictionary into the common
golden-harness TSV: `lemma <TAB> form <TAB> features`.

apertium-urd is the **independent** second oracle: a hand-built lttoolbox
morphological dictionary with no Wiktionary lineage, so its agreement
with kaikki is the two-oracle gate. Rather than shell out to `lt-expand`
(which would make CI depend on lttoolbox), the `.dix` is expanded here
directly — the monodix is flat (no nested paradigms), so every verb entry
is `stem + paradigm-suffix`:

    <e lm="اتر"><i>اتر</i><par n="بول__vblex_iv"/></e>
    <pardef n="بول__vblex_iv"> … <e><p><l>وں</l><r>…<s n="prs"/><s n="p1"/><s n="sg"/></r></p></e> …

so the surface of each cell is `<i>` + the paradigm entry's `<l>`, and the
citation infinitive is `<i>` + the `inf/nom/m` entry's `<l>` (اتر + نا).

apertium tags the *synthetic* core only (no participle-plus-copula
analytic layer); that core is exactly the overlap with kaikki. Its
synthetic future is written joined (اترونگا) where kaikki writes it apart
(اتروں گا), so the future slots fall out of the intersection as a
disagreement rather than an agreement — documented in docs/urd/oracles.md.

Tag mapping (apertium → shared bundle):
  inf;nom;m          → V;NFIN;LGSPEC1   infinitive (اترنا)
  inf;obl            → V;NFIN;LGSPEC2   oblique infinitive (اترنے)
  stem               → V;2;SG;IMP;INFM  intimate imperative = bare stem (اتر)
  imp;p2;pl          → V;2;SG;IMP;FORM  familiar imperative (اترو)
  imp;p2;frm;pl      → V;2;PL;IMP;FORM  polite imperative (اترئیے)
  prs;pN;num         → V;N;num;SBJV     subjunctive (اتروں …)
  fut;pN;g;num       → V;N;num;SBJV;g   synthetic future (joined; see above)
  impf;g;num         → V;V.PTCP;g;num;IPFV   imperfective participle
  perf;g;num         → V;V.PTCP;g;num;PFV     perfective participle

Usage: python3 scripts/urd/apertium_to_tsv.py \
           data/urd/apertium-urd/apertium-urd.urd.dix > data/urd/apertium.tsv
"""

import sys
import xml.etree.ElementTree as ET

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from norm import normalize  # noqa: E402

PN = {"p1": "1", "p2": "2", "p3": "3"}
NUM = {"sg": "SG", "pl": "PL"}
GEN = {"m": "MASC", "f": "FEM"}


def surface(elem):
    """The surface text of an <l>/<r>: its character data plus a space for
    every <b/> word boundary; <s> analysis tags contribute nothing."""
    if elem is None:
        return ""
    out = [elem.text or ""]
    for child in elem:
        if child.tag == "b":
            out.append(" ")
        elif child.tag != "s":
            out.append(child.text or "")
        out.append(child.tail or "")
    return "".join(out)


def tags_of(r):
    return [s.get("n") for s in r.findall("s")]


def features(tags):
    """Map an apertium tag list to the shared bundle, or None to skip."""
    t = set(tags)
    if "vblex" not in t or "neg" in t:
        return None
    if "inf" in t:
        if "obl" in t:
            return "V;NFIN;LGSPEC2"
        return "V;NFIN;LGSPEC1" if "m" in t else None
    if "stem" in t:
        return "V;2;SG;IMP;INFM"  # bare stem = intimate imperative
    p = next((PN[x] for x in tags if x in PN), None)
    n = next((NUM[x] for x in tags if x in NUM), None)
    g = next((GEN[x] for x in tags if x in GEN), None)
    # The participles inflect for gender/number but not person, so they
    # are matched before the finite person guard below.
    if "impf" in t and g and n:
        return f"V;V.PTCP;{g};{n};IPFV"
    if "perf" in t and g and n:
        return f"V;V.PTCP;{g};{n};PFV"
    if "imp" in t:
        if p != "2" or n != "pl":
            return None
        return "V;2;PL;IMP;FORM" if "frm" in t else "V;2;SG;IMP;FORM"
    if not p or not n:
        return None
    if "prs" in t:
        return f"V;{p};{n};SBJV"
    if "fut" in t and g:
        return f"V;{p};{n};SBJV;{g}"
    return None


def main(path):
    tree = ET.parse(path)
    root = tree.getroot()

    # Paradigms: name -> list of (suffix, tag list).
    pardefs = {}
    for pd in root.iter("pardef"):
        cells = []
        for e in pd.findall("e"):
            p = e.find("p")
            if p is None:
                continue
            cells.append((surface(p.find("l")), tags_of(p.find("r"))))
        pardefs[pd.get("n")] = cells

    rows = set()
    for section in root.iter("section"):
        for e in section.findall("e"):
            par = e.find("par")
            i = e.find("i")
            if par is None or i is None:
                continue
            name = par.get("n")
            cells = pardefs.get(name)
            if not cells or not name.endswith(("_tv", "_iv", "_vblex")):
                continue
            stem = surface(i)
            # Citation infinitive = stem + the inf;nom;m suffix.
            infinitive = None
            for suf, tags in cells:
                if "inf" in tags and "nom" in tags and "m" in tags:
                    infinitive = normalize(stem + suf)
                    break
            if not infinitive:
                continue
            for suf, tags in cells:
                feat = features(tags)
                if not feat:
                    continue
                form = normalize(stem + suf)
                # Fold the optional feminine-plural participle nasal
                # (اتراتیں → اتراتی) — some apertium paradigms write it,
                # others do not, and the engine, like Hindi, keeps a
                # single (number-less) feminine participle cell. Confined
                # to the participle so the subjunctive plural (اتریں) is
                # untouched.
                if "PTCP" in feat and "FEM;PL" in feat and form.endswith("یں"):
                    form = form[:-1]
                if form:
                    rows.add((infinitive, form, feat))

    out = sys.stdout
    for lemma, form, feat in sorted(rows):
        out.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "data/urd/apertium-urd/apertium-urd.urd.dix")
