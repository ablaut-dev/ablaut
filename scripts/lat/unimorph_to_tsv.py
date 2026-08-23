#!/usr/bin/env python3
"""Filter the UniMorph Latin table to the scored present-system
active-indicative core, keeping the feature strings verbatim (UniMorph is
already lemma<TAB>form<TAB>features and is the reference scheme the kaikki
adapter is aligned to). Macrons are kept — both oracles carry them.

Usage: unimorph_to_tsv.py data/lat/unimorph-lat.tsv > data/lat/unimorph.tsv
"""
import sys

CELLS = {
    "V;IND;ACT;PRS;1;SG", "V;IND;ACT;PRS;2;SG", "V;IND;ACT;PRS;3;SG",
    "V;IND;ACT;PRS;1;PL", "V;IND;ACT;PRS;2;PL", "V;IND;ACT;PRS;3;PL",
    "V;IND;ACT;PST;IPFV;1;SG", "V;IND;ACT;PST;IPFV;2;SG",
    "V;IND;ACT;PST;IPFV;3;SG", "V;IND;ACT;PST;IPFV;1;PL",
    "V;IND;ACT;PST;IPFV;2;PL", "V;IND;ACT;PST;IPFV;3;PL",
    "V;IND;ACT;FUT;1;SG", "V;IND;ACT;FUT;2;SG", "V;IND;ACT;FUT;3;SG",
    "V;IND;ACT;FUT;1;PL", "V;IND;ACT;FUT;2;PL", "V;IND;ACT;FUT;3;PL",
    "V;IMP;ACT;PRS;2;SG", "V;IMP;ACT;PRS;2;PL",
    "V;NFIN;ACT;PRS",
}


def main(path):
    for line in open(path):
        a = line.rstrip("\n").split("\t")
        if len(a) >= 3 and a[2] in CELLS and a[1] and " " not in a[1]:
            print(f"{a[0]}\t{a[1]}\t{a[2]}")


if __name__ == "__main__":
    main(sys.argv[1])
