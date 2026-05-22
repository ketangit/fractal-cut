use crate::tile::{Cell, Tile};

#[derive(Debug, Clone, Copy)]
pub struct DiagonalConnection {
    pub p1: Tile,
    pub p2: Tile,
    pub p2_taken: bool,
    pub slope: f64,
    pub quad: u8,
    pub cell: Cell,
}

impl DiagonalConnection {
    pub fn new(p1: Tile, p2: Tile, p2_taken: bool) -> Self {
        let slope = (p2.y - p1.y) as f64 / (p2.x - p1.x) as f64;
        let ccx = p1.x.min(p2.x);
        let ccy = p1.y.min(p2.y);
        let cell = Cell::new(ccx, ccy);
        let quad = if slope > 0.0 {
            if p2.y > p1.y { 3 } else { 1 }
        } else {
            if p2.y > p1.y { 2 } else { 0 }
        };
        DiagonalConnection { p1, p2, p2_taken, slope, quad, cell }
    }

    /// Matches JS `DiagonalConnection.FromPointAndQuad(p1, quadrant, p2_taken)`.
    pub fn from_point_and_quad(p1: Tile, quadrant: u8, p2_taken: bool) -> Self {
        let p2 = match quadrant {
            0 => Tile::new(p1.x + 1, p1.y - 1),
            1 => Tile::new(p1.x - 1, p1.y - 1),
            2 => Tile::new(p1.x - 1, p1.y + 1),
            3 => Tile::new(p1.x + 1, p1.y + 1),
            _ => unreachable!(),
        };
        DiagonalConnection::new(p1, p2, p2_taken)
    }

    /// Structural equality: same cell position, same slope sign, same p2_taken.
    /// Matches JS `eq()`.
    pub fn eq(&self, other: &DiagonalConnection) -> bool {
        self.cell.x == other.cell.x
            && self.cell.y == other.cell.y
            && (self.slope - other.slope).abs() < 1e-9
            && self.p2_taken == other.p2_taken
    }
}
