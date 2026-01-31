---
id: src_01KGAHSVDN8HRDMTRPVTHV8MMW
title: priorities
ingested_at: 2026-01-31T17:32:23.349123+00:00
original_path: /Users/jesse/cto/priorities.md
tags: []
processing_status: complete
content_hash: sha256:972a8772a4dce9da4d39d3b7a60e53bb5ddbf15b97c97732b69f2fdb6c63af0a
---
# CTO Priorities

## Immediate (from Carl sync 2025-01-23)

1. ~~**TMUX sizing fix**~~ ✓ Done (2179185a)

2. ~~**Postgres patterns**~~ ✓ Validated (2025-01-24)
   - Hard snapshots work - no PG coordination needed
   - WAL crash recovery handles unannounced snapshots
   - Enables: fork prod DB anytime, instant test fixtures, rollback migrations

3. ~~**SSH improvements**~~ ✓ Tests pass (2025-01-25)
   - Multi-connection, SFTP working
   - Zed spins but it's broken on normal linux too - not a Sparks issue

4. **spark-pg service** ✓ Working (2025-01-25)
   - `spark-pg new/fork/connect/status` implemented
   - Uses spark-client crate directly
   - Pattern established for more services (redis, etc.)
   - Open: inside vs outside spark complexity

5. **Dev Spark containers with KVM**

6. **🔥 Dev/Prod unification** (NEW - blocking)
   - Currently: different scripts/flows/settings for dev vs prod
   - Need: single path with flag difference
   - Affects: deploy, setup, verify

## Current Top 3 Priorities

1. **Sparks: Self-hosting development**
   - Get Sparks to where it develops itself
   - Trust, git workflows, source vs data
   - Goal: LEGOs that snap together faster than alternatives

2. **Internal tooling for LoopWork**
   - Switch from iMessage to internal chat tool (Loop chat - first Spark native app)
   - Voice call/session logging with Carl
   - Shared skills repo at loop.work/skills
   - **Conversation recording for AI context** - record all conversations (Claude Code sessions, chats, calls) so future AI can browse history when needed

3. **Explore agentic imagery**
   - What is "Cursor for images"?
   - Understand what agenticness means for media
   - Q1 focus: images, then video
   - See: MattF's Prologue CAD (meetings/2026-01-24-mattf-prologue-cad.md) - same pattern for CAD: generate variations, rapid human evaluation, genetic algorithms for creativity

## This Week (2025-01-23)

- [x] TMUX sizing fix
- [x] Postgres patterns (hard snapshots work!)
- [ ] SSH multi-connection + SCP/SFTP + verification tests (Zed: spins on connect, progress from yesterday's uname errors)
- [ ] Dev Spark with KVM
- [ ] Continue agentic imagery exploration

## Open Questions

- Port 8080 documentation gap - need standardized HTTP service patterns
- How to structure skills repo (manual sync -> CLI tool)
- External vs internal separation for Picnic

## Big Picture

**The thesis**: AI tools feel dead when frozen into apps. The agentic nature IS the product. Multi-user collaboration needs rethinking - maybe agent-mediated.

**Spark Native Apps**: Agentic computing framework. Apps that dynamically spin up Sparks as part of the application (e.g., Loop chat channels spawning dedicated Spark instances).

**Strategic options** (~$2M runway):
1. Build toward product/revenue
2. Build reputation/awareness for acquihire

## Parking Lot

- Video (after images validated)
- What from Vibewire is worth keeping?
- ZOMG learnings to apply
- Skills CLI tool (after manual sync works)
- **Granola integration**: Get meeting transcripts/summaries automatically instead of manual sharing
- **Readwise Reader integration**: Sync reading activity, highlights, notes (see `ideas/reader-plan.md`)
- **Changelog service**: Auto-generate commit/day/week/release summaries from git (see `ideas/changelog-service.md`)

---
*Last updated: 2025-01-24*
