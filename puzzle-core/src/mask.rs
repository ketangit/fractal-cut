/// Server-side custom border rasterization using kurbo.
/// Browser path: JS plotpath() → Uint8Array → set_mask().
/// Server path: SVG path string → kurbo arc-length sampling → maskgrid.
///
/// The adaptive sampling algorithm matches JS plotpath() (index.html:36-62):
///   uinc = 1/128, step along arc length, subdivide when adjacent cells gap > 1,
///   down to uinc < 1/65536 before forcing a point.

#[cfg(not(feature = "bindgen"))]
pub mod server {
    use kurbo::{BezPath, ParamCurve, ParamCurveArclen, Shape};

    const UINC_START: f64 = 0.0078125;           // 1/128
    const UINC_MIN: f64 = 0.0000152587890625;    // 1/65536

    #[derive(Debug, Clone, Copy)]
    struct Pt {
        x: i32,
        y: i32,
        rx: f64,
        ry: f64,
    }

    /// Rasterize one SVG path string into grid coordinates.
    /// `dia` = 2 * crad (tile diameter).
    fn plot_path(path_d: &str, dia: f64) -> Vec<Pt> {
        let bez = BezPath::from_svg(path_d).unwrap_or_default();
        let total_len = bez.perimeter(1e-6);
        if total_len <= 0.0 {
            return Vec::new();
        }

        let mut cp: Vec<Pt> = Vec::new();
        let mut uinc = UINC_START;
        let mut u = 0.0f64;

        while u <= 1.0 {
            // Sample point at arc-length parameter u
            let arc_u = u * total_len;
            let p_real = sample_at_arclen(&bez, arc_u, total_len);
            let px = (p_real.0 / dia).round() as i32;
            let py = (p_real.1 / dia).round() as i32;
            let pt = Pt { x: px, y: py, rx: p_real.0, ry: p_real.1 };

            if let Some(last) = cp.last() {
                if (last.x - pt.x).abs() > 1 || (last.y - pt.y).abs() > 1 {
                    // Too far — back up and halve step
                    u -= uinc;
                    uinc /= 2.0;
                    if uinc < UINC_MIN {
                        uinc = UINC_START;
                        cp.push(pt);
                    }
                    u += uinc;
                    continue;
                }
            }
            cp.push(pt);
            u += uinc;
        }
        cp
    }

    /// Sample a BezPath at a given arc-length position.
    fn sample_at_arclen(bez: &BezPath, target_len: f64, _total_len: f64) -> (f64, f64) {
        // Walk segments to find the right one
        let mut accumulated = 0.0f64;
        for seg in bez.segments() {
            let seg_len = seg.arclen(1e-6);
            if accumulated + seg_len >= target_len {
                let local_t = if seg_len > 0.0 {
                    ((target_len - accumulated) / seg_len).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let p = seg.eval(local_t);
                return (p.x, p.y);
            }
            accumulated += seg_len;
        }
        // Fallback: end point
        if let Some(last) = bez.segments().last() {
            let p = last.eval(1.0);
            (p.x, p.y)
        } else {
            (0.0, 0.0)
        }
    }

    /// Full flood-fill mask generation — port of JS fillmask().
    /// Returns (maskgrid, nrows, ncols) where maskgrid[row*ncols+col] = 1 if outside/border.
    pub fn fill_mask(
        path_strings: &[&str],
        crad: f64,
        frame: f64,
        nrows: usize,
        ncols: usize,
    ) -> Vec<u8> {
        let dia = 2.0 * crad;
        let mut maskgrid = vec![0u8; nrows * ncols];

        let mut all_pts = Vec::new();
        for path_d in path_strings {
            let pts = plot_path(path_d, dia);
            for pt in pts {
                if pt.y >= 0 && pt.x >= 0 && pt.y < nrows as i32 && pt.x < ncols as i32 {
                    maskgrid[pt.y as usize * ncols + pt.x as usize] = 1;
                }
                all_pts.push(pt);
            }
        }

        // Flood-fill from outside to mark exterior cells
        let cross = [(-1i32, 0i32), (1, 0), (0, 1), (0, -1)];
        let mut reg_num: u8 = 3;
        loop {
            let mut grown = false;
            for i in -1i32..(nrows as i32 + 1) {
                for j in -1i32..(ncols as i32 + 1) {
                    let is_boundary = i < 0 || j < 0 || i >= nrows as i32 || j >= ncols as i32;
                    let is_seed = !is_boundary && {
                        let v = maskgrid[i as usize * ncols + j as usize];
                        v > 1 && v < reg_num
                    };
                    if !is_boundary && !is_seed {
                        continue;
                    }
                    let mut stack = vec![(i, j)];
                    while let Some((pr, pc)) = stack.pop() {
                        for &(dr, dc) in &cross {
                            let ii = pr + dr;
                            let jj = pc + dc;
                            if ii < 0 || jj < 0 || ii >= nrows as i32 || jj >= ncols as i32 {
                                continue;
                            }
                            let idx = ii as usize * ncols + jj as usize;
                            if maskgrid[idx] == 0 {
                                stack.push((ii, jj));
                            }
                            if maskgrid[idx] < 2 {
                                maskgrid[idx] = reg_num;
                                grown = true;
                            }
                        }
                    }
                }
            }
            if !grown {
                break;
            }
            reg_num += 1;
        }

        // Keep only odd-numbered regions (interior) and re-mark borders
        for v in maskgrid.iter_mut() {
            *v = *v % 2;
        }

        // Re-stamp border points with frame expansion
        for pt in &all_pts {
            if pt.y >= 0 && pt.x >= 0 && pt.y < nrows as i32 && pt.x < ncols as i32 {
                maskgrid[pt.y as usize * ncols + pt.x as usize] = 1;
                let mut bordered = false;
                let mut extent = 1i32;
                while !bordered {
                    bordered = true;
                    for di in -extent..=extent {
                        for dj in -extent..=extent {
                            if di.abs() != extent && dj.abs() != extent {
                                continue;
                            }
                            let cx = pt.x + dj;
                            let cy = pt.y + di;
                            if cx > 0 && cy > 0 && cx < ncols as i32 && cy < nrows as i32 {
                                let dist = ((cx as f64 * dia - pt.rx).powi(2)
                                    + (cy as f64 * dia - pt.ry).powi(2))
                                    .sqrt();
                                if dist < crad + frame {
                                    maskgrid[cy as usize * ncols + cx as usize] = 1;
                                    bordered = false;
                                }
                            }
                        }
                    }
                    extent += 1;
                }
            }
        }

        maskgrid
    }
}
