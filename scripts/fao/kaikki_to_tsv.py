#!/usr/bin/env python3
"""Convert the kaikki.org Faroese verb extraction to the shared TSV, under
the same UniMorph-fao feature strings so the harness can intersect the two
into agreement gold.

Faroese is cited by its infinitive (mostly -a). The scored synthetic core is
the infinitive, the present and past indicative (person/number, with the
1/2/3-plural syncretised to UniMorph's numberless `V;IND;PRS;3` / `V;IND;PST;3`
plural cell), the two imperatives, the supine (UniMorph `V.CVB`) and the
present and past participles. kaikki's Faroese tables are sparse — reliably
only the supine and the third-person past — but they give a genuine second
attestation of those cells. The adjectival declension of the past participle
(gender/case forms) is skipped: only single-word finite/nonfinite cells are
kept, matching UniMorph's scope.
"""
import json
import sys

P = {"first-person": "1", "second-person": "2", "third-person": "3"}
N = {"singular": "SG", "plural": "PL"}
DROP = {"error-unrecognized-form", "table-tags", "inflection-template",
        "romanization", "alternative", "canonical", "multiword-construction"}
# Case/gender tags mark the adjectival declension of the participles, which
# UniMorph does not carry as verb cells — skip any form bearing one.
ADJ = {"nominative", "accusative", "dative", "genitive",
       "masculine", "feminine", "neuter"}


def feature(tags):
    t = {x.lower() for x in tags}
    if t & DROP or t & ADJ:
        return None
    if "supine" in t:
        return "V.CVB"
    if "infinitive" in t:
        return "V;NFIN"
    if "participle" in t:
        if "present" in t:
            return "V.PTCP.PRS"
        if "past" in t:
            return "V.PTCP.PST"
        return None
    p = next((v for k, v in P.items() if k in t), None)
    n = next((v for k, v in N.items() if k in t), None)
    if "imperative" in t and n:
        return f"V;IMP;2;{n}"
    tense = "PRS" if "present" in t else ("PST" if "past" in t else None)
    if tense is None:
        return None
    # Plural present/past collapse to UniMorph's numberless third-person cell.
    if n == "PL":
        return f"V;IND;{tense};3"
    if p and n == "SG":
        return f"V;IND;{tense};{p};SG"
    return None


def main(path):
    for line in open(path):
        try:
            e = json.loads(line)
        except json.JSONDecodeError:
            continue
        if e.get("pos") != "verb":
            continue
        lemma = e.get("word", "")
        if not lemma or " " in lemma:
            continue
        rows = set()
        for f in e.get("forms", []):
            form = f.get("form", "").strip()
            if not form or form == "-" or " " in form:
                continue
            feat = feature(f.get("tags", []))
            if feat:
                rows.add((form, feat))
        for form, feat in sorted(rows):
            print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
