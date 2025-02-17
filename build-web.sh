cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --out-dir deploy --out-name bevy_game --target web target/wasm32-unknown-unknown/release/roguecowboy.wasm

mkdir -p ./deploy
cp ./target/wasm32-unknown-unknown/release/roguecowboy.wasm ./deploy/
cp index.html ./deploy/
cp -r assets deploy/ || true # Try to copy, but ignore if it can't copy if source directory does not exist
