/// Room routes:
///   GET /api/rooms/:code/mates  — all students in a room

use axum::{extract::{Path, State}, Json};
use rusqlite::params;
use serde_json::{json, Value};

use crate::{error::{AppError, Result}, AppState};

pub async fn room_mates(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<Value>> {
    let result = tokio::task::spawn_blocking(move || -> std::result::Result<Vec<Value>, AppError> {
        let conn = state.db.lock().expect("DB lock poisoned");

        // Verify room exists
        let room_exists: bool = conn.query_row(
            "SELECT COUNT(*) FROM rooms WHERE code = ?1",
            params![code],
            |row| row.get::<_, i64>(0),
        ).unwrap_or(0) > 0;

        if !room_exists {
            return Err(AppError::NotFound(format!("Room '{}' not found", code)));
        }

        let mut stmt = conn.prepare(
            "SELECT s.id, s.room_code, s.name, s.roll_no, s.year, s.division, s.course,
                    s.tagline, s.avatar_url, s.skills,
                    (SELECT COUNT(*) FROM projects p WHERE p.student_id = s.id) AS project_count
             FROM students s
             WHERE s.room_code = ?1
             ORDER BY s.year ASC, s.division ASC, s.name ASC",
        )?;

        let rows = stmt.query_map(params![code], |row| {
            let skills_json: String = row.get(9)?;
            let skills: Vec<String> = serde_json::from_str(&skills_json).unwrap_or_default();
            Ok(json!({
                "id":            row.get::<_, String>(0)?,
                "room_code":     row.get::<_, String>(1)?,
                "name":          row.get::<_, String>(2)?,
                "roll_no":       row.get::<_, String>(3)?,
                "year":          row.get::<_, String>(4)?,
                "division":      row.get::<_, String>(5)?,
                "course":        row.get::<_, String>(6)?,
                "tagline":       row.get::<_, String>(7)?,
                "avatar_url":    row.get::<_, Option<String>>(8)?,
                "skills":        skills,
                "project_count": row.get::<_, i64>(10)?,
            }))
        })?;

        let mates: rusqlite::Result<Vec<Value>> = rows.collect();
        Ok(mates?)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(Value::Array(result)))
}
