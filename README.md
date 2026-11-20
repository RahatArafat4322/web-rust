# Nextcircuit Official Creator Hub — Rust + WebAssembly Edition

This is the complete **Rust + WebAssembly (`wasm32-unknown-unknown`)** port of the Nextcircuit Creator Hub. All application logic, DOM rendering, API fetching (YouTube Data API v3, live GitHub API, `software.json`), URL sanitization, typewriter text animation, and HTML5 Canvas particle/circuit-pulse animations are written in Rust (`src/lib.rs`) using `wasm-bindgen` and `web-sys`.

## Local Development

1. Install Rust (`https://rustup.rs/`) and add the WebAssembly target + Trunk bundler:
   ```bash
   rustup target add wasm32-unknown-unknown
   cargo install trunk
   ```
2. Copy your image assets (`image_499882.png`, `youtube.png`, `github.png`, `windows.png`, `linux.png`, `android.png`, `discord.png`, `facebook.png`, `linkedin.png`) into this project folder (or a `static/` folder configured in `Trunk.toml`).
3. Run locally:
   ```bash
   trunk serve
   ```
4. Build optimized production files into `dist/`:
   ```bash
   trunk build --release
   ```

## Deploying to Vercel

The included `vercel.json` automatically installs Rust + Trunk during Vercel's build step, compiles `src/lib.rs` to WebAssembly, and serves `dist/` with strict HTTP security headers.
