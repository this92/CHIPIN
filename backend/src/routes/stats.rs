/// GET /api/stats — global stats for the login page counter

use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::{error::{AppError, Result}, AppState};

pub async fn global_stats(State(state): State<AppState>) -> Result<Json<Value>> {
    let result = tokio::task::spawn_blocking(move || -> std::result::Result<Value, AppError> {
        let conn = state.db.lock().expect("DB lock poisoned");

        let students: i64 = conn.query_row("SELECT COUNT(*) FROM students",  [], |r| r.get(0)).unwrap_or(0);
        let projects: i64 = conn.query_row("SELECT COUNT(*) FROM projects",  [], |r| r.get(0)).unwrap_or(0);
        let rooms:    i64 = conn.query_row("SELECT COUNT(*) FROM rooms WHERE code IN (SELECT DISTINCT room_code FROM students)", [], |r| r.get(0)).unwrap_or(0);

        Ok(json!({
            "students": students,
            "projects": projects,
            "rooms":    rooms,
        }))
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    Ok(Json(result))
}
