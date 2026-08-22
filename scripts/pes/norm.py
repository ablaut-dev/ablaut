#!/usr/bin/env python3
"""Perso-Arabic normalization — the Python twin of `src/perso_arabic.rs`.

Both must agree character-for-character: the engine emits normalized
Persian and these adapters normalize the oracle forms the same way, so
they compare equal. See the Rust module for the rationale.
"""

# Zero-width joiners + directional/bidi control marks.
_ZERO_WIDTH = "‌‍‎‏؜"
# Arabic → Perso-Arabic letter folding (conservative; آ madda is kept).
_FOLD = {
    "ك": "ک",  # ك kaf → ک keheh
    "ي": "ی",  # ي yeh → ی
    "ى": "ی",  # ى alef maqṣūra → ی
    "ة": "ه",  # ة teh marbūṭa → ه
    "أ": "ا",  # أ → ا
    "إ": "ا",  # إ → ا
    "ٱ": "ا",  # ٱ → ا
}


def _is_diacritic(c):
    o = ord(c)
    return 0x064B <= o <= 0x0652 or o == 0x0670 or o == 0x0640


def normalize(s):
    out = []
    for c in s:
        if c in _ZERO_WIDTH or _is_diacritic(c):
            continue
        out.append(_FOLD.get(c, c))
    return "".join(out)
