#!/usr/bin/env python3
"""Extract the verb inflections from the dexonline database dump into
the lemma<TAB>form<TAB>features TSV the Romanian golden harness reads.

Usage: python3 scripts/ron/dexonline_to_tsv.py data/ron/dex-database.sql.gz \
           > data/ron/dexonline.tsv

Streams the SQL dump: Lexeme rows give lemma spellings (modelType V),
InflectedForm rows give (form, lexemeId, inflectionId); the fixed
Inflection ids 49-86 map to the shared feature schema. The "long"
infinitive/participle/gerund nominal forms are skipped.
"""
import gzip
import sys

FEAT = {
    49: "V;NFIN",
    51: "V;IMP;SG;2",
    52: "V.PTCP;PST;MASC;SG",
    53: "V;GER",
    54: "V;IND;PRS;SG;1", 55: "V;IND;PRS;SG;2", 56: "V;IND;PRS;SG;3",
    57: "V;IND;PRS;PL;1", 58: "V;IND;PRS;PL;2", 59: "V;IND;PRS;PL;3",
    60: "V;SBJV;PRS;SG;1", 61: "V;SBJV;PRS;SG;2", 62: "V;SBJV;PRS;SG;3",
    63: "V;SBJV;PRS;PL;1", 64: "V;SBJV;PRS;PL;2", 65: "V;SBJV;PRS;PL;3",
    66: "V;IND;PST;IPFV;SG;1", 67: "V;IND;PST;IPFV;SG;2", 68: "V;IND;PST;IPFV;SG;3",
    69: "V;IND;PST;IPFV;PL;1", 70: "V;IND;PST;IPFV;PL;2", 71: "V;IND;PST;IPFV;PL;3",
    72: "V;IND;PST;PFV;SG;1", 73: "V;IND;PST;PFV;SG;2", 74: "V;IND;PST;PFV;SG;3",
    75: "V;IND;PST;PFV;PL;1", 76: "V;IND;PST;PFV;PL;2", 77: "V;IND;PST;PFV;PL;3",
    78: "V;IND;PST;PQP;SG;1", 79: "V;IND;PST;PQP;SG;2", 80: "V;IND;PST;PQP;SG;3",
    81: "V;IND;PST;PQP;PL;1", 82: "V;IND;PST;PQP;PL;2", 83: "V;IND;PST;PQP;PL;3",
    86: "V;IMP;PL;2",
}

def tuples(payload):
    """Yield top-level parenthesized tuples, respecting quotes."""
    depth, q, start = 0, False, None
    i = 0
    while i < len(payload):
        c = payload[i]
        if q:
            if c == "\\":
                i += 2
                continue
            if c == "'":
                q = False
        elif c == "'":
            q = True
        elif c == "(":
            if depth == 0:
                start = i + 1
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0 and start is not None:
                yield payload[start:i]
                start = None
        i += 1


def fields(tup):
    """Split a SQL tuple respecting quoted strings."""
    out, cur, q = [], [], False
    i = 0
    while i < len(tup):
        c = tup[i]
        if q:
            if c == "\\":
                cur.append(tup[i + 1]); i += 2; continue
            if c == "'":
                q = False
            else:
                cur.append(c)
        elif c == "'":
            q = True
        elif c == ",":
            out.append("".join(cur)); cur = []
        else:
            cur.append(c)
        i += 1
    out.append("".join(cur))
    return out


def main(path):
    lex = {}
    inflected = []
    with gzip.open(path, "rt", encoding="utf-8", errors="replace") as f:
        for line in f:
            if line.startswith("INSERT INTO `Lexeme` "):
                for tup in tuples(line[line.index("VALUES") + 6 :]):
                    fs = fields(tup)
                    if len(fs) > 14 and fs[14] in ("V", "VT"):
                        lex[fs[0]] = fs[2]
            elif line.startswith("INSERT INTO `InflectedForm` "):
                for tup in tuples(line[line.index("VALUES") + 6 :]):
                    fs = fields(tup)
                    # id, form, formNoAccent, formUtf8General, lexemeId, inflectionId
                    if len(fs) > 5:
                        infl = fs[5]
                        if infl.isdigit() and int(infl) in FEAT:
                            inflected.append((fs[4], fs[2], int(infl)))
    seen = set()
    for lexeme_id, form, infl in inflected:
        lemma = lex.get(lexeme_id)
        if not lemma or " " in form or not form:
            continue
        row = (lemma, form, FEAT[infl])
        if row not in seen:
            seen.add(row)
            sys.stdout.write(f"{lemma}\t{form}\t{FEAT[infl]}\n")


if __name__ == "__main__":
    main(sys.argv[1])
