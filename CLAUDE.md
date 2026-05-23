# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

An isomorphic fractal jigsaw puzzle stack where the **same Rust geometry engine runs in both the browser (WebAssembly via wasm-bindgen) and on the server (WebAssembly via Chicory JVM runtime)**. The same seed/config produces identical SVG output in both environments.

## Repository Layout

```
puzzle-core/   Rust library — dual-ABI Wasm (browser + server)
frontend/      Next.js 15 app — configurator UI, 2D/3D preview
backend/       Spring Boot 4.0 — REST API, Chicory Wasm runtime, PostgreSQL
Dockerfile     Multi-stage build (4 stages)
docker-compose.yml  Local dev with postgres:18.2
```

## puzzle-core (Rust)

### Build commands
```bash
cd puzzle-core

# Browser Wasm (wasm-bindgen glue → pkg/)
wasm-pack build --release --target web --out-dir ../pkg -- --features bindgen

# Server Wasm (C-ABI, no bindgen → target/wasm32-unknown-unknown/release/)
cargo build --release --target wasm32-unknown-unknown

# Run all tests
cargo test --release

# Run a single test
cargo test --release prng::tests::golden_seed_42
```

### Dual-ABI architecture
- **`feature = "bindgen"`** → browser build. Exposes `PuzzleHandle` via `#[wasm_bindgen]`. Built with `wasm-pack`.
- **no features** → server build. Exposes three `#[no_mangle] extern "C"` functions: `get_input_ptr()`, `get_output_ptr()`, `generate_puzzle(input_len) → output_len`. Built with plain `cargo build --target wasm32-unknown-unknown`.
- The server Wasm uses 64KB static `INPUT_BUF` and 4MB static `OUTPUT_BUF`. JSON config is written into the input buffer; SVG is read from the output buffer.

### Critical invariants (do not break)
- **PRNG**: `x = seed.sin() * 10000.0; seed += 1.0; x - x.floor()`. Must use `x - x.floor()`, never `x % 1.0`. `seed` is `f64`. GNU libm and V8 libm differ by ~1–2 ULP for some inputs (e.g. `sin(18)`); test tolerance is `1e-10`.
- **CellGrid naming**: `CellGrid::new(nrow, ncol)` receives `(ncols, nrows)` from the outer context — the constructor args are swapped from their names, matching the original JS bug. `is_tile_valid` uses `v.x < self.nrow && v.y < self.ncol`. Do not "fix" this.
- **`cellmap` size**: allocated as `ncol * nrow` (not `(ncol-1)*(nrow-1)`) because the JS sparse-array auto-extended when indexed beyond its declared size.
- **`coloring_seed`**: captured after `generate()` + all `fill_holes()` calls complete. `export_svg(mode=3)` restores this snapshot before generating fill colors.
- **`wasm-opt = false`**: set in `Cargo.toml` metadata because `wasm-opt` (binaryen) is not available in the CI/offline environment.

### Module responsibilities
| Module | Role |
|--------|------|
| `prng.rs` | Seeded deterministic PRNG with snapshot/restore |
| `types.rs` | `PuzzleConfig` with manual JSON parsing (no serde derive — `syn` unavailable offline) |
| `grid.rs` | `CellGrid` with the swapped `nrow`/`ncol` naming |
| `jigsaw.rs` | `CircleFractalJigsaw` — generate, fill_holes, 4 SVG export modes |
| `arc.rs` | `Arc::svg()` for 3 tile shapes × 4 quads |
| `mask.rs` | Browser: receives pre-rasterized `Uint8Array`; server: kurbo arc-length sampling of SVG path |
| `frame.rs` | Pure SVG frame generation |

## frontend (Next.js 14)

### Build & dev commands
```bash
cd frontend
pnpm install
pnpm dev          # http://localhost:3000
pnpm build        # production build (output: standalone)
pnpm run lint
pnpm exec playwright test e2e/  # E2E tests (requires backend running)
```

### Wasm loading
The browser Wasm binary lives at `frontend/lib/wasm/` (colocated with the JS glue). This is required for webpack's `asyncWebAssembly` experiment to resolve the `.wasm` file via `import.meta.url`. **Do not move the `.wasm` file to `public/`** — webpack cannot resolve it there.

After `wasm-pack build`, copy outputs:
```bash
cp pkg/puzzle_core.js pkg/puzzle_core_bg.wasm pkg/puzzle_core.d.ts pkg/puzzle_core_bg.wasm.d.ts frontend/lib/wasm/
```

`usePuzzleWasm` uses a module-level singleton (`cached` + `initPromise`) so Wasm is only initialized once across React re-renders.

### Hook data flow
```
usePuzzleWasm        → loads Wasm module once (useEffect, client-only)
usePuzzleGenerator   → takes moduleRef, runs generate() + fill_holes() in setTimeout(0)
useBorderMask        → SVG file → flatten.js → plotpath.ts → Uint8Array → passed to set_mask()
```

`plotpath.ts` is the only file using browser DOM APIs (`getTotalLength`/`getPointAtLength`). It replicates `index.html:36-62`: `uinc = 1/128`, subdivide when adjacent grid cells gap > 1, floor at `uinc < 1/65536`.

### 3D preview
`PuzzlePreview3D` wraps `PuzzleScene` with `dynamic(() => import('./PuzzleScene'), { ssr: false })` to prevent WebGL errors during SSR. The R3F scene extrudes SVG paths 6mm, flips Y with `rotation={[Math.PI, 0, 0]}`, scales SVG units to scene with `scale={0.1}`.

### SVG export modes
| Mode | Description |
|------|-------------|
| 0 | Individual pieces (may overlap) |
| 1 | Non-overlapping |
| 2 | Single merged path (Trotec laser workaround) |
| 3 | Colored preview |

## backend (Spring Boot 4.0)

### Build & run commands
```bash
cd backend
mvn package -DskipTests          # build fat jar
mvn test -Dtest=PuzzleServiceTest  # unit tests only (no DB required)
java -jar target/backend-*.jar   # run (needs DB_URL, DB_USER, DB_PASS env vars)
```

### Chicory Wasm integration
`PuzzleWasmService` uses the Chicory 1.0.0 API:
```java
WasmModule module = Parser.parse(wasmBytes);      // com.dylibso.chicory.wasm.Parser
Instance inst = Instance.builder(module).build(); // com.dylibso.chicory.runtime.Instance
ExportFunction fn = inst.export("generate_puzzle");
long outputLen = fn.apply(inputLen)[0];           // ExportFunction.apply(long...) → long[]
inst.memory().write(ptr, bytes);
inst.memory().readBytes(ptr, len);
```

`Instance` is **not thread-safe**. A `BlockingQueue<Instance>` bounded by `Runtime.availableProcessors()` is used as a pool in `PuzzleWasmService`. One instance per request, returned to the pool in `finally`.

The server Wasm is bundled at `src/main/resources/wasm/puzzle_core_server.wasm`. Path is configurable via `puzzle.wasm.path` property.

### Database
PostgreSQL with Flyway migrations in `src/main/resources/db/migration/`. Three tables: `puzzle_configs`, `puzzle_outputs` (cached per config+mode with `wasm_sha256` for audit), `orders`. `puzzle_outputs` has a `UNIQUE(config_id, mode)` constraint — `export()` checks cache before regenerating.

### REST API
- `POST /api/puzzle/generate` → saves config, generates mode-3 preview SVG, returns `{ id, svgPreview, pieceCount, widthMm, heightMm }`
- `GET /api/puzzle/{id}/export?mode=0-3` → returns `image/svg+xml` (cached after first generation)
- `POST /api/orders` + `GET /api/orders/{id}`

## Local development

```bash
docker compose build --no-cache backend
docker compose up   # starts postgres:18.2, backend :8080, frontend :3000
```

Environment variables for backend: `DB_URL`, `DB_USER`, `DB_PASS`, `PORT` (default 8080), `WASM_PATH` (default `wasm/puzzle_core_server.wasm`).

## CI/CD (GitHub Actions)

`.github/workflows/deploy.yml` runs four jobs in sequence:
1. **wasm** — builds both Wasm targets, runs `cargo test --release`, uploads artifacts
2. **frontend** — downloads browser Wasm artifact, copies to `lib/wasm/`, runs `pnpm build`
3. **backend** — downloads server Wasm artifact, places at `src/main/resources/wasm/puzzle_core_server.wasm`, runs `mvn package`
4. **isomorphic-parity** — starts backend against a live postgres service, runs Playwright (`e2e/isomorphic.spec.ts`) asserting browser SVG === server SVG for modes 1, 2, 3 (mode 0 excluded — path ordering may differ without affecting cut quality)
5. **deploy** — builds Docker images, pushes to GCR, deploys two Cloud Run services via Workload Identity Federation

Deploy requires secrets: `GCP_PROJECT_ID`, `GCP_WORKLOAD_IDENTITY_PROVIDER`, `GCP_SERVICE_ACCOUNT`, `DB_URL`, `DB_USER`, `DB_PASS`, `API_URL`.

## Dockerfile stages

| Stage | Base | Output |
|-------|------|--------|
| `wasm-builder` | `rust:1.83` | Both Wasm binaries |
| `frontend-builder` | `node:24-alpine` | Next.js standalone build |
| `backend-builder` | `maven:3.9-eclipse-temurin-25` | Spring Boot fat jar |
| `runtime` | `eclipse-temurin:25-jre` | Backend container |
| `frontend-runtime` | `gcr.io/distroless/nodejs24-debian12` | Frontend container (runs `node server.js`) |
