use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};
use rusqlite_migration::{Migrations, M};
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::signaling::CameraSlot;

const MIGRATIONS: &[M] = &[M::up(
    r#"
    CREATE TABLE projects (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      description TEXT NOT NULL DEFAULT '',
      created_at TEXT NOT NULL,
      updated_at TEXT NOT NULL
    );
    CREATE TABLE cameras (
      id TEXT PRIMARY KEY,
      project_id TEXT NOT NULL,
      name TEXT NOT NULL,
      role TEXT NOT NULL,
      display_order INTEGER NOT NULL,
      created_at TEXT NOT NULL,
      updated_at TEXT NOT NULL,
      FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
    );
    CREATE UNIQUE INDEX cameras_project_role ON cameras(project_id, role);
    CREATE TABLE settings (
      key TEXT PRIMARY KEY,
      value TEXT NOT NULL,
      updated_at TEXT NOT NULL
    );
    CREATE TABLE broadcast_sessions (
      id TEXT PRIMARY KEY,
      project_id TEXT NOT NULL,
      created_at TEXT NOT NULL,
      updated_at TEXT NOT NULL,
      FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
    );
    "#,
)];

fn now() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into())
}

fn migrations() -> Migrations<'static> {
    Migrations::from_slice(MIGRATIONS)
}

pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
                Some(err.to_string()),
            )
        })?;
    }
    let mut conn = Connection::open(path)?;
    configure(&mut conn)?;
    Ok(conn)
}

pub fn open_in_memory() -> rusqlite::Result<Connection> {
    let mut conn = Connection::open_in_memory()?;
    configure(&mut conn)?;
    Ok(conn)
}

fn configure(conn: &mut Connection) -> rusqlite::Result<()> {
    conn.pragma_update(None, "foreign_keys", true)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    migrations()
        .to_latest(conn)
        .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Camera {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub role: String,
    pub display_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionRow {
    pub id: String,
    pub project_id: String,
    pub created_at: String,
    pub updated_at: String,
}

pub fn create_project(conn: &Connection, name: &str, description: &str) -> rusqlite::Result<Project> {
    let name = name.trim();
    if name.is_empty() {
        return Err(rusqlite::Error::InvalidParameterName(
            "Project name is required.".into(),
        ));
    }
    let ts = now();
    let project = Project {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        description: description.trim().to_string(),
        created_at: ts.clone(),
        updated_at: ts.clone(),
    };
    conn.execute(
        "INSERT INTO projects (id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            project.id,
            project.name,
            project.description,
            project.created_at,
            project.updated_at
        ],
    )?;
    insert_camera(conn, &project.id, "Camera 1", CameraSlot::Cam1.as_str(), 1, &ts)?;
    insert_camera(conn, &project.id, "Camera 2", CameraSlot::Cam2.as_str(), 2, &ts)?;
    Ok(project)
}

fn insert_camera(
    conn: &Connection,
    project_id: &str,
    name: &str,
    role: &str,
    order: i64,
    ts: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO cameras (id, project_id, name, role, display_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            Uuid::new_v4().to_string(),
            project_id,
            name,
            role,
            order,
            ts,
            ts
        ],
    )?;
    Ok(())
}

pub fn list_projects(conn: &Connection) -> rusqlite::Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, created_at, updated_at FROM projects ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    })?;
    rows.collect()
}

pub fn get_project(conn: &Connection, id: &str) -> rusqlite::Result<Option<Project>> {
    conn.query_row(
        "SELECT id, name, description, created_at, updated_at FROM projects WHERE id = ?1",
        params![id],
        |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        },
    )
    .optional()
}

pub fn list_cameras(conn: &Connection, project_id: &str) -> rusqlite::Result<Vec<Camera>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, role, display_order, created_at, updated_at
         FROM cameras WHERE project_id = ?1 ORDER BY display_order ASC",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(Camera {
            id: row.get(0)?,
            project_id: row.get(1)?,
            name: row.get(2)?,
            role: row.get(3)?,
            display_order: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;
    rows.collect()
}

pub fn rename_camera(conn: &Connection, camera_id: &str, name: &str) -> rusqlite::Result<Option<Camera>> {
    let name = name.trim();
    if name.is_empty() {
        return Err(rusqlite::Error::InvalidParameterName(
            "Camera name is required.".into(),
        ));
    }
    let ts = now();
    let changed = conn.execute(
        "UPDATE cameras SET name = ?1, updated_at = ?2 WHERE id = ?3",
        params![name, ts, camera_id],
    )?;
    if changed == 0 {
        return Ok(None);
    }
    conn.query_row(
        "SELECT id, project_id, name, role, display_order, created_at, updated_at FROM cameras WHERE id = ?1",
        params![camera_id],
        |row| {
            Ok(Camera {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                role: row.get(3)?,
                display_order: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    )
    .optional()
}

pub fn setting(conn: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    let ts = now();
    conn.execute(
        "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![key, value, ts],
    )?;
    Ok(())
}

pub fn create_session(conn: &Connection, project_id: &str, session_id: &str) -> rusqlite::Result<SessionRow> {
    if get_project(conn, project_id)?.is_none() {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    let ts = now();
    conn.execute(
        "INSERT INTO broadcast_sessions (id, project_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![session_id, project_id, ts, ts],
    )?;
    set_setting(conn, "active_session_id", session_id)?;
    set_setting(conn, "last_project_id", project_id)?;
    Ok(SessionRow {
        id: session_id.to_string(),
        project_id: project_id.to_string(),
        created_at: ts.clone(),
        updated_at: ts,
    })
}

pub fn list_session_ids(conn: &Connection) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT id FROM broadcast_sessions")?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    rows.collect()
}

pub fn get_session(conn: &Connection, session_id: &str) -> rusqlite::Result<Option<SessionRow>> {
    conn.query_row(
        "SELECT id, project_id, created_at, updated_at FROM broadcast_sessions WHERE id = ?1",
        params![session_id],
        |row| {
            Ok(SessionRow {
                id: row.get(0)?,
                project_id: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        },
    )
    .optional()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_list_project_persists_cameras() {
        let conn = open_in_memory().unwrap();
        let project = create_project(&conn, "Demo", "first").unwrap();
        let listed = list_projects(&conn).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "Demo");
        let cameras = list_cameras(&conn, &project.id).unwrap();
        assert_eq!(cameras.len(), 2);
        assert_eq!(cameras[0].role, "cam1");
        assert_eq!(cameras[1].role, "cam2");
    }

    #[test]
    fn empty_project_name_is_rejected() {
        let conn = open_in_memory().unwrap();
        assert!(create_project(&conn, "  ", "").is_err());
    }

    #[test]
    fn settings_and_session_roundtrip_on_reopen() {
        let dir = std::env::temp_dir().join(format!("link-studio-db-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("link-studio.sqlite");
        let project_id;
        {
            let conn = open(&path).unwrap();
            let project = create_project(&conn, "Keep", "").unwrap();
            project_id = project.id.clone();
            create_session(&conn, &project.id, "abc123").unwrap();
            set_setting(&conn, "caption_text", "Hello").unwrap();
        }
        let conn = open(&path).unwrap();
        assert_eq!(list_projects(&conn).unwrap()[0].id, project_id);
        assert_eq!(setting(&conn, "active_session_id").unwrap().as_deref(), Some("abc123"));
        assert_eq!(setting(&conn, "caption_text").unwrap().as_deref(), Some("Hello"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn foreign_keys_prevent_orphan_cameras() {
        let conn = open_in_memory().unwrap();
        let err = conn.execute(
            "INSERT INTO cameras (id, project_id, name, role, display_order, created_at, updated_at)
             VALUES ('x', 'missing', 'Cam', 'cam1', 1, 't', 't')",
            [],
        );
        assert!(err.is_err());
    }
}
