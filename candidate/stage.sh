#!/bin/bash
cd ..

python3 ./soc_oss/mbox_client.py

python3 ./cram_core.py

#rm -rf build/gateware/build/documentation/_build

#sphinx-build -M latexpdf build/gateware/build/documentation/ build/gateware/build/documentation/_build
#sphinx-build -M html build/gateware/build/documentation/ build/gateware/build/documentation/_build
cd candidate

cp ../build/software/soc.svd .
cp ../build/software/core.svd .

mkdir -p libs
mkdir -p tb

cp ../sim_support/mbox_v0.1.sv ./libs/
cp ../sim_support/mbox_client.v ./libs/
cp ../sim_support/ram_1w_1ra.v ./libs/
cp ../sim_support/ram_1w_1rs.v ./libs/
cp ../sim_support/bio_tb.v ./tb/
cp ../VexRiscv/VexRiscv_CramSoC.v ./libs/
cp ../VexRiscv/memory_AesZknPlugin_rom_storage_Rom_1rs.v ./libs/
cp ../build/gateware/cram_axi.v .

# copy over the AXI rtl models
cp ../deps/verilog-axi/rtl/axi_ram.v ./libs/
cp ../deps/verilog-axi/rtl/axi_axil_adapter.v ./libs/
cp ../deps/verilog-axi/rtl/axi_axil_adapter_rd.v ./libs/
cp ../deps/verilog-axi/rtl/axi_axil_adapter_wr.v ./libs/
cp ../deps/verilog-axi/rtl/axi_crossbar.v ./libs/
cp ../deps/verilog-axi/rtl/axi_crossbar_addr.v ./libs/
cp ../deps/verilog-axi/rtl/axi_crossbar_rd.v ./libs/
cp ../deps/verilog-axi/rtl/axi_crossbar_wr.v ./libs/
cp ../deps/verilog-axi/rtl/arbiter.v ./libs/
cp ../deps/verilog-axi/rtl/axi_register_wr.v ./libs/
cp ../deps/verilog-axi/rtl/axi_register_rd.v ./libs/
cp ../deps/verilog-axi/rtl/priority_encoder.v ./libs/
cp ../deps/verilog-axi/rtl/axi_adapter_wr.v ./libs/
cp ../deps/verilog-axi/rtl/axi_adapter_rd.v ./libs/
cp ../deps/verilog-axi/rtl/axi_adapter.v ./libs/

rm -rf bio/*
cp -r ../deps/bio/* bio/
mkdir bio/libs/

# used by BIO
cp ../deps/verilog-axi/rtl/axil_crossbar.v ./bio/libs/
cp ../deps/verilog-axi/rtl/axil_crossbar_addr.v ./bio/libs/
cp ../deps/verilog-axi/rtl/axil_crossbar_rd.v ./bio/libs/
cp ../deps/verilog-axi/rtl/axil_crossbar_wr.v ./bio/libs/
cp ../deps/verilog-axi/rtl/axil_register_wr.v ./bio/libs/
cp ../deps/verilog-axi/rtl/axil_register_rd.v ./bio/libs/
cp ../deps/verilog-axi/rtl/axil_reg_if.v ./bio/libs/
cp ../deps/verilog-axi/rtl/axil_reg_if_rd.v ./bio/libs/
cp ../deps/verilog-axi/rtl/axil_reg_if_wr.v ./bio/libs/
cp ../deps/verilog-axi/rtl/axil_cdc.v ./bio/libs/
cp ../deps/verilog-axi/rtl/axil_cdc_wr.v ./bio/libs/
cp ../deps/verilog-axi/rtl/axil_cdc_rd.v ./bio/libs/
cp ../sim_support/cdc_level_to_pulse.sv ./bio/libs/

# used by BIO; maybe substitute with CM7 HDK option if performance is better
cp ../deps/axi2ahb/axi2ahb.v ./bio/libs/
cp ../deps/axi2ahb/axi2ahb_cmd.v ./bio/libs/
cp ../deps/axi2ahb/axi2ahb_ctrl.v ./bio/libs/
cp ../deps/axi2ahb/axi2ahb_rd_fifo.v ./bio/libs/
cp ../deps/axi2ahb/axi2ahb_wr_fifo.v ./bio/libs/
cp ../deps/axi2ahb/prgen_fifo.v ./bio/libs/

# copy over PIO rtl models
#rm -rf pio/*
#cp ../deps/pio/upstream/src/*.v pio/
# remove the legacy top model
#rm -f pio/pio.v
# add the correct top model
#cp ../deps/pio/pio_apb.sv pio/
#cp ../deps/pio/rp_pio.sv pio/
# add support modules
#cp ../sim_support/cdc_blinded.v .

#rm -rf docs
#mkdir docs
#cp -r ../build/gateware/build/documentation/_build/html/* docs/
#cp ../build/gateware/build/documentation/_build/latex/cramiumsocrisc-vcorecomplex.pdf docs/

# sync the docs to the web
rsync -a --delete ../build/gateware/build/documentation/_build/html/* bunnie@ci.betrusted.io:/var/cramium-cpu/
rsync -a --delete ../build/doc/daric_doc/_build/html/* bunnie@ci.betrusted.io:/var/cramium/
