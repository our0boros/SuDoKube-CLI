use crate::cube::{CubeCoord, Face};
use rand::seq::SliceRandom;
use rand::{Rng, RngCore};
use std::collections::{HashMap, HashSet};

/// 波函数坍缩题解生成器。
/// 按"顶点 -> 边 -> 面内部"三阶段坍缩，符合数独立方体的几何约束。
pub struct WfcGenerator;

impl WfcGenerator {
    pub fn new() -> Self {
        Self
    }

    /// 生成完整题解。
    pub fn generate<R: Rng>(&mut self, rng: &mut R) -> Option<HashMap<CubeCoord, u8>> {
        for _ in 0..2000 {
            if let Some(solution) = self.try_generate(rng) {
                return Some(solution);
            }
        }
        None
    }

    fn try_generate<R: Rng>(&self, rng: &mut R) -> Option<HashMap<CubeCoord, u8>> {
        let mut solution: HashMap<CubeCoord, u8> = HashMap::new();
        let rng_core: &mut dyn RngCore = rng;

        // Phase 1: 坍缩 8 个顶点。
        let corners = corner_coords();
        if !self.solve_vertices(&corners, &mut solution, rng_core) {
            return None;
        }

        // Phase 2: 坍缩 12 条边的非角点，回溯保证相邻面边界合法。
        let edges = edge_segments();
        let mut edge_order: Vec<usize> = (0..edges.len()).collect();
        edge_order.shuffle(rng_core);
        if !self.solve_edges(&edges, &edge_order, 0, &mut solution, rng_core) {
            return None;
        }

        // Phase 3: 坍缩每个面的内部 7x7 区域。
        for face in Face::ALL.iter() {
            if !self.solve_face(*face, &mut solution, rng_core) {
                return None;
            }
        }

        if verify_solution(&solution) {
            Some(solution)
        } else {
            None
        }
    }

    fn solve_vertices(
        &self,
        corners: &[CubeCoord],
        solution: &mut HashMap<CubeCoord, u8>,
        rng: &mut dyn RngCore,
    ) -> bool {
        let mut order: Vec<usize> = (0..corners.len()).collect();
        order.shuffle(rng);
        self.backtrack_vertices(corners, &order, 0, solution, rng)
    }

    fn backtrack_vertices(
        &self,
        corners: &[CubeCoord],
        order: &[usize],
        idx: usize,
        solution: &mut HashMap<CubeCoord, u8>,
        rng: &mut dyn RngCore,
    ) -> bool {
        if idx == order.len() {
            return true;
        }
        let coord = corners[order[idx]];
        let mut candidates: Vec<u8> = (1..=9).collect();
        candidates.shuffle(rng);

        for value in candidates {
            let mut ok = true;
            for other in solution.keys() {
                if are_related(coord, *other) && solution[other] == value {
                    ok = false;
                    break;
                }
            }
            if !ok {
                continue;
            }
            solution.insert(coord, value);
            if self.backtrack_vertices(corners, order, idx + 1, solution, rng) {
                return true;
            }
            solution.remove(&coord);
        }
        false
    }

    fn solve_edges(
        &self,
        edges: &[(CubeCoord, CubeCoord, Vec<CubeCoord>)],
        order: &[usize],
        idx: usize,
        solution: &mut HashMap<CubeCoord, u8>,
        rng: &mut dyn RngCore,
    ) -> bool {
        if idx == order.len() {
            return true;
        }
        let (a, b, middle) = &edges[order[idx]];
        let va = *solution.get(a).unwrap();
        let vb = *solution.get(b).unwrap();
        let used: HashSet<u8> = [va, vb].into_iter().collect();
        let pool: Vec<u8> = (1..=9).filter(|v| !used.contains(v)).collect();

        // 生成中间 7 个坐标的所有合法赋值（排列）。
        let mut perms = Vec::new();
        generate_permutations(&pool, &mut Vec::new(), &mut perms);
        perms.shuffle(rng);

        for perm in perms {
            for (coord, &value) in middle.iter().zip(perm.iter()) {
                solution.insert(*coord, value);
            }
            if self.edges_locally_valid(solution, a, b, middle) {
                if self.solve_edges(edges, order, idx + 1, solution, rng) {
                    return true;
                }
            }
            for coord in middle.iter() {
                solution.remove(coord);
            }
        }
        false
    }

    /// 检查新赋值的这条边在两个相邻面的边界上是否造成重复。
    fn edges_locally_valid(
        &self,
        solution: &HashMap<CubeCoord, u8>,
        _a: &CubeCoord,
        _b: &CubeCoord,
        middle: &[CubeCoord],
    ) -> bool {
        // 取边上任意一个非角点，确定相邻的两个面。
        let Some(sample) = middle.first() else {
            return true;
        };
        for fc in sample.to_face_coords() {
            if !face_partial_valid(fc.face, solution) {
                return false;
            }
        }
        true
    }

    fn solve_face(
        &self,
        face: Face,
        solution: &mut HashMap<CubeCoord, u8>,
        rng: &mut dyn RngCore,
    ) -> bool {
        let mut grid = [[0u8; 9]; 9];
        for v in 0..9u8 {
            for u in 0..9u8 {
                let coord = face.to_cube(u, v);
                if let Some(&val) = solution.get(&coord) {
                    grid[v as usize][u as usize] = val;
                }
            }
        }

        if !is_partial_valid(&grid) {
            return false;
        }
        if solve_sudoku(&mut grid, rng) {
            for v in 0..9u8 {
                for u in 0..9u8 {
                    let coord = face.to_cube(u, v);
                    solution.insert(coord, grid[v as usize][u as usize]);
                }
            }
            true
        } else {
            false
        }
    }
}

fn corner_coords() -> Vec<CubeCoord> {
    let mut coords = Vec::with_capacity(8);
    for x in [0u8, 8u8] {
        for y in [0u8, 8u8] {
            for z in [0u8, 8u8] {
                coords.push(CubeCoord { x, y, z });
            }
        }
    }
    coords
}

fn edge_segments() -> Vec<(CubeCoord, CubeCoord, Vec<CubeCoord>)> {
    let mut edges = Vec::with_capacity(12);
    for y in [0u8, 8u8] {
        for z in [0u8, 8u8] {
            let a = CubeCoord { x: 0, y, z };
            let b = CubeCoord { x: 8, y, z };
            let middle: Vec<_> = (1..=7u8).map(|x| CubeCoord { x, y, z }).collect();
            edges.push((a, b, middle));
        }
    }
    for x in [0u8, 8u8] {
        for z in [0u8, 8u8] {
            let a = CubeCoord { x, y: 0, z };
            let b = CubeCoord { x, y: 8, z };
            let middle: Vec<_> = (1..=7u8).map(|y| CubeCoord { x, y, z }).collect();
            edges.push((a, b, middle));
        }
    }
    for x in [0u8, 8u8] {
        for y in [0u8, 8u8] {
            let a = CubeCoord { x, y, z: 0 };
            let b = CubeCoord { x, y, z: 8 };
            let middle: Vec<_> = (1..=7u8).map(|z| CubeCoord { x, y, z }).collect();
            edges.push((a, b, middle));
        }
    }
    edges
}

fn generate_permutations(pool: &[u8], current: &mut Vec<u8>, out: &mut Vec<Vec<u8>>) {
    if current.len() == pool.len() {
        out.push(current.clone());
        return;
    }
    for &v in pool {
        if current.contains(&v) {
            continue;
        }
        current.push(v);
        generate_permutations(pool, current, out);
        current.pop();
    }
}

fn are_related(a: CubeCoord, b: CubeCoord) -> bool {
    for fc_a in a.to_face_coords() {
        for fc_b in b.to_face_coords() {
            if fc_a.face != fc_b.face {
                continue;
            }
            if fc_a.u == fc_b.u || fc_a.v == fc_b.v {
                return true;
            }
            if fc_a.u / 3 == fc_b.u / 3 && fc_a.v / 3 == fc_b.v / 3 {
                return true;
            }
        }
    }
    false
}

fn face_partial_valid(face: Face, solution: &HashMap<CubeCoord, u8>) -> bool {
    let mut grid = [[0u8; 9]; 9];
    for v in 0..9u8 {
        for u in 0..9u8 {
            let coord = face.to_cube(u, v);
            if let Some(&val) = solution.get(&coord) {
                grid[v as usize][u as usize] = val;
            }
        }
    }
    is_partial_valid(&grid)
}

fn is_partial_valid(grid: &[[u8; 9]; 9]) -> bool {
    for v in 0..9 {
        let mut seen = [false; 10];
        for u in 0..9 {
            let val = grid[v][u] as usize;
            if val == 0 {
                continue;
            }
            if seen[val] {
                return false;
            }
            seen[val] = true;
        }
    }
    for u in 0..9 {
        let mut seen = [false; 10];
        for v in 0..9 {
            let val = grid[v][u] as usize;
            if val == 0 {
                continue;
            }
            if seen[val] {
                return false;
            }
            seen[val] = true;
        }
    }
    for bv in 0..3 {
        for bu in 0..3 {
            let mut seen = [false; 10];
            for v in bv * 3..bv * 3 + 3 {
                for u in bu * 3..bu * 3 + 3 {
                    let val = grid[v][u] as usize;
                    if val == 0 {
                        continue;
                    }
                    if seen[val] {
                        return false;
                    }
                    seen[val] = true;
                }
            }
        }
    }
    true
}

fn solve_sudoku(grid: &mut [[u8; 9]; 9], rng: &mut dyn RngCore) -> bool {
    if let Some((v, u)) = find_empty(grid) {
        let mut candidates: Vec<u8> = (1..=9).filter(|&val| is_safe(grid, v, u, val)).collect();
        candidates.shuffle(rng);
        for val in candidates {
            grid[v][u] = val;
            if solve_sudoku(grid, rng) {
                return true;
            }
            grid[v][u] = 0;
        }
        false
    } else {
        true
    }
}

fn find_empty(grid: &[[u8; 9]; 9]) -> Option<(usize, usize)> {
    for v in 0..9 {
        for u in 0..9 {
            if grid[v][u] == 0 {
                return Some((v, u));
            }
        }
    }
    None
}

fn is_safe(grid: &[[u8; 9]; 9], v: usize, u: usize, val: u8) -> bool {
    for x in 0..9 {
        if grid[v][x] == val {
            return false;
        }
    }
    for y in 0..9 {
        if grid[y][u] == val {
            return false;
        }
    }
    let bu = (u / 3) * 3;
    let bv = (v / 3) * 3;
    for y in bv..bv + 3 {
        for x in bu..bu + 3 {
            if grid[y][x] == val {
                return false;
            }
        }
    }
    true
}

/// 验证一个完整解是否满足所有面的数独规则。
pub fn verify_solution(solution: &HashMap<CubeCoord, u8>) -> bool {
    for face in Face::ALL.iter() {
        for v in 0..9u8 {
            let mut seen = [false; 10];
            for u in 0..9u8 {
                let coord = face.to_cube(u, v);
                let value = *solution.get(&coord).unwrap_or(&0) as usize;
                if value < 1 || value > 9 || seen[value] {
                    return false;
                }
                seen[value] = true;
            }
        }
        for u in 0..9u8 {
            let mut seen = [false; 10];
            for v in 0..9u8 {
                let coord = face.to_cube(u, v);
                let value = *solution.get(&coord).unwrap_or(&0) as usize;
                if value < 1 || value > 9 || seen[value] {
                    return false;
                }
                seen[value] = true;
            }
        }
        for bv in 0..3u8 {
            for bu in 0..3u8 {
                let mut seen = [false; 10];
                for v in bv * 3..bv * 3 + 3 {
                    for u in bu * 3..bu * 3 + 3 {
                        let coord = face.to_cube(u, v);
                        let value = *solution.get(&coord).unwrap_or(&0) as usize;
                        if value < 1 || value > 9 || seen[value] {
                            return false;
                        }
                        seen[value] = true;
                    }
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn generate_and_verify() {
        let mut rng = thread_rng();
        let mut generator = WfcGenerator::new();
        let solution = generator
            .generate(&mut rng)
            .expect("should generate a solution");
        assert!(verify_solution(&solution));
    }
}
