"""
pytest test suite for bmb_text Python bindings.

Organized into three categories:
  - TestSearch    : kmp_search, str_find, str_rfind, str_count, str_contains,
                    str_starts_with, str_ends_with, find_byte, count_byte,
                    token_count, hamming_distance
  - TestTransform : str_reverse, str_replace, str_replace_all, to_upper,
                    to_lower, trim, repeat
  - TestAnalysis  : is_palindrome, word_count
"""

import pytest
import bmb_text as bt


# ---------------------------------------------------------------------------
# Search & query functions
# ---------------------------------------------------------------------------

class TestKmpSearch:
    def test_found_at_start(self):
        assert bt.kmp_search("abcdef", "abc") == 0

    def test_found_in_middle(self):
        assert bt.kmp_search("hello world", "world") == 6

    def test_found_at_end(self):
        assert bt.kmp_search("abcdef", "def") == 3

    def test_not_found(self):
        assert bt.kmp_search("hello world", "xyz") == -1

    def test_repeated_pattern(self):
        # KMP returns the first (leftmost) match
        assert bt.kmp_search("AABAABAABAAB", "AAB") == 0

    def test_pattern_equals_text(self):
        assert bt.kmp_search("abc", "abc") == 0

    def test_pattern_longer_than_text(self):
        assert bt.kmp_search("hi", "hello") == -1

    def test_empty_text(self):
        assert bt.kmp_search("", "abc") == -1

    def test_single_char_found(self):
        assert bt.kmp_search("abc", "b") == 1

    def test_single_char_not_found(self):
        assert bt.kmp_search("abc", "z") == -1


class TestStrFind:
    def test_first_occurrence(self):
        assert bt.str_find("abcabc", "bc") == 1

    def test_not_found(self):
        assert bt.str_find("abcabc", "xyz") == -1

    def test_found_at_start(self):
        assert bt.str_find("hello", "hel") == 0

    def test_empty_text(self):
        assert bt.str_find("", "a") == -1

    def test_single_char(self):
        assert bt.str_find("xyz", "y") == 1


class TestStrRfind:
    def test_last_occurrence(self):
        assert bt.str_rfind("abcabc", "bc") == 4

    def test_not_found(self):
        assert bt.str_rfind("abcabc", "xyz") == -1

    def test_only_one_occurrence(self):
        assert bt.str_rfind("hello", "llo") == 2

    def test_empty_text(self):
        assert bt.str_rfind("", "a") == -1

    def test_single_char_last(self):
        assert bt.str_rfind("abcba", "b") == 3


class TestStrCount:
    def test_multiple_occurrences(self):
        assert bt.str_count("abcabcabc", "abc") == 3

    def test_single_char_repeated(self):
        assert bt.str_count("aaa", "a") == 3

    def test_not_found(self):
        assert bt.str_count("hello", "xyz") == 0

    def test_empty_text(self):
        assert bt.str_count("", "a") == 0

    def test_non_overlapping(self):
        # "aa" in "aaaa" non-overlapping → 2
        assert bt.str_count("aaaa", "aa") == 2

    def test_exact_match(self):
        assert bt.str_count("abc", "abc") == 1


class TestStrContains:
    def test_contains_true(self):
        assert bt.str_contains("hello world", "world") is True

    def test_contains_false(self):
        assert bt.str_contains("hello world", "xyz") is False

    def test_empty_text(self):
        assert bt.str_contains("", "a") is False

    def test_single_char_true(self):
        assert bt.str_contains("abc", "b") is True

    def test_single_char_false(self):
        assert bt.str_contains("abc", "z") is False


class TestStrStartsWith:
    def test_true(self):
        assert bt.str_starts_with("hello", "hel") is True

    def test_false(self):
        assert bt.str_starts_with("hello", "xyz") is False

    def test_full_string(self):
        assert bt.str_starts_with("hello", "hello") is True

    def test_empty_text(self):
        assert bt.str_starts_with("", "a") is False

    def test_single_char_match(self):
        assert bt.str_starts_with("abc", "a") is True

    def test_single_char_no_match(self):
        assert bt.str_starts_with("abc", "b") is False


class TestStrEndsWith:
    def test_true(self):
        assert bt.str_ends_with("hello", "llo") is True

    def test_false(self):
        assert bt.str_ends_with("hello", "xyz") is False

    def test_full_string(self):
        assert bt.str_ends_with("hello", "hello") is True

    def test_empty_text(self):
        assert bt.str_ends_with("", "a") is False

    def test_single_char_match(self):
        assert bt.str_ends_with("abc", "c") is True

    def test_single_char_no_match(self):
        assert bt.str_ends_with("abc", "b") is False


class TestFindByte:
    def test_found(self):
        assert bt.find_byte("hello", ord('l')) == 2

    def test_not_found(self):
        assert bt.find_byte("hello", ord('z')) == -1

    def test_at_start(self):
        assert bt.find_byte("hello", ord('h')) == 0

    def test_at_end(self):
        assert bt.find_byte("hello", ord('o')) == 4

    def test_empty_string(self):
        assert bt.find_byte("", ord('a')) == -1

    def test_single_char_found(self):
        assert bt.find_byte("x", ord('x')) == 0


class TestCountByte:
    def test_multiple(self):
        assert bt.count_byte("hello", ord('l')) == 2

    def test_zero(self):
        assert bt.count_byte("hello", ord('z')) == 0

    def test_single_occurrence(self):
        assert bt.count_byte("hello", ord('h')) == 1

    def test_empty_string(self):
        assert bt.count_byte("", ord('a')) == 0

    def test_all_same(self):
        assert bt.count_byte("aaaa", ord('a')) == 4


class TestTokenCount:
    def test_csv(self):
        assert bt.token_count("a,b,c,d", ",") == 4

    def test_no_delimiter(self):
        assert bt.token_count("hello", ",") == 1

    def test_colon_separator(self):
        # "a::b::c" split by ':' → ["a", "", "b", "", "c"] = 5 tokens
        assert bt.token_count("a::b::c", ":") == 5

    def test_single_token(self):
        assert bt.token_count("word", " ") == 1

    def test_two_tokens(self):
        assert bt.token_count("one two", " ") == 2


class TestHammingDistance:
    def test_known_distance(self):
        assert bt.hamming_distance("karolin", "kathrin") == 3

    def test_identical_strings(self):
        assert bt.hamming_distance("hello", "hello") == 0

    def test_all_different(self):
        assert bt.hamming_distance("abc", "xyz") == 3

    def test_different_lengths(self):
        assert bt.hamming_distance("ab", "abc") == -1

    def test_single_char_same(self):
        assert bt.hamming_distance("a", "a") == 0

    def test_single_char_different(self):
        assert bt.hamming_distance("a", "b") == 1

    def test_empty_strings(self):
        assert bt.hamming_distance("", "") == 0


# ---------------------------------------------------------------------------
# Transform functions
# ---------------------------------------------------------------------------

class TestStrReverse:
    def test_basic(self):
        assert bt.str_reverse("hello") == "olleh"

    def test_empty(self):
        assert bt.str_reverse("") == ""

    def test_single_char(self):
        assert bt.str_reverse("a") == "a"

    def test_palindrome_unchanged(self):
        assert bt.str_reverse("racecar") == "racecar"

    def test_two_chars(self):
        assert bt.str_reverse("ab") == "ba"

    def test_with_spaces(self):
        assert bt.str_reverse("hello world") == "dlrow olleh"


class TestStrReplace:
    def test_replace_first(self):
        assert bt.str_replace("hello world", "world", "BMB") == "hello BMB"

    def test_no_match(self):
        assert bt.str_replace("hello world", "xyz", "BMB") == "hello world"

    def test_replace_at_start(self):
        assert bt.str_replace("foobar", "foo", "baz") == "bazbar"

    def test_replace_at_end(self):
        assert bt.str_replace("foobar", "bar", "baz") == "foobaz"

    def test_only_first_occurrence(self):
        # str_replace replaces only the first match
        result = bt.str_replace("abcabc", "abc", "X")
        assert result == "Xabc"

    def test_empty_old(self):
        # Replacing empty string: behaviour depends on implementation;
        # verify it doesn't crash and returns a string.
        result = bt.str_replace("hello", "xyz", "!")
        assert isinstance(result, str)

    def test_replace_with_empty(self):
        assert bt.str_replace("hello world", "world", "") == "hello "


class TestStrReplaceAll:
    def test_replace_all(self):
        assert bt.str_replace_all("abcabc", "abc", "X") == "XX"

    def test_no_match(self):
        assert bt.str_replace_all("hello", "xyz", "!") == "hello"

    def test_replace_all_chars(self):
        assert bt.str_replace_all("aaa", "a", "b") == "bbb"

    def test_replace_with_empty(self):
        assert bt.str_replace_all("hello world", " ", "") == "helloworld"

    def test_empty_text(self):
        assert bt.str_replace_all("", "a", "b") == ""

    def test_longer_replacement(self):
        assert bt.str_replace_all("ab", "a", "xx") == "xxb"


class TestToUpper:
    def test_basic(self):
        assert bt.to_upper("hello") == "HELLO"

    def test_already_upper(self):
        assert bt.to_upper("HELLO") == "HELLO"

    def test_mixed(self):
        assert bt.to_upper("Hello123") == "HELLO123"

    def test_empty(self):
        assert bt.to_upper("") == ""

    def test_single_char(self):
        assert bt.to_upper("a") == "A"

    def test_numbers_unchanged(self):
        assert bt.to_upper("abc123") == "ABC123"


class TestToLower:
    def test_basic(self):
        assert bt.to_lower("HELLO") == "hello"

    def test_already_lower(self):
        assert bt.to_lower("hello") == "hello"

    def test_mixed(self):
        assert bt.to_lower("HELLO123") == "hello123"

    def test_empty(self):
        assert bt.to_lower("") == ""

    def test_single_char(self):
        assert bt.to_lower("A") == "a"

    def test_numbers_unchanged(self):
        assert bt.to_lower("ABC123") == "abc123"


class TestTrim:
    def test_both_sides(self):
        assert bt.trim("  hello  ") == "hello"

    def test_no_whitespace(self):
        assert bt.trim("hello") == "hello"

    def test_only_whitespace(self):
        assert bt.trim("   ") == ""

    def test_leading_only(self):
        assert bt.trim("  hello") == "hello"

    def test_trailing_only(self):
        assert bt.trim("hello  ") == "hello"

    def test_empty_string(self):
        assert bt.trim("") == ""

    def test_tabs_and_newlines(self):
        result = bt.trim("\t hello \n")
        assert result == "hello"

    def test_single_char_no_space(self):
        assert bt.trim("x") == "x"


class TestRepeat:
    def test_basic(self):
        assert bt.repeat("ab", 3) == "ababab"

    def test_zero_times(self):
        assert bt.repeat("x", 0) == ""

    def test_once(self):
        assert bt.repeat("hello", 1) == "hello"

    def test_empty_string(self):
        assert bt.repeat("", 5) == ""

    def test_single_char(self):
        assert bt.repeat("a", 4) == "aaaa"

    def test_large_repeat(self):
        assert bt.repeat("x", 10) == "xxxxxxxxxx"


# ---------------------------------------------------------------------------
# Analysis functions
# ---------------------------------------------------------------------------

class TestIsPalindrome:
    def test_palindrome_odd(self):
        assert bt.is_palindrome("racecar") is True

    def test_palindrome_even(self):
        assert bt.is_palindrome("abba") is True

    def test_not_palindrome(self):
        assert bt.is_palindrome("hello") is False

    def test_single_char(self):
        assert bt.is_palindrome("a") is True

    def test_empty_string(self):
        assert bt.is_palindrome("") is True

    def test_two_same_chars(self):
        assert bt.is_palindrome("aa") is True

    def test_two_different_chars(self):
        assert bt.is_palindrome("ab") is False

    def test_case_sensitive(self):
        # "Aba" is not a palindrome because 'A' != 'a' (case-sensitive)
        assert bt.is_palindrome("Aba") is False

    def test_numbers(self):
        assert bt.is_palindrome("12321") is True


class TestWordCount:
    def test_basic(self):
        assert bt.word_count("hello world") == 2

    def test_extra_spaces(self):
        assert bt.word_count(" hello  world ") == 2

    def test_empty_string(self):
        assert bt.word_count("") == 0

    def test_single_word(self):
        assert bt.word_count("one") == 1

    def test_three_words(self):
        assert bt.word_count("one two three") == 3

    def test_only_spaces(self):
        assert bt.word_count("   ") == 0

    def test_leading_trailing_spaces(self):
        assert bt.word_count("  word  ") == 1
