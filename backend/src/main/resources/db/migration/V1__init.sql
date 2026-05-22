CREATE TABLE puzzle_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    seed INT NOT NULL,
    ncols SMALLINT NOT NULL,
    nrows SMALLINT NOT NULL,
    min_piece SMALLINT NOT NULL,
    max_piece SMALLINT NOT NULL,
    tile_radius NUMERIC(8,4) NOT NULL,
    frame NUMERIC(8,4) NOT NULL,
    frame_corner NUMERIC(8,4) NOT NULL,
    arc_shape SMALLINT NOT NULL DEFAULT 0,
    border_svg TEXT,
    border_scale NUMERIC(8,4) NOT NULL DEFAULT 1.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE puzzle_outputs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    config_id UUID NOT NULL REFERENCES puzzle_configs(id),
    mode SMALLINT NOT NULL CHECK (mode BETWEEN 0 AND 3),
    svg_data TEXT NOT NULL,
    piece_count INT NOT NULL,
    wasm_sha256 CHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (config_id, mode)
);

CREATE TABLE orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    config_id UUID NOT NULL REFERENCES puzzle_configs(id),
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
