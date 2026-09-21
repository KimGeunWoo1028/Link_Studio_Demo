# Link Studio First Demo — MVP Scope

Source of truth for **this demo**: the First Demo implementation prompt.
Source of truth for **the finished product**: `docs/Link_Studio_RFP.pdf`.

When they conflict, this document wins for the current demo.

## In scope (P0/P1)

Tauri 2 desktop app, local HTTP+HTTPS server, SQLite, WebSocket signaling, P2P WebRTC, two cameras, manual PGM switch, PIP, caption, QR, PGM URL, disconnect/reconnect, Wake Lock when supported.

## Out of scope

Auth, Supabase, PostgreSQL, cloud, SFU, AI/MediaPipe, real ESP32, production TURN, full OBS automation.

## Hardware gates (NOT VERIFIED until a person runs them)

TEST-H1 … TEST-H8: phone QR, permission, preview, two phones, LAN switch, OBS, iOS, Android.
