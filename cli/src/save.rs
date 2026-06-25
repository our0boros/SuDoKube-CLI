use chrono::{Local, NaiveDateTime};
use rusqlite::{Connection, Result, params};
use std::collections::HashMap;
use std::path::Path;
use sudokube_core::cube::{CubeCoord, iter_surface_coords};
use sudokube_core::game_state::GameState;

pub const DB_PATH: &str = "sudokube.db";

/// 一条历史对局记录。
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GameRecord {
    pub id: i64,
    pub started_at: NaiveDateTime,
    pub finished_at: Option<NaiveDateTime>,
    pub elapsed_seconds: i64,
    pub difficulty: String,
    pub completed: bool,
    pub answer: HashMap<CubeCoord, u8>,
    pub puzzle: HashMap<CubeCoord, u8>,
    pub given: HashMap<CubeCoord, u8>,
}

pub fn init_db() -> Result<Connection> {
    let conn = Connection::open(Path::new(DB_PATH))?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS games (
            id INTEGER PRIMARY KEY,
            started_at TEXT NOT NULL,
            finished_at TEXT,
            elapsed_seconds INTEGER,
            difficulty TEXT,
            completed BOOLEAN,
            answer TEXT NOT NULL,
            puzzle TEXT NOT NULL,
            given TEXT NOT NULL,
            moves TEXT
        )",
        [],
    )?;
    // 兼容旧表：添加 given 列。
    let _ = conn.execute(
        "ALTER TABLE games ADD COLUMN given TEXT NOT NULL DEFAULT ''",
        [],
    );
    // 设置表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
        [],
    )?;
    Ok(conn)
}

pub fn save_setting(key: &str, value: &str) -> Result<()> {
    let conn = init_db()?;
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

pub fn load_setting(key: &str) -> Result<Option<String>> {
    let conn = init_db()?;
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query(params![key])?;
    match rows.next()? {
        Some(row) => Ok(Some(row.get(0)?)),
        None => Ok(None),
    }
}

pub fn save_game(state: &GameState) -> Result<i64> {
    let conn = init_db()?;
    let coords: Vec<CubeCoord> = iter_surface_coords().collect();
    let answer_str = serialize_solution(&state.solution, &coords);
    let puzzle_str = state.grid.serialize(&coords);
    let given_str = serialize_given(&state.grid, &coords);
    let started = Local::now().naive_local();
    let finished = if state.completed { Some(started) } else { None };

    if let Some(id) = state.id {
        conn.execute(
            "UPDATE games SET
                started_at = ?1,
                finished_at = ?2,
                elapsed_seconds = ?3,
                difficulty = ?4,
                completed = ?5,
                answer = ?6,
                puzzle = ?7,
                given = ?8,
                moves = ?9
             WHERE id = ?10",
            params![
                started.format("%Y-%m-%d %H:%M:%S").to_string(),
                finished.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
                state.elapsed_seconds as i64,
                state.difficulty.as_str(),
                state.completed,
                answer_str,
                puzzle_str,
                given_str,
                "",
                id
            ],
        )?;
        Ok(id)
    } else {
        conn.execute(
            "INSERT INTO games (started_at, finished_at, elapsed_seconds, difficulty, completed, answer, puzzle, given, moves)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                started.format("%Y-%m-%d %H:%M:%S").to_string(),
                finished.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
                state.elapsed_seconds as i64,
                state.difficulty.as_str(),
                state.completed,
                answer_str,
                puzzle_str,
                given_str,
                ""
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }
}

#[allow(dead_code)]
pub fn load_history(limit: usize) -> Result<Vec<GameRecord>> {
    load_records("ORDER BY started_at DESC LIMIT ?1", limit)
}

pub fn load_unfinished(limit: usize) -> Result<Vec<GameRecord>> {
    load_records("WHERE completed = 0 ORDER BY started_at DESC LIMIT ?1", limit)
}

fn load_records(where_clause: &str, limit: usize) -> Result<Vec<GameRecord>> {
    let conn = init_db()?;
    let sql = format!(
        "SELECT id, started_at, finished_at, elapsed_seconds, difficulty, completed, answer, puzzle, given
         FROM games {}",
        where_clause
    );
    let mut stmt = conn.prepare(&sql)?;

    let rows = stmt.query_map(params![limit as i64], |row| {
        let started: String = row.get(1)?;
        let finished: Option<String> = row.get(2)?;
        let answer: String = row.get(6)?;
        let puzzle: String = row.get(7)?;
        let given: String = row.get(8)?;
        Ok(GameRecord {
            id: row.get(0)?,
            started_at: NaiveDateTime::parse_from_str(&started, "%Y-%m-%d %H:%M:%S")
                .unwrap_or_else(|_| Local::now().naive_local()),
            finished_at: finished
                .and_then(|d| NaiveDateTime::parse_from_str(&d, "%Y-%m-%d %H:%M:%S").ok()),
            elapsed_seconds: row.get(3)?,
            difficulty: row.get(4)?,
            completed: row.get(5)?,
            answer: deserialize_solution(&answer),
            puzzle: deserialize_solution(&puzzle),
            given: deserialize_solution(&given),
        })
    })?;

    rows.collect()
}

fn serialize_solution(solution: &HashMap<CubeCoord, u8>, coords: &[CubeCoord]) -> String {
    coords
        .iter()
        .map(|c| solution.get(c).map_or('0', |&v| (b'0' + v) as char))
        .collect()
}

fn serialize_given(grid: &sudokube_core::cube::CubeGrid, coords: &[CubeCoord]) -> String {
    coords
        .iter()
        .map(|c| match grid.get(c) {
            Some(cell) if cell.given => cell.user_value.map_or('0', |v| (b'0' + v) as char),
            _ => '0',
        })
        .collect()
}

fn deserialize_solution(data: &str) -> HashMap<CubeCoord, u8> {
    let coords: Vec<CubeCoord> = iter_surface_coords().collect();
    coords
        .iter()
        .zip(data.chars())
        .filter_map(|(c, ch)| {
            let v = ch as u8 - b'0';
            if v >= 1 && v <= 9 {
                Some((*c, v))
            } else {
                None
            }
        })
        .collect()
}

pub fn delete_game(id: i64) -> Result<()> {
    let conn = init_db()?;
    conn.execute("DELETE FROM games WHERE id = ?1", params![id])?;
    Ok(())
}
