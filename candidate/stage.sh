#!/bin/bash

cp ../sim_support/ram_1w_1ra.v .
cp ../sim_support/ram_1w_1rs.v .
cp ../deps/pythondata-cpu-vexriscv/pythondata_cpu_vexriscv/verilog/VexRiscv_CranSoC.v_toplevel_memory_AesPlugin_rom_storage.bin .
cp ../deps/pythondata-cpu-vexriscv/pythondata_cpu_vexriscv/verilog/VexRiscv_CranSoC.v .
cp ../build/gateware/cram_axi.v .

# copy over the AXI rtl models
cp ../deps/verilog-axi/rtl/axi_ram.v .
cp ../deps/verilog-axi/rtl/axi_axil_adapter.v .
cp ../deps/verilog-axi/rtl/axi_axil_adapter_rd.v .
cp ../deps/verilog-axi/rtl/axi_axil_adapter_wr.v .
cp ../deps/verilog-axi/rtl/axi_crossbar.v .
cp ../deps/verilog-axi/rtl/axi_crossbar_addr.v .
cp ../deps/verilog-axi/rtl/axi_crossbar_rd.v .
cp ../deps/verilog-axi/rtl/axi_crossbar_wr.v .
cp ../deps/verilog-axi/rtl/arbiter.v .
cp ../deps/verilog-axi/rtl/axi_register_wr.v .
cp ../deps/verilog-axi/rtl/axi_register_rd.v .
cp ../deps/verilog-axi/rtl/priority_encoder.v .
cp ../deps/verilog-axi/rtl/axi_adapter_wr.v .
cp ../deps/verilog-axi/rtl/axi_adapter_rd.v .
cp ../deps/verilog-axi/rtl/axi_adapter.v .
