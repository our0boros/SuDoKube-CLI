use crate::cube::{CubeCoord, CubeGrid, Difficulty};
use std::collections::HashMap;

/// 当前对局状态。
#[derive(Debug, Clone)]
pub struct GameState {
    pub grid: CubeGrid,
    pub solution: HashMap<CubeCoord, u8>,
    pub difficulty: Difficulty,
    pub selected: Option<CubeCoord>,
    pub highlight_number: Option<u8>,
    pub started_at: f64,
    pub elapsed_seconds: u64,
    pub completed: bool,
    pub history: Vec<MoveRecord>,
    pub history_index: usize,
    pub auto_next: bool,
    pub id: Option<i64>,
}

impl GameState {
    pub fn new(grid: CubeGrid, solution: HashMap<CubeCoord, u8>, difficulty: Difficulty) -> Self {
        Self {
            grid,
            solution,
            difficulty,
            selected: None,
            highlight_number: None,
            started_at: 0.0,
            elapsed_seconds: 0,
            completed: false,
            history: Vec::new(),
            history_index: 0,
            auto_next: false,
            id: None,
        }
    }

    pub fn set_value(&mut self, coord: CubeCoord, value: Option<u8>) {
        if let Some(cell) = self.grid.get_mut(&coord) {
            if cell.given {
                return;
            }
            let old = cell.user_value;
            if old == value {
                return;
            }
            cell.user_value = value;
            self.push_history(MoveRecord { coord, old });
        }
    }

    pub fn hint(&mut self) {
        if let Some(coord) = self.selected {
            if let Some(&answer) = self.solution.get(&coord) {
                self.set_value(coord, Some(answer));
            }
        }
    }

    pub fn undo(&mut self) {
        if self.history_index == 0 {
            return;
        }
        self.history_index -= 1;
        let record = self.history[self.history_index].clone();
        if let Some(cell) = self.grid.get_mut(&record.coord) {
            cell.user_value = record.old;
        }
    }

    pub fn give_up(&mut self) {
        for (coord, cell) in self.grid.cells.iter_mut() {
            if !cell.given {
                cell.user_value = self.solution.get(coord).copied();
            }
        }
        self.completed = true;
    }

    fn push_history(&mut self, record: MoveRecord) {
        self.history.truncate(self.history_index);
        self.history.push(record);
        self.history_index += 1;
    }

    pub fn check_completion(&mut self) -> bool {
        if !self.completed && self.grid.is_complete() && self.grid.is_filled_correctly() {
            self.completed = true;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct MoveRecord {
    pub coord: CubeCoord,
    pub old: Option<u8>,
}
