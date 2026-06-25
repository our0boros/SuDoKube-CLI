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
    deserialize_solution_from(data, &coords)
}

pub fn deserialize_solution_from(data: &str, coords: &[CubeCoord]) -> HashMap<CubeCoord, u8> {
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

pub fn deserialize_grid_from(puzzle_data: &str, given_data: &str, coords: &[CubeCoord]) -> sudokube_core::cube::CubeGrid {
    use sudokube_core::cube::Cell;
    let mut cells = HashMap::new();
    let given: HashMap<_, _> = coords.iter()
        .zip(given_data.chars())
        .filter_map(|(c, ch)| {
            let v = ch as u8 - b'0';
            if v >= 1 && v <= 9 { Some((*c, v)) } else { None }
        })
        .collect();
    for (c, ch) in coords.iter().zip(puzzle_data.chars()) {
        let v = ch as u8 - b'0';
        let is_given = given.contains_key(c);
        let user_value = if is_given {
            Some(v)
        } else if v >= 1 && v <= 9 {
            Some(v)
        } else {
            None
        };
        let answer = given.get(c).copied().unwrap_or(v.max(1));
        cells.insert(*c, Cell { answer, given: is_given, user_value });
    }
    sudokube_core::cube::CubeGrid { cells }
}

pub fn delete_game(id: i64) -> Result<()> {
    let conn = init_db()?;
    conn.execute("DELETE FROM games WHERE id = ?1", params![id])?;
    Ok(())
}

// ── Export / Import ──

/// Export a game as a string. If encrypted, apply simple XOR obfuscation.
pub fn export_game(state: &GameState, encrypted: bool) -> String {
    let coords: Vec<CubeCoord> = iter_surface_coords().collect();
    let answer_str = serialize_solution(&state.solution, &coords);
    let puzzle_str = state.grid.serialize(&coords);
    let given_str = serialize_given(&state.grid, &coords);

    // Format: SUDOKUBE|difficulty|answer|puzzle|given
    let raw = format!("SUDOKUBE|{}|{}|{}|{}", state.difficulty.as_str(), answer_str, puzzle_str, given_str);

    if encrypted {
        // Simple XOR with rotating key
        let key = b"SuDoKube2024";
        let encoded: Vec<u8> = raw.bytes().enumerate().map(|(i, b)| b ^ key[i % key.len()]).collect();
        format!("SUDOKUBE!{}", base64_encode(&encoded))
    } else {
        raw
    }
}

/// Import a game from a string. Returns (difficulty, answer, puzzle, given).
pub fn import_game(data: &str) -> Option<(String, String, String, String)> {
    let raw = if data.starts_with("SUDOKUBE!") {
        // Encrypted
        let b64 = &data[9..];
        let decoded = base64_decode(b64)?;
        let key = b"SuDoKube2024";
        let decrypted: Vec<u8> = decoded.iter().enumerate().map(|(i, &b)| b ^ key[i % key.len()]).collect();
        String::from_utf8(decrypted).ok()?
    } else {
        data.to_string()
    };

    let parts: Vec<&str> = raw.splitn(5, '|').collect();
    if parts.len() != 5 || parts[0] != "SUDOKUBE" {
        return None;
    }
    Some((parts[1].to_string(), parts[2].to_string(), parts[3].to_string(), parts[4].to_string()))
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        result.push(CHARS[((b0 >> 2) & 0x3F) as usize] as char);
        result.push(CHARS[(((b0 << 4) | (b1 >> 4)) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[(((b1 << 2) | (b2 >> 6)) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(b2 & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let input = input.trim_end_matches('=');
    let mut result = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0u32;
    for c in input.chars() {
        let val = CHARS.iter().position(|&x| x as char == c)? as u32;
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            result.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Some(result)
}

/// Copy text to clipboard (Windows)
pub fn copy_to_clipboard(text: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("clip")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(ref mut stdin) = child.stdin {
                    stdin.write_all(text.as_bytes())?;
                }
                child.wait()
            })
            .is_ok()
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = text;
        false
    }
}

/// Get next available game ID
pub fn next_available_id() -> i64 {
    let conn = match init_db() {
        Ok(c) => c,
        Err(_) => return 1,
    };
    conn.query_row("SELECT COALESCE(MAX(id), 0) + 1 FROM games", [], |row| row.get(0))
        .unwrap_or(1)
}
