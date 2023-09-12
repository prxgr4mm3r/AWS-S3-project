#!/usr/bin/env bash

# Start the postgresql database
service postgresql start

# Build the project
cargo build --release

# Copy the binary to the root of the project
cp target/release/mys3cli /usr/bin/
