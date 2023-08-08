#!/bin/bash
cd ..

python3 ./mbox_client.py

python3 ./cram_core.py

#rm -rf build/gateware/build/documentation/_build

#sphinx-build -M latexpdf build/gateware/build/documentation/ build/gateware/build/documentation/_build
#sphinx-build -M html build/gateware/build/documentation/ build/gateware/build/documentation/_build
cd candidate

cp ../build/software/soc.svd .
cp ../build/software/core.svd .

cp ../sim_support/mbox_v0.1.sv .
cp ../sim_support/mbox_client.v .
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

# copy over PIO rtl models
rm -rf pio/*
cp ../deps/pio/upstream/src/*.v pio/
# remove the legacy top model
rm -f pio/pio.v
# add the correct top model
cp ../deps/pio/pio_apb.sv pio/
cp ../deps/pio/rp_pio.sv pio/
# add support modules
cp ../sim_support/cdc_blinded.v .

rm -rf docs
mkdir docs
cp -r ../build/gateware/build/documentation/_build/html/* docs/
cp ../build/gateware/build/documentation/_build/latex/cramiumsocrisc-vcorecomplex.pdf docs/

scp -r docs/* bunnie@ci.betrusted.io:/var/cramium/
