rm kernel
cargo build -Zbuild-std --target x86_64-unknown-none || exit 1
cp target/x86_64-unknown-none/debug/kernel .