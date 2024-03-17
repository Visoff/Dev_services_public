#/bin/bash
cargo build --release
if [ -z "$2" ]; then
    net=""
else
    net="-n localhost:$2"
fi
./target/release/dev_services -p $1 $net
