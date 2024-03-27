build-web:
    cargo build --release --target wasm32-unknown-unknown \
      && wasm-bindgen target/wasm32-unknown-unknown/release/roguelike_game.wasm \
         --out-dir wasm --no-modules --no-typescript

run-web: build-web
    python3 -m http.server 6969 --directory wasm/
