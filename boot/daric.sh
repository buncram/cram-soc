cp betrusted-boot/link.x.straight betrusted-boot/link.x # to be used in conjunction with --feature gdb-load
cargo xtask boot-image --feature daric --feature full-chip --feature gdb-load # --feature apb-test # --feature pio-test

riscv-none-elf-objdump -h target/riscv32imac-unknown-none-elf/release/betrusted-boot > boot.lst
riscv-none-elf-nm -r --size-sort --print-size target/riscv32imac-unknown-none-elf/release/betrusted-boot | rustfilt >> boot.lst
riscv-none-elf-objdump target/riscv32imac-unknown-none-elf/release/betrusted-boot -S -d | rustfilt >> boot.lst
