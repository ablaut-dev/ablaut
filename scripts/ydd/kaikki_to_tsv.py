#!/usr/bin/env python3
"""kaikki.org Yiddish verbs -> shared TSV, the synthetic (one-word) core.

Yiddish is a Germanic language written right-to-left in the Hebrew script.
kaikki gives the present tense as pronoun+verb pairs ("איך בענטש") and the
imperative with a parenthetical subject ("בענטש (דו)"); we recover the bare
synthetic verb form and read person/number off the pronoun. The scored core:

  * V;NFIN            infinitive (citation)
  * V;PRS;p;n         present indicative, 1/2/3 x SG/PL
  * V;IMP;2;SG/PL     the two imperatives
  * V.PTCP;PRS        present participle (…דיק)
  * V.PTCP;PST        past participle (the ge-…-t / strong-ablaut form)

The past, pluperfect, future and future-perfect are periphrastic (auxiliary +
participle) and multi-word, so they are dropped. Any form containing a space
(separable-prefix presents/imperatives such as "קום אָן") is skipped as well —
this is a single-word engine.
"""
import json, sys

# Subject pronoun -> person;number for the present paradigm.
PRON = {
    "איך": "1;SG", "דו": "2;SG",
    "ער": "3;SG", "זי": "3;SG", "עס": "3;SG",
    "מיר": "1;PL", "איר": "2;PL", "זיי": "3;PL",
}
SKIP = {"romanization", "alternative", "table-tags", "inflection-template",
        "error-unrecognized-form"}


def cells(entry):
    out = {}
    for f in entry.get("forms", []):
        tags = f.get("tags", [])
        form = f.get("form", "").strip()
        if set(tags) & SKIP or not form or form == "-":
            continue
        if tags == ["infinitive"] and " " not in form:
            out["V;NFIN"] = form
        elif tags == ["participle", "present"] and " " not in form:
            out["V.PTCP;PRS"] = form
        elif tags == ["participle", "past"] and " " not in form:
            out["V.PTCP;PST"] = form
        elif tags == ["present"]:
            t = form.split()
            if len(t) == 2 and t[0] in PRON and " " not in t[1]:
                out["V;PRS;" + PRON[t[0]]] = t[1]
        elif tags == ["imperative"]:
            if form.endswith("(דו)"):
                v = form[:-4].strip()
                if v and " " not in v:
                    out["V;IMP;2;SG"] = v
            elif form.endswith("(איר)"):
                v = form[:-5].strip()
                if v and " " not in v:
                    out["V;IMP;2;PL"] = v
    return out


def main(path):
    for line in open(path):
        try:
            e = json.loads(line)
        except Exception:
            continue
        if e.get("pos") != "verb":
            continue
        lemma = e.get("word", "").strip()
        if not lemma or " " in lemma:
            continue
        c = cells(e)
        if "V;NFIN" not in c:
            continue
        for feat, form in sorted(c.items()):
            print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
