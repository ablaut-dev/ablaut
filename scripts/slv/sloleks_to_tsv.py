#!/usr/bin/env python3
"""Convert the Sloleks 3.0 SQLite mirror to the shared TSV."""
import sqlite3
import sys

NUM = {"singular": "SG", "dual": "DU", "plural": "PL"}
PER = {"first": "1", "second": "2", "third": "3"}
GEN = {"masculine": "M", "feminine": "F", "neuter": "N"}


def main(path):
    con = sqlite3.connect(path)
    q = """SELECT le.lemma, wf.vform, wf.number, wf.gender, wf.person,
                  wf.negative, wf.form
           FROM word_forms wf
           JOIN lexical_entries le ON wf.lexical_entry_id = le.id
           WHERE le.category = 'verb'"""
    for lemma, vform, number, gender, person, negative, form in con.execute(q):
        if not form or " " in form or " " in (lemma or "") or negative == "yes":
            continue
        feat = None
        if vform == "infinitive":
            feat = "V;NFIN"
        elif vform == "supine":
            feat = "V;SUP"
        elif vform == "present" and number in NUM and person in PER:
            feat = f"V;IND;PRS;{NUM[number]};{PER[person]}"
        elif vform == "imperative" and number in NUM and person in PER:
            feat = f"V;IMP;{NUM[number]};{PER[person]}"
        elif vform == "participle" and number in NUM and gender in GEN:
            feat = f"V.PTCP;PST;{GEN[gender]};{NUM[number]}"
        if feat:
            print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
