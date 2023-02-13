#!/usr/bin/env bash
TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target x86_64-unknown-linux-musl && cargo build --release;
scp ./target/release/{pipe_status,pipe_status_server} delos:/Volumes/workstation_home/software/bin &
scp ./target/x86_64-unknown-linux-musl/release/{pipe_status,pipe_status_server} civmcluster1:/cm/shared/workstation_code_dev/bin &
wait
echo "transfer complete"

