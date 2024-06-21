cp betrusted-boot/link.x.straight betrusted-boot/link.x # to be used in conjunction with --feature gdb-load
cargo xtask boot-image --no-default-features --feature daric --feature full-chip --feature gdb-load --feature pl230-test # --feature apb-test # --feature cram-fpga <- use this to compile for cramium FPGA devboard

# simulation build - for testing rram
# cargo xtask boot-image --no-default-features --feature daric --feature gdb-load --feature rram-testing

riscv-none-elf-objdump -h target/riscv32imac-unknown-none-elf/release/betrusted-boot > boot.lst
riscv-none-elf-nm -r --size-sort --print-size target/riscv32imac-unknown-none-elf/release/betrusted-boot | rustfilt >> boot.lst
riscv-none-elf-objdump target/riscv32imac-unknown-none-elf/release/betrusted-boot -S -d | rustfilt >> boot.lst

#scp boot.bin bunnie@10.0.245.155:code/jtag-tools/
rsync -aiv -e 'ssh -i ~/.ssh/id_s32 -4 -p 22132 -c aes128-gcm@openssh.com' boot.bin bunnie@silico.dscloud.biz:boot.bin

md5sum boot.bin
