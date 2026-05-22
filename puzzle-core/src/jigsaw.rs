use crate::arc::Arc;
use crate::diagonal::DiagonalConnection;
use crate::frame::create_frame;
use crate::grid::CellGrid;
use crate::prng::Prng;
use crate::tile::Tile;
use crate::types::PuzzleConfig;

pub struct CircleFractalJigsaw {
    pub ncols: usize,
    pub nrows: usize,
    /// maskgrid[row][col]: 1 = outside border (excluded), 0 = inside
    pub maskgrid: Vec<Vec<u8>>,
    grid: CellGrid,
    /// Each piece is a Vec of DiagonalConnections.
    pub pieces: Vec<Vec<DiagonalConnection>>,
    min_piece: usize,
    max_piece: usize,
    pub coloring_seed: f64,
}

impl CircleFractalJigsaw {
    pub fn new(cfg: &PuzzleConfig) -> Self {
        let ncols = cfg.ncols;
        let nrows = cfg.nrows;
        let maskgrid = vec![vec![0u8; ncols]; nrows];
        // CellGrid constructor: new CellGrid(ncols, nrows) → nrow=ncols, ncol=nrows
        let grid = CellGrid::new(ncols, nrows);
        CircleFractalJigsaw {
            ncols,
            nrows,
            maskgrid,
            grid,
            pieces: Vec::new(),
            min_piece: cfg.min_piece,
            max_piece: cfg.max_piece,
            coloring_seed: cfg.seed,
        }
    }

    /// Apply a pre-rasterized mask (from browser plotpath+floodfill or server kurbo path).
    /// mask is row-major: mask[row * cols + col].
    pub fn set_mask(&mut self, mask: &[u8], rows: usize, cols: usize) {
        self.nrows = rows;
        self.ncols = cols;
        self.maskgrid = vec![vec![0u8; cols]; rows];
        for r in 0..rows {
            for c in 0..cols {
                self.maskgrid[r][c] = mask[r * cols + c];
            }
        }
        self.grid = CellGrid::new(cols, rows);
        for r in 0..rows {
            for c in 0..cols {
                if self.maskgrid[r][c] != 0 {
                    self.grid.visit_tile(&Tile::new(c as i32, r as i32));
                }
            }
        }
    }

    fn create_piece(&mut self, prng: &mut Prng) {
        let target_len =
            prng.uniform(self.min_piece as f64, self.max_piece as f64).round() as usize;
        let vi = self.grid.random_empty_tile(prng);
        let mut mytiles = vec![vi];
        let mut myconnections: Vec<DiagonalConnection> = Vec::new();
        self.grid.visit_tile(&vi);

        while self.grid.nunvisited() > 0 && mytiles.len() < target_len {
            // possible_connections needs mutable self but also borrows mytiles
            // We avoid the borrow conflict by working on a snapshot of tiles
            let pcs = self.possible_connections_snapshot(&mytiles, false);
            if pcs.is_empty() {
                break;
            }
            let chosen = &pcs[prng.uniform(0.0, pcs.len() as f64).floor() as usize];
            let chosen = *chosen;
            myconnections.push(chosen);
            mytiles.push(chosen.p2);
            self.grid.occupy_cell(&chosen.cell);
            self.grid.visit_tile(&chosen.p2);
        }

        if mytiles.len() >= self.min_piece {
            self.pieces.push(myconnections);
        } else {
            for c in &myconnections {
                self.grid.liberate_cell(&c.cell);
            }
        }
    }

    /// possible_connections variant that takes an immutable slice (no tile mutation).
    /// JS mutates `v.hasconnections` for optimization; we skip that optimization
    /// and accept the minor perf cost to avoid unsafe aliasing.
    fn possible_connections_snapshot(
        &self,
        mytiles: &[Tile],
        allow_partials: bool,
    ) -> Vec<DiagonalConnection> {
        let neighbors: [(i32, i32); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
        let mut pcs = Vec::new();
        for v in mytiles.iter() {
            if !v.has_connections && !allow_partials {
                continue;
            }
            for &(nx, ny) in &neighbors {
                let cpt = Tile::new(v.x + nx, v.y + ny);
                if !self.grid.is_tile_valid(&cpt) {
                    continue;
                }
                if self.maskgrid[cpt.y as usize][cpt.x as usize] != 0 {
                    continue;
                }
                if mytiles.iter().any(|nv| nv.eq_pos(&cpt)) {
                    continue;
                }
                let dc =
                    DiagonalConnection::new(*v, cpt, !self.grid.is_tile_visited(&cpt));
                if !self.grid.is_cell_empty(&dc.cell) {
                    continue;
                }
                if allow_partials || !self.grid.is_tile_visited(&cpt) {
                    pcs.push(dc);
                }
            }
        }
        pcs
    }

    /// Replicates JS `fillholes(allowpartials)`.
    pub fn fill_holes(&mut self, allow_partials: bool) -> bool {
        let mut filled = false;
        // Sort pieces by ascending length (ascending connection count)
        self.pieces.sort_by_key(|p| p.len());
        let n = self.pieces.len();
        for pi in 0..n {
            let mut tiles = {
                let p = &self.pieces[pi];
                if p.is_empty() {
                    Vec::new()
                } else {
                    let mut v = vec![p[0].p1];
                    for con in p.iter() {
                        v.push(con.p2);
                    }
                    v
                }
            };
            let tiles_snapshot = tiles.clone();
            for vi in 0..tiles_snapshot.len() {
                let v = tiles_snapshot[vi];
                let pcs = self.possible_connections_snapshot_single(v, &tiles, allow_partials);
                let filtered: Vec<DiagonalConnection> = pcs
                    .into_iter()
                    .filter(|pc| !tiles.iter().any(|t| t.eq_pos(&pc.p2)))
                    .collect();
                for pc in filtered {
                    tiles.push(pc.p2);
                    self.pieces[pi].push(pc);
                    filled = true;
                    self.grid.occupy_cell(&pc.cell);
                    self.grid.visit_tile(&pc.p2);
                }
            }
        }
        filled
    }

    fn possible_connections_snapshot_single(
        &self,
        v: Tile,
        mytiles: &[Tile],
        allow_partials: bool,
    ) -> Vec<DiagonalConnection> {
        let neighbors: [(i32, i32); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
        let mut pcs = Vec::new();
        for &(nx, ny) in &neighbors {
            let cpt = Tile::new(v.x + nx, v.y + ny);
            if !self.grid.is_tile_valid(&cpt) {
                continue;
            }
            if self.maskgrid[cpt.y as usize][cpt.x as usize] != 0 {
                continue;
            }
            if mytiles.iter().any(|nv| nv.eq_pos(&cpt)) {
                continue;
            }
            let dc = DiagonalConnection::new(v, cpt, !self.grid.is_tile_visited(&cpt));
            if !self.grid.is_cell_empty(&dc.cell) {
                continue;
            }
            if allow_partials || !self.grid.is_tile_visited(&cpt) {
                pcs.push(dc);
            }
        }
        pcs
    }

    /// Replicates JS `regenerategrid()` — resets grid and rebuilds from pieces + maskgrid.
    fn regenerate_grid(&mut self) {
        self.grid.reset();
        for i in 0..self.nrows {
            for j in 0..self.ncols {
                if self.maskgrid[i][j] != 0 {
                    self.grid.occupy_cell(&crate::tile::Cell::new(j as i32, i as i32));
                }
            }
        }
        for p in &self.pieces {
            for c in p {
                if !self.grid.is_tile_visited(&c.p1) {
                    self.grid.visit_tile(&c.p1);
                }
                if c.p2_taken && !self.grid.is_tile_visited(&c.p2) {
                    self.grid.visit_tile(&c.p2);
                }
                self.grid.occupy_cell(&c.cell);
            }
        }
    }

    /// Main generation loop. Matches JS `generate()`.
    pub fn generate(&mut self, prng: &mut Prng) {
        while self.grid.nunvisited() > 0 {
            self.create_piece(prng);
        }
        self.regenerate_grid();
    }

    pub fn n_pieces(&self) -> usize {
        self.pieces.len()
    }

    // ── SVG export helpers ────────────────────────────────────────────────

    fn piece_arcs(p: &[DiagonalConnection], rad: f64, frame: f64) -> Vec<Arc> {
        let mut arcs = Vec::new();
        Self::add_arcs(&p[0], p, &mut arcs, rad, frame, true);
        arcs
    }

    fn add_arcs(
        con: &DiagonalConnection,
        connections: &[DiagonalConnection],
        arcs: &mut Vec<Arc>,
        rad: f64,
        frame: f64,
        first: bool,
    ) {
        let newarc = match con.quad {
            0 => Arc::new(Tile::new(con.p1.x + 1, con.p1.y), rad, frame, 1, 1),
            1 => Arc::new(Tile::new(con.p1.x, con.p1.y - 1), rad, frame, 2, 1),
            2 => Arc::new(Tile::new(con.p1.x - 1, con.p1.y), rad, frame, 3, 1),
            3 => Arc::new(Tile::new(con.p1.x, con.p1.y + 1), rad, frame, 0, 1),
            _ => unreachable!(),
        };
        arcs.push(newarc);

        if con.p2_taken {
            for q in [(con.quad + 3) % 4, (con.quad + 4) % 4, (con.quad + 5) % 4] {
                let pct = DiagonalConnection::from_point_and_quad(con.p2, q, true);
                let pcnt = DiagonalConnection::from_point_and_quad(con.p2, q, false);
                if connections.iter().any(|c| c.eq(&pct)) {
                    Self::add_arcs(&pct, connections, arcs, rad, frame, false);
                } else if connections.iter().any(|c| c.eq(&pcnt)) {
                    Self::add_arcs(&pcnt, connections, arcs, rad, frame, false);
                } else {
                    arcs.push(Arc::new(con.p2, rad, frame, q, 0));
                }
            }
        } else {
            arcs.push(Arc::new(con.p2, rad, frame, (con.quad + 2) % 4, 1));
        }

        let newarc2 = match con.quad {
            0 => Arc::new(Tile::new(con.p1.x, con.p1.y - 1), rad, frame, 3, 1),
            1 => Arc::new(Tile::new(con.p1.x - 1, con.p1.y), rad, frame, 0, 1),
            2 => Arc::new(Tile::new(con.p1.x, con.p1.y + 1), rad, frame, 1, 1),
            3 => Arc::new(Tile::new(con.p1.x + 1, con.p1.y), rad, frame, 2, 1),
            _ => unreachable!(),
        };
        arcs.push(newarc2);

        if first {
            for q in [(con.quad + 1) % 4, (con.quad + 2) % 4, (con.quad + 3) % 4] {
                let pct = DiagonalConnection::from_point_and_quad(con.p1, q, true);
                let pcnt = DiagonalConnection::from_point_and_quad(con.p1, q, false);
                if connections.iter().any(|c| c.eq(&pct)) {
                    Self::add_arcs(&pct, connections, arcs, rad, frame, false);
                } else if connections.iter().any(|c| c.eq(&pcnt)) {
                    Self::add_arcs(&pcnt, connections, arcs, rad, frame, false);
                } else {
                    arcs.push(Arc::new(con.p1, rad, frame, q, 0));
                }
            }
        }
    }

    fn svg_header(width: f64, height: f64) -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\" ?>\
<svg baseProfile=\"full\" height=\"{}mm\" version=\"1.1\" \
viewBox=\"0 0 {} {}\" width=\"{}mm\" \
xmlns=\"http://www.w3.org/2000/svg\" \
xmlns:ev=\"http://www.w3.org/2001/xml-events\" \
xmlns:xlink=\"http://www.w3.org/1999/xlink\"><defs />",
            height, width, height, width
        )
    }

    fn frame_path_element(&self, rad: f64, frame: f64, frame_corner: f64) -> String {
        format!(
            "<path fill=\"none\" stroke=\"black\" stroke-width=\"0.1\" d=\"{}\"></path>",
            create_frame(self.ncols, self.nrows, rad, frame, frame_corner)
        )
    }

    fn multipaths(&self, frame: f64, rad: f64, arcshape: u8) -> Vec<String> {
        self.pieces
            .iter()
            .map(|p| {
                let arcs = Self::piece_arcs(p, rad, frame);
                let mut d = format!("M{},{} ", arcs[0].sp.x, arcs[0].sp.y);
                for a in &arcs {
                    d += &a.svg(arcshape);
                }
                d += "Z";
                d
            })
            .collect()
    }

    /// Mode 0: individual pieces with overlap.
    pub fn export_svg(&self, frame: f64, rad: f64, frame_corner: f64, arcshape: u8) -> String {
        let width = self.ncols as f64 * 2.0 * rad + 2.0 * frame;
        let height = self.nrows as f64 * 2.0 * rad + 2.0 * frame;
        let mut data = Self::svg_header(width, height);
        for p in &self.pieces {
            let arcs = Self::piece_arcs(p, rad, frame);
            data += &format!(
                "<path fill=\"none\" stroke=\"black\" stroke-width=\"0.1\" d=\"M{},{} ",
                arcs[0].sp.x, arcs[0].sp.y
            );
            for a in &arcs {
                data += &a.svg(arcshape);
            }
            data += "Z\"></path>";
        }
        data += &self.frame_path_element(rad, frame, frame_corner);
        data += "</svg>";
        data
    }

    /// Mode 2: non-overlapping single path (Trotec workaround).
    pub fn export_svg_nooverlap_singlepath(
        &self,
        frame: f64,
        rad: f64,
        frame_corner: f64,
        arcshape: u8,
    ) -> String {
        let width = self.ncols as f64 * 2.0 * rad + 2.0 * frame;
        let height = self.nrows as f64 * 2.0 * rad + 2.0 * frame;
        let mut data = Self::svg_header(width, height);
        let mut all_arcs: Vec<Arc> = Vec::new();
        data += "<path fill=\"none\" stroke=\"black\" stroke-width=\"0.1\" d=\"";
        let mut current_x = -1i32;
        let mut current_y = -1i32;
        for p in &self.pieces {
            let arcs = Self::piece_arcs(p, rad, frame);
            for a in arcs {
                if !all_arcs.iter().any(|na| na.eq(&a)) {
                    all_arcs.push(Arc::new(a.cp, a.rad, 0.0, a.quad, a.sign)); // push a copy
                    if a.sp.x != current_x || a.sp.y != current_y {
                        data += &format!("M{},{} ", a.sp.x, a.sp.y);
                    }
                    data += &a.svg(arcshape);
                    current_x = a.ep.x;
                    current_y = a.ep.y;
                }
            }
        }
        data += "\"></path>";
        data += &self.frame_path_element(rad, frame, frame_corner);
        data += "</svg>";
        data
    }

    /// Mode 1: non-overlapping, multiple paths.
    pub fn export_svg_nooverlap(
        &self,
        frame: f64,
        rad: f64,
        frame_corner: f64,
        arcshape: u8,
    ) -> String {
        let width = self.ncols as f64 * 2.0 * rad + 2.0 * frame;
        let height = self.nrows as f64 * 2.0 * rad + 2.0 * frame;
        let mut data = Self::svg_header(width, height);
        let mut all_arcs: Vec<Arc> = Vec::new();
        for p in &self.pieces {
            let arcs = Self::piece_arcs(p, rad, frame);
            let mut in_path = false;
            let mut path = String::new();
            for a in arcs {
                if all_arcs.iter().any(|na| na.eq(&a)) {
                    if in_path {
                        path += "\"></path>";
                        data += &path;
                        in_path = false;
                    }
                } else {
                    all_arcs.push(Arc::new(a.cp, a.rad, 0.0, a.quad, a.sign));
                    if !in_path {
                        path = format!(
                            "<path fill=\"none\" stroke=\"black\" stroke-width=\"0.1\" d=\"M{},{} ",
                            a.sp.x, a.sp.y
                        );
                        in_path = true;
                    }
                    path += &a.svg(arcshape);
                }
            }
            if in_path {
                path += "\"></path>";
                data += &path;
            }
        }
        data += &self.frame_path_element(rad, frame, frame_corner);
        data += "</svg>";
        data
    }

    /// Mode 3: colored pieces. Uses coloring_seed to replay PRNG from snapshot.
    pub fn export_svg_colored(
        &self,
        frame: f64,
        rad: f64,
        frame_corner: f64,
        arcshape: u8,
        coloring_seed: f64,
    ) -> String {
        let width = self.ncols as f64 * 2.0 * rad + 2.0 * frame;
        let height = self.nrows as f64 * 2.0 * rad + 2.0 * frame;
        let mut data = Self::svg_header(width, height);
        let mut prng = Prng::new(coloring_seed);
        let paths = self.multipaths(frame, rad, arcshape);
        for p in &paths {
            let color = prng.uniform(0.0, 16777216.0).floor() as u32;
            data += &format!(
                "<path fill=\"#{:06x}\" stroke=\"black\" stroke-width=\"{}\" d=\"",
                color,
                rad / 20.0
            );
            data += p;
            data += "\"></path>";
        }
        data += &self.frame_path_element(rad, frame, frame_corner);
        data += "</svg>";
        data
    }

    /// Dispatch to one of the 4 export modes.
    pub fn export(&self, mode: u8, frame: f64, rad: f64, frame_corner: f64, arcshape: u8) -> String {
        match mode {
            0 => self.export_svg(frame, rad, frame_corner, arcshape),
            1 => self.export_svg_nooverlap(frame, rad, frame_corner, arcshape),
            2 => self.export_svg_nooverlap_singlepath(frame, rad, frame_corner, arcshape),
            3 => self.export_svg_colored(frame, rad, frame_corner, arcshape, self.coloring_seed),
            _ => String::new(),
        }
    }
}
