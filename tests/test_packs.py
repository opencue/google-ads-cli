"""Tests that every shipped example TOML loads cleanly. Catches handcraft
syntax errors before users hit them."""

import pathlib

import pytest


REPO_ROOT = pathlib.Path(__file__).resolve().parent.parent


def _all_example_tomls():
    return list((REPO_ROOT / "examples").rglob("*.toml"))


@pytest.mark.parametrize("path", _all_example_tomls(),
                         ids=lambda p: str(p.relative_to(REPO_ROOT)))
def test_example_toml_parses(path):
    import tomllib
    with open(path, "rb") as f:
        data = tomllib.load(f)
    assert isinstance(data, dict)


def test_example_profile_has_required_keys():
    import tomllib
    path = REPO_ROOT / "examples" / "profile.toml"
    with open(path, "rb") as f:
        data = tomllib.load(f)
    assert "customer_id" in data
    assert "packs_root" in data
    # placeholder must clearly NOT be a real customer id
    assert data["customer_id"].startswith("INSERT_"), \
        "Placeholder must scream 'replace me'"


def test_example_account_assets_has_required_arrays():
    import tomllib
    path = REPO_ROOT / "examples" / "packs" / "account-assets.toml"
    with open(path, "rb") as f:
        data = tomllib.load(f)
    assert isinstance(data.get("sitelinks", []), list)
    assert isinstance(data.get("callouts", []), list)
    assert isinstance(data.get("snippets", []), list)


def test_example_brand_packs_pass_width_check(gads):
    import tomllib
    for brand_toml in (REPO_ROOT / "examples" / "packs" / "brands").glob("*.toml"):
        with open(brand_toml, "rb") as f:
            data = tomllib.load(f)
        for h in data.get("headlines", []):
            assert gads._google_width(h) <= 30, \
                f"{brand_toml.name} headline > 30 google-width: {h!r}"
        for d in data.get("descriptions", []):
            assert gads._google_width(d) <= 90, \
                f"{brand_toml.name} description > 90 google-width: {d!r}"
        for lh in data.get("long_headlines", []):
            assert gads._google_width(lh) <= 90, \
                f"{brand_toml.name} long headline > 90 google-width: {lh!r}"


def test_example_brand_search_pack_passes_width_check(gads):
    import tomllib
    path = REPO_ROOT / "examples" / "packs" / "brand-search.toml"
    with open(path, "rb") as f:
        data = tomllib.load(f)
    for h in data.get("headlines", []):
        assert gads._google_width(h) <= 30
    for d in data.get("descriptions", []):
        assert gads._google_width(d) <= 90
