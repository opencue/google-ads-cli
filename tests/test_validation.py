"""Tests for the validation helpers — character width counting + TOML
string escaping. These bugs cost us hours during development (en-dash
incident + JSON-escape close-call), so they're locked down here."""


# ─── _google_width ───────────────────────────────────────────────────────

def test_google_width_pure_ascii(gads):
    assert gads._google_width("Superway") == 8
    assert gads._google_width("") == 0


def test_google_width_counts_hungarian_diacritics_as_one(gads):
    # á/é/í/ó/ö/ü/ő/ű — all count as 1 in Google's accounting
    assert gads._google_width("kéréseik") == 8


def test_google_width_en_dash_counts_as_two(gads):
    # The bug that bit us: "Superway – akár −25%" was Python-len 30 but
    # Google rejected as "Too long" because en-dash + minus each count as 2.
    assert gads._google_width("a – b") == 6     # 1 + 1 + 2 + 1 + 1
    assert gads._google_width("a — b") == 6     # em-dash also counts 2
    assert gads._google_width("a − b") == 6     # minus sign too


def test_google_width_ascii_hyphen_counts_as_one(gads):
    # Workaround: use ASCII hyphen-minus to dodge the rule
    assert gads._google_width("a - b") == 5


# ─── _toml_escape ────────────────────────────────────────────────────────

def test_toml_escape_passthrough(gads):
    assert gads._toml_escape("hello") == "hello"


def test_toml_escape_quotes_doubled(gads):
    assert gads._toml_escape('she said "hi"') == 'she said \\"hi\\"'


def test_toml_escape_backslashes(gads):
    assert gads._toml_escape("c:\\path") == "c:\\\\path"


# ─── _parse_duration ─────────────────────────────────────────────────────

def test_parse_duration_units(gads):
    assert gads._parse_duration("30s") == 30
    assert gads._parse_duration("5m") == 300
    assert gads._parse_duration("2h") == 7200
    assert gads._parse_duration("7d") == 604800


def test_parse_duration_case_insensitive(gads):
    assert gads._parse_duration("7D") == 7 * 86400
    assert gads._parse_duration("  3h  ") == 3 * 3600


def test_parse_duration_bare_int_is_seconds(gads):
    assert gads._parse_duration("90") == 90


def test_parse_duration_rejects_bad(gads):
    import pytest
    with pytest.raises(ValueError):
        gads._parse_duration("forever")
    with pytest.raises(ValueError):
        gads._parse_duration("")
