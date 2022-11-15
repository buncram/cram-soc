#!/bin/bash

cp ../sim_support/ram_1w_1ra.v .
cp ../sim_support/ram_1w_1rs.v .
cp ../deps/pythondata-cpu-vexriscv/pythondata_cpu_vexriscv/verilog/VexRiscv_CranSoC.v_toplevel_memory_AesPlugin_rom_storage.bin .
cp ../deps/pythondata-cpu-vexriscv/pythondata_cpu_vexriscv/verilog/VexRiscv_CranSoC.v .
cp ../build/gateware/cram_axi.v .
