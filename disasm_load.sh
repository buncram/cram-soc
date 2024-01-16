#!/bin/bash

# Generates listings for binary objects so that codezoom.py can do its thing
# Called by verilate.sh

PREFIX="../xous-core"

riscv-none-elf-objdump -h $PREFIX/target/riscv32imac-unknown-xous-elf/release/loader > listings/load.lst
riscv-none-elf-nm -r --size-sort --print-size $PREFIX/target/riscv32imac-unknown-xous-elf/release/loader | rustfilt >> listings/load.lst
riscv-none-elf-objdump $PREFIX/target/riscv32imac-unknown-xous-elf/release/loader -S -d | rustfilt >> listings/load.lst

riscv-none-elf-objdump -h $PREFIX/target/riscv32imac-unknown-xous-elf/release/xous-kernel > listings/kernel.lst
riscv-none-elf-nm -r --size-sort --print-size $PREFIX/target/riscv32imac-unknown-xous-elf/release/xous-kernel | rustfilt >> listings/kernel.lst
riscv-none-elf-objdump $PREFIX/target/riscv32imac-unknown-xous-elf/release/xous-kernel -S -d | rustfilt >> listings/kernel.lst

riscv-none-elf-objdump -h $PREFIX/target/riscv32imac-unknown-xous-elf/release/cram-console > listings/console.lst
riscv-none-elf-nm -r --size-sort --print-size $PREFIX/target/riscv32imac-unknown-xous-elf/release/cram-console | rustfilt >> listings/console.lst
riscv-none-elf-objdump $PREFIX/target/riscv32imac-unknown-xous-elf/release/cram-console -S -d | rustfilt >> listings/console.lst

riscv-none-elf-objdump -h $PREFIX/target/riscv32imac-unknown-xous-elf/release/xous-ticktimer > listings/ticktimer.lst
riscv-none-elf-nm -r --size-sort --print-size $PREFIX/target/riscv32imac-unknown-xous-elf/release/xous-ticktimer | rustfilt >> listings/ticktimer.lst
riscv-none-elf-objdump $PREFIX/target/riscv32imac-unknown-xous-elf/release/xous-ticktimer -S -d | rustfilt >> listings/ticktimer.lst

riscv-none-elf-objdump -h $PREFIX/target/riscv32imac-unknown-xous-elf/release/xous-log > listings/log.lst
riscv-none-elf-nm -r --size-sort --print-size $PREFIX/target/riscv32imac-unknown-xous-elf/release/xous-log | rustfilt >> listings/log.lst
riscv-none-elf-objdump $PREFIX/target/riscv32imac-unknown-xous-elf/release/xous-log -S -d | rustfilt >> listings/log.lst

riscv-none-elf-objdump -h $PREFIX/target/riscv32imac-unknown-xous-elf/release/xous-names > listings/names.lst
riscv-none-elf-nm -r --size-sort --print-size $PREFIX/target/riscv32imac-unknown-xous-elf/release/xous-names | rustfilt >> listings/names.lst
riscv-none-elf-objdump $PREFIX/target/riscv32imac-unknown-xous-elf/release/xous-names -S -d | rustfilt >> listings/names.lst
