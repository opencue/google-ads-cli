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

## Build

```bash
# Requires Rust 1.75+. Install via https://rustup.rs/
cd tui
cargo build --release
./target/release/gads-tui
```

Or install globally:

```bash
cargo install --path tui
gads-tui
```

## Prerequisites

- `gads` Python CLI on `PATH` (see `../README.md`)
- A configured profile (`gads init`) or `GADS_PROFILE` env var set

## Keys

| Key      | What it does                          |
|---------:|---------------------------------------|
| `q` `Esc`| Quit                                  |
| `r`      | Force refresh                         |
| `↑`/`k`  | Move selection up                     |
| `↓`/`j`  | Move selection down                   |
| `Ctrl+C` | Quit (same as `q`)                    |

## Roadmap

- Inline mutations: `b` to change budget, `p` to toggle status, `e` to
  enable a campaign without leaving the TUI
- `s` to open `gads suggest` output in a side panel
- Asset group drill-down (Enter on selected campaign)
- Multi-profile view: switch profiles with `Tab`
- Live spend deltas with sparklines
- Alerts panel: lost-IS spikes, optimization-score drops, budget hits

## Single-binary install

Pre-built binaries for common targets are on the GitHub Releases page
(coming soon — for now, build from source).
