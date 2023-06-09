python3 ./cram_soc.py --simulator xsim --svd-only

cd .\boot

cargo xtask boot-image --feature sim --feature daric --feature pio-test

riscv-none-elf-objdump -h target/riscv32imac-unknown-none-elf/release/betrusted-boot > boot.lst
riscv-none-elf-nm -r --size-sort --print-size target/riscv32imac-unknown-none-elf/release/betrusted-boot | rustfilt >> boot.lst
riscv-none-elf-objdump target/riscv32imac-unknown-none-elf/release/betrusted-boot -S -d | rustfilt >> boot.lst

cd ..\

python3 ./cram_soc.py --simulator xsim --bios .\boot\boot.bin
