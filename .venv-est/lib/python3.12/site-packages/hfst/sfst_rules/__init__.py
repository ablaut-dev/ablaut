"""
functions for creating transducers implementing two-level rules
"""

from ..libhfst import coercion
from ..libhfst import deep_coercion
from ..libhfst import deep_restriction
from ..libhfst import deep_restriction_and_coercion
from ..libhfst import left_replace_down
from ..libhfst import left_replace_down_karttunen
from ..libhfst import left_replace_left
from ..libhfst import left_replace_right
from ..libhfst import left_replace_up
# from ..libhfst import left_replace_up  # TODO wrong name?
from ..libhfst import replace_down
from ..libhfst import replace_down_karttunen
from ..libhfst import replace_left
from ..libhfst import replace_right
from ..libhfst import replace_up
from ..libhfst import restriction
from ..libhfst import restriction_and_coercion
from ..libhfst import surface_coercion
from ..libhfst import surface_restriction
from ..libhfst import surface_restriction_and_coercion
from ..libhfst import two_level_if
from ..libhfst import two_level_if_and_only_if
from ..libhfst import two_level_only_if


__all__ = [
    "coercion",
    "deep_coercion",
    "deep_restriction",
    "deep_restriction_and_coercion",
    "left_replace_down",
    "left_replace_down_karttunen",
    "left_replace_left",
    "left_replace_right",
    "left_replace_up",
    "left_replace_up",
    "replace_down",
    "replace_down_karttunen",
    "replace_left",
    "replace_right",
    "replace_up",
    "restriction",
    "restriction_and_coercion",
    "surface_coercion",
    "surface_restriction",
    "surface_restriction_and_coercion",
    "two_level_if",
    "two_level_if_and_only_if",
    "two_level_only_if",
]
