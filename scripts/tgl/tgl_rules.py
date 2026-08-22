#!/usr/bin/env python3
"""Reference implementation of the Tagalog productive rules.

This is the Python twin of `src/tgl.rs`: the same infixation, CV-
reduplication, mang- nasal assimilation, o->u raising, h-insertion and
the d->r / ni- allomorph variants. It exists so the lexicon miner
(mine_verbs.py) and the Rust engine derive forms the same way; the Rust
side is the source of truth for the shipped conjugator.
"""

import re

V = "aeiouAEIOU"
SLOTS = [
    "V;PFV;AGFOC", "V;IPFV;AGFOC", "V;AGFOC;LGSPEC1",
    "V;PFV;PFOC", "V;IPFV;PFOC", "V;PFOC;LGSPEC1",
]


def onset_len(w):
    i = 0
    while i < len(w) and w[i] not in V:
        i += 1
    return i


def redup(root):
    """Copy the first CV (or the first vowel, if vowel-initial)."""
    i = onset_len(root)
    if i >= len(root):
        return root
    if i == 0:
        return root[0] + root
    return root[0] + root[i] + root


def infix(w, af):
    """Insert -af- after the first consonant; prefix it if vowel-initial."""
    if w[0] in V:
        return af + w
    return w[0] + af + w[1:]


def raise_last_o(root):
    idx = root.rfind("o")
    return root if idx == -1 else root[:idx] + "u" + root[idx + 1:]


def suffix(root, s, h):
    r = raise_last_o(root)
    if r and r[-1] in V and h:
        return r + "h" + s
    return r + s


def _nasal(c):
    if c == "" or c in "pb":
        return "m"
    if c in "tdsn":
        return "n"
    return "ng"  # k, g, l, r, w, y, h, ng, vowel


def _mang_stem(root):
    c = root[0] if root[0] not in V else ""
    delete = c in "ptks"          # fortis onsets fuse into the nasal
    stem = root[1:] if delete else root
    return _nasal(c), stem


def gen(root, actor, patient, hins):
    """The single best surface form for each slot."""
    out = {}
    r = redup(root)
    if actor == "um":
        out["V;PFV;AGFOC"] = infix(root, "um")
        out["V;IPFV;AGFOC"] = infix(r, "um")
        out["V;AGFOC;LGSPEC1"] = r
    elif actor == "mag":
        h = "-" if root[0] in V else ""
        out["V;PFV;AGFOC"] = "nag" + h + root
        out["V;IPFV;AGFOC"] = "nag" + h + r
        out["V;AGFOC;LGSPEC1"] = "mag" + h + r
    elif actor == "mang":
        n, stem = _mang_stem(root)
        rs = redup(stem)
        h = "-" if stem[0] in V else ""
        out["V;PFV;AGFOC"] = "na" + n + h + stem
        out["V;IPFV;AGFOC"] = "na" + n + h + rs
        out["V;AGFOC;LGSPEC1"] = "ma" + n + h + rs
    elif actor == "ma":
        out["V;PFV;AGFOC"] = "na" + root
        out["V;IPFV;AGFOC"] = "na" + r
        out["V;AGFOC;LGSPEC1"] = "ma" + r
    if patient == "in":
        out["V;PFV;PFOC"] = infix(root, "in")
        out["V;IPFV;PFOC"] = infix(r, "in")
        out["V;PFOC;LGSPEC1"] = suffix(r, "in", hins)
    elif patient == "an":
        out["V;PFV;PFOC"] = infix(root, "in") + "an"
        out["V;IPFV;PFOC"] = infix(r, "in") + "an"
        out["V;PFOC;LGSPEC1"] = suffix(r, "an", hins)
    elif patient == "i":
        out["V;PFV;PFOC"] = "i" + infix(root, "in")
        out["V;IPFV;PFOC"] = "i" + infix(r, "in")
        out["V;PFOC;LGSPEC1"] = "i" + r
    return out


def variants_of(root, actor, patient, hins):
    """All accepted spellings per slot (adds the d->r and ni- allomorphs)."""
    base = gen(root, actor, patient, hins)
    out = {ft: {f} for ft, f in base.items()}
    r = redup(root)
    if root and root[0] == "d":
        for ft in list(out):
            for f in list(out[ft]):
                nf = re.sub(r"(?<=[aeiou])d(?=[aeiou])", "r", f, count=1)
                if nf != f:
                    out[ft].add(nf)
    if patient in ("in", "an") and root[0] not in V:
        suf = "an" if patient == "an" else ""
        out.setdefault("V;PFV;PFOC", set()).add("ni" + root + suf)
        out.setdefault("V;IPFV;PFOC", set()).add("ni" + r + suf)
    return out
