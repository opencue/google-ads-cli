# google-ads-cli (`gads`)

A single-file Python CLI for managing **any** Google Ads account from the
shell — including mutations the read-only MCPs can't do. Designed to be
driven by humans **and** by LLM agents like Claude Code.

- Zero dependencies (uses Python 3.11+ `tomllib` + `urllib`).
- Auth via `gcloud` ADC + a developer token env var. No OAuth flow.
- **Dry-run by default** on every mutation. `--apply` commits.
- Account-specific data lives in declarative **TOML packs**, not in code.
- Switch accounts with `--profile <name>` or `GADS_PROFILE` env var.

## Why

The official `google-ads-mcp` is read-only. Google's UI is slow for bulk
work. Google Ads Editor is desktop-only. This CLI fills the gap with a
focused mutation surface (asset extensions, headlines, brand campaigns,
asset-group splits) and pairs well with an LLM agent that needs to do real
work in an account end-to-end.

## Quick start

```bash
# 1. Auth
gcloud auth application-default login \
    --scopes=https://www.googleapis.com/auth/adwords,https://www.googleapis.com/auth/cloud-platform
export GOOGLE_ADS_DEVELOPER_TOKEN=your_dev_token_here

# 2. Drop the script somewhere on PATH
sudo install gads /usr/local/bin/

# 3. Configure your first account
mkdir -p ~/.config/gads/profiles
cp examples/profile.toml ~/.config/gads/profiles/myaccount.toml
$EDITOR ~/.config/gads/profiles/myaccount.toml   # set customer_id + packs_root

echo 'default_profile = "myaccount"' > ~/.config/gads/config.toml

# 4. Read commands work immediately
gads list-campaigns
gads stats --days 30
gads stats-vs-prev --days 14         # last 14d vs prior 14d
```

## Commands

### Read-only

| Command | What it does |
|---|---|
| `list-profiles` | List configured profiles |
| `list-campaigns` | Active campaigns + budgets + bid strategy |
| `list-conversions` | Conversion action inventory |
| `list-recommendations` | Pending Google recommendations |
| `list-assets <ag_id>` | Assets in an asset group |
| `list-account-assets` | Account-level sitelinks/callouts/snippets |
| `list-adgroups [--campaign X]` | Ad groups (filter optional) |
| `list-keywords [--campaign X]` | Keywords with match types + quality scores |
| `stats [--days N \| --start ... --end ...]` | Campaign-level KPIs |
| `stats-vs-prev --days N` | Last N days vs prior N days delta table |
| `gaql "<query>"` | Run any GAQL query (debug) |

### Mutating (dry-run unless `--apply`)

| Command | What it does |
|---|---|
| `add-sitelink <text> <url>` | Single sitelink at account scope |
| `add-callout <text>` | Single callout |
| `add-snippet <header> <val1> <val2> <val3>...` | Structured snippet (min 3 values) |
| `seed-account-assets --pack <file>` | Bulk: sitelinks + callouts + snippets from TOML |
| `add-text-assets <ag_id> --field-type {HEADLINE,DESCRIPTION,LONG_HEADLINE,BUSINESS_NAME} "<t1>"...` | Bulk text assets to an asset group |
| `add-business-name <ag_id> <name>` | BUSINESS_NAME (Brand-Guidelines-aware: links at campaign level if needed) |
| `upload-image <ag_id> <file> --field-type {LOGO,SQUARE_MARKETING_IMAGE,...}` | Upload image asset + link to group |
| `add-youtube-video <ag_id> <video_id_or_url>` | YouTube video asset + link |
| `replace-headlines <ag_id> --pack <file>` | Atomic: remove old headlines + add new pack |
| `create-brand-campaign --pack <file>` | Full Search campaign: budget + geo + ad group + keywords + RSA (PAUSED) |
| `split-asset-group <src_ag> --brand-packs <p1,p2,p3>` | Spawn N new asset groups from brand packs (copies images from source) |
| `delete-conversion <id>` | Delete a conversion action |
| `disable-url-expansion <campaign_id>` / `enable-url-expansion` | PMAX URL expansion toggle |
| `set-bid <campaign_id> --strategy <S> [--target <X>]` | Change bid strategy |
| `set-budget <campaign_id> --daily <amount>` | Change daily budget |

## Profiles + packs

A profile is a TOML file at `~/.config/gads/profiles/<name>.toml`. It tells
the CLI which Google Ads account you're operating on and where its pack
files live.

```toml
# ~/.config/gads/profiles/myaccount.toml
customer_id = "1234567890"
# login_customer_id = "9876543210"     # set if account is under an MCC

packs_root = "/home/user/projects/myaccount/gads-packs"

[brand]
name = "ACME Tires"
homepage = "https://www.example.com/"

[locale]
language = "en"
currency = "USD"
geo_targets = [2840]    # geoTargetConstants/2840 = United States
```

A pack is a TOML file with bulk content — sitelinks, headlines, brand
copy. The same `gads` binary runs against any pack. See [examples/](examples)
for the full set of pack templates.

```bash
# Switch accounts via flag or env var
gads --profile bigclient list-campaigns
GADS_PROFILE=bigclient gads stats --days 30

# Bulk seed from a pack file
gads seed-account-assets --pack account-assets --apply
#                              └── resolves to <packs_root>/account-assets.toml

# Pack name resolution order:
#   1. absolute path
#   2. <packs_root>/<arg>
#   3. <packs_root>/<arg>.toml
#   4. <packs_root>/brands/<arg>.toml
```

## Using `gads` from Claude Code

`gads` is designed to be driven by LLM agents. Drop a skill file at
`~/.claude/skills/gads-cli/SKILL.md` (template in `docs/`). After that,
Claude Code sessions auto-discover the CLI and pick it over walking
through the Ads UI step-by-step.

Common LLM-driven workflows:

- **"Audit the account"** → `gads list-campaigns && gads stats-vs-prev --days 14 && gads list-recommendations`
- **"Ship the audit recommendations"** → human writes packs, agent runs `gads seed-account-assets`, `gads replace-headlines`, `gads create-brand-campaign` with the packs
- **"Generate the weekly report"** → `gads stats-vs-prev --days 7` (markdown report command planned)

## Safety

- Every mutating command is **dry-run by default**. Add `--apply` to commit.
- Mutations log to stdout; expect to see resource IDs you can use to undo.
- Sitelink URLs are HEAD-checked before apply (`--skip-url-check` to bypass).
- Character widths follow Google's display-width counting (en-dash and minus
  count as 2), not Python `len()` — caught at validation time.

## Roadmap

- `gads cleanup-orphans` — find + delete unattached budgets/assets
- `gads set-status <campaign|adgroup|asset_group> {ENABLED|PAUSED}`
- `gads health` — one-command account dashboard
- `gads report --weekly` — markdown 2-week riport from `stats-vs-prev`
- `gads snapshot list/restore` — undo support
- `--json` / `--csv` output flags for piping
- Atomic multi-step via `googleAds:mutate` bulk endpoint (no more orphans on partial failure)
- Negative-keyword management
- `gads sync <yaml>` — declarative account state, drift detection

## Limitations

- Requires Python 3.11+ (`tomllib` is stdlib only from 3.11).
- Manager (MCC) account hierarchy: set `login_customer_id` in profile.
- No `delete-asset-group` / `delete-campaign` yet — destructive deletes still happen in the UI.
- Cannot upload images larger than 5,120,000 bytes (Google API limit).
- PMAX with Brand Guidelines: BUSINESS_NAME and LOGO link at campaign scope (handled automatically).

## Origin

This started as one-off automation for managing a single Google Ads
account during a paid audit engagement. Generalised into a portable CLI
once the value of "any agent can run this against any account" became
clear. Read-only command patterns ported from
[Bin-Huang/google-ads-open-cli](https://github.com/Bin-Huang/google-ads-open-cli).

## License

MIT — see [LICENSE](LICENSE).
