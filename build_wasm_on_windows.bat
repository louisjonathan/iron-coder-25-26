#install prereqs
cargo install --locked trunk
rustup target add wasm32-unknown-unknown

#run trunk and clean up after
trunk serve
trunk clean
