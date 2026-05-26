# gads-tui — live terminal dashboard

A Rust TUI companion to the `gads` Python CLI. Auto-refreshing
campaigns table with status / budget / bid strategy at a glance.

```
┌─ gads-tui · profile=myaccount ──────────────────────────────────────┐
├─ Campaigns ────────────────────────────────────────────────────────┤
│ ID            STATUS    TYPE              BID                BUDGET│
│▸23857652151   ENABLED   PERFORMANCE_MAX   MAXIMIZE_CONV_VAL  1000  │
│ 23875784109   PAUSED    SEARCH            MAXIMIZE_CONV       1500  │
│                                                                     │
└─ q quit  r refresh  ↑↓/jk navigate  ·  3s since refresh  ──────────┘
```

## Why a separate Rust binary

- **Performance**: 30fps refresh without Python's interpreter startup tax.
- **Single file**: compiles to a ~1MB stripped static binary.
- **Stays out of the way of the Python CLI**: `gads` stays single-file
  zero-deps. `gads-tui` is opt-in.

The two binaries talk via `gads --format json` — the TUI shells out to
the Python CLI for data. So everything the TUI shows is data the CLI
already exposes.

## Install

### Pre-built binary (no Rust toolchain required)

Grab the latest archive for your platform from the
[Releases page](https://github.com/NagyVikt/google-ads-cli/releases):

```bash
# Linux x86_64
curl -L https://github.com/NagyVikt/google-ads-cli/releases/latest/download/gads-tui-x86_64-unknown-linux-gnu.tar.gz \
    | tar xz
sudo install gads-tui /usr/local/bin/

# macOS Apple Silicon
curl -L https://github.com/NagyVikt/google-ads-cli/releases/latest/download/gads-tui-aarch64-apple-darwin.tar.gz \
    | tar xz
sudo install gads-tui /usr/local/bin/

# macOS Intel
curl -L https://github.com/NagyVikt/google-ads-cli/releases/latest/download/gads-tui-x86_64-apple-darwin.tar.gz \
    | tar xz
sudo install gads-tui /usr/local/bin/

# Windows x86_64 — download from the Releases page and extract the .exe
```

Binaries are produced by [`.github/workflows/release.yml`](../.github/workflows/release.yml)
on every `v*.*.*` tag push and attached to the corresponding GitHub
Release.

### From source

```bash
# Requires Rust 1.75+. Install via https://rustup.rs/
cd tui
cargo build --release
./target/release/gads-tui

# Or install globally:
cargo install --path tui
gads-tui
```

## Prerequisites

- `gads` Python CLI on `PATH` (see `../README.md`)
- A configured profile (`gads init`) or `GADS_PROFILE` env var set

## Keys

| Key      | What it does                                          |
|---------:|-------------------------------------------------------|
| `q` `Esc`| Quit (closes any open modal first)                    |
| `r`      | Force refresh                                         |
| `p`      | Toggle PAUSED ↔ ENABLED on the selected campaign      |
| `b`      | Set daily budget on the selected campaign (text input)|
| `s`      | Open `gads suggest` modal — prioritized fix punch list|
| `Tab`    | Cycle to the next configured profile (multi-account)  |
| `↑`/`k`  | Move selection up                                     |
| `↓`/`j`  | Move selection down                                   |
| `Ctrl+C` | Quit (same as `q`)                                    |

### Multi-profile (Tab)

`gads-tui` reads `gads list-profiles --format json` on startup. With
2+ profiles configured, `Tab` cycles to the next one — the campaign
table re-fetches, all shell-outs route through the new
`GADS_PROFILE=<name>`. Useful for agencies / anyone managing several
accounts from one shell.

### Budget modal

Press `b` on the selected campaign. A centered modal shows:

```
┌─ Set daily budget · Enter to commit · Esc to cancel ─┐
│                                                       │
│   Campaign: Sales-Performance Max-1                   │
│   Current:  1000 /day                                 │
│                                                       │
│   New:      1200_ /day                                │
│                                                       │
└───────────────────────────────────────────────────────┘
```

Only digit keys + backspace are accepted. Enter shells out to
`gads set-budget <id> --daily <value> --apply`. Esc cancels.

## Safety

- `p` shells out to `gads set-status campaign <id> <new> --apply`.
- All TUI-triggered mutations set `GADS_NO_AUTOSNAPSHOT=1` so the
  snapshot dir doesn't balloon during a keypress-heavy session.
- Take a manual snapshot before launching if you want one:
  `gads snapshot save before-tui-session`

## Roadmap

- Asset group drill-down (Enter on selected campaign)
- Live spend deltas with sparklines
- Alerts panel: lost-IS spikes, optimization-score drops, budget hits
- Inline `gads cleanup-orphans` / `snapshot save` from the dashboard
- Pre-built static binaries on GitHub Releases (in progress)

## Single-binary install

Pre-built binaries for common targets are on the GitHub Releases page
(coming soon — for now, build from source).
