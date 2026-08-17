#!/usr/bin/env python3
"""Convert the kaikki.org Finnish verb extraction to the shared TSV.
Negatives, perfect tenses and other periphrastics are multi-word and
out of scope."""
import json
import sys

SKIP = {"error-unrecognized-form", "table-tags", "inflection-template",
        "archaic", "obsolete", "rare", "dialectal", "negative", "class",
        "connegative"}
PER = {"first-person": "1", "second-person": "2", "third-person": "3"}


def slot(tags):
    n = "SG" if "singular" in tags else "PL" if "plural" in tags else None
    p = next((v for k, v in PER.items() if k in tags), None)
    if "infinitive" in tags:
        return None
    if "participle" in tags:
        return None
    if "passive" in tags:
        if "indicative" in tags and "present" in tags:
            return "V;IND;PRS;PASS"
        if "indicative" in tags and "past" in tags:
            return "V;IND;PST;PASS"
        if "conditional" in tags:
            return "V;COND;PASS"
        if "potential" in tags:
            return "V;POT;PASS"
        if "imperative" in tags:
            return "V;IMP;PASS"
        return None
    if not (n and p):
        return None
    if "indicative" in tags and "present" in tags:
        return f"V;IND;PRS;{n};{p}"
    if "indicative" in tags and "past" in tags:
        return f"V;IND;PST;{n};{p}"
    if "conditional" in tags:
        return f"V;COND;{n};{p}"
    if "potential" in tags:
        return f"V;POT;{n};{p}"
    if "imperative" in tags:
        return f"V;IMP;{n};{p}"
    return None


def main(path):
    for line in open(path):
        e = json.loads(line)
        lemma = e.get("word", "")
        if not lemma or " " in lemma or not lemma[0].islower():
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
            rows.add((lemma, "V;NFIN"))
            for form, feat in sorted(rows):
                print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
