/// Project routes:
///   GET  /api/projects?student_id=...  — list projects for a student
///   POST /api/projects                 — create a project

use axum::{extract::{Query, State}, Json};
use rusqlite::params;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{error::{AppError, Result}, models::CreateProjectRequest, AppState};

#[derive(Debug, Deserialize)]
pub struct ProjectQuery {
    pub student_id: Option<String>,
}

// ── GET /api/projects ──────────────────────────────────────────────────────────
pub async fn list_projects(
    State(state): State<AppState>,
    Query(q): Query<ProjectQuery>,
) -> Result<Json<Value>> {
    let result = tokio::task::spawn_blocking(move || -> std::result::Result<Vec<Value>, AppError> {
        let conn = state.db.lock().expect("DB lock poisoned");
        let result_vec = match q.student_id {
            Some(sid) => {
                let mut s = conn.prepare(
                    "SELECT id, student_id, name, url, created_at FROM projects WHERE student_id = ?1 ORDER BY created_at DESC"
                )?;
                let rows = s.query_map(params![sid], project_row)?;
                rows.collect::<rusqlite::Result<Vec<Value>>>()
            }
            None => {
                let mut s = conn.prepare(
                    "SELECT id, student_id, name, url, created_at FROM projects ORDER BY created_at DESC LIMIT 50"
                )?;
                let rows = s.query_map([], project_row)?;
                rows.collect::<rusqlite::Result<Vec<Value>>>()
            }
        };
        Ok(result_vec?)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(Value::Array(result)))
}

fn project_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Value> {
    Ok(json!({
        "id":         row.get::<_, String>(0)?,
        "student_id": row.get::<_, String>(1)?,
        "name":       row.get::<_, String>(2)?,
        "link":       row.get::<_, String>(3)?,
        "created_at": row.get::<_, String>(4)?,
    }))
}

// ── POST /api/projects ─────────────────────────────────────────────────────────
pub async fn create_project(
    State(state): State<AppState>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<Value>> {
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest("Project name is required".into()));
    }

    let result = tokio::task::spawn_blocking(move || -> std::result::Result<Value, AppError> {
        let conn = state.db.lock().expect("DB lock poisoned");
        let new_id = Uuid::new_v4().to_string();
        let url    = req.url.unwrap_or_default();

        // Verify student exists
        let exists: bool = conn.query_row(
            "SELECT COUNT(*) FROM students WHERE id = ?1",
            params![req.student_id],
            |row| row.get::<_, i64>(0),
        ).unwrap_or(0) > 0;

        if !exists {
            return Err(AppError::NotFound("Student not found".into()));
        }

        conn.execute(
            "INSERT INTO projects (id, student_id, name, url) VALUES (?1, ?2, ?3, ?4)",
            params![new_id, req.student_id, name, url],
        )?;

        Ok(json!({
            "id":         new_id,
            "student_id": req.student_id,
            "name":       name,
            "link":       url,
        }))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(result))
}
