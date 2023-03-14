#!/bin/bash
cd ..

python3 ./cram_core.py

rm -rf build/gateware/build/documentation/_build

sphinx-build -M latexpdf build/gateware/build/documentation/ build/gateware/build/documentation/_build
sphinx-build -M html build/gateware/build/documentation/ build/gateware/build/documentation/_build
cd candidate

cp ../sim_support/ram_1w_1ra.v .
cp ../sim_support/ram_1w_1rs.v .
cp ../VexRiscv/VexRiscv_CramSoC.v_toplevel_memory_AesPlugin_rom_storage.bin .
cp ../VexRiscv/VexRiscv_CramSoC.v .
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

rm -rf docs
mkdir docs
cp -r ../build/gateware/build/documentation/_build/html/* docs/
cp ../build/gateware/build/documentation/_build/latex/cramiumsocrisc-vcorecomplex.pdf docs/

scp -r docs/* bunnie@ci.betrusted.io:/var/cramium/
