# Architecture (First Demo)

Local-first. Media is P2P WebRTC. Signaling is a local WebSocket in the Tauri process (Axum).

```
Phone browser --WebRTC--> Director / PGM tabs
     |                         ^
     | WebSocket               |
     v                         |
Axum HTTP :8787 (0.0.0.0)      |
Axum HTTPS :8443 (0.0.0.0) ----+
     |
     + SQLite (projects, cameras, settings, sessions)
```

- HTTP 8787: Director API, same-PC PGM/OBS (`http://127.0.0.1` is a secure context).
- HTTPS 8443: phone camera pages. Local CA in `certs/ca.pem`.
- Camera is the WebRTC offerer. Each camera has one PeerConnection per receiver.
- PGM layout is HTML/CSS on each viewer using broadcast `pgm` state, not a video compositor.
