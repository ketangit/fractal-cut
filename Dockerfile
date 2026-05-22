# Stage 1: Build both Wasm targets from the same Rust source
FROM rust:1.79 AS wasm-builder
WORKDIR /build
COPY puzzle-core/ puzzle-core/

# Install wasm-pack (browser bindgen target)
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

WORKDIR /build/puzzle-core

# Browser Wasm (feature=bindgen, wasm-bindgen glue)
RUN wasm-pack build --release --target web --out-dir ../pkg

# Server Wasm (no features, C-ABI)
RUN cargo build --release --target wasm32-unknown-unknown

# Assert both binaries derive from the same source by comparing Cargo.lock hash
RUN sha256sum Cargo.lock | tee /build/cargo_lock.sha256

# Stage 2: Build Next.js frontend
FROM node:22-alpine AS frontend-builder
WORKDIR /app

RUN corepack enable && corepack prepare pnpm@latest --activate

COPY frontend/package.json frontend/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

COPY frontend/ ./
# Copy browser Wasm artifacts from wasm-builder
COPY --from=wasm-builder /build/pkg/puzzle_core.js lib/wasm/puzzle_core.js
COPY --from=wasm-builder /build/pkg/puzzle_core_bg.wasm lib/wasm/puzzle_core_bg.wasm
COPY --from=wasm-builder /build/pkg/puzzle_core.d.ts lib/wasm/puzzle_core.d.ts
COPY --from=wasm-builder /build/pkg/puzzle_core_bg.wasm.d.ts lib/wasm/puzzle_core_bg.wasm.d.ts

RUN pnpm run build

# Stage 3: Build Spring Boot backend
FROM eclipse-temurin:21-jdk AS backend-builder
WORKDIR /app

COPY backend/pom.xml ./
COPY backend/src/ src/
# Copy server Wasm from wasm-builder
COPY --from=wasm-builder /build/puzzle-core/target/wasm32-unknown-unknown/release/puzzle_core.wasm \
     src/main/resources/wasm/puzzle_core_server.wasm

RUN apt-get update && apt-get install -y maven && apt-get clean
RUN mvn package -DskipTests -q

# Stage 4: Runtime — Spring Boot fat jar + Next.js standalone
FROM eclipse-temurin:21-jre AS runtime
WORKDIR /app

COPY --from=backend-builder /app/target/backend-*.jar app.jar

EXPOSE 8080
ENTRYPOINT ["java", "-jar", "app.jar"]

# Separate target for frontend-only container
FROM node:22-alpine AS frontend-runtime
WORKDIR /app

COPY --from=frontend-builder /app/.next/standalone ./
COPY --from=frontend-builder /app/.next/static ./.next/static
COPY --from=frontend-builder /app/public ./public

EXPOSE 3000
ENV NODE_ENV=production
ENTRYPOINT ["node", "server.js"]
