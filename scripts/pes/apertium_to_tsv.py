#!/usr/bin/env python3
"""Convert `lt-expand` output of apertium-pes into the common
golden-harness TSV: `lemma <TAB> form <TAB> features`.

apertium-pes is the independent second oracle: a hand-built lttoolbox
morphological dictionary with no Wiktionary lineage. `lt-expand` walks it
into `surface:lemma<tags>` pairs (see scripts/pes/fetch_apertium.sh); this
maps its finite tag set onto the same bundle kaikki_to_tsv.py emits, so
the harness can intersect the two.

Tag mapping (apertium → shared bundle):
  <past>            → V;IND;PST     simple past (کردم)
  <cont><pii>       → V;IND;PST     past imperfective (می‌کردم) — kaikki
                                    files this under the same past slot
  <cont><pri>       → V;IND;PRS     present indicative (می‌کنم)
  <prs>             → V;SBJV        present subjunctive (بکنم)
  <pprf>            → V;IND;PRF     present perfect (کرده‌ام)
  <imp>             → V;IMP;2       imperative (بکن) — 2nd person only
  <inf>             → V;NFIN
  <pp>              → V;PTCP;PST

Usage: lt-expand apertium-pes.pes.dix | python3 scripts/pes/apertium_to_tsv.py > data/pes/apertium.tsv
"""
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from norm import normalize  # noqa: E402

PN = {"p1": "1", "p2": "2", "p3": "3"}
NUM = {"sg": "SG", "pl": "PL"}


def features(tags):
    t = set(tags)
    if "vblex" not in t or "neg" in t:
        return None  # affirmative finite/non-finite only
    if "inf" in t:
        return "V;NFIN"
    if "pp" in t:
        return "V;PTCP;PST"
    p = next((PN[x] for x in tags if x in PN), None)
    n = next((NUM[x] for x in tags if x in NUM), None)
    if "imp" in t:
        return f"V;IMP;2;{n}" if p == "2" and n else None
    if not p or not n:
        return None
    pn = f"{p};{n}"
    if "prs" in t:
        return f"V;SBJV;{pn}"
    if "pprf" in t:
        return f"V;IND;PRF;{pn}"
    if "cont" in t and "pri" in t:
        return f"V;IND;PRS;{pn}"
    if "cont" in t and "pii" in t:
        return f"V;IND;PST;{pn}"
    if "past" in t:
        return f"V;IND;PST;{pn}"
    return None


def main():
    seen = set()
    for line in sys.stdin:
        line = line.rstrip("\n")
        if ":" not in line or "<vblex>" not in line:
            continue
        surface, lexical = line.split(":", 1)
        # lt-expand marks direction-restricted entries `surface:>:lemma…`
        # (RL-only) or `surface:<:lemma…` (LR-only); drop the marker.
        if lexical[:2] in (">:", "<:"):
            lexical = lexical[2:]
        # lexical = lemma<tag1><tag2>...
        lemma, _, rest = lexical.partition("<")
        tags = [t for t in ("<" + rest).replace(">", " ").split("<") if t.strip()]
        tags = [t.strip() for t in tags]
        feat = features(tags)
        if not feat:
            continue
        lemma = normalize(lemma)
        form = normalize(surface)
        if not lemma or not form:
            continue
        row = (lemma, form, feat)
        if row not in seen:
            seen.add(row)
            print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main()
