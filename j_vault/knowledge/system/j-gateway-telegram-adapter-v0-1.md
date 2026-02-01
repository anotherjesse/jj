---
id: mem_01KGAKSRNZ3S4A3QVRY65H6BAT
title: J Gateway Telegram adapter (v0.1)
type: system
status: active
tags:
- j
- gateway
- telegram
- adapter
- long-polling
- dedupe
- security
confidence: 0.8
created_at: 2026-01-31T18:07:17.695538Z
updated_at: 2026-01-31T18:07:17.695538Z
sources:
- thread_id: ''
  event_ids:
  - src_01KGAKQYA2PRMPRV128VBJMAR2
supersedes: []
summary: 'J Gateway v0.1 Telegram adapter: config-enabled long-polling getUpdates with durable offset, tg:<chat_id> sessions, update/message dedupe.'
---
## J Gateway v0.1 Telegram adapter
- Enabled via config: `[telegram].enabled = true`.
- Uses **long polling** via Telegram Bot API `getUpdates`.
- Tracks a durable `offset` / last processed update to avoid duplicate processing.

### Session mapping
- `session_key = "tg:<chat_id>"`.
- (Future option) include thread/topic: `tg:<chat_id>:t:<message_thread_id>`.

### Dedupe
- Deduplicate by `update_id` and/or `(chat_id, message_id)`.
- Store last processed update state durably (suggested: `sessions.json` or a separate `telegram_state.json`).

### Inbound rules
- Ignore non-text messages in v0.1.
- Handle `/start` and `/help` specially.
- Optional/expected allowlist: accept only configured `allow_chat_ids` (default deny unless configured).

### Outbound
- Send assistant **final** response via `sendMessage(chat_id, text)`.
- Optional: `sendChatAction` typing indicators.
- Streaming to Telegram is explicitly optional/deferred (may skip in v0.1).

### Failure handling
- On Telegram API errors: backoff and continue.
- On gateway shutdown: stop polling loop.