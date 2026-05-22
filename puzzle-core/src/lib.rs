pub mod arc;
pub mod diagonal;
pub mod frame;
pub mod grid;
pub mod jigsaw;
pub mod mask;
pub mod prng;
pub mod tile;
pub mod types;

use jigsaw::CircleFractalJigsaw;
use prng::Prng;
use types::PuzzleConfig;

// ── Browser ABI (wasm-bindgen) ────────────────────────────────────────────────

#[cfg(feature = "bindgen")]
mod browser {
    use super::*;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct PuzzleHandle {
        jig: CircleFractalJigsaw,
        prng: Prng,
        cfg: PuzzleConfig,
    }

    #[wasm_bindgen]
    impl PuzzleHandle {
        #[wasm_bindgen(constructor)]
        pub fn new(config_json: &str) -> Result<PuzzleHandle, JsValue> {
            #[cfg(feature = "bindgen")]
            console_error_panic_hook::set_once();

            let cfg = PuzzleConfig::from_json(config_json)
                .map_err(|e| JsValue::from_str(&e))?;
            let prng = Prng::new(cfg.seed);
            let jig = CircleFractalJigsaw::new(&cfg);
            Ok(PuzzleHandle { jig, prng, cfg })
        }

        pub fn generate(&mut self) {
            self.jig.generate(&mut self.prng);
            self.jig.coloring_seed = self.prng.snapshot();
        }

        pub fn fill_holes(&mut self) -> bool {
            self.jig.fill_holes(false)
        }

        pub fn fill_holes_partial(&mut self) -> bool {
            self.jig.fill_holes(true)
        }

        /// Receive pre-rasterized mask from JS plotpath().
        pub fn set_mask(&mut self, mask: &[u8], rows: u32, cols: u32) {
            self.jig.set_mask(mask, rows as usize, cols as usize);
        }

        pub fn export_svg(&self, mode: u8) -> String {
            self.jig.export(mode, self.cfg.frame, self.cfg.tile_radius, self.cfg.frame_corner, self.cfg.arc_shape)
        }

        pub fn piece_count(&self) -> u32 {
            self.jig.n_pieces() as u32
        }

        pub fn coloring_seed(&self) -> f64 {
            self.jig.coloring_seed
        }
    }
}

// ── Server ABI (C-ABI, no wasm-bindgen) ──────────────────────────────────────

#[cfg(not(feature = "bindgen"))]
mod server_abi {
    use super::*;

    static mut INPUT_BUF: [u8; 65536] = [0u8; 65536];
    static mut OUTPUT_BUF: [u8; 4194304] = [0u8; 4194304];

    #[no_mangle]
    pub extern "C" fn get_input_ptr() -> *mut u8 {
        core::ptr::addr_of_mut!(INPUT_BUF) as *mut u8
    }

    #[no_mangle]
    pub extern "C" fn get_output_ptr() -> *const u8 {
        core::ptr::addr_of!(OUTPUT_BUF) as *const u8
    }

    /// Returns SVG byte length in OUTPUT_BUF, or 0 on error.
    #[no_mangle]
    pub extern "C" fn generate_puzzle(input_len: u32) -> u32 {
        let json_bytes = unsafe {
            core::slice::from_raw_parts(core::ptr::addr_of!(INPUT_BUF) as *const u8, input_len as usize)
        };
        let Ok(cfg) = PuzzleConfig::from_slice(json_bytes) else {
            return 0;
        };

        let mut prng = Prng::new(cfg.seed);
        let mut jig = CircleFractalJigsaw::new(&cfg);

        // Handle server-side custom border via kurbo
        if let Some(ref border_svg) = cfg.custom_border_svg {
            use crate::mask::server::fill_mask;
            let dia = 2.0 * cfg.tile_radius;
            let ncols = (border_svg.len() as f64 / dia).ceil() as usize + 1;
            let nrows = ncols; // placeholder; real dims from SVG viewBox parsing
            let mask = fill_mask(&[border_svg.as_str()], cfg.tile_radius, cfg.frame, nrows, ncols);
            jig.set_mask(&mask, nrows, ncols);
        }

        jig.generate(&mut prng);
        while jig.fill_holes(false) {}
        jig.fill_holes(true);
        jig.coloring_seed = prng.snapshot();

        let svg = jig.export(
            cfg.arc_shape,
            cfg.frame,
            cfg.tile_radius,
            cfg.frame_corner,
            cfg.arc_shape,
        );
        let bytes = svg.as_bytes();
        const OUT_LEN: usize = 4194304;
        if bytes.len() > OUT_LEN {
            return 0;
        }
        unsafe {
            core::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                core::ptr::addr_of_mut!(OUTPUT_BUF) as *mut u8,
                bytes.len(),
            );
        }
        bytes.len() as u32
    }
}
