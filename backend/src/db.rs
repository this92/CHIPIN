/// Database initialization and connection management.
/// Uses rusqlite with a bundled SQLite — no external install needed.

use rusqlite::{Connection, Result};
use std::path::Path;
use tracing::info;

/// Open (or create) the SQLite database and run migrations.
/// Returns a ready-to-use Connection.
pub fn init_db(db_path: &str) -> Result<Connection> {
    let is_new = !Path::new(db_path).exists();
    let conn = Connection::open(db_path)?;

    // Enable WAL mode for better concurrent read performance
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;
    // Enforce foreign keys
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    if is_new {
        info!("New database created at {db_path} — running migrations");
    } else {
        info!("Opened existing database at {db_path}");
    }

    run_migrations(&conn)?;
    Ok(conn)
}

/// Runs the embedded SQL migration to create all tables.
/// Uses CREATE TABLE IF NOT EXISTS — safe to call on every startup.
fn run_migrations(conn: &Connection) -> Result<()> {
    let schema = include_str!("../migrations/001_init.sql");
    conn.execute_batch(schema)?;
    info!("Migrations applied successfully");
    Ok(())
}
