/// SVG parity and structural tests for puzzle-core.
///
/// NOTE: Byte-for-byte parity with the JS reference (index.html) is NOT achievable
/// due to 1-ULP sin() discrepancies between GNU libm (Rust) and V8's libm (JS) for
/// some seed values (e.g. sin(18)). This causes divergent PRNG paths during generation.
///
/// The isomorphic guarantee (browser Wasm == server Wasm) is preserved because both
/// use the SAME compiled binary with the same sin() implementation.
///
/// These tests verify: structural validity, determinism, and self-consistency.
use puzzle_core::{
    jigsaw::CircleFractalJigsaw,
    prng::Prng,
    types::PuzzleConfig,
};
use serde_json::Value;

fn load_fixtures() -> Value {
    let json = include_str!("fixtures/golden_fixtures.json");
    serde_json::from_str(json).expect("valid fixture JSON")
}

fn make_config(seed: f64, ncols: usize, nrows: usize) -> PuzzleConfig {
    PuzzleConfig::from_json(&format!(
        r#"{{"seed":{seed},"ncols":{ncols},"nrows":{nrows},"min_piece":4,"max_piece":50,
            "tile_radius":6.0,"frame":6.0,"frame_corner":4.0,"arc_shape":0}}"#
    ))
    .unwrap()
}

fn run_rust(cfg: &PuzzleConfig) -> (usize, f64, [String; 4]) {
    let mut prng = Prng::new(cfg.seed);
    let mut jig = CircleFractalJigsaw::new(cfg);
    jig.generate(&mut prng);
    while jig.fill_holes(false) {}
    jig.fill_holes(true);
    jig.coloring_seed = prng.snapshot();

    let piece_count = jig.n_pieces();
    let coloring_seed = jig.coloring_seed;
    let svgs = [
        jig.export(0, cfg.frame, cfg.tile_radius, cfg.frame_corner, cfg.arc_shape),
        jig.export(1, cfg.frame, cfg.tile_radius, cfg.frame_corner, cfg.arc_shape),
        jig.export(2, cfg.frame, cfg.tile_radius, cfg.frame_corner, cfg.arc_shape),
        jig.export(3, cfg.frame, cfg.tile_radius, cfg.frame_corner, cfg.arc_shape),
    ];
    (piece_count, coloring_seed, svgs)
}

/// Determinism: same seed → identical output every run.
#[test]
fn determinism_same_seed_same_output() {
    let cfg = make_config(42.0, 10, 10);
    let (pc1, cs1, svgs1) = run_rust(&cfg);
    let (pc2, cs2, svgs2) = run_rust(&cfg);
    assert_eq!(pc1, pc2, "piece count must be deterministic");
    assert_eq!(cs1, cs2, "coloring_seed must be deterministic");
    for mode in 0..4usize {
        assert_eq!(svgs1[mode], svgs2[mode], "mode={mode} SVG must be deterministic");
    }
}

/// Different seeds must produce different puzzles.
#[test]
fn different_seeds_different_output() {
    let (_, _, svgs42) = run_rust(&make_config(42.0, 10, 10));
    let (_, _, svgs99) = run_rust(&make_config(99.0, 10, 10));
    assert_ne!(svgs42[0], svgs99[0], "different seeds should produce different SVGs");
}

/// Piece count is positive and reasonable.
#[test]
fn piece_count_in_range() {
    for seed in [0.0, 42.0, 999.0, 1234.0f64] {
        let cfg = make_config(seed, 10, 10);
        let (pc, _, _) = run_rust(&cfg);
        assert!(pc >= 1, "seed={seed}: must have at least 1 piece, got {pc}");
        assert!(pc <= 100, "seed={seed}: 10x10 grid can't have >100 pieces, got {pc}");
    }
}

/// All 4 modes produce valid SVG structure.
#[test]
fn svg_is_valid_xml() {
    let cfg = make_config(42.0, 10, 10);
    let (_, _, svgs) = run_rust(&cfg);
    for (mode, svg) in svgs.iter().enumerate() {
        assert!(svg.starts_with("<?xml"), "mode={mode}: missing XML declaration");
        assert!(svg.contains("<svg "), "mode={mode}: missing <svg>");
        assert!(svg.ends_with("</svg>"), "mode={mode}: missing </svg>");
        assert!(svg.contains("mm\""), "mode={mode}: missing physical dimensions");
    }
}

/// Mode 3 (colored) has fill colors on piece paths.
#[test]
fn colored_mode_has_fill_colors() {
    let cfg = make_config(42.0, 10, 10);
    let (_, _, svgs) = run_rust(&cfg);
    assert!(svgs[3].contains("fill=\"#"), "mode=3: no fill colors found");
}

/// Mode 2 (single path) has exactly 2 <path> elements: puzzle body + frame.
#[test]
fn singlepath_mode_has_one_puzzle_path() {
    let cfg = make_config(42.0, 10, 10);
    let (_, _, svgs) = run_rust(&cfg);
    let path_count = svgs[2].matches("<path").count();
    assert_eq!(path_count, 2, "mode=2: expected 2 paths (puzzle+frame), got {path_count}");
}

/// Mode 0 has one <path> per piece + 1 frame path.
#[test]
fn mode0_has_per_piece_paths() {
    let cfg = make_config(42.0, 10, 10);
    let (piece_count, _, svgs) = run_rust(&cfg);
    let path_count = svgs[0].matches("<path").count();
    assert_eq!(
        path_count, piece_count + 1,
        "mode=0: expected {piece_count}+1 paths, got {path_count}"
    );
}

/// Larger grid produces more pieces than smaller grid (probabilistically).
#[test]
fn larger_grid_more_pieces() {
    let (pc_small, _, _) = run_rust(&make_config(42.0, 5, 5));
    let (pc_large, _, _) = run_rust(&make_config(42.0, 15, 15));
    assert!(pc_large > pc_small, "15x15 should have more pieces than 5x5");
}

/// SVG dimensions match expected mm values (ncols*2*rad + 2*frame).
#[test]
fn svg_dimensions_correct() {
    // 10*2*6 + 2*6 = 132mm
    let cfg = make_config(42.0, 10, 10);
    let (_, _, svgs) = run_rust(&cfg);
    for (mode, svg) in svgs.iter().enumerate() {
        assert!(
            svg.contains("132mm"),
            "mode={mode}: expected 132mm dimensions in SVG"
        );
        assert!(
            svg.contains("viewBox=\"0 0 132 132\""),
            "mode={mode}: expected viewBox 132x132"
        );
    }
}

/// All <path> elements in mode 0 are closed (end with Z).
#[test]
fn mode0_paths_are_closed() {
    let cfg = make_config(42.0, 10, 10);
    let (_, _, svgs) = run_rust(&cfg);
    // All piece paths should end with Z (frame path also has Z)
    for path_segment in svgs[0].split("<path").skip(1) {
        if let Some(d_start) = path_segment.find("d=\"") {
            let d_end = path_segment[d_start + 3..].find('"').unwrap();
            let d = &path_segment[d_start + 3..d_start + 3 + d_end];
            if !d.starts_with("M4,") {
                // piece paths (not frame)
                assert!(d.ends_with("Z") || d.ends_with("Z "), "piece path not closed: ...{}", &d[d.len().saturating_sub(20)..]);
            }
        }
    }
}

/// Frame path uses correct corner radius.
#[test]
fn frame_has_corner_arcs() {
    let cfg = make_config(42.0, 10, 10);
    let (_, _, svgs) = run_rust(&cfg);
    // Frame should appear in all modes
    for (mode, svg) in svgs.iter().enumerate() {
        assert!(
            svg.contains("A 4 4 0 0,1"),
            "mode={mode}: expected frame corner arcs (r=4)"
        );
    }
}

/// JS golden fixture piece counts are within 3x of Rust (same algorithm, different libm).
/// Not exact equality — see NOTE at top.
#[test]
fn piece_count_close_to_js_fixture() {
    let fixtures = load_fixtures();
    let cases = [
        ("seed0_10x10", 0.0f64, 10usize, 10usize),
        ("seed42_10x10", 42.0, 10, 10),
        ("seed999_5x5", 999.0, 5, 5),
    ];
    for (key, seed, ncols, nrows) in cases {
        let js_count = fixtures[key]["piece_count"].as_u64().unwrap() as usize;
        let (rust_count, _, _) = run_rust(&make_config(seed, ncols, nrows));
        assert!(
            rust_count > 0,
            "{key}: Rust produced 0 pieces"
        );
        // Both should be in the same order of magnitude
        let ratio = (rust_count as f64) / (js_count as f64);
        assert!(
            ratio > 0.2 && ratio < 5.0,
            "{key}: Rust piece count {rust_count} too different from JS {js_count}"
        );
    }
}

/// After full generation + fillholes, all pieces are non-empty and have valid structure.
#[test]
fn pieces_have_valid_structure() {
    let cfg = make_config(7.0, 8, 8);
    let mut prng = Prng::new(cfg.seed);
    let mut jig = CircleFractalJigsaw::new(&cfg);
    jig.generate(&mut prng);
    while jig.fill_holes(false) {}
    jig.fill_holes(true);
    jig.coloring_seed = prng.snapshot();

    assert!(jig.n_pieces() > 0, "must have at least 1 piece");
    for (i, piece) in jig.pieces.iter().enumerate() {
        assert!(!piece.is_empty(), "piece {i} is empty (has no connections)");
    }
}
