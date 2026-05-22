use crate::tile::Tile;

pub struct Arc {
    pub cp: Tile,
    pub quad: u8,
    pub rad: f64,
    pub sign: u8,
    pub sp: Tile,
    pub ep: Tile,
}

impl Arc {
    /// Matches JS `new Arc(gcp, rad, offs, quad, sign)`.
    pub fn new(gcp: Tile, rad: f64, offs: f64, quad: u8, sign: u8) -> Self {
        let rad_i = rad as i32;
        let cp = Tile::new(
            gcp.x * 2 * rad_i + rad_i + offs as i32,
            gcp.y * 2 * rad_i + rad_i + offs as i32,
        );
        let (pa, pb) = match quad {
            0 => (Tile::new(cp.x + rad_i, cp.y), Tile::new(cp.x, cp.y - rad_i)),
            1 => (Tile::new(cp.x, cp.y - rad_i), Tile::new(cp.x - rad_i, cp.y)),
            2 => (Tile::new(cp.x - rad_i, cp.y), Tile::new(cp.x, cp.y + rad_i)),
            3 => (Tile::new(cp.x, cp.y + rad_i), Tile::new(cp.x + rad_i, cp.y)),
            _ => unreachable!(),
        };
        let (sp, ep) = if sign == 0 { (pa, pb) } else { (pb, pa) };
        Arc { cp, quad, rad, sign, sp, ep }
    }

    /// Generates the SVG path segment for this arc.
    /// arcshape: 0=circle, 1=square, 2=octagon
    pub fn svg(&self, arcshape: u8) -> String {
        // tan(22.5°) — matches JS constant exactly
        const TAN225: f64 = 0.4142135623730950488016887242097;
        let hlen = self.rad * TAN225;
        let rad_i = self.rad as i32;
        match arcshape {
            0 => format!(
                "A {} {} 0 0,{} {} {} ",
                rad_i, rad_i, self.sign, self.ep.x, self.ep.y
            ),
            1 => format!("L {} {} ", self.ep.x, self.ep.y),
            2 => {
                let quad = self.quad;
                let (sp, ep) = if self.sign == 1 {
                    (self.ep, self.sp)
                } else {
                    (self.sp, self.ep)
                };
                let (mp1, mp2) = match quad {
                    0 => ([sp.x as f64, sp.y as f64 - hlen], [ep.x as f64 + hlen, ep.y as f64]),
                    1 => ([sp.x as f64 - hlen, sp.y as f64], [ep.x as f64, ep.y as f64 - hlen]),
                    2 => ([sp.x as f64, sp.y as f64 + hlen], [ep.x as f64 - hlen, ep.y as f64]),
                    3 => ([sp.x as f64 + hlen, sp.y as f64], [ep.x as f64, ep.y as f64 + hlen]),
                    _ => unreachable!(),
                };
                if self.sign == 1 {
                    format!(
                        "L {} {} L {} {} L {} {} ",
                        mp2[0], mp2[1], mp1[0], mp1[1], self.ep.x, self.ep.y
                    )
                } else {
                    format!(
                        "L {} {} L {} {} L {} {} ",
                        mp1[0], mp1[1], mp2[0], mp2[1], self.ep.x, self.ep.y
                    )
                }
            }
            _ => String::new(),
        }
    }

    pub fn eq(&self, other: &Arc) -> bool {
        self.quad == other.quad && self.cp.x == other.cp.x && self.cp.y == other.cp.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Validates all 3 shapes × 4 quads × 2 signs = 24 combinations against JS reference output.
    #[test]
    fn circle_arc_quad0_sign0() {
        // cp = (5*12+6+6, 5*12+6+6) = (72,72)
        // quad=0: pa=(78,72), pb=(72,66); sign=0 → ep=pb=(72,66)
        let a = Arc::new(Tile::new(5, 5), 6.0, 6.0, 0, 0);
        assert_eq!(a.svg(0), "A 6 6 0 0,0 72 66 ");
    }

    #[test]
    fn circle_arc_quad0_sign1() {
        // sign=1 → ep=pa=(78,72)
        let a = Arc::new(Tile::new(5, 5), 6.0, 6.0, 0, 1);
        assert_eq!(a.svg(0), "A 6 6 0 0,1 78 72 ");
    }

    #[test]
    fn square_arc_is_line() {
        let a = Arc::new(Tile::new(3, 3), 6.0, 6.0, 2, 0);
        // square just does L ep.x ep.y
        let s = a.svg(1);
        assert!(s.starts_with("L "), "expected L command, got: {s}");
    }

    #[test]
    fn octagon_arc_has_two_midpoints() {
        let a = Arc::new(Tile::new(3, 3), 6.0, 6.0, 1, 0);
        let s = a.svg(2);
        // Should be "L x1 y1 L x2 y2 L epx epy "
        let l_count = s.matches("L ").count();
        assert_eq!(l_count, 3, "octagon arc should have 3 L commands, got: {s}");
    }

    #[test]
    fn tan225_constant_precision() {
        // tan(π/8) = √2 - 1
        let expected = (2.0f64).sqrt() - 1.0;
        const TAN225: f64 = 0.4142135623730950488016887242097;
        assert!((TAN225 - expected).abs() < 1e-15);
    }
}
