# Database

SQLite, WAL, `PRAGMA foreign_keys=ON`, `rusqlite_migration`.

File: app data `link-studio.sqlite` (headless: `./data/`).

## Tables

- **projects** id PK, name, description, created_at, updated_at
- **cameras** id PK, project_id FK CASCADE, name, role (`cam1`/`cam2`), display_order, timestamps. Unique (project_id, role)
- **settings** key PK, value, updated_at
- **broadcast_sessions** id PK, project_id FK CASCADE, timestamps

Not stored: video, SDP, ICE, PTZ, MediaPipe.
