#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tile {
    pub x: i32,
    pub y: i32,
    pub has_connections: bool,
}

impl Tile {
    pub fn new(x: i32, y: i32) -> Self {
        Tile { x, y, has_connections: true }
    }

    pub fn eq_pos(&self, other: &Tile) -> bool {
        self.x == other.x && self.y == other.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
}

impl Cell {
    pub fn new(x: i32, y: i32) -> Self {
        Cell { x, y }
    }
}
