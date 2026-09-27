# Chord Anatomy: build the Dioxus web bundle, then serve it as static files with nginx.
# The app runs entirely in the browser (WebAssembly); there is no server-side code.

# ── Build ────────────────────────────────────────────────────────────────────────────
FROM rust:1-bookworm AS build

RUN rustup target add wasm32-unknown-unknown

# The Dioxus CLI must match the dioxus crate version in Cargo.lock (0.7.10).
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
      https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
 && cargo binstall -y dioxus-cli@0.7.10

WORKDIR /app
COPY . .

# dx fetches the matching wasm-bindgen and wasm-opt itself.
RUN dx bundle --platform web --release

# ── Serve ────────────────────────────────────────────────────────────────────────────
FROM nginx:1.27-alpine

COPY deploy/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/target/dx/sinephonic/release/web/public /usr/share/nginx/html

EXPOSE 8080
