# Fractal Cut — Isomorphic Jigsaw Puzzle Generator

Design laser-cut fractal jigsaw puzzles with a unified geometry engine running in **both the browser (WebAssembly) and on the server (Chicory JVM runtime)**. Same seed, same config → identical SVG output everywhere.

## Features

- **Seeded deterministic puzzle generation** — reproducible results with `seed` parameter
- **2D & 3D preview** — React Three Fiber visualization with real-time extrusion
- **Custom borders** — load SVG paths to constrain puzzle to custom shapes
- **4 SVG export modes**:
  - Mode 0: Individual pieces (overlapping)
  - Mode 1: Non-overlapping pieces
  - Mode 2: Single merged path (Trotec laser workaround)
  - Mode 3: Colored preview
- **Isomorphic guarantee** — browser and server produce byte-identical SVG from the same config
- **Production-ready** — containerized, Cloud Run deployable, PostgreSQL-backed

## Quick Start

### Local Development (Docker)

```bash
docker compose up
```

Opens:
- Frontend: http://localhost:3000
- Backend API: http://localhost:8080
- PostgreSQL: localhost:5432

### Without Docker

**Requirements:**
- Rust 1.83+ (wasm-pack)
- Node.js 24+ (pnpm)
- Java 25+ (Maven)
- PostgreSQL 18.2+

**Build:**
```bash
# Rust Wasm engine
cd puzzle-core && cargo build --release && wasm-pack build --release --target web

# Next.js frontend
cd frontend && pnpm install && pnpm build

# Spring Boot backend
cd backend && mvn package -DskipTests
```

**Run:**
```bash
# Backend (needs DB_URL, DB_USER, DB_PASS)
java -jar backend/target/backend-*.jar

# Frontend (standalone)
cd frontend && pnpm start
```

## Architecture

### puzzle-core (Rust/WebAssembly)

Geometry engine ported from the original JavaScript. Compiled to **two Wasm targets** from identical source:

| Target | ABI | Use Case |
|--------|-----|----------|
| Browser | wasm-bindgen | React hooks in Next.js frontend |
| Server | C-ABI (Chicory) | Spring Boot Wasm runtime |

**Key modules:**
- `prng.rs` — Seeded PRNG matching JS `Math.sin(seed) * 10000`
- `jigsaw.rs` — Fractal piece generation, fill_holes algorithm
- `arc.rs` — 3 tile shapes (circular, square, octagonal)
- `mask.rs` — Custom border rasterization (kurbo for server, plotpath for browser)

### frontend (Next.js 14)

Single-page configurator UI with:
- **Sliders** for all puzzle parameters (seed, cols, rows, tile radius, etc.)
- **2D SVG preview** — rendered directly
- **3D preview** — R3F Canvas with ExtrudeGeometry, 6mm depth
- **Custom border upload** — SVG file → mask → passed to Wasm
- **4 export buttons** — download SVG in any mode

Wasm binary is colocated at `lib/wasm/` (required for webpack's `asyncWebAssembly` resolver).

### backend (Spring Boot 4.0)

REST API:
- `POST /api/puzzle/generate` — create config, generate mode-3 preview
- `GET /api/puzzle/{id}/export?mode=0-3` — download SVG (cached per mode)
- `POST /api/orders` — create order
- `GET /api/orders/{id}` — retrieve order

**Wasm integration:** `PuzzleWasmService` uses Chicory 1.0.0 to instantiate the server Wasm. Instances are pooled (one per CPU core) and reused across requests via `BlockingQueue`.

**Database:** PostgreSQL with Flyway migrations. Three tables:
- `puzzle_configs` — seed, dimensions, parameters
- `puzzle_outputs` — cached SVG per config+mode (with `wasm_sha256` audit field)
- `orders` — user orders

## Development

### Rust Changes

```bash
cd puzzle-core

# Unit tests (PRNG golden values, SVG structural validation)
cargo test --release

# Build browser Wasm
wasm-pack build --release --target web --out-dir ../pkg

# Copy to frontend
cp pkg/puzzle_core*.{js,wasm,d.ts} ../frontend/lib/wasm/

# Build server Wasm
cargo build --release --target wasm32-unknown-unknown
```

**Critical invariants** (see `CLAUDE.md`):
- PRNG: `x = seed.sin() * 10000.0; seed += 1.0; x - x.floor()` (not `%`)
- `CellGrid::new(nrow, ncol)` receives `(ncols, nrows)` — naming is intentionally swapped to match original JS
- `cellmap` allocated as `ncol * nrow` (not `(ncol-1)*(nrow-1)`)

### Frontend Changes

```bash
cd frontend
pnpm dev      # http://localhost:3000 with auto-reload
pnpm build    # production build (output: standalone)
pnpm run lint
```

### Backend Changes

```bash
cd backend
mvn compile
mvn test -Dtest=PuzzleServiceTest  # unit tests (no DB required)
mvn package -DskipTests             # build fat jar
```

Requires `DB_URL`, `DB_USER`, `DB_PASS` environment variables.

## Deployment

### Cloud Run

The `.github/workflows/deploy.yml` workflow:

1. Builds both Wasm targets → runs `cargo test`
2. Builds frontend → runs `pnpm build`
3. Builds backend → runs `mvn package`
4. **Isomorphic parity gate** — Playwright E2E test asserts browser SVG === server SVG (modes 1–3)
5. Pushes Docker images to Google Container Registry
6. Deploys two Cloud Run services (backend + frontend) via Workload Identity Federation

**Required GitHub secrets:**
- `GCP_PROJECT_ID` — Google Cloud project
- `GCP_WORKLOAD_IDENTITY_PROVIDER` — Workload Identity pool provider
- `GCP_SERVICE_ACCOUNT` — Service account email
- `DB_URL` — Neon/Supabase PostgreSQL connection string
- `DB_USER`, `DB_PASS` — Database credentials
- `API_URL` — Public API endpoint (for frontend env var)

**Docker multi-stage build:**
```
Stage 1: wasm-builder (rust:1.83)        → both Wasm binaries
Stage 2: frontend-builder (node:24)      → Next.js standalone
Stage 3: backend-builder (openjdk:25)    → Spring Boot fat jar + minimal JRE (jlink)
Stage 4: runtime (distroless base)       → Backend container
Stage 5: frontend-runtime (distroless)   → Frontend container
```

**Container optimizations:**
- jlink: ~50MB minimal JRE (vs 300MB+ default)
- distroless: no shell, no package managers, minimal attack surface

## Testing

### Unit Tests

**Rust (puzzle-core):**
```bash
cargo test --release
```

Tests PRNG golden values and SVG structural validity.

**Java (backend):**
```bash
mvn test -Dtest=PuzzleServiceTest
```

JSON config parsing and serialization.

### E2E Tests

**Isomorphic parity (CI gate):**
```bash
pnpm exec playwright test e2e/isomorphic.spec.ts
```

Generates puzzle in browser, captures SVG. Calls server `/api/puzzle/generate` with same config, captures SVG. Asserts equality (modes 1, 2, 3).

## Troubleshooting

### Wasm not loading in browser

Check that:
1. `lib/wasm/puzzle_core_bg.wasm` exists (webpack `asyncWebAssembly` resolves via `import.meta.url`)
2. `next.config.mjs` has `webpack.experiments.asyncWebAssembly = true`
3. Browser DevTools → Network tab: verify `.wasm` request succeeds with 200 OK

### Backend Wasm fails to instantiate

1. Verify `src/main/resources/wasm/puzzle_core_server.wasm` exists
2. Check `application.properties`: `puzzle.wasm.path=wasm/puzzle_core_server.wasm`
3. Confirm server Wasm exports `get_input_ptr`, `get_output_ptr`, `generate_puzzle`:
   ```bash
   wasm-objdump -x puzzle_core_server.wasm | grep "function "
   ```

### Large Docker image size

The `backend/target/` directory (56MB jar) persists in git history. To remove permanently:
```bash
git filter-repo --path backend/target/ --invert-paths --force
git push --force-with-lease
```

## License

MIT. Original jigsaw algorithm based on [proceduraljigsaw/Fractalpuzzlejs](https://github.com/proceduraljigsaw/Fractalpuzzlejs).

## Links

- **GitHub:** https://github.com/ketangit/fractal-cut
- **Live demo:** (deploy to Cloud Run to access)
- **Playwright E2E:** `frontend/e2e/isomorphic.spec.ts`
- **Architecture docs:** `CLAUDE.md`
