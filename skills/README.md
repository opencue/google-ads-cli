# Skills bundle

Eight Claude Code skills that pair well with the `gads` CLI. Drop the
ones you want into `~/.claude/skills/` and Claude Code will auto-load
them in any session.

## Install

```bash
# Install ALL bundled skills
mkdir -p ~/.claude/skills
cp -r skills/*/ ~/.claude/skills/

# Or pick specific ones
cp -r skills/gads-cli ~/.claude/skills/
cp -r skills/ads      ~/.claude/skills/
```

After install, start a new Claude Code session — the skills are
discoverable automatically.

## What's in here

| Skill | Pairs with `gads` because... |
|---|---|
| **gads-cli** | Teaches Claude Code how to use `gads` itself. Drop this first. |
| **ads** | High-level paid-ad strategy across Google, Meta, LinkedIn. `gads` is the execution arm; `ads` is the brain. |
| **ad-creative** | Bulk RSA headline / description / primary-text generation. Output flows straight into `gads`' brand-search and headline packs. |
| **copywriting** | Long-form copy (homepage, landing page) — feeds final URLs that `gads` sitelinks/RSAs point at. |
| **cro** | Conversion-rate-optimization audit of the landing pages your ads send traffic to. Catches the "ad works but page doesn't convert" failure mode. |
| **seo-audit** | Technical + on-page SEO audit. Same surface as `gads` but for organic; the two complement each other. |
| **ai-seo** | Optimization for AI search (ChatGPT, Perplexity, AI Overviews). Where paid ad budget can't reach. |
| **market-ads** | Slash-command flavored `/market ads <url>` skill — generates full multi-platform ad campaigns from one URL prompt. |

## Provenance / attribution

The 7 skills besides `gads-cli` were copied from the bundle author's local
`~/.claude/skills/` library and may have originated from upstream
publishers (e.g. agentskills.ai-style libraries). They are included here
under fair use for personal/team use of this repo.

**If you fork or redistribute publicly:** verify the upstream license
for each skill before re-publishing. `gads-cli/SKILL.md` is MIT-licensed
under this repo's LICENSE; the others may have different terms.

See [NOTICE.md](./NOTICE.md) for detail.

## How agents discover skills

When Claude Code starts a session, it scans `~/.claude/skills/` for
directories containing a `SKILL.md` with valid YAML frontmatter. Each
skill's `description:` field tells the agent when to invoke it.

A skill is just a markdown file with a YAML header. The body can be
plain prose, code snippets, decision trees — whatever helps the agent
do the right thing.
