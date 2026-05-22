use crate::tile::{Cell, Tile};
use crate::prng::Prng;

/// Faithfully replicates JS CellGrid including the seemingly-swapped naming:
///   JS: new CellGrid(ncols, nrows) → this.nrow = ncols, this.ncol = nrows
///   istilevalid: v.x < this.nrow && v.y < this.ncol
pub struct CellGrid {
    /// = ncols in the outer context (JS constructor arg 1)
    pub nrow: usize,
    /// = nrows in the outer context (JS constructor arg 2)
    pub ncol: usize,
    /// size = ncol * nrow (i.e. nrows * ncols)
    visited: Vec<bool>,
    /// size = (ncol-1) * (nrow-1)
    cellmap: Vec<bool>,
    nunvisited: usize,
}

impl CellGrid {
    pub fn new(nrow: usize, ncol: usize) -> Self {
        let n = ncol * nrow;
        // JS initialises cellmap as (ncol-1)*(nrow-1) but indexes it with
        // c.y * nrow + c.x, which can reach (ncol-2)*nrow + (nrow-2).
        // JS sparse arrays silently extend; we allocate the full ncol*nrow to cover
        // all reachable indices.
        CellGrid {
            nrow,
            ncol,
            visited: vec![false; n],
            cellmap: vec![false; n],
            nunvisited: n,
        }
    }

    pub fn nunvisited(&self) -> usize {
        self.nunvisited
    }

    pub fn reset(&mut self) {
        self.visited.fill(false);
        self.cellmap.fill(false);
        self.nunvisited = self.ncol * self.nrow;
    }

    pub fn is_tile_valid(&self, v: &Tile) -> bool {
        v.x >= 0 && v.x < self.nrow as i32 && v.y >= 0 && v.y < self.ncol as i32
    }

    pub fn is_tile_visited(&self, v: &Tile) -> bool {
        self.visited[v.y as usize * self.nrow + v.x as usize]
    }

    pub fn is_cell_empty(&self, c: &Cell) -> bool {
        !self.cellmap[c.y as usize * self.nrow + c.x as usize]
    }

    pub fn visit_tile(&mut self, v: &Tile) {
        let idx = v.y as usize * self.nrow + v.x as usize;
        if !self.visited[idx] {
            self.visited[idx] = true;
            self.nunvisited -= 1;
        }
    }

    pub fn occupy_cell(&mut self, c: &Cell) {
        let idx = c.y as usize * self.nrow + c.x as usize;
        if !self.cellmap[idx] {
            self.cellmap[idx] = true;
        }
    }

    pub fn liberate_cell(&mut self, c: &Cell) {
        self.cellmap[c.y as usize * self.nrow + c.x as usize] = false;
    }

    /// Replicates JS `randomemptytile()` — collects all unvisited indices,
    /// picks one via uniform(0, len), maps index → (x, y).
    pub fn random_empty_tile(&self, prng: &mut Prng) -> Tile {
        let empty: Vec<usize> = self
            .visited
            .iter()
            .enumerate()
            .filter(|(_, &v)| !v)
            .map(|(i, _)| i)
            .collect();
        let idx = empty[prng.uniform(0.0, empty.len() as f64).floor() as usize];
        let y = idx / self.nrow;
        let x = idx % self.nrow;
        Tile::new(x as i32, y as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grid_all_unvisited() {
        let g = CellGrid::new(5, 4);
        assert_eq!(g.nunvisited(), 20);
    }

    #[test]
    fn visit_decrements_nunvisited() {
        let mut g = CellGrid::new(5, 4);
        g.visit_tile(&Tile::new(0, 0));
        assert_eq!(g.nunvisited(), 19);
        // visiting same tile again is idempotent
        g.visit_tile(&Tile::new(0, 0));
        assert_eq!(g.nunvisited(), 19);
    }

    #[test]
    fn tile_validity_uses_swapped_naming() {
        // nrow=5 (ncols), ncol=4 (nrows)
        // valid x: 0..5, valid y: 0..4
        let g = CellGrid::new(5, 4);
        assert!(g.is_tile_valid(&Tile::new(4, 3)));
        assert!(!g.is_tile_valid(&Tile::new(5, 0))); // x == nrow → invalid
        assert!(!g.is_tile_valid(&Tile::new(0, 4))); // y == ncol → invalid
    }

    #[test]
    fn reset_restores_state() {
        let mut g = CellGrid::new(3, 3);
        g.visit_tile(&Tile::new(0, 0));
        g.visit_tile(&Tile::new(1, 1));
        g.reset();
        assert_eq!(g.nunvisited(), 9);
        assert!(!g.is_tile_visited(&Tile::new(0, 0)));
    }
}
