# SoC Candidate Files

## Contents

- [wrapper](./cram_axi.v)
- [CPU core](./VexRiscv_CranSoC.v)
- [AES ROM for core](./VexRiscv_CranSoC.v_toplevel_memory_AesPlugin_rom_storage.bin)
- [Register file abstract model](./ram_1w_1ra.v)
- [Cache memory abstract model](./ram_1w_1rs.v)

## Description

This is a customized VexRiscv core in a wrapper that provides the following bus interfaces:
- 64-bit AXI-4 instruction cache bus (read-only cached)
- 32-bit AXI-4 data cache bus (r/w cached)
- 32-bit AXI-lite peripheral bus (r/w uncached)
- All busses run at ACLK speed

The core itself contains the following features:
- VexRiscv CPU (simple, in-order RV32-IMAC with pipelining)
- Static branch prediction
- 4k, 4-way D-cache
- 4k, 4-way I-cache
- MMU and 8-entry TLB
- AES instruction extensions
- Non-cached regions (available for I/O):
  - 0x4000_0000-0x5FFF_FFFF (routed to p_bus)
  - 0xA000_0000-0xFFFF_FFFF (not routed to p_bus)
- Peripheral AXI-lite routed via crossbar
  - Only 0x4000_0000-0x5FFF_FFF is routed to peripheral AXI

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
- (stretch) Replicated PC/SP/SATP/TLB for anti-glitch hardening
- (very stretch) ECC on cache for anti-glitch hardening

## Update History:

Nov 16, 2022:
- Fetching & running instructions from AXI RAM located at 0x6000_0000-0x6100_0000
- I/O read/write / non-cacheable access tested
- Much, much more testing required (many bugs to be found still)

