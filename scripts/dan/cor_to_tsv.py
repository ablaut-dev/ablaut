#!/usr/bin/env python3
"""Convert COR (Det Centrale Ordregister, Dansk Sprognævn) to the
shared TSV. Columns: id, lemma, ?, tag, form, status."""
import sys

TAGS = {
    "vb.inf.akt": "V;NFIN;ACT",
    "vb.inf.pass": "V;NFIN;PASS",
    "vb.præs.akt": "V;PRS;ACT",
    "vb.præs.pass": "V;PRS;PASS",
    "vb.præt.akt": "V;PST;ACT",
    "vb.præt.pass": "V;PST;PASS",
    "vb.imp": "V;IMP",
    "vb.præs.part": "V.PTCP;PRS",
    "vb.perf.part": "V.PTCP;PST",
}


def main(path):
    for line in open(path, encoding="utf-8"):
        f = line.rstrip("\n").split("\t")
        if len(f) < 5:
            continue
        lemma, tag, form = f[1], f[3], f[4]
        feat = TAGS.get(tag)
        if not feat or not form or " " in form or " " in lemma or form == "-":
            continue
        print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
