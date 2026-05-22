use serde_json::Value;

#[derive(Debug, Clone)]
pub struct PuzzleConfig {
    pub seed: f64,
    pub ncols: usize,
    pub nrows: usize,
    pub min_piece: usize,
    pub max_piece: usize,
    pub tile_radius: f64,
    pub frame: f64,
    pub frame_corner: f64,
    pub arc_shape: u8,
    pub custom_border_svg: Option<String>,
    pub border_scale: f64,
}

impl PuzzleConfig {
    pub fn from_json(s: &str) -> Result<Self, String> {
        let v: Value = serde_json::from_str(s).map_err(|e| e.to_string())?;
        Self::from_value(&v)
    }

    pub fn from_slice(b: &[u8]) -> Result<Self, String> {
        let v: Value = serde_json::from_slice(b).map_err(|e| e.to_string())?;
        Self::from_value(&v)
    }

    fn from_value(v: &Value) -> Result<Self, String> {
        Ok(PuzzleConfig {
            seed: v["seed"].as_f64().ok_or("missing seed")?,
            ncols: v["ncols"].as_u64().ok_or("missing ncols")? as usize,
            nrows: v["nrows"].as_u64().ok_or("missing nrows")? as usize,
            min_piece: v["min_piece"].as_u64().ok_or("missing min_piece")? as usize,
            max_piece: v["max_piece"].as_u64().ok_or("missing max_piece")? as usize,
            tile_radius: v["tile_radius"].as_f64().ok_or("missing tile_radius")?,
            frame: v["frame"].as_f64().ok_or("missing frame")?,
            frame_corner: v["frame_corner"].as_f64().ok_or("missing frame_corner")?,
            arc_shape: v["arc_shape"].as_u64().unwrap_or(0) as u8,
            custom_border_svg: v["custom_border_svg"].as_str().map(String::from),
            border_scale: v["border_scale"].as_f64().unwrap_or(1.0),
        })
    }
}
