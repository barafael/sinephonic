# Chord Anatomy: build the WebAssembly analysis and serve web/ as static files with nginx.
# (GitHub Pages is the primary deployment, see .github/workflows/pages.yml.)

FROM rust:1-bookworm AS build
RUN rustup target add wasm32-unknown-unknown
# Must match the wasm-bindgen version pinned in crates/anatomy-wasm/Cargo.toml.
RUN curl -L --proto '=https' --tlsv1.2 -sSf \
      https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
 && cargo binstall -y wasm-bindgen-cli@0.2.129
WORKDIR /app
COPY . .
RUN ./scripts/build-web.sh

FROM nginx:1.27-alpine
COPY deploy/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/web /usr/share/nginx/html
EXPOSE 8080
