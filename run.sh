#/bin/bash
cargo build --release
./target/release/dev_services -f setup.json
