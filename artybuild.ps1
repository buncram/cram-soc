python3 ./cram_core.py
python3 ./cram_arty.py

cd .\boot

cp betrusted-boot\link.x.straight betrusted-boot\link.x
cargo xtask boot-image --feature daric --feature gdb-load --feature arty # --feature pio-test

riscv-none-elf-objdump -h target/riscv32imac-unknown-none-elf/release/betrusted-boot > boot.lst
riscv-none-elf-nm -r --size-sort --print-size target/riscv32imac-unknown-none-elf/release/betrusted-boot | rustfilt >> boot.lst
riscv-none-elf-objdump target/riscv32imac-unknown-none-elf/release/betrusted-boot -S -d | rustfilt >> boot.lst

scp target/riscv32imac-unknown-none-elf/release/betrusted-boot bunnie@192.168.137.37:

cd ..\

python3 ./cram_arty.py --build --bios .\boot\boot.bin
