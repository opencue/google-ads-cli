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

| Key      | What it does                                          |
|---------:|-------------------------------------------------------|
| `q` `Esc`| Quit (closes suggest modal first if open)             |
| `r`      | Force refresh                                         |
| `p`      | Toggle PAUSED ↔ ENABLED on the selected campaign      |
| `s`      | Open `gads suggest` modal — prioritized fix punch list|
| `↑`/`k`  | Move selection up                                     |
| `↓`/`j`  | Move selection down                                   |
| `Ctrl+C` | Quit (same as `q`)                                    |

## Safety

- `p` shells out to `gads set-status campaign <id> <new> --apply`.
- All TUI-triggered mutations set `GADS_NO_AUTOSNAPSHOT=1` so the
  snapshot dir doesn't balloon during a keypress-heavy session.
- Take a manual snapshot before launching if you want one:
  `gads snapshot save before-tui-session`

## Roadmap

- `b` to change budget (modal text input)
- `e` (alias for ENABLED-only) and `d` (alias for PAUSED-only)
- Asset group drill-down (Enter on selected campaign)
- Multi-profile view: switch profiles with `Tab`
- Live spend deltas with sparklines
- Alerts panel: lost-IS spikes, optimization-score drops, budget hits

## Single-binary install

Pre-built binaries for common targets are on the GitHub Releases page
(coming soon — for now, build from source).
