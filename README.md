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
# 1. Install the script
sudo install gads /usr/local/bin/

# 2. Auth (one-time, ~2 min)
gcloud auth application-default login \
    --scopes=https://www.googleapis.com/auth/adwords,https://www.googleapis.com/auth/cloud-platform
export GOOGLE_ADS_DEVELOPER_TOKEN=your_dev_token_here    # add to .bashrc/.zshrc

# 3. Interactive wizard — creates ~/.config/gads/profiles/<name>.toml
gads init

# 4. Verify everything works
gads auth-check

# 5. Now read commands work immediately
gads list-campaigns
gads stats --days 30
gads stats-vs-prev --days 14         # last 14d vs prior 14d
```

Skipping the wizard? Copy `examples/profile.toml` to
`~/.config/gads/profiles/<name>.toml` by hand and edit. Then
`echo 'default_profile = "<name>"' > ~/.config/gads/config.toml`.

## Commands

### Onboarding / meta

| Command | What it does |
|---|---|
| `init` | Interactive wizard to create your first profile |
| `auth-check` | Preflight: dev token + ADC + customer reachability |
| `commands` | JSON manifest of every command — for LLM agent discovery |
| `list-profiles` | List configured profiles |

### Read-only

| Command | What it does |
|---|---|
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
| `set-status <campaign\|ad_group\|asset_group> <id> {ENABLED\|PAUSED}` | Enable / pause a resource |

## Profiles + packs

A profile is a TOML file at `~/.config/gads/profiles/<name>.toml`. It tells
the CLI which Google Ads account you're operating on and where its pack
files live.

```toml
# ~/.config/gads/profiles/myaccount.toml
customer_id = "INSERT_CUSTOMER_ID_HERE"
# login_customer_id = "INSERT_MCC_CUSTOMER_ID_HERE"   # only if under an MCC

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

`gads` is designed to be driven by LLM agents. Drop the bundled
`skills/gads-cli/SKILL.md` into `~/.claude/skills/` and Claude Code
will auto-discover it.

```bash
cp -r skills/gads-cli ~/.claude/skills/
```

Or install the whole bundle of 8 marketing/ads skills that pair with
`gads` — see [skills/README.md](skills/README.md).

After install, Claude Code prefers the CLI over walking you through the
Ads UI step-by-step.

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

## Live terminal dashboard (`gads-tui`)

A Rust companion binary that reads from `gads --format json list-campaigns`
and shows a live, auto-refreshing dashboard. See [`tui/README.md`](tui/README.md).

```bash
cd tui
cargo build --release
./target/release/gads-tui
```

Builds to a ~650KB stripped static binary. The Python CLI stays
zero-dep; the TUI is opt-in and only needed if you want the live view.

## Roadmap

- `gads health` — one-command account dashboard
- `gads snapshot save/list/restore` — undo support for mutations
- `gads sync <yaml>` — Terraform-style declarative state with drift detection
- `gads lint <pack>` — offline pack validator (width, required fields, URL HEAD)
- Atomic multi-step via `googleAds:mutate` bulk endpoint — prevents orphans on partial failure
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
