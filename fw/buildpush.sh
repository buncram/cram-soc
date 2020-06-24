#!/bin/bash

# argument 1 is the target for copy

if [ -z "$1" ]
then
    echo "Usage: $0 ssh-target [privatekey]"
    echo "Missing ssh-target argument."
    echo "Assumes betrusted-scripts repo is cloned on repository at ~/code/betrused-scripts/"
    exit 0
fi

# case of no private key specified
if [ -z "$2" ]
then
CC=riscv64-unknown-elf-gcc cargo +nightly build --release && ./rust-rom.sh && scp /tmp/betrusted-soc.bin $1:code/betrusted-scripts/ && scp ../build/gateware/encrypted.bin $1:code/betrusted-scripts/ && scp ../test/csr.csv $1:code/betrusted-scripts/soc-csr.csv
else
# there is a private key
CC=riscv64-unknown-elf-gcc cargo +nightly build --release && ./rust-rom.sh && scp -i $2 /tmp/betrusted-soc.bin $1:code/betrusted-scripts/ && scp -i $2 ../build/gateware/encrypted.bin $1:code/betrusted-scripts/ && scp -i $2 ../test/csr.csv $1:code/betrusted-scripts/soc-csr.csv
fi

