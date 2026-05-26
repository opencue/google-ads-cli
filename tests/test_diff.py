"""Tests for _compute_diff — the heart of plan/sync/snapshot-restore.

Each test monkeypatches `_live_campaigns_state` and
`_live_account_assets_state` so we never hit the real API."""


def _patch_live(monkeypatch, gads, campaigns=None, assets=None):
    monkeypatch.setattr(gads, "_live_campaigns_state",
                        lambda cid: campaigns or [])
    monkeypatch.setattr(gads, "_live_account_assets_state",
                        lambda cid: assets or {"sitelinks": [], "callouts": [], "snippets": []})


# ─── Campaigns ───────────────────────────────────────────────────────────

def test_no_drift_returns_empty_list(monkeypatch, gads):
    _patch_live(monkeypatch, gads, campaigns=[
        {"id": "1", "name": "A", "status": "ENABLED",
         "channel_type": "SEARCH", "budget_daily": 100},
    ])
    state = {
        "account": {"customer_id": "x"},
        "campaigns": [
            {"id": "1", "name": "A", "status": "ENABLED", "budget_daily": 100},
        ],
    }
    assert gads._compute_diff(state, "x") == []


def test_status_change_detected(monkeypatch, gads):
    _patch_live(monkeypatch, gads, campaigns=[
        {"id": "1", "name": "A", "status": "PAUSED",
         "channel_type": "SEARCH", "budget_daily": 100},
    ])
    state = {
        "account": {"customer_id": "x"},
        "campaigns": [
            {"id": "1", "name": "A", "status": "ENABLED", "budget_daily": 100},
        ],
    }
    changes = gads._compute_diff(state, "x")
    assert len(changes) == 1
    ch = changes[0]
    assert ch["operation"] == "update"
    assert ch["field"] == "status"
    assert ch["current"] == "PAUSED"
    assert ch["desired"] == "ENABLED"


def test_budget_change_detected(monkeypatch, gads):
    _patch_live(monkeypatch, gads, campaigns=[
        {"id": "1", "name": "A", "status": "ENABLED",
         "channel_type": "SEARCH", "budget_daily": 100},
    ])
    state = {
        "account": {"customer_id": "x"},
        "campaigns": [
            {"id": "1", "name": "A", "status": "ENABLED", "budget_daily": 250},
        ],
    }
    changes = gads._compute_diff(state, "x")
    fields_changed = [c["field"] for c in changes if c["operation"] == "update"]
    assert fields_changed == ["budget_daily"]


def test_campaign_only_in_state_file_is_flagged(monkeypatch, gads):
    _patch_live(monkeypatch, gads, campaigns=[])
    state = {
        "account": {"customer_id": "x"},
        "campaigns": [{"id": "1", "name": "Ghost",
                       "status": "ENABLED", "budget_daily": 100}],
    }
    changes = gads._compute_diff(state, "x")
    assert any(c["operation"] == "missing_in_live" for c in changes)


def test_campaign_only_live_is_flagged(monkeypatch, gads):
    _patch_live(monkeypatch, gads, campaigns=[
        {"id": "9", "name": "Live Only", "status": "ENABLED",
         "channel_type": "SEARCH", "budget_daily": 100},
    ])
    state = {"account": {"customer_id": "x"}, "campaigns": []}
    changes = gads._compute_diff(state, "x")
    assert any(c["operation"] == "only_in_live" for c in changes)


# ─── Account-level assets ────────────────────────────────────────────────

def test_sitelink_create_via_no_id(monkeypatch, gads):
    _patch_live(monkeypatch, gads, assets={
        "sitelinks": [], "callouts": [], "snippets": [],
    })
    state = {
        "account": {"customer_id": "x"},
        "sitelinks": [
            {"text": "New", "url": "https://example.com/new",
             "d1": "desc1", "d2": "desc2"},
        ],
    }
    changes = gads._compute_diff(state, "x")
    assert len(changes) == 1
    assert changes[0]["kind"] == "sitelink"
    assert changes[0]["operation"] == "create"
    assert changes[0]["data"]["text"] == "New"


def test_sitelink_prune_when_live_extra(monkeypatch, gads):
    _patch_live(monkeypatch, gads, assets={
        "sitelinks": [
            {"id": "777", "text": "Old", "url": "https://example.com/old",
             "d1": "", "d2": ""},
        ],
        "callouts": [], "snippets": [],
    })
    state = {
        "account": {"customer_id": "x"},
        "sitelinks": [],   # explicit empty → "I want NO sitelinks"
    }
    changes = gads._compute_diff(state, "x")
    assert len(changes) == 1
    assert changes[0]["operation"] == "prune"
    assert changes[0]["data"]["id"] == "777"


def test_sitelink_not_managed_when_key_omitted(monkeypatch, gads):
    """If the state file doesn't mention `sitelinks` at all, we don't
    diff them — that's how users opt out of managing a kind."""
    _patch_live(monkeypatch, gads, assets={
        "sitelinks": [
            {"id": "777", "text": "Old", "url": "https://example.com/old",
             "d1": "", "d2": ""},
        ],
        "callouts": [], "snippets": [],
    })
    state = {"account": {"customer_id": "x"}}  # no `sitelinks` key
    changes = gads._compute_diff(state, "x")
    assert changes == []


def test_callout_create(monkeypatch, gads):
    _patch_live(monkeypatch, gads, assets={
        "sitelinks": [], "callouts": [], "snippets": [],
    })
    state = {
        "account": {"customer_id": "x"},
        "callouts": [{"text": "New callout"}],
    }
    changes = gads._compute_diff(state, "x")
    assert len(changes) == 1
    assert changes[0]["kind"] == "callout"
    assert changes[0]["operation"] == "create"


def test_snippet_create_with_values(monkeypatch, gads):
    _patch_live(monkeypatch, gads, assets={
        "sitelinks": [], "callouts": [], "snippets": [],
    })
    state = {
        "account": {"customer_id": "x"},
        "snippets": [{"header": "Brands", "values": ["A", "B", "C"]}],
    }
    changes = gads._compute_diff(state, "x")
    assert len(changes) == 1
    assert changes[0]["data"]["header"] == "Brands"
    assert changes[0]["data"]["values"] == ["A", "B", "C"]


# ─── v3: keywords, asset-group text assets, RSAs ─────────────────────────

def _patch_v3(monkeypatch, gads, **overrides):
    """Patch all v3 collectors to return the given lists (default empty)."""
    monkeypatch.setattr(gads, "_live_campaigns_state", lambda cid: overrides.get("campaigns", []))
    monkeypatch.setattr(gads, "_live_account_assets_state",
                        lambda cid: {"sitelinks": [], "callouts": [], "snippets": []})
    monkeypatch.setattr(gads, "_live_ad_groups_state",
                        lambda cid: overrides.get("ad_groups", []))
    monkeypatch.setattr(gads, "_live_keywords_state",
                        lambda cid: overrides.get("keywords", []))
    monkeypatch.setattr(gads, "_live_asset_group_text_assets_state",
                        lambda cid: overrides.get("asset_group_text_assets", []))
    monkeypatch.setattr(gads, "_live_responsive_search_ads_state",
                        lambda cid: overrides.get("responsive_search_ads", []))


def test_keyword_create_via_no_id(monkeypatch, gads):
    _patch_v3(monkeypatch, gads)
    state = {
        "account": {"customer_id": "x"},
        "keywords": [
            {"ad_group_id": "999", "text": "best widgets", "match_type": "EXACT"},
        ],
    }
    changes = gads._compute_diff(state, "x")
    assert len(changes) == 1
    assert changes[0]["kind"] == "keyword"
    assert changes[0]["operation"] == "create"
    assert changes[0]["data"]["text"] == "best widgets"


def test_keyword_prune_when_live_extra(monkeypatch, gads):
    _patch_v3(monkeypatch, gads, keywords=[
        {"id": "k1", "ad_group_id": "999", "text": "stale", "match_type": "PHRASE", "status": "ENABLED"},
    ])
    state = {"account": {"customer_id": "x"}, "keywords": []}
    changes = gads._compute_diff(state, "x")
    assert len(changes) == 1
    assert changes[0]["operation"] == "prune"
    assert changes[0]["data"]["id"] == "k1"


def test_asset_group_text_asset_create(monkeypatch, gads):
    _patch_v3(monkeypatch, gads)
    state = {
        "account": {"customer_id": "x"},
        "asset_group_text_assets": [
            {"asset_group_id": "ag1", "field_type": "HEADLINE",
             "text": "New brand headline"},
        ],
    }
    changes = gads._compute_diff(state, "x")
    assert len(changes) == 1
    assert changes[0]["kind"] == "asset_group_text_asset"
    assert changes[0]["operation"] == "create"
    assert changes[0]["data"]["field_type"] == "HEADLINE"


def test_rsa_create_with_inline_copy(monkeypatch, gads):
    _patch_v3(monkeypatch, gads)
    state = {
        "account": {"customer_id": "x"},
        "responsive_search_ads": [{
            "ad_group_id": "999",
            "status": "ENABLED",
            "final_url": "https://example.com/",
            "headlines": ["H1", "H2", "H3"],
            "descriptions": ["D1", "D2"],
        }],
    }
    changes = gads._compute_diff(state, "x")
    assert len(changes) == 1
    assert changes[0]["kind"] == "responsive_search_ad"
    assert changes[0]["data"]["headlines"] == ["H1", "H2", "H3"]


def test_v3_keys_omitted_means_unmanaged(monkeypatch, gads):
    """Same opt-out rule as account assets: omitting `keywords` from the
    state file means we don't manage them — even if live has them."""
    _patch_v3(monkeypatch, gads, keywords=[
        {"id": "k1", "ad_group_id": "999", "text": "live", "match_type": "EXACT", "status": "ENABLED"},
    ])
    state = {"account": {"customer_id": "x"}}    # no `keywords` key
    changes = gads._compute_diff(state, "x")
    assert changes == []
