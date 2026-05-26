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
