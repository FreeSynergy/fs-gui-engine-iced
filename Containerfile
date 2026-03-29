# fs-gui-engine-iced — multi-stage container build
#
# This library crate has no standalone binary.  The container image is used
# as a build-time base by downstream crates (fs-desktop, fs-apps) that
# link against fs-gui-engine-iced.  It can also be used to run the unit
# test suite in CI without installing Rust locally.

# Stage 1: Build + test
FROM docker.io/rust:1.83-slim AS builder

WORKDIR /build

# System packages required for iced (Wayland/X11 display libraries are NOT
# needed at compile time — only at runtime on the host).
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libfontconfig1-dev \
    libfreetype6-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace dependencies first for layer caching.
COPY fs-libs/  fs-libs/
COPY fs-render/ fs-render/
COPY fs-gui-engine-iced/ fs-gui-engine-iced/

WORKDIR /build/fs-gui-engine-iced
RUN cargo test --release
RUN cargo build --release

# Stage 2: Minimal artifact image (library + test binary)
FROM docker.io/debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libfontconfig1 \
    libfreetype6 \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled library artefacts so downstream images can COPY --from.
COPY --from=builder \
    /build/fs-gui-engine-iced/target/release/deps/libfs_gui_engine_iced*.rlib \
    /usr/local/lib/freesynergy/

LABEL org.opencontainers.image.source="https://github.com/FreeSynergy/fs-gui-engine-iced"
LABEL org.opencontainers.image.description="FreeSynergy iced render engine (library)"
LABEL org.opencontainers.image.licenses="MIT"
