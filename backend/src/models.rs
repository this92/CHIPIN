/// Shared data models (request/response types) for the CHIPIN backend.
/// All structs derive Serialize + Deserialize for Axum JSON handling.

use serde::{Deserialize, Serialize};

// ── ROOM ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub code:       String,
    pub label:      String,
    pub created_at: String,
}

// ── STUDENT ───────────────────────────────────────────────────────────────────

/// Full student record — returned by GET /api/students/:id
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Student {
    pub id:            String,
    pub room_code:     String,
    pub name:          String,
    pub roll_no:       String,
    pub year:          String,
    pub division:      String,
    pub course:        String,
    pub tagline:       String,
    pub about:         String,
    pub avatar_url:    Option<String>,
    /// JSON array stored in DB, deserialized on read
    pub skills:        Vec<String>,
    pub project_count: i64,
    pub created_at:    String,
    pub updated_at:    String,
}

/// Lightweight student summary — used in list/grid views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentSummary {
    pub id:            String,
    pub room_code:     String,
    pub name:          String,
    pub roll_no:       String,
    pub year:          String,
    pub division:      String,
    pub course:        String,
    pub tagline:       String,
    pub avatar_url:    Option<String>,
    pub skills:        Vec<String>,
    pub project_count: i64,
}

// ── PROJECT ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id:         String,
    pub student_id: String,
    pub name:       String,
    pub url:        String,
    pub created_at: String,
}

// ── CONTRIBUTION ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contribution {
    pub student_id: String,
    pub date:       String,
    pub count:      i64,
}

// ── REQUEST BODIES ────────────────────────────────────────────────────────────

/// POST /api/login
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub room_code: String,
    pub name:      String,
    pub roll_no:   String,
    pub year:      String,
    pub division:  String,
}

/// PUT /api/students/:id
#[derive(Debug, Deserialize)]
pub struct UpdateStudentRequest {
    pub name:       Option<String>,
    pub tagline:    Option<String>,
    pub about:      Option<String>,
    pub avatar_url: Option<String>,
    pub skills:     Option<Vec<String>>,
}

/// POST /api/projects
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub student_id: String,
    pub name:       String,
    pub url:        Option<String>,
}

/// POST /api/contributions
#[derive(Debug, Deserialize)]
pub struct LogContributionRequest {
    pub student_id: String,
    pub date:       String,   // ISO date string "YYYY-MM-DD"
}

// ── QUERY PARAMS ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct StudentQuery {
    pub room_code: Option<String>,
    pub name:      Option<String>,
    pub year:      Option<String>,
    pub division:  Option<String>,
}

// ── GLOBAL STATS ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct GlobalStats {
    pub students: i64,
    pub projects: i64,
    pub rooms:    i64,
}
