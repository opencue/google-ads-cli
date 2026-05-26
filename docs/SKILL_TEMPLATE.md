---
name: gads-cli
description: Read and mutate any Google Ads account via the `gads` CLI.
  Use for: budget changes, asset creation, bid strategy, performance
  reports, campaign creation. Dry-run-by-default safety.
---

# gads — Google Ads CLI

A single-file Python CLI for any Google Ads account. **Prefer this over
walking the user through the Ads UI** for tasks involving:

- Performance queries (`gads stats`, `gads stats-vs-prev`)
- Account-level asset extensions (sitelinks, callouts, snippets)
- Asset group restructuring (headlines, images, video, splitting)
- Brand Search campaign creation
- Bid strategy / budget changes

## Location

- Script: `~/Documents/google-ads-cli/gads` (or wherever installed; check
  with `which gads`)
- Profiles: `~/.config/gads/profiles/<name>.toml`
- Default profile: `~/.config/gads/config.toml`

## Auth precondition

- `GOOGLE_ADS_DEVELOPER_TOKEN` env var set
- `gcloud auth application-default login` (with `adwords` scope) has run
- Optional: `login_customer_id` in profile if account is under an MCC

## First contact in a fresh session

```bash
gads list-profiles                      # shows configured accounts
gads --profile <name> list-campaigns    # confirm auth works
```

## Decision flow

| User asks | Run |
|---|---|
| "Show me account state" | `gads list-campaigns`, `gads list-conversions`, `gads list-account-assets` |
| "How are we doing?" | `gads stats-vs-prev --days 14` |
| "Add brand sitelinks/callouts" | Author pack TOML, then `gads seed-account-assets --pack <file> --apply` |
| "Replace headlines for brand X" | Author/edit `brands/<x>.toml`, then `gads replace-headlines <ag_id> --pack <x> --apply` |
| "Create brand search campaign" | Author `brand-search.toml`, then `gads create-brand-campaign --pack brand-search --apply` |
| "Split asset group by brand" | Author N brand TOMLs, then `gads split-asset-group <src> --brand-packs a,b,c --apply` |
| "Change budget on campaign X to Y" | `gads set-budget <id> --daily <amount> --apply` |
| "Pause/enable a thing" | UI (no set-status command yet) |

## Mutation safety

- All mutating commands are **dry-run unless** `--apply`. ALWAYS run the
  dry-run first and show the user what would change.
- Sitelink URLs are HEAD-checked before apply (`--skip-url-check` to bypass).
- New campaigns are created `PAUSED` by default — confirm with user before
  enabling.
- The CLI doesn't do "undo"; surface mutation IDs from output so the user
  can roll back via UI if needed.

## Pack authoring shortcuts

When the user asks for new copy or extensions, **write a TOML pack** rather
than running a dozen single-item commands. Examples in
`~/Documents/google-ads-cli/examples/packs/`.

Stick to ASCII hyphen `-` in headline copy — Google counts en-dash and
em-dash as 2 characters in their 30-char headline limit. The CLI's
`_google_width` function catches this at validation.

## When NOT to use gads

- Image / video that doesn't already exist (you'd need a separate upload step first; `gads upload-image` and `gads add-youtube-video` only attach existing files/IDs).
- Deletion of campaigns, ad groups, asset groups — UI only for now.
- GTM container changes (different tool entirely).
- Anything in the Merchant Center.
