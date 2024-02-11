#!/bin/bash

cd ./nfs-autoshare-daemon
cargo build --release
cd ../nfs-autoshare-client
cargo build --release

