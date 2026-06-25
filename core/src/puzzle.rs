use crate::cube::{CubeCoord, CubeGrid, Difficulty, iter_surface_coords};
use rand::Rng;
use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};

/// 基于完整题解生成一个可解谜题。
/// 挖空策略：随机打乱后逐个尝试隐藏，并通过求解器确保当前题目仍有唯一解。
pub fn generate_puzzle<R: Rng + ?Sized>(
    solution: &HashMap<CubeCoord, u8>,
    difficulty: Difficulty,
    rng: &mut R,
) -> CubeGrid {
    let mut coords: Vec<CubeCoord> = iter_surface_coords().collect();
    coords.shuffle(rng);

    let mut given: HashSet<CubeCoord> = coords.iter().copied().collect();
    let target = difficulty.hidden_count();

    for coord in coords {
        if given.len() <= 386 - target {
            break;
        }
        given.remove(&coord);
        let puzzle = build_partial_solution(solution, &given);
        if count_solutions(&puzzle, 2) != 1 {
            // 会导致多解或无解，恢复该格。
            given.insert(coord);
        }
    }

    CubeGrid::from_solution(solution.clone(), &given)
}

fn build_partial_solution(
    solution: &HashMap<CubeCoord, u8>,
    given: &HashSet<CubeCoord>,
) -> HashMap<CubeCoord, u8> {
    solution
        .iter()
        .filter(|(c, _)| given.contains(c))
        .map(|(&c, &v)| (c, v))
        .collect()
}

/// 统计基于部分赋值的完整解数量（上限为 limit）。
pub fn count_solutions(partial: &HashMap<CubeCoord, u8>, limit: usize) -> usize {
    let all: HashSet<u8> = (1..=9).collect();
    let mut candidates: HashMap<CubeCoord, HashSet<u8>> =
        iter_surface_coords().map(|c| (c, all.clone())).collect();
    let mut assigned: HashMap<CubeCoord, u8> = HashMap::new();

    // 应用初始赋值。
    for (&coord, &value) in partial.iter() {
        if !assign(&mut candidates, &mut assigned, coord, value) {
            return 0;
        }
    }

    let mut count = 0;
    let mut order: Vec<CubeCoord> = iter_surface_coords().collect();
    order.sort_by_key(|c| candidates.get(c).map_or(10, |s| s.len()));
    solve_count(&mut candidates, &mut assigned, &order, limit, &mut count);
    count
}

fn solve_count(
    candidates: &mut HashMap<CubeCoord, HashSet<u8>>,
    assigned: &mut HashMap<CubeCoord, u8>,
    order: &[CubeCoord],
    limit: usize,
    count: &mut usize,
) {
    if assigned.len() == order.len() {
        *count += 1;
        return;
    }
    if *count >= limit {
        return;
    }

    let coord = order
        .iter()
        .filter(|c| !assigned.contains_key(c))
        .min_by_key(|c| candidates.get(*c).map_or(10, |s| s.len()));

    let Some(coord) = coord else { return };
    let values: Vec<u8> = candidates
        .get(coord)
        .map(|s| s.iter().copied().collect())
        .unwrap_or_default();

    for value in values {
        let snap_c = candidates.clone();
        let snap_a = assigned.clone();
        if assign(candidates, assigned, *coord, value) {
            solve_count(candidates, assigned, order, limit, count);
        }
        *candidates = snap_c;
        *assigned = snap_a;
        if *count >= limit {
            return;
        }
    }
}

fn assign(
    candidates: &mut HashMap<CubeCoord, HashSet<u8>>,
    assigned: &mut HashMap<CubeCoord, u8>,
    coord: CubeCoord,
    value: u8,
) -> bool {
    for related in coord.related() {
        if let Some(&v) = assigned.get(&related) {
            if v == value {
                return false;
            }
        }
    }

    assigned.insert(coord, value);
    candidates.insert(coord, [value].into_iter().collect());

    let queue: Vec<CubeCoord> = coord.related();
    let mut head = 0;
    while head < queue.len() {
        let cur = queue[head];
        head += 1;
        if assigned.contains_key(&cur) {
            continue;
        }
        let set = match candidates.get_mut(&cur) {
            Some(s) => s,
            None => continue,
        };
        if set.remove(&value) && set.len() == 1 {
            let forced = *set.iter().next().unwrap();
            if !assign(candidates, assigned, cur, forced) {
                return false;
            }
        } else if set.is_empty() {
            return false;
        }
    }

    true
}
