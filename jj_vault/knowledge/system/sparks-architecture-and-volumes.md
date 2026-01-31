---
id: mem_01KGAKNA2V25KBX642TYNVY3H7
title: Sparks architecture and volumes
type: system
status: active
tags:
- sparks
- btrfs
- containers
- volumes
- secrets
- performance
- gcs
- raid10
- fsync
confidence: 0.72
created_at: 2026-01-31T18:04:51.675864Z
updated_at: 2026-01-31T18:21:58.444944Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKKWB9TDHWX5V9423PZ1BG
- thread_id: ''
  event_ids:
  - src_01KGAMJZD7RGD1SHMHS18R8A2G
supersedes: []
---
## Sparks architecture/system facts
- Each “spark” runs in an **isolated container** with its own root filesystem.
- Root filesystems are backed by **btrfs snapshots** (supports create/restore and hard snapshotting).
- Auto-mounted volumes:
  - `/spark/bin` (global shared executables)
  - `/spark/all` (global shared data)
  - `/spark/proj` (per-project shared data)
- Supports persistent cross-spark volumes via `spark data`.
- Supports 1Password integration via `spark secrets`.
- Ephemeral sparks: using `-` to auto-delete a spark on exit.

## Validated behavior
- 2025-01-24: Postgres survived **unannounced** btrfs snapshots; WAL crash recovery handled it. Implication: no need to coordinate with apps before forking; “fork anything anytime.”


## Performance/ops claims (2026-01-24)
- Fast fork/snapshot time: ~**0.25s** per fork/snapshot (btrfs-based).
- Tracks diffs across copy-on-write snapshots.
- Streams state to **GCS** for backup.

## “Eat My Data” mode
- An optimization mode intended for cloud environments.
- **Disables fsync** to prioritize throughput.
- Uses **RAID10** with **local SSDs**.
- Claimed disk speedup: ~**6,000×**.
