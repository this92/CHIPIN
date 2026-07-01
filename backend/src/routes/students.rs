/// Student routes:
///   GET  /api/students              — list students (filterable)
///   GET  /api/students/:id          — get single student
///   PUT  /api/students/:id          — update own profile

use axum::{
    extract::{Path, Query, State},
    Json,
};
use rusqlite::params;
use serde_json::{json, Value};

use crate::{
    error::{AppError, Result},
    models::{StudentQuery, UpdateStudentRequest},
    AppState,
};

// ── GET /api/students ──────────────────────────────────────────────────────────
pub async fn list_students(
    State(state): State<AppState>,
    Query(params): Query<StudentQuery>,
) -> Result<Json<Value>> {
    let result = tokio::task::spawn_blocking(move || -> std::result::Result<Vec<Value>, AppError> {
        let conn = state.db.lock().expect("DB lock poisoned");

        let mut sql = String::from(
            "SELECT s.id, s.room_code, s.name, s.roll_no, s.year, s.division, s.course,
                    s.tagline, s.avatar_url, s.skills,
                    (SELECT COUNT(*) FROM projects p WHERE p.student_id = s.id) AS project_count
             FROM students s WHERE 1=1"
        );
        let mut args: Vec<String> = Vec::new();
        let mut idx = 1usize;

        if let Some(rc) = &params.room_code {
            sql.push_str(&format!(" AND s.room_code = ?{idx}"));
            args.push(rc.trim().to_string());
            idx += 1;
        }
        if let Some(name) = &params.name {
            sql.push_str(&format!(" AND s.name LIKE ?{idx}"));
            args.push(format!("%{}%", name.trim()));
            idx += 1;
        }
        if let Some(year) = &params.year {
            sql.push_str(&format!(" AND s.year = ?{idx}"));
            args.push(year.trim().to_uppercase());
            idx += 1;
        }
        if let Some(div) = &params.division {
            sql.push_str(&format!(" AND s.division = ?{idx}"));
            args.push(div.trim().to_uppercase());
            // idx += 1; // would be needed for further params
        }
        sql.push_str(" ORDER BY s.name ASC");

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(
            rusqlite::params_from_iter(args.iter()),
            |row| {
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
            },
        )?;

        let students: rusqlite::Result<Vec<Value>> = rows.collect();
        Ok(students?)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(Value::Array(result)))
}

// ── GET /api/students/:id ──────────────────────────────────────────────────────
pub async fn get_student(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let result = tokio::task::spawn_blocking(move || -> std::result::Result<Value, AppError> {
        let conn = state.db.lock().expect("DB lock poisoned");

        let student: Option<Value> = conn.query_row(
            "SELECT s.id, s.room_code, s.name, s.roll_no, s.year, s.division, s.course,
                    s.tagline, s.about, s.avatar_url, s.skills, s.created_at, s.updated_at,
                    (SELECT COUNT(*) FROM projects p WHERE p.student_id = s.id) AS project_count
             FROM students s WHERE s.id = ?1",
            params![id],
            |row| {
                let skills_json: String = row.get(10)?;
                let skills: Vec<String> = serde_json::from_str(&skills_json).unwrap_or_default();
                // Fetch projects
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
        ).ok();

        if student.is_none() {
            return Err(AppError::NotFound(format!("Student '{}' not found", id)));
        }

        // Fetch projects separately
        let student_id_clone = id.clone();
        let mut proj_stmt = conn.prepare(
            "SELECT id, name, url, created_at FROM projects WHERE student_id = ?1 ORDER BY created_at DESC"
        )?;
        let projects: Vec<Value> = proj_stmt.query_map(params![student_id_clone], |row| {
            Ok(json!({
                "id":         row.get::<_, String>(0)?,
                "name":       row.get::<_, String>(1)?,
                "link":       row.get::<_, String>(2)?,
                "created_at": row.get::<_, String>(3)?,
            }))
        })?.filter_map(|r| r.ok()).collect();

        // Fetch contributions
        let mut contrib_stmt = conn.prepare(
            "SELECT date, count FROM contributions WHERE student_id = ?1 ORDER BY date DESC LIMIT 365"
        )?;
        let contributions: Vec<Value> = contrib_stmt.query_map(params![id], |row| {
            Ok(json!({ "date": row.get::<_, String>(0)?, "count": row.get::<_, i64>(1)? }))
        })?.filter_map(|r| r.ok()).collect();

        // Merge projects + contributions into student
        let mut s = student.unwrap();
        s["projects"]      = Value::Array(projects);
        s["contributions"] = Value::Array(contributions);

        Ok(s)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(result))
}

// ── PUT /api/students/:id ──────────────────────────────────────────────────────
pub async fn update_student(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateStudentRequest>,
) -> Result<Json<Value>> {
    let result = tokio::task::spawn_blocking(move || -> std::result::Result<Value, AppError> {
        let conn = state.db.lock().expect("DB lock poisoned");

        // Build dynamic SET clause
        let mut sets  = Vec::<String>::new();
        let mut args  = Vec::<Box<dyn rusqlite::types::ToSql>>::new();
        let mut idx   = 1usize;

        macro_rules! maybe_set {
            ($field:literal, $val:expr) => {
                if let Some(v) = $val {
                    sets.push(format!("{} = ?{idx}", $field));
                    args.push(Box::new(v.clone()));
                    idx += 1;
                }
            };
        }

        maybe_set!("name",       req.name);
        maybe_set!("tagline",    req.tagline);
        maybe_set!("about",      req.about);
        maybe_set!("avatar_url", req.avatar_url);

        if let Some(skills) = req.skills {
            let json_skills = serde_json::to_string(&skills).unwrap_or_else(|_| "[]".into());
            sets.push(format!("skills = ?{idx}"));
            args.push(Box::new(json_skills));
            idx += 1;
        }

        if sets.is_empty() {
            return Err(AppError::BadRequest("No fields to update".into()));
        }

        sets.push("updated_at = datetime('now')".into());
        args.push(Box::new(id.clone()));

        let sql = format!("UPDATE students SET {} WHERE id = ?{idx}", sets.join(", "));
        let affected = conn.execute(&sql, rusqlite::params_from_iter(args.iter().map(|b| b.as_ref())))?;

        if affected == 0 {
            return Err(AppError::NotFound(format!("Student '{}' not found", id)));
        }

        Ok(json!({ "ok": true, "id": id }))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(result))
}
