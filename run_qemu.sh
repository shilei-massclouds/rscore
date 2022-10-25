#!/bin/sh
set -e

cargo run

cd ../qemu
./zstart.sh
cd -
