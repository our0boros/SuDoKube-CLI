use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, BTreeSet};

/// 立方体表面的三维坐标。每个坐标在 0..9 之间，且至少一维为 0 或 8。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CubeCoord {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

/// 立方体的六个面。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Face {
    Right,  // +X, x = 8
    Left,   // -X, x = 0
    Top,    // +Y, y = 8
    Bottom, // -Y, y = 0
    Front,  // +Z, z = 8
    Back,   // -Z, z = 0
}

/// 某个面上的局部坐标。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FaceCoord {
    pub face: Face,
    pub u: u8,
    pub v: u8,
}

impl Face {
    pub const ALL: [Face; 6] = [
        Face::Right,
        Face::Left,
        Face::Top,
        Face::Bottom,
        Face::Front,
        Face::Back,
    ];

    /// 单字符代号：前/后/左/右/上/下
    pub fn short_code(&self) -> &'static str {
        match self {
            Face::Front => "F",
            Face::Back => "B",
            Face::Left => "L",
            Face::Right => "R",
            Face::Top => "T",
            Face::Bottom => "U",
        }
    }

    /// 将面局部坐标转换为唯一的立方体表面坐标。
    pub fn to_cube(&self, u: u8, v: u8) -> CubeCoord {
        match self {
            Face::Right => CubeCoord { x: 8, y: u, z: v },
            Face::Left => CubeCoord { x: 0, y: v, z: u },
            Face::Top => CubeCoord { x: u, y: 8, z: v },
            Face::Bottom => CubeCoord { x: v, y: 0, z: u },
            Face::Front => CubeCoord { x: u, y: v, z: 8 },
            Face::Back => CubeCoord { x: v, y: u, z: 0 },
        }
    }

    /// 该面的行索引：按局部 v 值。
    pub fn row(&self, row_idx: u8) -> Vec<CubeCoord> {
        (0..9u8).map(|u| self.to_cube(u, row_idx)).collect()
    }

    /// 该面的列索引：按局部 u 值。
    pub fn col(&self, col_idx: u8) -> Vec<CubeCoord> {
        (0..9u8).map(|v| self.to_cube(col_idx, v)).collect()
    }

    /// 该面指定 3x3 宫内的所有坐标。
    pub fn box_coords(&self, box_u: u8, box_v: u8) -> Vec<CubeCoord> {
        let base_u = box_u * 3;
        let base_v = box_v * 3;
        (base_v..base_v + 3)
            .flat_map(move |v| (base_u..base_u + 3).map(move |u| self.to_cube(u, v)))
            .collect()
    }

    /// 包含指定局部坐标 (u,v) 的 3x3 宫。
    pub fn box_at(&self, u: u8, v: u8) -> Vec<CubeCoord> {
        self.box_coords(u / 3, v / 3)
    }

    /// 返回该面的四条边中，与相邻面共享的边的信息：
    /// (边名称, 相邻面, 相邻面中的对应边行/列索引, 是否沿 u 方向)
    pub fn shared_edges(&self) -> [SharedEdge; 4] {
        match self {
            Face::Right => [
                SharedEdge::new(Face::Bottom, 8, true), // v = 0 -> Bottom max-v
                SharedEdge::new(Face::Top, 8, true),    // v = 8 -> Top max-v
                SharedEdge::new(Face::Back, 8, false),  // u = 0 -> Back max-v
                SharedEdge::new(Face::Front, 8, false), // u = 8 -> Front max-u
            ],
            Face::Left => [
                SharedEdge::new(Face::Bottom, 0, true), // v = 0
                SharedEdge::new(Face::Top, 0, true),    // v = 8
                SharedEdge::new(Face::Back, 0, false),  // u = 0
                SharedEdge::new(Face::Front, 0, false), // u = 8
            ],
            Face::Top => [
                SharedEdge::new(Face::Back, 8, true),   // v = 0
                SharedEdge::new(Face::Front, 8, true),  // v = 8
                SharedEdge::new(Face::Left, 8, false),  // u = 0
                SharedEdge::new(Face::Right, 8, false), // u = 8
            ],
            Face::Bottom => [
                SharedEdge::new(Face::Front, 0, true),  // v = 0
                SharedEdge::new(Face::Back, 0, true),   // v = 8
                SharedEdge::new(Face::Left, 0, false),  // u = 0
                SharedEdge::new(Face::Right, 0, false), // u = 8
            ],
            Face::Front => [
                SharedEdge::new(Face::Bottom, 8, true), // v = 0
                SharedEdge::new(Face::Top, 8, true),    // v = 8
                SharedEdge::new(Face::Left, 8, false),  // u = 0
                SharedEdge::new(Face::Right, 8, false), // u = 8
            ],
            Face::Back => [
                SharedEdge::new(Face::Bottom, 0, true), // v = 0
                SharedEdge::new(Face::Top, 0, true),    // v = 8
                SharedEdge::new(Face::Right, 0, false), // u = 0
                SharedEdge::new(Face::Left, 0, false),  // u = 8
            ],
        }
    }
}

impl FaceCoord {
    #[allow(dead_code)]
    pub fn to_cube(&self) -> CubeCoord {
        self.face.to_cube(self.u, self.v)
    }
}

impl CubeCoord {
    /// 判断是否为合法表面坐标。
    pub fn is_surface(&self) -> bool {
        self.x < 9
            && self.y < 9
            && self.z < 9
            && (self.x == 0
                || self.x == 8
                || self.y == 0
                || self.y == 8
                || self.z == 0
                || self.z == 8)
    }

    /// 返回该立方体坐标所属的所有面局部坐标（1~3 个）。
    pub fn to_face_coords(&self) -> Vec<FaceCoord> {
        let mut result = Vec::with_capacity(3);
        if self.x == 8 {
            result.push(FaceCoord {
                face: Face::Right,
                u: self.y,
                v: self.z,
            });
        }
        if self.x == 0 {
            result.push(FaceCoord {
                face: Face::Left,
                u: self.z,
                v: self.y,
            });
        }
        if self.y == 8 {
            result.push(FaceCoord {
                face: Face::Top,
                u: self.x,
                v: self.z,
            });
        }
        if self.y == 0 {
            result.push(FaceCoord {
                face: Face::Bottom,
                u: self.z,
                v: self.x,
            });
        }
        if self.z == 8 {
            result.push(FaceCoord {
                face: Face::Front,
                u: self.x,
                v: self.y,
            });
        }
        if self.z == 0 {
            result.push(FaceCoord {
                face: Face::Back,
                u: self.y,
                v: self.x,
            });
        }
        result
    }

    /// 返回与该坐标在数独约束下相关的所有坐标（去重，不含自身）。
    /// 即：在每个所属面中，同行、同列、同宫的所有格子。
    pub fn related(&self) -> Vec<CubeCoord> {
        let mut set = HashSet::new();
        for fc in self.to_face_coords() {
            for c in fc.face.row(fc.v) {
                set.insert(c);
            }
            for c in fc.face.col(fc.u) {
                set.insert(c);
            }
            for c in fc.face.box_at(fc.u, fc.v) {
                set.insert(c);
            }
        }
        set.remove(self);
        set.into_iter().collect()
    }

    /// 返回与该坐标共享行或列的跨面坐标。
    /// 对每条共享边，若该坐标位于某面的边行/列，则把相邻面对应边行/列的坐标加入。
    pub fn extended_related(&self) -> Vec<CubeCoord> {
        let mut set = HashSet::new();
        for fc in self.to_face_coords() {
            let edges = fc.face.shared_edges();
            // 四条边顺序固定为：min-v, max-v, min-u, max-u
            let edge_indices = [
                (fc.v == 0, &edges[0]),
                (fc.v == 8, &edges[1]),
                (fc.u == 0, &edges[2]),
                (fc.u == 8, &edges[3]),
            ];
            for (on_edge, shared) in edge_indices.iter() {
                if !on_edge {
                    continue;
                }
                if shared.along_u {
                    for u in 0..9u8 {
                        set.insert(shared.neighbor.to_cube(u, shared.index));
                    }
                } else {
                    for v in 0..9u8 {
                        set.insert(shared.neighbor.to_cube(shared.index, v));
                    }
                }
            }
        }
        set.remove(self);
        set.into_iter().collect()
    }
}

/// 描述两个相邻面之间共享边的关系。
#[derive(Debug, Clone, Copy)]
pub struct SharedEdge {
    pub neighbor: Face,
    /// 在相邻面中固定的行或列索引（0..9）。
    pub index: u8,
    /// 若 true，表示相邻面中沿 u 方向变化（即固定 v = index）。
    pub along_u: bool,
}

impl SharedEdge {
    fn new(neighbor: Face, index: u8, along_u: bool) -> Self {
        Self {
            neighbor,
            index,
            along_u,
        }
    }
}

/// 遍历所有立方体表面坐标（386 个）。
pub fn iter_surface_coords() -> impl Iterator<Item = CubeCoord> {
    (0..9u8).flat_map(move |x| {
        (0..9u8).flat_map(move |y| {
            (0..9u8).filter_map(move |z| {
                let c = CubeCoord { x, y, z };
                if c.is_surface() { Some(c) } else { None }
            })
        })
    })
}

/// 谜题难度配置。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    /// 该难度下需要隐藏的数字数量（约占 386 格子的比例）。
    pub fn hidden_count(&self) -> usize {
        match self {
            Difficulty::Easy => 100,
            Difficulty::Medium => 160,
            Difficulty::Hard => 280,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Difficulty::Easy => "简单",
            Difficulty::Medium => "中等",
            Difficulty::Hard => "困难",
        }
    }
}

/// 游戏格子。
#[derive(Debug, Clone)]
pub struct Cell {
    pub answer: u8,
    pub given: bool,
    pub user_value: Option<u8>,
    /// 草稿标记集合（1-9），使用 BTreeSet 保证有序
    pub draft: BTreeSet<u8>,
    /// 草稿是否可见（关闭草稿模式时隐藏，擦除时重新显示）
    pub draft_visible: bool,
}

impl Cell {
    pub fn new(answer: u8, given: bool) -> Self {
        Self {
            answer,
            given,
            user_value: if given { Some(answer) } else { None },
            draft: BTreeSet::new(),
            draft_visible: true,
        }
    }

    pub fn is_correct(&self) -> bool {
        self.user_value == Some(self.answer)
    }

    /// 切换草稿数字：若已存在则擦除，否则添加
    pub fn toggle_draft(&mut self, n: u8) {
        if self.draft.contains(&n) {
            self.draft.remove(&n);
        } else {
            self.draft.insert(n);
        }
    }

    /// 擦除草稿数字，同时标记草稿为可见
    pub fn erase_draft(&mut self, n: u8) {
        if self.draft.remove(&n) {
            self.draft_visible = true;
        }
    }
}

/// 当前游戏棋盘。
#[derive(Debug, Clone)]
pub struct CubeGrid {
    pub cells: HashMap<CubeCoord, Cell>,
}

impl CubeGrid {
    pub fn from_solution(solution: HashMap<CubeCoord, u8>, given: &HashSet<CubeCoord>) -> Self {
        let cells = solution
            .iter()
            .map(|(&coord, &answer)| {
                let is_given = given.contains(&coord);
                (coord, Cell::new(answer, is_given))
            })
            .collect();
        Self { cells }
    }

    pub fn get(&self, coord: &CubeCoord) -> Option<&Cell> {
        self.cells.get(coord)
    }

    pub fn get_mut(&mut self, coord: &CubeCoord) -> Option<&mut Cell> {
        self.cells.get_mut(coord)
    }

    /// 检查当前所有已填数字是否与答案一致。
    pub fn is_filled_correctly(&self) -> bool {
        self.cells.values().all(|c| c.is_correct())
    }

    /// 检查是否所有格子都已填满。
    pub fn is_complete(&self) -> bool {
        self.cells.values().all(|c| c.user_value.is_some())
    }

    /// 将棋盘序列化为固定顺序字符串（空值用 0）。
    pub fn serialize(&self, coords: &[CubeCoord]) -> String {
        coords
            .iter()
            .map(|c| match self.cells.get(c) {
                Some(cell) => cell.user_value.map_or('0', |v| (b'0' + v) as char),
                None => '0',
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_count_is_386() {
        let coords: Vec<_> = iter_surface_coords().collect();
        assert_eq!(coords.len(), 386);
    }

    #[test]
    fn coord_round_trip() {
        for cube in iter_surface_coords() {
            let face_coords = cube.to_face_coords();
            for fc in face_coords {
                assert_eq!(fc.to_cube(), cube);
            }
        }
    }

    #[test]
    fn related_does_not_contain_self() {
        let c = CubeCoord { x: 8, y: 4, z: 4 };
        let rel = c.related();
        assert!(!rel.contains(&c));
    }
}
