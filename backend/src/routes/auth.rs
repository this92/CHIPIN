/// POST /api/login
///
/// Accepts a room code + student info. Creates the room if it doesn't exist,
/// then creates or returns the existing student record for that roll number.

use axum::{extract::State, Json};
use rusqlite::params;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    models::LoginRequest,
    AppState,
};

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<Value>> {
    // ── Validate input ────────────────────────────────────────────────────────
    let room_code = req.room_code.trim().to_string();
    let name      = req.name.trim().to_string();
    let roll_no   = req.roll_no.trim().to_uppercase();
    let year      = req.year.trim().to_uppercase();
    let division  = req.division.trim().to_uppercase();

    if room_code.is_empty() { return Err(AppError::BadRequest("room_code is required".into())); }
    if name.is_empty()      { return Err(AppError::BadRequest("name is required".into())); }
    if roll_no.is_empty()   { return Err(AppError::BadRequest("roll_no is required".into())); }

    if !["FY", "SY", "TY"].contains(&year.as_str()) {
        return Err(AppError::BadRequest(format!("Invalid year '{year}'. Must be FY, SY, or TY.")));
    }
    if !["A", "B", "C", "D"].contains(&division.as_str()) {
        return Err(AppError::BadRequest(format!("Invalid division '{division}'. Must be A, B, C, or D.")));
    }

    // ── Database work (sync, runs in blocking thread) ─────────────────────────
    let log_name = name.clone();
    let log_room = room_code.clone();
    let result = tokio::task::spawn_blocking(move || -> std::result::Result<Value, AppError> {
        let conn = state.db.lock().expect("DB lock poisoned");

        // 1. Ensure room exists (upsert)
        let room_label = match room_code.as_str() {
            "107" => "BCA — All Divisions",
            "108" => "BCS — All Divisions",
            "109" => "BSc IT — All Divisions",
            _     => "Unknown Course",
        };
        conn.execute(
            "INSERT OR IGNORE INTO rooms (code, label) VALUES (?1, ?2)",
            params![room_code, room_label],
        )?;

        // 2. Check if a student with this roll_no already exists in this room
        let existing: Option<String> = conn.query_row(
            "SELECT id FROM students WHERE roll_no = ?1 AND room_code = ?2",
            params![roll_no, room_code],
            |row| row.get(0),
        ).ok();

        let student_id = if let Some(id) = existing {
            // Update name in case they changed it
            conn.execute(
                "UPDATE students SET name = ?1, year = ?2, division = ?3, updated_at = datetime('now') WHERE id = ?4",
                params![name, year, division, id],
            )?;
            id
        } else {
            // Create new student
            let new_id = Uuid::new_v4().to_string();
            let course = match room_code.as_str() {
                "107" => "BCA",
                "108" => "BCS",
                "109" => "BSc IT",
                _     => "Unknown",
            };
            conn.execute(
                "INSERT INTO students (id, room_code, name, roll_no, year, division, course)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![new_id, room_code, name, roll_no, year, division, course],
            )?;
            new_id
        };

        // 3. Return full student record
        let student: Value = conn.query_row(
            "SELECT s.id, s.room_code, s.name, s.roll_no, s.year, s.division, s.course,
                    s.tagline, s.about, s.avatar_url, s.skills, s.created_at, s.updated_at,
                    (SELECT COUNT(*) FROM projects p WHERE p.student_id = s.id) AS project_count
             FROM students s WHERE s.id = ?1",
            params![student_id],
            |row| {
                let skills_json: String = row.get(10)?;
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
                    "about":         row.get::<_, String>(8)?,
                    "avatar_url":    row.get::<_, Option<String>>(9)?,
                    "skills":        skills,
                    "project_count": row.get::<_, i64>(13)?,
                    "created_at":    row.get::<_, String>(11)?,
                    "updated_at":    row.get::<_, String>(12)?,
                }))
            },
        )?;

        Ok(student)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    tracing::info!("Login: {} joined room {}", log_name, log_room);
    Ok(Json(result))
}
