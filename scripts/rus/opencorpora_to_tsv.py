#!/usr/bin/env python3
"""Convert the OpenCorpora dictionary (dict.opcorpora.txt) into the
lemma<TAB>form<TAB>features TSV that the Russian golden harness reads.

Usage: python3 scripts/rus/opencorpora_to_tsv.py data/rus/dict.opcorpora.txt \
           > data/rus/opencorpora.tsv

OpenCorpora is a hand-checked Russian morphology lexicon of provenance
wholly independent of Wiktionary — the required second oracle.

Format: blank-line-separated blocks; each block is a numeric lexeme id
followed by `FORM<TAB>grammemes` lines (grammemes comma/space separated,
all uppercase). Critically, OpenCorpora splits one verb into *separate*
consecutive lexemes: a VERB block (finite forms), an INFN block (the
infinitive), and PRTF/PRTS/GRND blocks (participles/gerunds). The
infinitive that keys the harness lives only in the INFN block, so each
VERB block is paired with the next INFN block inside the same contiguous
verbal run — a pairing that is 1:1 for all but a handful of slang
lexemes.

Only the finite paradigm is emitted, in the schema shared with
scripts/rus/kaikki_to_tsv.py. Forms are lowercased (OpenCorpora prints
uppercase); ё is preserved. Perfective non-past is tagged `futr`
(→ V;IND;FUT), imperfective present `pres` (→ V;IND;PRS). The standard
2nd-person imperative is `impr,excl`; the hortative `impr,incl` (возьмём
= let's take) is skipped.
"""
import re
import sys

VERBAL = {"VERB", "INFN", "PRTF", "PRTS", "GRND"}


def parse_form(line):
    form, rest = line.split("\t", 1)
    toks = [t for t in re.split(r"[,\s]+", rest.strip()) if t]
    return form, toks, set(toks)


def blocks(path):
    buf = []
    for raw in open(path, encoding="utf-8"):
        line = raw.rstrip("\n")
        if line == "":
            if buf:
                forms = [parse_form(l) for l in buf[1:] if "\t" in l]
                if forms:
                    yield forms
            buf = []
        else:
            buf.append(line)
    if buf:
        forms = [parse_form(l) for l in buf[1:] if "\t" in l]
        if forms:
            yield forms


def pos(block):
    return block[0][1][0]


def finite_features(tags):
    """Map a VERB-form grammeme set to a feature string, or None."""
    n = "SG" if "sing" in tags else ("PL" if "plur" in tags else None)
    p = "1" if "1per" in tags else ("2" if "2per" in tags else ("3" if "3per" in tags else None))
    if "impr" in tags:
        if "incl" in tags:  # hortative возьмём — not the 2nd-person imperative
            return None
        return f"V;IMP;{n};2" if n else None
    if "past" in tags:
        if n == "PL":
            return "V;IND;PST;PL"
        g = "MASC" if "masc" in tags else ("FEM" if "femn" in tags else ("NEUT" if "neut" in tags else None))
        return f"V;IND;PST;{g};SG" if g else None
    if not (p and n):
        return None
    if "futr" in tags:
        return f"V;IND;FUT;{n};{p}"
    if "pres" in tags:
        return f"V;IND;PRS;{n};{p}"
    return None


def main(path):
    seen = set()
    out = sys.stdout
    # Group the file into maximal contiguous runs of verbal lexemes, then
    # pair each VERB block with the next unused INFN block in the run.
    run = []

    def emit_run(run):
        infn_positions = [i for i, b in enumerate(run) if pos(b) == "INFN"]
        used = set()
        for i, b in enumerate(run):
            if pos(b) != "VERB":
                continue
            nxt = [j for j in infn_positions if j > i and j not in used]
            if not nxt:
                nxt = [j for j in infn_positions if j not in used]
            if not nxt:
                continue
            j = nxt[0]
            used.add(j)
            lemma = run[j][0][0].lower()
            for form, toks, tags in b:
                feat = finite_features(tags)
                if feat is None:
                    continue
                lform = form.lower()
                row = (lemma, lform, feat)
                if row not in seen:
                    seen.add(row)
                    out.write(f"{lemma}\t{form.lower()}\t{feat}\n")
            # the infinitive itself
            inf = run[j][0][0].lower()
            row = (lemma, inf, "V;NFIN")
            if row not in seen:
                seen.add(row)
                out.write(f"{lemma}\t{inf}\tV;NFIN\n")

    for b in blocks(path):
        if pos(b) in VERBAL:
            run.append(b)
        else:
            if run:
                emit_run(run)
                run = []
    if run:
        emit_run(run)


if __name__ == "__main__":
    main(sys.argv[1])
