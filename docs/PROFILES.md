# Profiles & packs reference

`gads` keeps account-specific state out of the CLI by using two layers:

- **Profile**: which account, what locale, where the packs live.
- **Pack**: the actual content — sitelinks, headlines, brand copy.

Both are TOML files.

## Profile

```
~/.config/gads/
├── config.toml                     # global: default profile
└── profiles/
    ├── myaccount.toml              # one TOML per Google Ads account
    └── otheraccount.toml
```

### `~/.config/gads/config.toml`

```toml
default_profile = "myaccount"
```

When you don't pass `--profile <name>` and `GADS_PROFILE` env var isn't
set, the CLI loads the `default_profile`.

### `~/.config/gads/profiles/<name>.toml`

```toml
customer_id = "INSERT_CUSTOMER_ID_HERE"
# login_customer_id = "INSERT_MCC_CUSTOMER_ID_HERE"   # only if under an MCC

packs_root = "/abs/path/to/pack/files"

[brand]
name = "ACME Tires"
homepage = "https://www.example.com/"

[locale]
language = "en"
currency = "USD"
geo_targets = [2840]                    # United States = 2840

[defaults]
campaign_status_on_create = "PAUSED"    # PAUSED until human review
url_check_before_apply = true           # HEAD-check sitelink URLs
```

**TOML scoping gotcha**: top-level scalars / arrays (`customer_id`,
`packs_root`, etc.) MUST appear BEFORE any `[section]` block. Otherwise
TOML parses them as fields of the most recent section. Same applies inside
pack files — keep `headlines = [...]` and other arrays above any
`[[table]]` blocks.

## Packs

A pack is a TOML file consumed by a specific command. The `--pack` arg
resolves to a pack file in this order:

1. Absolute path
2. `<packs_root>/<arg>`
3. `<packs_root>/<arg>.toml`
4. `<packs_root>/brands/<arg>.toml`

So `gads replace-headlines 6714 --pack brand-a` picks up
`<packs_root>/brands/brand-a.toml` automatically.

### account-assets.toml — `seed-account-assets`

| Field | Type | Notes |
|---|---|---|
| `callouts` (top-level) | array of strings | each ≤ 25 chars |
| `[[sitelinks]]` blocks | each has `text`, `url`, `d1`, `d2` | text ≤ 25, d1/d2 ≤ 35 |
| `[[snippets]]` blocks | each has `header`, `values` | header must be a Google enum (Brands, Types, Models, Styles, ...); ≥3 values, each ≤25 chars |

### brand-search.toml — `create-brand-campaign`

| Field | Type | Notes |
|---|---|---|
| `name` | string | campaign name |
| `final_url` | string | RSA destination |
| `daily_budget` | float | in account currency |
| `bidding_strategy` | string | `MAXIMIZE_CONVERSIONS`, `MAXIMIZE_CONVERSION_VALUE`, or `MANUAL_CPC` |
| `status_on_create` | string | `PAUSED` or `ENABLED` |
| `ad_group_bid` | int | fallback CPC bid in account currency |
| `headlines` | array of strings | RSA, 3-15 items, each ≤ 30 Google-width |
| `descriptions` | array of strings | RSA, 2-4 items, each ≤ 90 Google-width |
| `[[keywords]]` blocks | each has `text`, `match_type` | match_type: `EXACT`, `PHRASE`, `BROAD` |

Geo targeting comes from the profile's `[locale].geo_targets` array.

### brands/<brand>.toml — `replace-headlines` and `split-asset-group`

| Field | Type | Notes |
|---|---|---|
| `name` | string | display name for the brand |
| `asset_group_name` | string | what the new asset group will be called |
| `final_url` | string | the asset group's landing URL |
| `headlines` | array of strings | 3-15 items, each ≤ 30 Google-width |
| `descriptions` | array of strings | 2-5 items, each ≤ 90 Google-width |
| `long_headlines` | array of strings | 1-5 items, each ≤ 90 Google-width |

## Google-width vs Python `len()`

Google Ads counts en-dash (`–`), em-dash (`—`), and minus (`−`) as 2
characters each in headline/callout/sitelink-text limits. Python's
`len()` counts them as 1. The CLI uses Google's accounting, so a 31-char
Python string with one en-dash will be rejected at validation as "32 > 30".

Workaround: use ASCII hyphen-minus (`-`) which counts as 1 in both.

## Multiple accounts

Drop a new TOML into `~/.config/gads/profiles/`. Switch with:

```bash
gads --profile bigclient list-campaigns
GADS_PROFILE=bigclient gads stats --days 30
```

Or change the default once for all subsequent commands:

```bash
sed -i 's/default_profile = .*/default_profile = "bigclient"/' \
    ~/.config/gads/config.toml
```
