#!/usr/bin/env python3
"""Convert the kaikki.org Estonian verb extraction to the shared TSV.
Negated and periphrastic (multi-word) forms are out of scope."""
import json
import sys

SKIP = {"error-unrecognized-form", "table-tags", "inflection-template",
        "archaic", "obsolete", "rare", "dialectal", "negative", "class"}
PER = {"first-person": "1", "second-person": "2", "third-person": "3"}


def slot(tags):
    n = "SG" if "singular" in tags else "PL" if "plural" in tags else None
    p = next((v for k, v in PER.items() if k in tags), None)
    if "infinitive-ma" in tags:
        if "inessive" in tags:
            return "V;NFIN;MA;INE"
        if "elative" in tags:
            return "V;NFIN;MA;ELA"
        if "translative" in tags:
            return "V;NFIN;MA;TRANSL"
        if "abessive" in tags:
            return "V;NFIN;MA;ABE"
        if "passive" in tags:
            return "V;NFIN;MA;PASS"
        if "nominative" in tags or "illative" in tags:
            return "V;NFIN;MA"
        return None
    if "infinitive-da" in tags:
        return "V;NFIN;DA" if "present" in tags else None
    if "quotative" in tags:
        if "perfect" in tags:
            return None
        return "V;QUOT" if "active" in tags else None
    if "participle" in tags:
        tense = "PRS" if "present" in tags else "PST" if "past" in tags else None
        voice = "ACT" if "active" in tags else "PASS" if "passive" in tags else None
        return f"V.PTCP;{tense};{voice}" if tense and voice else None
    if "imperative" in tags:
        if "perfect" in tags:
            return None
        if "impersonal" in tags:
            return "V;IMP;IMPRS"
        return f"V;IMP;{n};{p}" if n and p else None
    if "conditional" in tags:
        tense = "PRS" if "present" in tags else "PRF" if "perfect" in tags else None
        if not tense:
            return None
        if "impersonal" in tags:
            return "V;COND;PRS;IMPRS" if tense == "PRS" else None
        return f"V;COND;{tense};{n};{p}" if n and p else None
    if "indicative" in tags:
        tense = "PRS" if "present" in tags else "PST" if "past" in tags else None
        if not tense:
            return None
        if "impersonal" in tags:
            return f"V;IND;{tense};IMPRS"
        return f"V;IND;{tense};{n};{p}" if n and p else None
    return None


def main(path):
    for line in open(path):
        e = json.loads(line)
        lemma = e.get("word", "")
        if not lemma or " " in lemma or not lemma.endswith("ma"):
            continue
        rows = set()
        for f in e.get("forms", []):
            tags = set(f.get("tags", []))
            if tags & SKIP or f.get("source") != "conjugation":
                continue
            form = f.get("form", "")
            if not form or " " in form or form == "-":
                continue
            feat = slot(tags)
            if feat:
                rows.add((form, feat))
        if len(rows) >= 10:
            for form, feat in sorted(rows):
                print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
