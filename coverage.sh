#!/bin/bash

rm ./target/coverage/data/*.profraw
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='./target/coverage/data/cargo-test-%p-%m.profraw' cargo test
grcov ./target/coverage/data/ --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/html
xdg-open target/coverage/html/index.html
