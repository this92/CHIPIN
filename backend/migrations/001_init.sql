-- CHIPIN Database Schema
-- Run automatically on first start via db.rs

-- ── ROOMS ─────────────────────────────────────────────────────────────────────
-- A room is identified by a short code (e.g. "107").
-- Students who join with the same code all belong to the same room.
CREATE TABLE IF NOT EXISTS rooms (
    code        TEXT PRIMARY KEY,          -- e.g. "107"
    label       TEXT NOT NULL,             -- e.g. "BCA — All Divisions"
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Seed default rooms
INSERT OR IGNORE INTO rooms (code, label) VALUES
    ('107', 'BCA — All Divisions'),
    ('108', 'BCS — All Divisions'),
    ('109', 'BSc IT — All Divisions');

-- ── STUDENTS ──────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS students (
    id          TEXT PRIMARY KEY,           -- UUID v4
    room_code   TEXT NOT NULL REFERENCES rooms(code),
    name        TEXT NOT NULL,
    roll_no     TEXT NOT NULL,
    year        TEXT NOT NULL CHECK(year IN ('FY','SY','TY')),
    division    TEXT NOT NULL CHECK(division IN ('A','B','C','D')),
    course      TEXT NOT NULL DEFAULT 'BCA',
    tagline     TEXT NOT NULL DEFAULT '',
    about       TEXT NOT NULL DEFAULT '',
    avatar_url  TEXT,
    skills      TEXT NOT NULL DEFAULT '[]',   -- JSON array stored as text
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── PROJECTS ──────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS projects (
    id          TEXT PRIMARY KEY,           -- UUID v4
    student_id  TEXT NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    url         TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ── CONTRIBUTIONS ─────────────────────────────────────────────────────────────
-- Tracks daily contribution count per student (GitHub-style heatmap data)
CREATE TABLE IF NOT EXISTS contributions (
    student_id  TEXT NOT NULL REFERENCES students(id) ON DELETE CASCADE,
    date        TEXT NOT NULL,              -- ISO date: "2026-06-15"
    count       INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (student_id, date)
);

-- ── INDEXES ───────────────────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_students_room     ON students(room_code);
CREATE INDEX IF NOT EXISTS idx_students_name     ON students(name);
CREATE INDEX IF NOT EXISTS idx_projects_student  ON projects(student_id);
CREATE INDEX IF NOT EXISTS idx_contrib_student   ON contributions(student_id);
CREATE INDEX IF NOT EXISTS idx_contrib_date      ON contributions(date);
