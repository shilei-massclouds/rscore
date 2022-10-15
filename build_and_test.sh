#!/bin/sh
set -e

make
cd ../qemu
./zstart.sh
cd -
