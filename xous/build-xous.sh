#!/bin/sh

# notes:
# xous-stage1.bin written to 0x2050_0000 (64k erase block size)
# xous-kernel.bin written to 0x2051_0000 => passed as a0 arg to xous-stage1.bin
# This is handled in part by betrusted-scripts, with provision-xous.sh
# stage1 and kernel are merged into xous.img by this script.

cd xous-kernel && cargo build --release
cd ..
cd xous-stage1 && cargo build --release
cd ..
cd xous-tools && cargo build --release
cd ..

riscv64-unknown-elf-objcopy xous-stage1/target/riscv32i-unknown-none-elf/release/xous-stage1 -O binary xous-stage1.bin
dd if=/dev/null of=xous-stage1.bin bs=1 count=1 seek=65536
xous-tools/target/release/create-image --csv ../test/csr.csv --kernel xous-kernel/target/riscv32i-unknown-none-elf/release/xous-kernel xous-kernel.bin
cat xous-stage1.bin xous-kernel.bin > xous.img

if [ $# -gt 0 ]
then
    if [ -z "$2" ]
    then
	scp xous.img $1:code/betrusted-scripts/
	scp ../build/gateware/encrypted.bin $1:code/betrusted-scripts/
    else
	scp -i $2 xous.img $1:code/betrusted-scripts/
	scp -i $2 ../build/gateware/encrypted.bin $1:code/betrusted-scripts/
    fi
else
    echo "Copy to target with $0 <user@host> <ssh-id>"
fi
