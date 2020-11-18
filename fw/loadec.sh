sudo wishbone-tool 0x40080000 --burst-source ../../betrusted-ec/target/riscv32i-unknown-none-elf/release/bt-ec.bin
sudo wishbone-tool 0x40080000 --burst-length 159744 > bt-ec.v
diff -s bt-ec.v ../../betrusted-ec/target/riscv32i-unknown-none-elf/release/bt-ec.bin
