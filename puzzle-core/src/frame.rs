/// Replicates JS `createframe()` — pure SVG string generation, no DOM.
pub fn create_frame(ncols: usize, nrows: usize, rad: f64, frame: f64, frame_corner: f64) -> String {
    let width = ncols as f64 * 2.0 * rad + 2.0 * frame;
    let height = nrows as f64 * 2.0 * rad + 2.0 * frame;
    let fc = frame_corner;

    let mut d = format!("M{},{} ", fc, 0.0);
    d += &format!("H {}", width - fc);
    if fc > 0.0 {
        d += &format!("A {} {} 0 0,1 {} {} ", fc, fc, width, fc);
    }
    d += &format!("V {}", height - fc);
    if fc > 0.0 {
        d += &format!("A {} {} 0 0,1 {} {} ", fc, fc, width - fc, height);
    }
    d += &format!("H {}", fc);
    if fc > 0.0 {
        d += &format!("A {} {} 0 0,1 {} {} ", fc, fc, 0.0, height - fc);
    }
    d += &format!("V {}", fc);
    if fc > 0.0 {
        d += &format!("A {} {} 0 0,1 {} {} ", fc, fc, fc, 0.0);
    }
    d += "Z";
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_no_corner() {
        let d = create_frame(10, 10, 6.0, 6.0, 0.0);
        assert!(d.starts_with("M0,0 "));
        assert!(d.ends_with('Z'));
        assert!(!d.contains('A'));
    }

    #[test]
    fn frame_with_corner_has_arcs() {
        let d = create_frame(10, 10, 6.0, 6.0, 4.0);
        // 4 corners → 4 A commands (no leading space — follows directly after H/V)
        assert_eq!(d.matches("A ").count(), 4);
    }

    #[test]
    fn frame_dimensions() {
        // width = ncols*2*rad + 2*frame = 10*12 + 12 = 132
        // height = nrows*2*rad + 2*frame = 10*12 + 12 = 132
        let d = create_frame(10, 10, 6.0, 6.0, 4.0);
        assert!(d.contains("132"), "expected width/height 132 in: {d}");
    }
}
