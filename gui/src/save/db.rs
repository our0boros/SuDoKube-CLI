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
            moves TEXT
        )",
        [],
    )?;
    Ok(conn)
}

pub fn save_game(state: &GameState) -> Result<i64> {
    let conn = init_db()?;
    let coords: Vec<CubeCoord> = iter_surface_coords().collect();
    let answer_str = serialize_solution(&state.solution, &coords);
    let puzzle_str = state.grid.serialize(&coords);
    let started = Local::now().naive_local();
    let finished = if state.completed { Some(started) } else { None };

    conn.execute(
        "INSERT INTO games (started_at, finished_at, elapsed_seconds, difficulty, completed, answer, puzzle, moves)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            started.format("%Y-%m-%d %H:%M:%S").to_string(),
            finished.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
            state.elapsed_seconds as i64,
            state.difficulty.as_str(),
            state.completed,
            answer_str,
            puzzle_str,
            ""
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn load_history(limit: usize) -> Result<Vec<GameRecord>> {
    let conn = init_db()?;
    let mut stmt = conn.prepare(
        "SELECT id, started_at, finished_at, elapsed_seconds, difficulty, completed, answer, puzzle
         FROM games ORDER BY started_at DESC LIMIT ?1",
    )?;

    let rows = stmt.query_map(params![limit as i64], |row| {
        let started: String = row.get(1)?;
        let finished: Option<String> = row.get(2)?;
        let answer: String = row.get(6)?;
        let puzzle: String = row.get(7)?;
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
