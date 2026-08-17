"""

Hfst Xerox-type rule functions and classes.

"""

from ..libhfst import after
from ..libhfst import before
from ..libhfst import replace
from ..libhfst import replace_epenthesis
from ..libhfst import replace_leftmost_longest_match
from ..libhfst import replace_leftmost_shortest_match
from ..libhfst import replace_rightmost_longest_match
from ..libhfst import replace_rightmost_shortest_match
from ..libhfst import Rule

# these functions had to be renamed in the swig interface
# to prevent name collision
from ..libhfst import xerox_replace_left as replace_left
from ..libhfst import xerox_restriction as restriction

from .. import libhfst

__all__ = [
    "after",
    "before",
    "replace",
    "replace_epenthesis",
    "replace_leftmost_longest_match",
    "replace_leftmost_shortest_match",
    "replace_rightmost_longest_match",
    "replace_rightmost_shortest_match",
    "Rule",
    "replace_left",
    "restriction",
]


class ReplaceType:
    """
    Replace type in Xerox-type rules.

    Attributes:

        REPL_UP:      Match contexts on input level
        REPL_DOWN:    Match contexts on output level
        REPL_RIGHT:   Match left contexts on input level and right contexts on output level
        REPL_LEFT:    Match left contexts on output level and right contexts on input level

    """  # noqa: E501
    REPL_UP = libhfst.REPL_UP
    REPL_DOWN = libhfst.REPL_DOWN
    REPL_RIGHT = libhfst.REPL_RIGHT
    REPL_LEFT = libhfst.REPL_LEFT
