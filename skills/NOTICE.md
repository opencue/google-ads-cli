# NOTICE — third-party skill attribution

The `skills/` directory in this repo contains 8 Claude Code skill files.
Their licensing status:

| Skill | Origin | License |
|---|---|---|
| `gads-cli/` | Authored for this repo | MIT (this repo's LICENSE) |
| `ads/` | Local install from `~/.agents/skills/ads/` | Unknown upstream — bundled as-is |
| `ad-creative/` | Local install from `~/.agents/skills/ad-creative/` | Unknown upstream — bundled as-is |
| `seo-audit/` | Local install from `~/.agents/skills/seo-audit/` | Unknown upstream — bundled as-is |
| `ai-seo/` | Local install from `~/.agents/skills/ai-seo/` | Unknown upstream — bundled as-is |
| `cro/` | Local install from `~/.agents/skills/cro/` | Unknown upstream — bundled as-is |
| `copywriting/` | Local install from `~/.agents/skills/copywriting/` | Unknown upstream — bundled as-is |
| `market-ads/` | Local install from `~/.claude/skills/market-ads/` | Unknown upstream — bundled as-is |

The "unknown upstream" skills are bundled here because they pair well
with `gads` for any user who already has them locally — packaging keeps
the install one-step.

**If you redistribute this repo publicly:** verify upstream licensing
for each skill before publishing your fork. The bundle author makes no
warranty about redistribution rights for the 7 unattributed skills. If
an upstream rights-holder requests removal, file an issue and we'll
move the skill into a separate "recommended skills install script"
instead.
