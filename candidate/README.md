# SoC Candidate Files

## Contents

- [wrapper](./cram_axi.v)
- [CPU core](./VexRiscv_CranSoC.v)
- [AES ROM for core](./VexRiscv_CranSoC.v_toplevel_memory_AesPlugin_rom_storage.bin)
- [Register file abstract model](./ram_1w_1ra.v)
- [Cache memory abstract model](./ram_1w_1rs.v)
- Source files for the AXI pathway

## Description

Please refer to the [documentation](docs/index.html)

## Roadmap

- Banked interrupts
   - 16 banks of interrupts
   - Each bank has 16 inputs
   - 256 total interrupts
   - Must group interrupts logically to banks (do not mix/match functions across banks)
- SATP break-out
   - Provides core_user signal
- WFI signal break-out
   - Must confirm AXI bus is relinquished when WFI active
- Lot and lots of testing
  - Synthetic test benches exercising corner cases
  - Verification of memory access partitioning through d-bus xbar
  - AXI/AXIlite bus signaling verification
  - Exception handling
  - WFI verification
  - core_user verification
  - Bootable Xous image on FPGA model
- (stretch) Replicated PC/SP/SATP/TLB for anti-glitch hardening
- (very stretch) ECC on cache for anti-glitch hardening

## Update History:

Nov 16, 2022:
- Fetching & running instructions from AXI RAM located at 0x6000_0000-0x6100_0000
- I/O read/write / non-cacheable access tested
- Much, much more testing required (many bugs to be found still)

Dec 23, 2022:
- Add `coreuser` signal
- Rework AXI pathway to use `verilog-axi` framework
- Add auto-generated documentation tree, synchronized to SpinalHDL source for CPU
