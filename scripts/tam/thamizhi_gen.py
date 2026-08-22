#!/usr/bin/env python3
"""Generate the ThamizhiMorph oracle for the Tamil golden harness.

ThamizhiMorph (K. Sarveswaran, Apache-2.0) is a foma finite-state
transducer for literary Tamil, independent of Wiktionary — so its
agreement with kaikki is evidence rather than an echo. It ships as an
*analyser*; this script drives it as a *generator*.

How it works. Each verb class has its own FST (verb-c3, verb-c4,
verb-c11, verb-c12, verb-c62 and verb-c-rest for classes 1,2,5-10,
13-18). A class's paradigm is a fixed list of upper-side analysis
strings — `<root>+verb+fin+sim+strong+past=த்+3sgm=ஆன்` — that is the
same for every root in the class; the per-root surface (செய்தான்) is
produced by the FST's rewrite rules. So for every kaikki lemma we
prepend the lemma to each class's analysis strings and apply the FST
downward (`flookup -i`); a lemma that is not in a class yields `+?` and
is dropped. A lemma can be listed in more than one class (padi is both
weak and strong), and ThamizhiMorph over-generates for suppletives
(it derives *vaakiraen* for வா); both are harmless here — the harness
scores only the slots where this oracle and kaikki agree, so a wrong
ThamizhiMorph form simply drops out of the intersection.

The analysis strings are read from the lexc sources (extracted from
foma/ThamizhiMorph-Verbs.zip by scripts/tam/fetch_thamizhi.sh); their
`=morpheme` realisations are kept for generation and stripped for the
feature mapping. caus/euph/opt/neg/sandhi variants are out of scope.

Output: `lemma <tab> form <tab> features` on the schema shared with
scripts/tam/kaikki_to_tsv.py.
"""
import os
import re
import subprocess
import sys

# PNG suffix tag -> schema token (2sgh, 3sge are not in the shared schema)
PNG = {
    "1sg": "1SG", "1pl": "1PL", "2sg": "2SG", "2pl": "2PL",
    "3sgm": "3SGM", "3sgf": "3SGF", "3sghe": "3SGH", "3sgn": "3SGN",
    "3ple": "3PLE", "3pln": "3PLN",
}
TENSE = {"past": "PST", "pres": "PRS", "fut": "FUT"}


def feature(skeleton):
    """Map a realisation-stripped analysis skeleton to a feature bundle.

    `skeleton` is the tag string minus the lemma and minus every
    `=morpheme`, e.g. `+verb+fin+sim+strong+past+3sgm`.
    """
    t = skeleton.strip("+").split("+")
    if t[:1] != ["verb"]:
        return None
    t = t[1:]
    # Anything caus/euph/neg/optative/complex/sandhi is out of scope.
    if {"caus", "euph", "neg", "opt", "complex"} & set(t):
        return None
    if any(x.startswith("sandhi") or x in ("moodpart", "negpart") for x in t):
        return None
    if t[:2] == ["fin", "sim"]:
        rest = t[2:]
        if rest[:1] == ["imp"]:
            if rest[1:] == ["2sg"]:
                return "V;IMP;SG"
            if rest[1:] == ["2pl"]:
                return "V;IMP;PL"
            return None
        # strong/weak, tense, png
        rest = [x for x in rest if x not in ("strong", "weak")]
        if len(rest) == 2 and rest[0] in TENSE and rest[1] in PNG:
            feat = f"V;{TENSE[rest[0]]};{PNG[rest[1]]}"
            # The future neuter plural is out of the shared cell set: the
            # two oracles disagree by convention rather than by error —
            # kaikki has a distinct -வன/-ப்பன plural (செய்வன), while
            # ThamizhiMorph reuses the -உம் neuter singular for it.
            return None if feat == "V;FUT;3PLN" else feat
        return None
    if t[:2] == ["nonfin", "sim"]:
        rest = t[2:]
        if rest == ["inf"]:
            return "V;INF"
        if rest == ["vpart"]:
            return "V;CVB"
        if rest == ["con"]:
            return "V;COND"
        if rest == ["past", "adjpart"]:
            return "V;PTCP;PST"
        if rest == ["pres", "adjpart"]:
            return "V;PTCP;PRS"
        if rest == ["futANDadjpart"]:
            return "V;PTCP;FUT"
    return None


def tagsuffixes(lexc_path):
    """The class's terminal analysis strings whose skeleton maps to a
    schema slot, as (analysis_suffix, feature) pairs."""
    out = []
    for line in open(lexc_path, encoding="utf-8"):
        line = line.rstrip("\n")
        if "#;" not in line or ":" not in line:
            continue
        upper = line.split(":", 1)[0].strip().replace("%", "")
        if not upper.startswith("+verb"):
            continue
        skeleton = re.sub(r"=[^+]*", "", upper)
        feat = feature(skeleton)
        if not feat:
            continue
        # The honorific 3sghe realises as both ஆர் and ஆர்கள்; kaikki's
        # honorific singular is ஆர், and the ஆர்கள் spelling is already
        # covered by 3ple, so keep only the ஆர் realisation for 3SGH.
        if feat.endswith("3SGH") and not upper.endswith("=ஆர்"):
            continue
        out.append((upper, feat))
    return out


# fst basename -> lexc basename
CLASSES = [
    ("verb-c3", "ThamizhiVerbs-C3.lexc"),
    ("verb-c4", "ThamizhiVerbs-C4.lexc"),
    ("verb-c11", "ThamizhiVerbs-C11.lexc"),
    ("verb-c12", "ThamizhiVerbs-C12.lexc"),
    ("verb-c62", "ThamizhiVerbs-C62.lexc"),
    ("verb-c-rest", "ThamizhiVerbs-otherthan-3-4-62-11-12.lexc"),
]


# Tamil dependent vowel signs; a pulli (virama) immediately followed by
# one is a morpheme-boundary artifact of the raw FST output (அகழ்ுங்கள்)
# and the pulli must drop (அகழுங்கள்) — a consonant cannot carry both.
VOWEL_SIGNS = "ாிீுூெேைொோௌ"
PULLI = "்"


def clean(surface):
    out = []
    for ch in surface:
        if ch in VOWEL_SIGNS and out and out[-1] == PULLI:
            out.pop()
        out.append(ch)
    return "".join(out)


FRONT = "ிீெேை"   # front vowel signs: take a ய glide
BACK = "ாுூொோ"     # back vowel signs: take a வ glide


def glide_uu(surface):
    """Insert the missing glide before a suffix -உங்கள் that the FST
    glued straight onto a vowel-final root (அணிுங்கள் → அணியுங்கள்)."""
    marker = "ு" + "ங்கள்"
    i = surface.find(marker)
    if i > 0 and surface[i - 1] in FRONT + BACK:
        glide = "ய" if surface[i - 1] in FRONT else "வ"
        return surface[:i] + glide + surface[i:]
    return surface


def generate(fst_path, lemmas, tags):
    """Apply the FST downward for every lemma × analysis-suffix."""
    inp = "".join(f"{lem}{suf}\n" for lem in lemmas for suf, _ in tags)
    p = subprocess.run(["flookup", "-i", fst_path], input=inp,
                       capture_output=True, text=True)
    feat_of = {suf: feat for suf, feat in tags}
    rows = set()
    for line in p.stdout.splitlines():
        if not line.strip() or "+?" in line:
            continue
        analysis, surface = line.split("\t")
        if not surface or surface == "0":
            continue
        # Recover lemma and suffix: the suffix begins at the first `+verb`.
        cut = analysis.find("+verb")
        lemma, suf = analysis[:cut], analysis[cut:]
        feat = feat_of.get(suf)
        if feat:
            surface = clean(surface)
            # ThamizhiMorph mechanical slip: for the -இன் past class it
            # keeps the -ன் before the அ-initial neuter/adjectival
            # endings (அகற்றினது, அகற்றின), where literary Tamil and
            # kaikki take the -இ stem with a ய glide (அகற்றியது,
            # அகற்றிய); the neuter plural is the bare stem (அகற்றின).
            if "past=இன்" in analysis:
                if feat == "V;PTCP;PST" and surface.endswith("ன"):
                    surface = surface[:-1] + "ய"
                elif feat == "V;PST;3SGN" and surface.endswith("னது"):
                    surface = surface[:-3] + "யது"
                elif feat == "V;PST;3PLN" and surface.endswith("னன"):
                    surface = surface[:-1]
            # ThamizhiMorph mechanical slip: on a vowel-final root the
            # imperative plural -உங்கள் is glued straight onto the vowel
            # (அணிுங்கள், அண்ணாுங்கள்) with no glide; literary Tamil and
            # kaikki insert ய after a front vowel (அணியுங்கள்) and வ after
            # a back vowel (அண்ணாவுங்கள்).
            if feat == "V;IMP;PL":
                surface = glide_uu(surface)
            rows.add((lemma, surface, feat))
            # The present marker has two interchangeable allomorphs,
            # கின்ற and கிற; ThamizhiMorph spells the present adjectival
            # participle with கின்ற only, kaikki with கிற. Both are
            # standard, so emit both (the finite present, where
            # ThamizhiMorph itself gives both, already carries the pair).
            if feat == "V;PTCP;PRS" and "கின்ற" in surface:
                rows.add((lemma, surface.replace("கின்ற", "கிற"), feat))
    return rows


def main(fst_dir, lexc_dir, lemmas_path):
    lemmas = [l.strip() for l in open(lemmas_path, encoding="utf-8") if l.strip()]
    rows = set()
    for fst_name, lexc_name in CLASSES:
        tags = tagsuffixes(os.path.join(lexc_dir, lexc_name))
        rows |= generate(os.path.join(fst_dir, fst_name + ".fst"), lemmas, tags)
    for lemma, form, feat in sorted(rows):
        print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    # argv: <fst_dir> <lexc_dir> <lemmas_file>
    main(sys.argv[1], sys.argv[2], sys.argv[3])
